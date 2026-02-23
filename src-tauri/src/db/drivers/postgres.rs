use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgPool, PgPoolOptions, Postgres};
use sqlx::{Executor, Row};
use tokio::sync::Mutex;

use crate::db::traits::{DbDriver, SqlDriver};
use crate::db::types::{pg_columns_to_defs, pg_row_to_cells};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::QueryResponse;
use crate::models::schema::{
    ColumnInfo, ContainerInfo, EnumInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo,
    RoutineInfo, SchemaInfo, SequenceInfo, TableInfo, TableStats,
};

pub struct PostgresDriver {
    pool: PgPool,
    txn_conn: Mutex<Option<PoolConnection<Postgres>>>,
}

impl PostgresDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let url = config.to_connection_url();
        let pool = PgPoolOptions::new()
            .max_connections(config.pool_max_connections)
            .idle_timeout(Duration::from_secs(config.pool_idle_timeout_secs))
            .acquire_timeout(Duration::from_secs(config.pool_acquire_timeout_secs))
            .connect(&url)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to PostgreSQL: {}", e)))?;

        Ok(Self {
            pool,
            txn_conn: Mutex::new(None),
        })
    }

    /// Execute a query using the transaction connection if active, otherwise pool.
    async fn execute_on<'e, E: Executor<'e, Database = Postgres>>(
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
            || upper.starts_with("TABLE")
            || upper.starts_with("VALUES");

        if is_select {
            let rows = sqlx::query(trimmed).fetch_all(executor).await?;
            let elapsed = start.elapsed().as_millis() as u64;

            let columns = if rows.is_empty() {
                Vec::new()
            } else {
                pg_columns_to_defs(&rows[0])
            };

            let row_count = rows.len();
            let data: Vec<Vec<_>> = rows.iter().map(|r| pg_row_to_cells(r)).collect();

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
impl DbDriver for PostgresDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Relational
    }

    fn dialect_hint(&self) -> &'static str {
        "postgres"
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
impl SqlDriver for PostgresDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT schema_name FROM information_schema.schemata \
             WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast') \
             ORDER BY schema_name",
        )
        .fetch_all(&self.pool)
        .await?;

        let schemas = rows
            .iter()
            .map(|row| {
                let name: String = row.get("schema_name");
                SchemaInfo { name }
            })
            .collect();

        Ok(schemas)
    }

    async fn get_tables(&self, schema: &str) -> Result<Vec<TableInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT table_name, table_type \
             FROM information_schema.tables \
             WHERE table_schema = $1 \
             ORDER BY table_name",
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await?;

        let tables = rows
            .iter()
            .map(|row| {
                let name: String = row.get("table_name");
                let table_type: String = row.get("table_type");
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
            "SELECT c.column_name, c.data_type, c.is_nullable, c.column_default, c.ordinal_position, \
             CASE WHEN tc.constraint_type = 'PRIMARY KEY' THEN true ELSE false END as is_pk \
             FROM information_schema.columns c \
             LEFT JOIN information_schema.key_column_usage kcu \
               ON c.table_schema = kcu.table_schema \
               AND c.table_name = kcu.table_name \
               AND c.column_name = kcu.column_name \
             LEFT JOIN information_schema.table_constraints tc \
               ON kcu.constraint_name = tc.constraint_name \
               AND kcu.table_schema = tc.table_schema \
               AND tc.constraint_type = 'PRIMARY KEY' \
             WHERE c.table_schema = $1 AND c.table_name = $2 \
             ORDER BY c.ordinal_position",
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let columns = rows
            .iter()
            .map(|row| {
                let name: String = row.get("column_name");
                let data_type: String = row.get("data_type");
                let is_nullable_str: String = row.get("is_nullable");
                let column_default: Option<String> = row.get("column_default");
                let ordinal_position: i32 = row.get("ordinal_position");
                let is_primary_key: bool = row.try_get("is_pk").unwrap_or(false);

                ColumnInfo {
                    name,
                    data_type,
                    is_nullable: is_nullable_str == "YES",
                    column_default,
                    is_primary_key,
                    ordinal_position,
                }
            })
            .collect();

        Ok(columns)
    }

    async fn get_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT i.relname as index_name, \
                    array_agg(a.attname ORDER BY array_position(ix.indkey, a.attnum)) as columns, \
                    ix.indisunique as is_unique, \
                    ix.indisprimary as is_primary, \
                    am.amname as index_type \
             FROM pg_index ix \
             JOIN pg_class t ON t.oid = ix.indrelid \
             JOIN pg_class i ON i.oid = ix.indexrelid \
             JOIN pg_namespace n ON n.oid = t.relnamespace \
             JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey) \
             JOIN pg_am am ON am.oid = i.relam \
             WHERE t.relname = $2 AND n.nspname = $1 \
             GROUP BY i.relname, ix.indisunique, ix.indisprimary, am.amname \
             ORDER BY i.relname",
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let indexes = rows
            .iter()
            .map(|row| {
                let name: String = row.get("index_name");
                let columns: Vec<String> = row.get("columns");
                let is_unique: bool = row.get("is_unique");
                let is_primary: bool = row.get("is_primary");
                let index_type: String = row.get("index_type");

                IndexInfo {
                    name,
                    columns,
                    is_unique,
                    is_primary,
                    index_type,
                }
            })
            .collect();

        Ok(indexes)
    }

    async fn get_foreign_keys(
        &self,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ForeignKeyInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT tc.constraint_name, \
                    kcu.column_name, \
                    ccu.table_name as referenced_table, \
                    ccu.table_schema as referenced_schema, \
                    ccu.column_name as referenced_column, \
                    rc.update_rule, \
                    rc.delete_rule \
             FROM information_schema.table_constraints tc \
             JOIN information_schema.key_column_usage kcu \
               ON tc.constraint_name = kcu.constraint_name \
               AND tc.table_schema = kcu.table_schema \
             JOIN information_schema.constraint_column_usage ccu \
               ON ccu.constraint_name = tc.constraint_name \
               AND ccu.table_schema = tc.table_schema \
             JOIN information_schema.referential_constraints rc \
               ON tc.constraint_name = rc.constraint_name \
               AND tc.table_schema = rc.constraint_schema \
             WHERE tc.constraint_type = 'FOREIGN KEY' \
               AND tc.table_schema = $1 \
               AND tc.table_name = $2 \
             ORDER BY tc.constraint_name, kcu.ordinal_position",
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let mut fk_map: HashMap<String, ForeignKeyInfo> = HashMap::new();

        for row in &rows {
            let name: String = row.get("constraint_name");
            let column: String = row.get("column_name");
            let referenced_table: String = row.get("referenced_table");
            let referenced_schema: String = row.get("referenced_schema");
            let referenced_column: String = row.get("referenced_column");
            let on_update: String = row.get("update_rule");
            let on_delete: String = row.get("delete_rule");

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
            "SELECT * FROM \"{}\".\"{}\" LIMIT {} OFFSET {}",
            schema, table, limit, offset
        );
        self.execute_raw(&sql).await
    }

    async fn get_row_count(&self, schema: &str, table: &str) -> Result<i64, AppError> {
        let sql = format!("SELECT COUNT(*) as count FROM \"{}\".\"{}\"", schema, table);
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
            .enumerate()
            .map(|(i, col)| format!("\"{}\" = ${}", col, i + 2))
            .collect();

        let sql = format!(
            "UPDATE \"{}\".\"{}\" SET \"{}\" = $1 WHERE {}",
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

        let cols: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c)).collect();
        let placeholders: Vec<String> = (1..=values.len()).map(|i| format!("${}", i)).collect();

        let sql = format!(
            "INSERT INTO \"{}\".\"{}\" ({}) VALUES ({})",
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
                .enumerate()
                .map(|(i, col)| format!("\"{}\" = ${}", col, i + 1))
                .collect();

            let sql = format!(
                "DELETE FROM \"{}\".\"{}\" WHERE {}",
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
            "SELECT c.reltuples::bigint AS row_count, \
                    pg_total_relation_size(c.oid) AS size_bytes \
             FROM pg_class c \
             JOIN pg_namespace n ON n.oid = c.relnamespace \
             WHERE n.nspname = $1 AND c.relname = $2",
        )
        .bind(schema)
        .bind(table)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let row_count: i64 = row.get("row_count");
                let size_bytes: i64 = row.get("size_bytes");
                let size_display = format_bytes(size_bytes);
                Ok(TableStats {
                    row_count: if row_count < 0 { 0 } else { row_count },
                    size_bytes: Some(size_bytes),
                    size_display: Some(size_display),
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

    async fn get_routines(&self, schema: &str) -> Result<Vec<RoutineInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT routine_name, routine_type, data_type \
             FROM information_schema.routines \
             WHERE routine_schema = $1 \
             ORDER BY routine_name",
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await?;

        let routines = rows
            .iter()
            .map(|row| {
                let name: String = row.get("routine_name");
                let routine_type: String = row.get("routine_type");
                let return_type: Option<String> = row.get("data_type");
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

    async fn get_sequences(&self, schema: &str) -> Result<Vec<SequenceInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT sequence_name, data_type \
             FROM information_schema.sequences \
             WHERE sequence_schema = $1 \
             ORDER BY sequence_name",
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await?;

        let sequences = rows
            .iter()
            .map(|row| {
                let name: String = row.get("sequence_name");
                let data_type: Option<String> = row.get("data_type");
                SequenceInfo {
                    name,
                    schema: schema.to_string(),
                    data_type,
                }
            })
            .collect();

        Ok(sequences)
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

    async fn get_enums(&self, schema: &str) -> Result<Vec<EnumInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT t.typname AS name, \
                    array_agg(e.enumlabel ORDER BY e.enumsortorder) AS variants \
             FROM pg_type t \
             JOIN pg_enum e ON e.enumtypid = t.oid \
             JOIN pg_namespace n ON n.oid = t.typnamespace \
             WHERE n.nspname = $1 \
             GROUP BY t.typname \
             ORDER BY t.typname",
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await?;

        let enums = rows
            .iter()
            .map(|row| {
                let name: String = row.get("name");
                let variants: Vec<String> = row.get("variants");
                EnumInfo {
                    name,
                    schema: schema.to_string(),
                    variants,
                }
            })
            .collect();

        Ok(enums)
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
