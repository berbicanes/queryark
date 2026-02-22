use std::time::Instant;

use async_trait::async_trait;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{Column, Row, TypeInfo, ValueRef};

use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo, SchemaInfo, TableInfo,
};

pub struct SqliteDriver {
    pool: SqlitePool,
}

impl SqliteDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let url = config.to_connection_url();
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to SQLite: {}", e)))?;

        Ok(Self { pool })
    }
}

fn sqlite_columns_to_defs(row: &sqlx::sqlite::SqliteRow) -> Vec<ColumnDef> {
    row.columns()
        .iter()
        .map(|col| ColumnDef {
            name: col.name().to_string(),
            data_type: col.type_info().name().to_string(),
        })
        .collect()
}

fn sqlite_row_to_cells(row: &sqlx::sqlite::SqliteRow) -> Vec<CellValue> {
    let columns = row.columns().len();
    let mut cells = Vec::with_capacity(columns);

    for i in 0..columns {
        let raw = row.try_get_raw(i);
        let is_null = match &raw {
            Ok(val) => val.is_null(),
            Err(_) => true,
        };

        if is_null {
            cells.push(CellValue::Null);
            continue;
        }

        let type_name = row.columns()[i].type_info().name().to_uppercase();

        let cell = match type_name.as_str() {
            "BOOLEAN" => match row.try_get::<bool, _>(i) {
                Ok(v) => CellValue::Bool(v),
                Err(_) => CellValue::Null,
            },
            "INTEGER" | "INT" | "BIGINT" | "SMALLINT" | "TINYINT" => {
                match row.try_get::<i64, _>(i) {
                    Ok(v) => CellValue::Int(v),
                    Err(_) => match row.try_get::<i32, _>(i) {
                        Ok(v) => CellValue::Int(v as i64),
                        Err(_) => CellValue::Null,
                    },
                }
            }
            "REAL" | "FLOAT" | "DOUBLE" => match row.try_get::<f64, _>(i) {
                Ok(v) => CellValue::Float(v),
                Err(_) => CellValue::Null,
            },
            "BLOB" => match row.try_get::<Vec<u8>, _>(i) {
                Ok(v) => CellValue::Binary(v),
                Err(_) => CellValue::Null,
            },
            _ => match row.try_get::<String, _>(i) {
                Ok(v) => CellValue::Text(v),
                Err(_) => match row.try_get::<i64, _>(i) {
                    Ok(v) => CellValue::Int(v),
                    Err(_) => match row.try_get::<f64, _>(i) {
                        Ok(v) => CellValue::Float(v),
                        Err(_) => CellValue::Null,
                    },
                },
            },
        };

        cells.push(cell);
    }

    cells
}

#[async_trait]
impl DbDriver for SqliteDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Relational
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let trimmed = sql.trim();
        let upper = trimmed.to_uppercase();

        let is_select = upper.starts_with("SELECT")
            || upper.starts_with("WITH")
            || upper.starts_with("EXPLAIN")
            || upper.starts_with("PRAGMA")
            || upper.starts_with("VALUES");

        if is_select {
            let rows = sqlx::query(trimmed).fetch_all(&self.pool).await?;
            let elapsed = start.elapsed().as_millis() as u64;

            let columns = if rows.is_empty() {
                Vec::new()
            } else {
                sqlite_columns_to_defs(&rows[0])
            };

            let row_count = rows.len();
            let data: Vec<Vec<_>> = rows.iter().map(|r| sqlite_row_to_cells(r)).collect();

            Ok(QueryResponse {
                columns,
                rows: data,
                row_count,
                execution_time_ms: elapsed,
                affected_rows: None,
            })
        } else {
            let result = sqlx::query(trimmed).execute(&self.pool).await?;
            let elapsed = start.elapsed().as_millis() as u64;
            let affected = result.rows_affected();

            Ok(QueryResponse {
                columns: Vec::new(),
                rows: Vec::new(),
                row_count: 0,
                execution_time_ms: elapsed,
                affected_rows: Some(affected),
            })
        }
    }

    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> {
        // SQLite has a single "main" database
        Ok(vec![ContainerInfo {
            name: "main".to_string(),
            container_type: "database".to_string(),
        }])
    }

    async fn get_items(&self, container: &str) -> Result<Vec<ItemInfo>, AppError> {
        let tables = self.get_tables(container).await?;
        Ok(tables.iter().map(ItemInfo::from).collect())
    }

    async fn get_item_fields(&self, container: &str, item: &str) -> Result<Vec<FieldInfo>, AppError> {
        let columns = self.get_columns(container, item).await?;
        Ok(columns.iter().map(FieldInfo::from).collect())
    }

    async fn get_item_data(&self, container: &str, item: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        SqlDriver::get_table_data(self, container, item, limit, offset).await
    }

    async fn get_item_count(&self, container: &str, item: &str) -> Result<i64, AppError> {
        SqlDriver::get_row_count(self, container, item).await
    }
}

#[async_trait]
impl SqlDriver for SqliteDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        Ok(vec![SchemaInfo {
            name: "main".to_string(),
        }])
    }

    async fn get_tables(&self, _schema: &str) -> Result<Vec<TableInfo>, AppError> {
        let rows = sqlx::query(
            "SELECT name, type FROM sqlite_master \
             WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' \
             ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        let tables = rows
            .iter()
            .map(|row| {
                let name: String = row.get("name");
                let table_type: String = row.get("type");
                TableInfo {
                    name,
                    schema: "main".to_string(),
                    table_type: table_type.to_uppercase(),
                    row_count: None,
                }
            })
            .collect();

        Ok(tables)
    }

    async fn get_columns(&self, _schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        let sql = format!("PRAGMA table_info(\"{}\")", table);
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;

        let columns = rows
            .iter()
            .map(|row| {
                let cid: i32 = row.get("cid");
                let name: String = row.get("name");
                let data_type: String = row.get("type");
                let notnull: bool = row.get("notnull");
                let dflt_value: Option<String> = row.get("dflt_value");
                let pk: bool = row.get("pk");

                ColumnInfo {
                    name,
                    data_type,
                    is_nullable: !notnull,
                    column_default: dflt_value,
                    is_primary_key: pk,
                    ordinal_position: cid + 1,
                }
            })
            .collect();

        Ok(columns)
    }

    async fn get_indexes(&self, _schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        let sql = format!("PRAGMA index_list(\"{}\")", table);
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;

        let mut indexes = Vec::new();

        for row in &rows {
            let name: String = row.get("name");
            let unique: bool = row.get("unique");
            let origin: String = row.get("origin");

            let info_sql = format!("PRAGMA index_info(\"{}\")", name);
            let info_rows = sqlx::query(&info_sql).fetch_all(&self.pool).await?;

            let columns: Vec<String> = info_rows
                .iter()
                .map(|r| r.get::<String, _>("name"))
                .collect();

            indexes.push(IndexInfo {
                name,
                columns,
                is_unique: unique,
                is_primary: origin == "pk",
                index_type: "btree".to_string(),
            });
        }

        Ok(indexes)
    }

    async fn get_foreign_keys(&self, _schema: &str, table: &str) -> Result<Vec<ForeignKeyInfo>, AppError> {
        let sql = format!("PRAGMA foreign_key_list(\"{}\")", table);
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;

        use std::collections::HashMap;
        let mut fk_map: HashMap<i32, ForeignKeyInfo> = HashMap::new();

        for row in &rows {
            let id: i32 = row.get("id");
            let table_ref: String = row.get("table");
            let from: String = row.get("from");
            let to: String = row.get("to");
            let on_update: String = row.get("on_update");
            let on_delete: String = row.get("on_delete");

            let entry = fk_map.entry(id).or_insert_with(|| ForeignKeyInfo {
                name: format!("fk_{}", id),
                columns: Vec::new(),
                referenced_table: table_ref,
                referenced_schema: "main".to_string(),
                referenced_columns: Vec::new(),
                on_update,
                on_delete,
            });

            entry.columns.push(from);
            entry.referenced_columns.push(to);
        }

        let mut foreign_keys: Vec<ForeignKeyInfo> = fk_map.into_values().collect();
        foreign_keys.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(foreign_keys)
    }

    async fn get_table_data(&self, _schema: &str, table: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        let sql = format!(
            "SELECT * FROM \"{}\" LIMIT {} OFFSET {}",
            table, limit, offset
        );
        self.execute_raw(&sql).await
    }

    async fn get_row_count(&self, _schema: &str, table: &str) -> Result<i64, AppError> {
        let sql = format!("SELECT COUNT(*) as count FROM \"{}\"", table);
        let row = sqlx::query(&sql).fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");
        Ok(count)
    }

    async fn update_cell(&self, _schema: &str, table: &str, column: &str, value: &str, pk_columns: Vec<String>, pk_values: Vec<String>) -> Result<(), AppError> {
        if pk_columns.len() != pk_values.len() || pk_columns.is_empty() {
            return Err(AppError::InvalidConfig("Invalid primary key specification".to_string()));
        }

        let where_clauses: Vec<String> = pk_columns
            .iter()
            .zip(pk_values.iter())
            .map(|(col, val)| format!("\"{}\" = '{}'", col, val.replace('\'', "''")))
            .collect();

        let escaped_value = value.replace('\'', "''");
        let sql = format!(
            "UPDATE \"{}\" SET \"{}\" = '{}' WHERE {}",
            table, column, escaped_value, where_clauses.join(" AND ")
        );

        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    async fn insert_row(&self, _schema: &str, table: &str, columns: Vec<String>, values: Vec<String>) -> Result<(), AppError> {
        if columns.len() != values.len() {
            return Err(AppError::InvalidConfig("Columns and values must have the same length".to_string()));
        }

        let cols: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c)).collect();
        let vals: Vec<String> = values.iter().map(|v| format!("'{}'", v.replace('\'', "''"))).collect();

        let sql = format!(
            "INSERT INTO \"{}\" ({}) VALUES ({})",
            table, cols.join(", "), vals.join(", ")
        );

        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_rows(&self, _schema: &str, table: &str, pk_columns: Vec<String>, pk_values_list: Vec<Vec<String>>) -> Result<u64, AppError> {
        if pk_columns.is_empty() {
            return Err(AppError::InvalidConfig("At least one primary key column is required".to_string()));
        }

        let mut total_affected: u64 = 0;

        for pk_values in &pk_values_list {
            if pk_columns.len() != pk_values.len() {
                return Err(AppError::InvalidConfig("Primary key columns and values must have the same length".to_string()));
            }

            let where_clauses: Vec<String> = pk_columns
                .iter()
                .zip(pk_values.iter())
                .map(|(col, val)| format!("\"{}\" = '{}'", col, val.replace('\'', "''")))
                .collect();

            let sql = format!(
                "DELETE FROM \"{}\" WHERE {}",
                table, where_clauses.join(" AND ")
            );

            let result = sqlx::query(&sql).execute(&self.pool).await?;
            total_affected += result.rows_affected();
        }

        Ok(total_affected)
    }
}
