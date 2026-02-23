use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sqlx::mysql::{MySql, MySqlPool, MySqlPoolOptions};
use sqlx::pool::PoolConnection;
use sqlx::{Executor, Row};
use tokio::sync::Mutex;

use crate::db::traits::{DbDriver, SqlDriver};
use crate::db::types::{mysql_columns_to_defs, mysql_row_to_cells};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::QueryResponse;
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo,
    RoutineInfo, SchemaInfo, TableInfo, TableStats,
};

pub struct MySqlDriver {
    pool: MySqlPool,
    txn_conn: Mutex<Option<PoolConnection<MySql>>>,
}

impl MySqlDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let url = config.to_connection_url();
        let pool = MySqlPoolOptions::new()
            .max_connections(config.pool_max_connections)
            .idle_timeout(Duration::from_secs(config.pool_idle_timeout_secs))
            .acquire_timeout(Duration::from_secs(config.pool_acquire_timeout_secs))
            .connect(&url)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to MySQL: {}", e)))?;

        Ok(Self {
            pool,
            txn_conn: Mutex::new(None),
        })
    }

    async fn execute_on<'e, E: Executor<'e, Database = MySql>>(
        executor: E,
        sql: &str,
    ) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let trimmed = sql.trim();
        let upper = trimmed.to_uppercase();

        let is_select = upper.starts_with("SELECT")
            || upper.starts_with("WITH")
            || upper.starts_with("SHOW")
            || upper.starts_with("EXPLAIN")
            || upper.starts_with("DESCRIBE")
            || upper.starts_with("DESC")
            || upper.starts_with("TABLE");

        if is_select {
            let rows = sqlx::query(trimmed).fetch_all(executor).await?;
            let elapsed = start.elapsed().as_millis() as u64;

            let columns = if rows.is_empty() {
                Vec::new()
            } else {
                mysql_columns_to_defs(&rows[0])
            };

            let row_count = rows.len();
            let data: Vec<Vec<_>> = rows.iter().map(|r| mysql_row_to_cells(r)).collect();

            Ok(QueryResponse {
                columns,
                rows: data,
                row_count,
                execution_time_ms: elapsed,
                affected_rows: None,
                truncated: false,
                max_rows_limit: None,
            })
        } else {
            let result = sqlx::query(trimmed).execute(executor).await?;
            let elapsed = start.elapsed().as_millis() as u64;
            let affected = result.rows_affected();

            Ok(QueryResponse {
                columns: Vec::new(),
                rows: Vec::new(),
                row_count: 0,
                execution_time_ms: elapsed,
                affected_rows: Some(affected),
                truncated: false,
                max_rows_limit: None,
            })
        }
    }
}

#[async_trait]
impl DbDriver for MySqlDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Relational
    }

    fn dialect_hint(&self) -> &'static str {
        "mysql"
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResponse, AppError> {
        let mut guard = self.txn_conn.lock().await;
        if let Some(ref mut conn) = *guard {
            Self::execute_on(&mut **conn, sql).await
        } else {
            drop(guard);
            Self::execute_on(&self.pool, sql).await
        }
    }

    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> {
        let schemas = self.get_schemas().await?;
        Ok(schemas.iter().map(ContainerInfo::from).collect())
    }

    async fn get_items(&self, container: &str) -> Result<Vec<ItemInfo>, AppError> {
        let tables = self.get_tables(container).await?;
        Ok(tables.iter().map(ItemInfo::from).collect())
    }

    async fn get_item_fields(
        &self,
        container: &str,
        item: &str,
    ) -> Result<Vec<FieldInfo>, AppError> {
        let columns = self.get_columns(container, item).await?;
        Ok(columns.iter().map(FieldInfo::from).collect())
    }

    async fn get_item_data(
        &self,
        container: &str,
        item: &str,
        limit: i64,
        offset: i64,
    ) -> Result<QueryResponse, AppError> {
        SqlDriver::get_table_data(self, container, item, limit, offset).await
    }

    async fn get_item_count(&self, container: &str, item: &str) -> Result<i64, AppError> {
        SqlDriver::get_row_count(self, container, item).await
    }
}

#[async_trait]
impl SqlDriver for MySqlDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT SCHEMA_NAME as name FROM information_schema.SCHEMATA \
             WHERE SCHEMA_NAME NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys') \
             ORDER BY SCHEMA_NAME",
        )
        .fetch_all(&self.pool)
        .await?;

        let schemas = rows
            .iter()
            .map(|row| {
                let name: String = row.get("name");
                SchemaInfo { name }
            })
            .collect();

        Ok(schemas)
    }

    async fn get_tables(&self, schema: &str) -> Result<Vec<TableInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT TABLE_NAME, TABLE_TYPE \
             FROM information_schema.TABLES \
             WHERE TABLE_SCHEMA = ? \
             ORDER BY TABLE_NAME",
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await?;

        let tables = rows
            .iter()
            .map(|row| {
                let name: String = row.get("TABLE_NAME");
                let table_type: String = row.get("TABLE_TYPE");
                TableInfo {
                    name,
                    schema: schema.to_string(),
                    table_type,
                    row_count: None,
                }
            })
            .collect();

        Ok(tables)
    }

    async fn get_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, \
                    ORDINAL_POSITION, COLUMN_KEY \
             FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? \
             ORDER BY ORDINAL_POSITION",
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let columns = rows
            .iter()
            .map(|row| {
                let name: String = row.get("COLUMN_NAME");
                let data_type: String = row.get("COLUMN_TYPE");
                let is_nullable_str: String = row.get("IS_NULLABLE");
                let column_default: Option<String> = row.get("COLUMN_DEFAULT");
                let ordinal_position: u32 = row.get("ORDINAL_POSITION");
                let column_key: String = row.get("COLUMN_KEY");

                ColumnInfo {
                    name,
                    data_type,
                    is_nullable: is_nullable_str == "YES",
                    column_default,
                    is_primary_key: column_key == "PRI",
                    ordinal_position: ordinal_position as i32,
                }
            })
            .collect();

        Ok(columns)
    }

    async fn get_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT INDEX_NAME, COLUMN_NAME, NON_UNIQUE, INDEX_TYPE, SEQ_IN_INDEX \
             FROM information_schema.STATISTICS \
             WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? \
             ORDER BY INDEX_NAME, SEQ_IN_INDEX",
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let mut index_map: HashMap<String, IndexInfo> = HashMap::new();

        for row in &rows {
            let name: String = row.get("INDEX_NAME");
            let column_name: String = row.get("COLUMN_NAME");
            let non_unique: i64 = row.get("NON_UNIQUE");
            let index_type: String = row.get("INDEX_TYPE");

            let entry = index_map.entry(name.clone()).or_insert_with(|| IndexInfo {
                name: name.clone(),
                columns: Vec::new(),
                is_unique: non_unique == 0,
                is_primary: name == "PRIMARY",
                index_type: index_type.clone(),
            });

            entry.columns.push(column_name);
        }

        let mut indexes: Vec<IndexInfo> = index_map.into_values().collect();
        indexes.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(indexes)
    }

    async fn get_foreign_keys(
        &self,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ForeignKeyInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT tc.CONSTRAINT_NAME, \
                    kcu.COLUMN_NAME, \
                    kcu.REFERENCED_TABLE_NAME, \
                    kcu.REFERENCED_TABLE_SCHEMA, \
                    kcu.REFERENCED_COLUMN_NAME, \
                    rc.UPDATE_RULE, \
                    rc.DELETE_RULE \
             FROM information_schema.TABLE_CONSTRAINTS tc \
             JOIN information_schema.KEY_COLUMN_USAGE kcu \
               ON tc.CONSTRAINT_NAME = kcu.CONSTRAINT_NAME \
               AND tc.TABLE_SCHEMA = kcu.TABLE_SCHEMA \
               AND tc.TABLE_NAME = kcu.TABLE_NAME \
             JOIN information_schema.REFERENTIAL_CONSTRAINTS rc \
               ON tc.CONSTRAINT_NAME = rc.CONSTRAINT_NAME \
               AND tc.CONSTRAINT_SCHEMA = rc.CONSTRAINT_SCHEMA \
             WHERE tc.CONSTRAINT_TYPE = 'FOREIGN KEY' \
               AND tc.TABLE_SCHEMA = ? \
               AND tc.TABLE_NAME = ? \
             ORDER BY tc.CONSTRAINT_NAME, kcu.ORDINAL_POSITION",
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let mut fk_map: HashMap<String, ForeignKeyInfo> = HashMap::new();

        for row in &rows {
            let name: String = row.get("CONSTRAINT_NAME");
            let column: String = row.get("COLUMN_NAME");
            let referenced_table: String = row.get("REFERENCED_TABLE_NAME");
            let referenced_schema: String = row.get("REFERENCED_TABLE_SCHEMA");
            let referenced_column: String = row.get("REFERENCED_COLUMN_NAME");
            let on_update: String = row.get("UPDATE_RULE");
            let on_delete: String = row.get("DELETE_RULE");

            let entry = fk_map.entry(name.clone()).or_insert_with(|| ForeignKeyInfo {
                name,
                columns: Vec::new(),
                referenced_table,
                referenced_schema,
                referenced_columns: Vec::new(),
                on_update,
                on_delete,
            });

            if !entry.columns.contains(&column) {
                entry.columns.push(column);
            }
            if !entry.referenced_columns.contains(&referenced_column) {
                entry.referenced_columns.push(referenced_column);
            }
        }

        let mut foreign_keys: Vec<ForeignKeyInfo> = fk_map.into_values().collect();
        foreign_keys.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(foreign_keys)
    }

    async fn get_table_data(
        &self,
        schema: &str,
        table: &str,
        limit: i64,
        offset: i64,
    ) -> Result<QueryResponse, AppError> {
        let sql = format!(
            "SELECT * FROM `{}`.`{}` LIMIT {} OFFSET {}",
            schema, table, limit, offset
        );
        self.execute_raw(&sql).await
    }

    async fn get_row_count(&self, schema: &str, table: &str) -> Result<i64, AppError> {
        let sql = format!("SELECT COUNT(*) as count FROM `{}`.`{}`", schema, table);
        let row = sqlx::query(&sql).fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");
        Ok(count)
    }

    async fn update_cell(
        &self,
        schema: &str,
        table: &str,
        column: &str,
        value: &str,
        pk_columns: Vec<String>,
        pk_values: Vec<String>,
    ) -> Result<(), AppError> {
        if pk_columns.len() != pk_values.len() {
            return Err(AppError::InvalidConfig(
                "Primary key columns and values must have the same length".to_string(),
            ));
        }

        if pk_columns.is_empty() {
            return Err(AppError::InvalidConfig(
                "At least one primary key column is required".to_string(),
            ));
        }

        let where_clauses: Vec<String> = pk_columns
            .iter()
            .map(|col| format!("`{}` = ?", col))
            .collect();

        let sql = format!(
            "UPDATE `{}`.`{}` SET `{}` = ? WHERE {}",
            schema,
            table,
            column,
            where_clauses.join(" AND ")
        );

        let mut query = sqlx::query(&sql).bind(value);
        for pk_val in &pk_values {
            query = query.bind(pk_val);
        }
        query.execute(&self.pool).await?;
        Ok(())
    }

    async fn insert_row(
        &self,
        schema: &str,
        table: &str,
        columns: Vec<String>,
        values: Vec<String>,
    ) -> Result<(), AppError> {
        if columns.len() != values.len() {
            return Err(AppError::InvalidConfig(
                "Columns and values must have the same length".to_string(),
            ));
        }

        let cols: Vec<String> = columns.iter().map(|c| format!("`{}`", c)).collect();
        let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();

        let sql = format!(
            "INSERT INTO `{}`.`{}` ({}) VALUES ({})",
            schema,
            table,
            cols.join(", "),
            placeholders.join(", ")
        );

        let mut query = sqlx::query(&sql);
        for val in &values {
            query = query.bind(val);
        }
        query.execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_rows(
        &self,
        schema: &str,
        table: &str,
        pk_columns: Vec<String>,
        pk_values_list: Vec<Vec<String>>,
    ) -> Result<u64, AppError> {
        if pk_columns.is_empty() {
            return Err(AppError::InvalidConfig(
                "At least one primary key column is required".to_string(),
            ));
        }

        let mut total_affected: u64 = 0;

        for pk_values in &pk_values_list {
            if pk_columns.len() != pk_values.len() {
                return Err(AppError::InvalidConfig(
                    "Primary key columns and values must have the same length".to_string(),
                ));
            }

            let where_clauses: Vec<String> = pk_columns
                .iter()
                .map(|col| format!("`{}` = ?", col))
                .collect();

            let sql = format!(
                "DELETE FROM `{}`.`{}` WHERE {}",
                schema,
                table,
                where_clauses.join(" AND ")
            );

            let mut query = sqlx::query(&sql);
            for pk_val in pk_values {
                query = query.bind(pk_val);
            }
            let result = query.execute(&self.pool).await?;
            total_affected += result.rows_affected();
        }

        Ok(total_affected)
    }

    async fn get_table_stats(&self, schema: &str, table: &str) -> Result<TableStats, AppError> {
        let row = sqlx::query(
            "SELECT TABLE_ROWS, DATA_LENGTH, INDEX_LENGTH \
             FROM information_schema.TABLES \
             WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?",
        )
        .bind(schema)
        .bind(table)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let row_count: Option<i64> = row.try_get("TABLE_ROWS").ok();
                let data_length: Option<i64> = row.try_get("DATA_LENGTH").ok();
                let index_length: Option<i64> = row.try_get("INDEX_LENGTH").ok();

                let size_bytes = match (data_length, index_length) {
                    (Some(d), Some(i)) => Some(d + i),
                    (Some(d), None) => Some(d),
                    _ => None,
                };

                let size_display = size_bytes.map(format_bytes);

                Ok(TableStats {
                    row_count: row_count.unwrap_or(0),
                    size_bytes,
                    size_display,
                })
            }
            None => {
                let count = self.get_row_count(schema, table).await?;
                Ok(TableStats {
                    row_count: count,
                    size_bytes: None,
                    size_display: None,
                })
            }
        }
    }

    async fn begin_transaction(&self) -> Result<(), AppError> {
        let mut guard = self.txn_conn.lock().await;
        if guard.is_some() {
            return Err(AppError::Database("Transaction already active".to_string()));
        }
        let mut conn = self.pool.acquire().await?;
        sqlx::query("BEGIN").execute(&mut *conn).await?;
        *guard = Some(conn);
        Ok(())
    }

    async fn commit_transaction(&self) -> Result<(), AppError> {
        let mut guard = self.txn_conn.lock().await;
        if let Some(ref mut conn) = *guard {
            sqlx::query("COMMIT").execute(&mut **conn).await?;
            *guard = None;
            Ok(())
        } else {
            Err(AppError::Database("No active transaction".to_string()))
        }
    }

    async fn rollback_transaction(&self) -> Result<(), AppError> {
        let mut guard = self.txn_conn.lock().await;
        if let Some(ref mut conn) = *guard {
            sqlx::query("ROLLBACK").execute(&mut **conn).await?;
            *guard = None;
            Ok(())
        } else {
            Err(AppError::Database("No active transaction".to_string()))
        }
    }

    async fn in_transaction(&self) -> Result<bool, AppError> {
        let guard = self.txn_conn.lock().await;
        Ok(guard.is_some())
    }

    async fn get_routines(&self, schema: &str) -> Result<Vec<RoutineInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT ROUTINE_NAME, ROUTINE_TYPE, DTD_IDENTIFIER \
             FROM information_schema.ROUTINES \
             WHERE ROUTINE_SCHEMA = ? \
             ORDER BY ROUTINE_NAME",
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await?;

        let routines = rows
            .iter()
            .map(|row| {
                let name: String = row.get("ROUTINE_NAME");
                let routine_type: String = row.get("ROUTINE_TYPE");
                let return_type: Option<String> = row.get("DTD_IDENTIFIER");
                RoutineInfo {
                    name,
                    schema: schema.to_string(),
                    routine_type,
                    return_type,
                }
            })
            .collect();

        Ok(routines)
    }
}

fn format_bytes(bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}
