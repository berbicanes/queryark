// Snowflake driver — REST-based via snowflake-api crate.

use std::sync::Arc;
use std::time::Instant;

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array,
    Int64Array, StringArray, RecordBatch,
};
use arrow::datatypes::DataType as ArrowDataType;
use async_trait::async_trait;
use snowflake_api::SnowflakeApi;

use crate::db::escape::escape_sql_literal;
use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo, SchemaInfo, TableInfo,
};

/// Convert Arrow RecordBatches into ColumnDefs and rows of CellValues.
fn arrow_batches_to_response(batches: &[RecordBatch]) -> (Vec<ColumnDef>, Vec<Vec<CellValue>>) {
    if batches.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let schema = batches[0].schema();
    let columns: Vec<ColumnDef> = schema
        .fields()
        .iter()
        .map(|f| ColumnDef {
            name: f.name().clone(),
            data_type: format!("{}", f.data_type()),
        })
        .collect();

    let mut rows = Vec::new();

    for batch in batches {
        for row_idx in 0..batch.num_rows() {
            let mut row = Vec::with_capacity(batch.num_columns());
            for col_idx in 0..batch.num_columns() {
                let col = batch.column(col_idx);
                if col.is_null(row_idx) {
                    row.push(CellValue::Null);
                    continue;
                }
                let cell = match col.data_type() {
                    ArrowDataType::Boolean => {
                        let arr = col.as_any().downcast_ref::<BooleanArray>().unwrap();
                        CellValue::Bool(arr.value(row_idx))
                    }
                    ArrowDataType::Int8 => {
                        let arr = col.as_any().downcast_ref::<Int8Array>().unwrap();
                        CellValue::Int(arr.value(row_idx) as i64)
                    }
                    ArrowDataType::Int16 => {
                        let arr = col.as_any().downcast_ref::<Int16Array>().unwrap();
                        CellValue::Int(arr.value(row_idx) as i64)
                    }
                    ArrowDataType::Int32 => {
                        let arr = col.as_any().downcast_ref::<Int32Array>().unwrap();
                        CellValue::Int(arr.value(row_idx) as i64)
                    }
                    ArrowDataType::Int64 => {
                        let arr = col.as_any().downcast_ref::<Int64Array>().unwrap();
                        CellValue::Int(arr.value(row_idx))
                    }
                    ArrowDataType::Float32 => {
                        let arr = col.as_any().downcast_ref::<Float32Array>().unwrap();
                        CellValue::Float(arr.value(row_idx) as f64)
                    }
                    ArrowDataType::Float64 => {
                        let arr = col.as_any().downcast_ref::<Float64Array>().unwrap();
                        CellValue::Float(arr.value(row_idx))
                    }
                    ArrowDataType::Utf8 => {
                        let arr = col.as_any().downcast_ref::<StringArray>().unwrap();
                        CellValue::Text(arr.value(row_idx).to_string())
                    }
                    _ => {
                        // Fallback: render as display string
                        let arr_str = arrow::util::display::array_value_to_string(col, row_idx)
                            .unwrap_or_default();
                        CellValue::Text(arr_str)
                    }
                };
                row.push(cell);
            }
            rows.push(row);
        }
    }

    (columns, rows)
}

pub struct SnowflakeDriver {
    client: Arc<SnowflakeApi>,
    database: String,
}

impl SnowflakeDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let account = config.snowflake_account.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("Snowflake account is required".to_string())
        })?;

        let username = config.username_or_default();
        if username.is_empty() {
            return Err(AppError::InvalidConfig("Snowflake username is required".to_string()));
        }

        let password = config.password_or_default();
        if password.is_empty() {
            return Err(AppError::InvalidConfig("Snowflake password is required".to_string()));
        }

        let database = config.database_or_default().to_string();

        let warehouse = config.snowflake_warehouse.as_deref().unwrap_or("COMPUTE_WH");
        let role = config.snowflake_role.as_deref();

        let api = SnowflakeApi::with_password_auth(
            account,
            Some(warehouse),
            Some(&database),
            None, // schema
            username,
            role,
            password,
        )
        .map_err(|e| AppError::Database(format!("Failed to create Snowflake client: {}", e)))?;

        // Test connectivity
        api.exec("SELECT 1")
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to Snowflake: {}", e)))?;

        Ok(Self {
            client: Arc::new(api),
            database,
        })
    }

    /// Execute a query and return Arrow-converted results.
    async fn query_to_response(&self, sql: &str) -> Result<(Vec<ColumnDef>, Vec<Vec<CellValue>>), AppError> {
        let result = self.client
            .exec(sql)
            .await
            .map_err(|e| AppError::Database(format!("Snowflake query error: {}", e)))?;

        match result {
            snowflake_api::QueryResult::Arrow(batches) => {
                Ok(arrow_batches_to_response(&batches))
            }
            snowflake_api::QueryResult::Json(json_result) => {
                // json_result is JsonResult { value: serde_json::Value, schema: Vec<FieldSchema> }
                let mut columns = Vec::new();
                let mut rows = Vec::new();

                if let serde_json::Value::Array(arr) = json_result.value {
                    for (idx, item) in arr.iter().enumerate() {
                        if let serde_json::Value::Object(map) = item {
                            if idx == 0 {
                                columns = map
                                    .keys()
                                    .map(|k| ColumnDef {
                                        name: k.clone(),
                                        data_type: "TEXT".to_string(),
                                    })
                                    .collect();
                            }
                            let row: Vec<CellValue> = columns
                                .iter()
                                .map(|col| match map.get(&col.name) {
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
                                })
                                .collect();
                            rows.push(row);
                        }
                    }
                }
                Ok((columns, rows))
            }
            snowflake_api::QueryResult::Empty => Ok((Vec::new(), Vec::new())),
        }
    }
}

#[async_trait]
impl DbDriver for SnowflakeDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Analytics
    }

    fn dialect_hint(&self) -> &'static str {
        "snowflake"
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let trimmed = sql.trim();
        let upper = trimmed.to_uppercase();

        let is_query = upper.starts_with("SELECT")
            || upper.starts_with("WITH")
            || upper.starts_with("SHOW")
            || upper.starts_with("DESCRIBE")
            || upper.starts_with("EXPLAIN")
            || upper.starts_with("LIST");

        if is_query {
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
            // DML / DDL
            self.client
                .exec(trimmed)
                .await
                .map_err(|e| AppError::Database(format!("Snowflake execute error: {}", e)))?;
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
impl SqlDriver for SnowflakeDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        let sql = format!("SHOW SCHEMAS IN DATABASE \"{}\"", escape_sql_literal(&self.database));
        let (_, rows) = self.query_to_response(&sql).await?;

        // SHOW SCHEMAS returns columns: created_on, name, is_default, ...
        // The "name" column is typically at index 1
        let schemas = rows
            .iter()
            .filter_map(|row| {
                let name = match row.get(1) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => return None,
                };
                if name == "INFORMATION_SCHEMA" {
                    None
                } else {
                    Some(SchemaInfo { name })
                }
            })
            .collect();

        Ok(schemas)
    }

    async fn get_tables(&self, schema: &str) -> Result<Vec<TableInfo>, AppError> {
        let sql = format!(
            "SHOW TABLES IN SCHEMA \"{}\".\"{}\"",
            escape_sql_literal(&self.database),
            escape_sql_literal(schema)
        );
        let (_, rows) = self.query_to_response(&sql).await?;

        // SHOW TABLES returns: created_on, name, database_name, schema_name, kind, ...
        let tables = rows
            .iter()
            .filter_map(|row| {
                let name = match row.get(1) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => return None,
                };
                let kind = match row.get(4) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => "TABLE".to_string(),
                };
                Some(TableInfo {
                    name,
                    schema: schema.to_string(),
                    table_type: kind,
                    row_count: None,
                })
            })
            .collect();

        Ok(tables)
    }

    async fn get_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        let sql = format!(
            "SHOW COLUMNS IN TABLE \"{}\".\"{}\".\"{}\"\n",
            escape_sql_literal(&self.database),
            escape_sql_literal(schema),
            escape_sql_literal(table)
        );
        let (columns_def, rows) = self.query_to_response(&sql).await?;

        // Find column indices by name for robustness
        let col_name_idx = columns_def.iter().position(|c| c.name.to_lowercase() == "column_name").unwrap_or(2);
        let col_type_idx = columns_def.iter().position(|c| c.name.to_lowercase() == "data_type").unwrap_or(3);
        let col_null_idx = columns_def.iter().position(|c| c.name.to_lowercase() == "is_nullable").unwrap_or(5);
        let col_default_idx = columns_def.iter().position(|c| c.name.to_lowercase() == "default").unwrap_or(4);

        let columns = rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                let name = match row.get(col_name_idx) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => return None,
                };

                // data_type from SHOW COLUMNS is JSON like {"type":"TEXT","length":16777216,...}
                let data_type = match row.get(col_type_idx) {
                    Some(CellValue::Text(v)) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(v) {
                            parsed.get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or(v)
                                .to_string()
                        } else {
                            v.clone()
                        }
                    }
                    _ => "TEXT".to_string(),
                };

                let is_nullable = match row.get(col_null_idx) {
                    Some(CellValue::Text(v)) => v.to_uppercase() == "Y" || v.to_uppercase() == "YES" || v.to_uppercase() == "TRUE",
                    Some(CellValue::Bool(b)) => *b,
                    _ => true,
                };

                let column_default = match row.get(col_default_idx) {
                    Some(CellValue::Text(v)) if !v.is_empty() => Some(v.clone()),
                    _ => None,
                };

                Some(ColumnInfo {
                    name,
                    data_type,
                    is_nullable,
                    column_default,
                    is_primary_key: false,
                    ordinal_position: (idx + 1) as i32,
                })
            })
            .collect();

        Ok(columns)
    }

    async fn get_indexes(&self, _schema: &str, _table: &str) -> Result<Vec<IndexInfo>, AppError> {
        Ok(Vec::new())
    }

    async fn get_foreign_keys(&self, schema: &str, table: &str) -> Result<Vec<ForeignKeyInfo>, AppError> {
        let sql = format!(
            "SELECT tc.CONSTRAINT_NAME, kcu.COLUMN_NAME, \
             rc.UNIQUE_CONSTRAINT_SCHEMA, rc2_kcu.TABLE_NAME AS REFERENCED_TABLE, \
             rc2_kcu.COLUMN_NAME AS REFERENCED_COLUMN \
             FROM \"{db}\".INFORMATION_SCHEMA.TABLE_CONSTRAINTS tc \
             JOIN \"{db}\".INFORMATION_SCHEMA.KEY_COLUMN_USAGE kcu \
               ON tc.CONSTRAINT_NAME = kcu.CONSTRAINT_NAME AND tc.TABLE_SCHEMA = kcu.TABLE_SCHEMA \
             JOIN \"{db}\".INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS rc \
               ON tc.CONSTRAINT_NAME = rc.CONSTRAINT_NAME AND tc.TABLE_SCHEMA = rc.CONSTRAINT_SCHEMA \
             LEFT JOIN \"{db}\".INFORMATION_SCHEMA.KEY_COLUMN_USAGE rc2_kcu \
               ON rc.UNIQUE_CONSTRAINT_NAME = rc2_kcu.CONSTRAINT_NAME AND rc.UNIQUE_CONSTRAINT_SCHEMA = rc2_kcu.TABLE_SCHEMA \
             WHERE tc.TABLE_SCHEMA = '{schema}' AND tc.TABLE_NAME = '{table}' AND tc.CONSTRAINT_TYPE = 'FOREIGN KEY' \
             ORDER BY tc.CONSTRAINT_NAME, kcu.ORDINAL_POSITION",
            db = escape_sql_literal(&self.database),
            schema = escape_sql_literal(schema),
            table = escape_sql_literal(table),
        );

        match self.query_to_response(&sql).await {
            Ok((_, rows)) => {
                let mut fk_map: std::collections::HashMap<String, ForeignKeyInfo> = std::collections::HashMap::new();
                for row in &rows {
                    let name = match row.get(0) { Some(CellValue::Text(v)) => v.clone(), _ => continue };
                    let column = match row.get(1) { Some(CellValue::Text(v)) => v.clone(), _ => continue };
                    let ref_schema = match row.get(2) { Some(CellValue::Text(v)) => v.clone(), _ => String::new() };
                    let ref_table = match row.get(3) { Some(CellValue::Text(v)) => v.clone(), _ => String::new() };
                    let ref_col = match row.get(4) { Some(CellValue::Text(v)) => v.clone(), _ => String::new() };

                    let entry = fk_map.entry(name.clone()).or_insert_with(|| ForeignKeyInfo {
                        name,
                        columns: Vec::new(),
                        referenced_table: ref_table,
                        referenced_schema: ref_schema,
                        referenced_columns: Vec::new(),
                        on_update: "NO ACTION".to_string(),
                        on_delete: "NO ACTION".to_string(),
                    });
                    entry.columns.push(column);
                    entry.referenced_columns.push(ref_col);
                }
                Ok(fk_map.into_values().collect())
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn get_table_data(&self, schema: &str, table: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        let sql = format!(
            "SELECT * FROM \"{}\".\"{}\" LIMIT {} OFFSET {}",
            escape_sql_literal(schema),
            escape_sql_literal(table),
            limit,
            offset
        );
        self.execute_raw(&sql).await
    }

    async fn get_row_count(&self, schema: &str, table: &str) -> Result<i64, AppError> {
        let sql = format!(
            "SELECT COUNT(*) AS cnt FROM \"{}\".\"{}\"",
            escape_sql_literal(schema),
            escape_sql_literal(table)
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

    async fn update_cell(
        &self,
        schema: &str,
        table: &str,
        column: &str,
        value: &str,
        pk_columns: Vec<String>,
        pk_values: Vec<String>,
    ) -> Result<(), AppError> {
        if pk_columns.len() != pk_values.len() || pk_columns.is_empty() {
            return Err(AppError::InvalidConfig("Invalid primary key specification".to_string()));
        }

        let where_clauses: Vec<String> = pk_columns
            .iter()
            .zip(pk_values.iter())
            .map(|(col, val)| format!("\"{}\" = '{}'", escape_sql_literal(col), escape_sql_literal(val)))
            .collect();

        let sql = format!(
            "UPDATE \"{}\".\"{}\" SET \"{}\" = '{}' WHERE {}",
            escape_sql_literal(schema),
            escape_sql_literal(table),
            escape_sql_literal(column),
            escape_sql_literal(value),
            where_clauses.join(" AND ")
        );

        self.execute_raw(&sql).await?;
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
            return Err(AppError::InvalidConfig("Columns and values must have the same length".to_string()));
        }

        let cols: Vec<String> = columns.iter().map(|c| format!("\"{}\"", escape_sql_literal(c))).collect();
        let vals: Vec<String> = values.iter().map(|v| format!("'{}'", escape_sql_literal(v))).collect();

        let sql = format!(
            "INSERT INTO \"{}\".\"{}\" ({}) VALUES ({})",
            escape_sql_literal(schema),
            escape_sql_literal(table),
            cols.join(", "),
            vals.join(", ")
        );

        self.execute_raw(&sql).await?;
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
            return Err(AppError::InvalidConfig("At least one primary key column is required".to_string()));
        }

        let mut total: u64 = 0;
        for pk_values in &pk_values_list {
            if pk_columns.len() != pk_values.len() {
                return Err(AppError::InvalidConfig(
                    "Primary key columns and values must have the same length".to_string(),
                ));
            }

            let where_clauses: Vec<String> = pk_columns
                .iter()
                .zip(pk_values.iter())
                .map(|(col, val)| format!("\"{}\" = '{}'", escape_sql_literal(col), escape_sql_literal(val)))
                .collect();

            let sql = format!(
                "DELETE FROM \"{}\".\"{}\" WHERE {}",
                escape_sql_literal(schema),
                escape_sql_literal(table),
                where_clauses.join(" AND ")
            );

            self.execute_raw(&sql).await?;
            total += 1;
        }

        Ok(total)
    }
}
