// BigQuery driver — REST-based via gcp-bigquery-client crate.

use std::time::Instant;

use async_trait::async_trait;
use gcp_bigquery_client::model::field_type::FieldType;
use gcp_bigquery_client::model::query_request::QueryRequest;
use gcp_bigquery_client::Client;

use crate::db::escape::escape_sql_literal;
use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{CloudAuth, ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo, SchemaInfo, TableInfo,
};

/// Format a BigQuery FieldType enum variant to a readable string.
fn field_type_to_string(ft: &FieldType) -> String {
    match ft {
        FieldType::String => "STRING".to_string(),
        FieldType::Bytes => "BYTES".to_string(),
        FieldType::Integer => "INTEGER".to_string(),
        FieldType::Int64 => "INT64".to_string(),
        FieldType::Float => "FLOAT".to_string(),
        FieldType::Float64 => "FLOAT64".to_string(),
        FieldType::Numeric => "NUMERIC".to_string(),
        FieldType::Bignumeric => "BIGNUMERIC".to_string(),
        FieldType::Boolean => "BOOLEAN".to_string(),
        FieldType::Bool => "BOOL".to_string(),
        FieldType::Timestamp => "TIMESTAMP".to_string(),
        FieldType::Date => "DATE".to_string(),
        FieldType::Time => "TIME".to_string(),
        FieldType::Datetime => "DATETIME".to_string(),
        FieldType::Record => "RECORD".to_string(),
        FieldType::Struct => "STRUCT".to_string(),
        FieldType::Geography => "GEOGRAPHY".to_string(),
        FieldType::Json => "JSON".to_string(),
    }
}

pub struct BigQueryDriver {
    client: Client,
    project_id: String,
}

impl BigQueryDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let project_id = config.database.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("BigQuery project ID is required (set as 'Database' field)".to_string())
        })?.to_string();

        let credentials_json = match &config.cloud_auth {
            Some(CloudAuth::GcpServiceAccount { credentials_json }) => credentials_json.clone(),
            _ => {
                return Err(AppError::InvalidConfig(
                    "BigQuery requires GCP service account credentials JSON".to_string(),
                ));
            }
        };

        let sa_key = serde_json::from_str(&credentials_json).map_err(|e| {
            AppError::InvalidConfig(format!("Invalid service account JSON: {}", e))
        })?;

        let client = Client::from_service_account_key(sa_key, false)
            .await
            .map_err(|e| AppError::Database(format!("Failed to create BigQuery client: {}", e)))?;

        // Test connection by listing datasets
        client
            .dataset()
            .list(&project_id, Default::default())
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to BigQuery: {}", e)))?;

        Ok(Self { client, project_id })
    }

    /// Execute a query and parse the result set.
    async fn query_to_response(&self, sql: &str) -> Result<(Vec<ColumnDef>, Vec<Vec<CellValue>>), AppError> {
        let req = QueryRequest::new(sql);

        let mut result = self
            .client
            .job()
            .query(&self.project_id, req)
            .await
            .map_err(|e| AppError::Database(format!("BigQuery query error: {}", e)))?;

        // Build column definitions from schema
        let schema_fields = result
            .query_response()
            .schema
            .as_ref()
            .and_then(|s| s.fields.as_ref());

        let columns: Vec<ColumnDef> = if let Some(fields) = schema_fields {
            fields
                .iter()
                .map(|f| ColumnDef {
                    name: f.name.clone(),
                    data_type: field_type_to_string(&f.r#type),
                })
                .collect()
        } else {
            // Fallback: use column_names() from ResultSet
            result.column_names().into_iter().map(|name| ColumnDef {
                name,
                data_type: "STRING".to_string(),
            }).collect()
        };

        let mut rows = Vec::new();

        // Iterate through all result rows
        while result.next_row() {
            let mut row = Vec::with_capacity(columns.len());
            for (col_idx, col_def) in columns.iter().enumerate() {
                let cell = match result.get_string(col_idx) {
                    Ok(Some(value)) => {
                        // Try to parse based on declared type
                        match col_def.data_type.as_str() {
                            "INTEGER" | "INT64" => {
                                value.parse::<i64>().map(CellValue::Int).unwrap_or(CellValue::Text(value))
                            }
                            "FLOAT" | "FLOAT64" | "NUMERIC" | "BIGNUMERIC" => {
                                value.parse::<f64>().map(CellValue::Float).unwrap_or(CellValue::Text(value))
                            }
                            "BOOLEAN" | "BOOL" => {
                                CellValue::Bool(value.to_lowercase() == "true")
                            }
                            "TIMESTAMP" | "DATETIME" | "DATE" | "TIME" => {
                                CellValue::Timestamp(value)
                            }
                            "RECORD" | "STRUCT" | "JSON" => {
                                CellValue::Json(value)
                            }
                            _ => CellValue::Text(value),
                        }
                    }
                    Ok(None) => CellValue::Null,
                    Err(_) => CellValue::Null,
                };
                row.push(cell);
            }
            rows.push(row);
        }

        Ok((columns, rows))
    }
}

#[async_trait]
impl DbDriver for BigQueryDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Analytics
    }

    fn dialect_hint(&self) -> &'static str {
        "bigquery"
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let trimmed = sql.trim();
        let upper = trimmed.to_uppercase();

        let is_query = upper.starts_with("SELECT")
            || upper.starts_with("WITH")
            || upper.starts_with("SHOW")
            || upper.starts_with("DESCRIBE")
            || upper.starts_with("EXPLAIN");

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
            let req = QueryRequest::new(trimmed);

            self.client
                .job()
                .query(&self.project_id, req)
                .await
                .map_err(|e| AppError::Database(format!("BigQuery execute error: {}", e)))?;

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

    async fn health_check(&self) -> Result<(), AppError> {
        self.execute_raw("SELECT 1").await.map(|_| ())
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
impl SqlDriver for BigQueryDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        let datasets = self
            .client
            .dataset()
            .list(&self.project_id, Default::default())
            .await
            .map_err(|e| AppError::Database(format!("BigQuery list datasets error: {}", e)))?;

        let schemas = datasets
            .datasets
            .iter()
            .map(|ds| SchemaInfo {
                name: ds.dataset_reference.dataset_id.clone(),
            })
            .collect();

        Ok(schemas)
    }

    async fn get_tables(&self, schema: &str) -> Result<Vec<TableInfo>, AppError> {
        let table_list = self
            .client
            .table()
            .list(&self.project_id, schema, Default::default())
            .await
            .map_err(|e| AppError::Database(format!("BigQuery list tables error: {}", e)))?;

        let tables = table_list
            .tables
            .unwrap_or_default()
            .iter()
            .map(|t| {
                let name = t.table_reference.table_id.clone();
                let table_type = t.r#type.clone().unwrap_or_else(|| "TABLE".to_string());
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
        let table_info = self
            .client
            .table()
            .get(&self.project_id, schema, table, None)
            .await
            .map_err(|e| AppError::Database(format!("BigQuery get table error: {}", e)))?;

        let fields = table_info
            .schema
            .fields
            .unwrap_or_default();

        let columns = fields
            .iter()
            .enumerate()
            .map(|(idx, f)| ColumnInfo {
                name: f.name.clone(),
                data_type: field_type_to_string(&f.r#type),
                is_nullable: f.mode.as_deref() != Some("REQUIRED"),
                column_default: None,
                is_primary_key: false,
                ordinal_position: (idx + 1) as i32,
            })
            .collect();

        Ok(columns)
    }

    async fn get_indexes(&self, _schema: &str, _table: &str) -> Result<Vec<IndexInfo>, AppError> {
        Ok(Vec::new())
    }

    async fn get_foreign_keys(&self, _schema: &str, _table: &str) -> Result<Vec<ForeignKeyInfo>, AppError> {
        Ok(Vec::new())
    }

    async fn get_table_data(&self, schema: &str, table: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        let sql = format!(
            "SELECT * FROM `{}`.`{}` LIMIT {} OFFSET {}",
            escape_sql_literal(schema),
            escape_sql_literal(table),
            limit,
            offset
        );
        self.execute_raw(&sql).await
    }

    async fn get_row_count(&self, schema: &str, table: &str) -> Result<i64, AppError> {
        let sql = format!(
            "SELECT COUNT(*) AS cnt FROM `{}`.`{}`",
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
            .map(|(col, val)| format!("`{}` = '{}'", escape_sql_literal(col), escape_sql_literal(val)))
            .collect();

        let sql = format!(
            "UPDATE `{}`.`{}` SET `{}` = '{}' WHERE {}",
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

        let cols: Vec<String> = columns.iter().map(|c| format!("`{}`", escape_sql_literal(c))).collect();
        let vals: Vec<String> = values.iter().map(|v| format!("'{}'", escape_sql_literal(v))).collect();

        let sql = format!(
            "INSERT INTO `{}`.`{}` ({}) VALUES ({})",
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
                .map(|(col, val)| format!("`{}` = '{}'", escape_sql_literal(col), escape_sql_literal(val)))
                .collect();

            let sql = format!(
                "DELETE FROM `{}`.`{}` WHERE {}",
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
