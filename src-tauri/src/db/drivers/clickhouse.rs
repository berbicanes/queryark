use std::time::Instant;

use async_trait::async_trait;
use clickhouse::Client;

use crate::db::escape::{escape_sql_literal, validate_identifier};
use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo, SchemaInfo, TableInfo,
};

pub struct ClickHouseDriver {
    client: Client,
}

impl ClickHouseDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let url = format!(
            "http://{}:{}",
            config.host_or_default(),
            config.port_or_default()
        );

        let mut client = Client::default().with_url(&url);

        if !config.username_or_default().is_empty() {
            client = client.with_user(config.username_or_default());
        }
        if !config.password_or_default().is_empty() {
            client = client.with_password(config.password_or_default());
        }

        let database = config.database_or_default().to_string();
        if !database.is_empty() {
            client = client.with_database(&database);
        }

        // Test connection
        client
            .query("SELECT 1")
            .fetch_one::<u8>()
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to ClickHouse: {}", e)))?;

        Ok(Self { client })
    }

    async fn query_to_response(&self, sql: &str) -> Result<(Vec<ColumnDef>, Vec<Vec<CellValue>>), AppError> {
        // Use JSONEachRow format for easier parsing
        let query_with_format = format!("{} FORMAT JSONEachRow", sql.trim().trim_end_matches(';'));

        let raw = self
            .client
            .query(&query_with_format)
            .fetch_all::<String>()
            .await;

        match raw {
            Ok(rows_str) => {
                if rows_str.is_empty() {
                    return Ok((Vec::new(), Vec::new()));
                }

                let mut columns = Vec::new();
                let mut rows = Vec::new();

                for (idx, row_str) in rows_str.iter().enumerate() {
                    let obj: serde_json::Value = serde_json::from_str(row_str)
                        .map_err(|e| AppError::Serialization(format!("Failed to parse ClickHouse row: {}", e)))?;

                    if let serde_json::Value::Object(map) = obj {
                        if idx == 0 {
                            columns = map
                                .keys()
                                .map(|k| ColumnDef {
                                    name: k.clone(),
                                    data_type: "String".to_string(),
                                })
                                .collect();
                        }

                        let row: Vec<CellValue> = columns
                            .iter()
                            .map(|col| {
                                match map.get(&col.name) {
                                    Some(serde_json::Value::Null) => CellValue::Null,
                                    Some(serde_json::Value::Bool(b)) => CellValue::Bool(*b),
                                    Some(serde_json::Value::Number(n)) => {
                                        if let Some(i) = n.as_i64() {
                                            CellValue::Int(i)
                                        } else if let Some(f) = n.as_f64() {
                                            CellValue::Float(f)
                                        } else {
                                            CellValue::Text(n.to_string())
                                        }
                                    }
                                    Some(serde_json::Value::String(s)) => CellValue::Text(s.clone()),
                                    Some(v) => CellValue::Json(v.to_string()),
                                    None => CellValue::Null,
                                }
                            })
                            .collect();

                        rows.push(row);
                    }
                }

                Ok((columns, rows))
            }
            Err(e) => Err(AppError::Database(format!("ClickHouse query error: {}", e))),
        }
    }
}

#[async_trait]
impl DbDriver for ClickHouseDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Analytics
    }

    fn dialect_hint(&self) -> &'static str {
        "clickhouse"
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let trimmed = sql.trim();
        let upper = trimmed.to_uppercase();

        let is_select = upper.starts_with("SELECT")
            || upper.starts_with("WITH")
            || upper.starts_with("SHOW")
            || upper.starts_with("DESCRIBE")
            || upper.starts_with("EXPLAIN");

        if is_select {
            let (columns, rows) = self.query_to_response(trimmed).await?;
            let elapsed = start.elapsed().as_millis() as u64;
            let row_count = rows.len();

            Ok(QueryResponse {
                columns,
                rows,
                row_count,
                execution_time_ms: elapsed,
                affected_rows: None,
                truncated: false,
                max_rows_limit: None,
            })
        } else {
            self.client
                .query(trimmed)
                .execute()
                .await
                .map_err(|e| AppError::Database(format!("ClickHouse execute error: {}", e)))?;

            let elapsed = start.elapsed().as_millis() as u64;

            Ok(QueryResponse {
                columns: Vec::new(),
                rows: Vec::new(),
                row_count: 0,
                execution_time_ms: elapsed,
                affected_rows: Some(0),
                truncated: false,
                max_rows_limit: None,
            })
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
impl SqlDriver for ClickHouseDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        let (_, rows) = self.query_to_response("SELECT name FROM system.databases ORDER BY name").await?;

        let schemas = rows
            .iter()
            .filter_map(|row| {
                if let Some(CellValue::Text(name)) = row.first() {
                    if name != "system" && name != "INFORMATION_SCHEMA" && name != "information_schema" {
                        Some(SchemaInfo { name: name.clone() })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(schemas)
    }

    async fn get_tables(&self, schema: &str) -> Result<Vec<TableInfo>, AppError> {
        validate_identifier(schema)?;
        let sql = format!(
            "SELECT name, engine FROM system.tables WHERE database = '{}' ORDER BY name",
            escape_sql_literal(schema)
        );
        let (_, rows) = self.query_to_response(&sql).await?;

        let tables = rows
            .iter()
            .filter_map(|row| {
                let name = match row.get(0) { Some(CellValue::Text(v)) => v.clone(), _ => return None };
                let engine = match row.get(1) { Some(CellValue::Text(v)) => v.clone(), _ => "MergeTree".to_string() };
                Some(TableInfo {
                    name,
                    schema: schema.to_string(),
                    table_type: engine,
                    row_count: None,
                })
            })
            .collect();

        Ok(tables)
    }

    async fn get_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        let sql = format!(
            "SELECT name, type, default_kind, default_expression, position \
             FROM system.columns \
             WHERE database = '{}' AND table = '{}' \
             ORDER BY position",
            escape_sql_literal(schema),
            escape_sql_literal(table)
        );
        let (_, rows) = self.query_to_response(&sql).await?;

        let columns = rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                let name = match row.get(0) { Some(CellValue::Text(v)) => v.clone(), _ => return None };
                let data_type = match row.get(1) { Some(CellValue::Text(v)) => v.clone(), _ => "String".to_string() };
                let default_kind = match row.get(2) { Some(CellValue::Text(v)) => v.clone(), _ => String::new() };
                let default_expr = match row.get(3) { Some(CellValue::Text(v)) => Some(v.clone()), _ => None };
                let is_nullable = data_type.starts_with("Nullable");

                Some(ColumnInfo {
                    name,
                    data_type,
                    is_nullable,
                    column_default: if default_kind.is_empty() { None } else { default_expr },
                    is_primary_key: false,
                    ordinal_position: (idx + 1) as i32,
                })
            })
            .collect();

        Ok(columns)
    }

    async fn get_indexes(&self, _schema: &str, _table: &str) -> Result<Vec<IndexInfo>, AppError> {
        // ClickHouse uses order-by keys rather than traditional indexes
        Ok(Vec::new())
    }

    async fn get_foreign_keys(&self, _schema: &str, _table: &str) -> Result<Vec<ForeignKeyInfo>, AppError> {
        // ClickHouse doesn't support foreign keys
        Ok(Vec::new())
    }

    async fn get_table_data(&self, schema: &str, table: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        let sql = format!(
            "SELECT * FROM `{}`.`{}` LIMIT {} OFFSET {}",
            schema, table, limit, offset
        );
        self.execute_raw(&sql).await
    }

    async fn get_row_count(&self, schema: &str, table: &str) -> Result<i64, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        let sql = format!(
            "SELECT count() as count FROM `{}`.`{}`",
            schema, table
        );
        let (_, rows) = self.query_to_response(&sql).await?;

        if let Some(row) = rows.first() {
            if let Some(CellValue::Int(count)) = row.first() {
                return Ok(*count);
            }
            if let Some(CellValue::Text(count)) = row.first() {
                return Ok(count.parse().unwrap_or(0));
            }
        }
        Ok(0)
    }

    async fn update_cell(&self, schema: &str, table: &str, column: &str, value: &str, pk_columns: Vec<String>, pk_values: Vec<String>) -> Result<(), AppError> {
        if pk_columns.len() != pk_values.len() || pk_columns.is_empty() {
            return Err(AppError::InvalidConfig("Invalid primary key specification".to_string()));
        }
        validate_identifier(schema)?;
        validate_identifier(table)?;
        validate_identifier(column)?;

        let where_clauses: Vec<String> = pk_columns
            .iter()
            .zip(pk_values.iter())
            .map(|(col, val)| {
                validate_identifier(col)?;
                Ok(format!("`{}` = '{}'", col, escape_sql_literal(val)))
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        let sql = format!(
            "ALTER TABLE `{}`.`{}` UPDATE `{}` = '{}' WHERE {}",
            schema, table, column, escape_sql_literal(value), where_clauses.join(" AND ")
        );

        self.execute_raw(&sql).await?;
        Ok(())
    }

    async fn insert_row(&self, schema: &str, table: &str, columns: Vec<String>, values: Vec<String>) -> Result<(), AppError> {
        if columns.len() != values.len() {
            return Err(AppError::InvalidConfig("Columns and values must have the same length".to_string()));
        }
        validate_identifier(schema)?;
        validate_identifier(table)?;

        let cols: Vec<String> = columns.iter().map(|c| {
            validate_identifier(c)?;
            Ok(format!("`{}`", c))
        }).collect::<Result<Vec<_>, AppError>>()?;
        let vals: Vec<String> = values.iter().map(|v| format!("'{}'", escape_sql_literal(v))).collect();

        let sql = format!(
            "INSERT INTO `{}`.`{}` ({}) VALUES ({})",
            schema, table, cols.join(", "), vals.join(", ")
        );

        self.execute_raw(&sql).await?;
        Ok(())
    }

    async fn delete_rows(&self, schema: &str, table: &str, pk_columns: Vec<String>, pk_values_list: Vec<Vec<String>>) -> Result<u64, AppError> {
        if pk_columns.is_empty() {
            return Err(AppError::InvalidConfig("At least one primary key column is required".to_string()));
        }
        validate_identifier(schema)?;
        validate_identifier(table)?;

        let mut total: u64 = 0;
        for pk_values in &pk_values_list {
            if pk_columns.len() != pk_values.len() {
                return Err(AppError::InvalidConfig("Primary key columns and values must have the same length".to_string()));
            }

            let where_clauses: Vec<String> = pk_columns
                .iter()
                .zip(pk_values.iter())
                .map(|(col, val)| {
                    validate_identifier(col)?;
                    Ok(format!("`{}` = '{}'", col, escape_sql_literal(val)))
                })
                .collect::<Result<Vec<_>, AppError>>()?;

            let sql = format!(
                "ALTER TABLE `{}`.`{}` DELETE WHERE {}",
                schema, table, where_clauses.join(" AND ")
            );

            self.execute_raw(&sql).await?;
            total += 1;
        }

        Ok(total)
    }
}
