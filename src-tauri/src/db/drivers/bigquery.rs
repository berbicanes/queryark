// BigQuery driver — REST-based, requires gcp-bigquery-client crate.
// Enabled via the "bigquery" feature flag.

use async_trait::async_trait;

use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::QueryResponse;
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo, SchemaInfo, TableInfo,
};

pub struct BigQueryDriver {
    _config: ConnectionConfig,
}

impl BigQueryDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let _project = config.database.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("BigQuery project ID is required".to_string())
        })?;
        Err(AppError::Database(
            "BigQuery driver requires the 'bigquery' feature flag to be enabled.".to_string(),
        ))
    }
}

#[async_trait]
impl DbDriver for BigQueryDriver {
    fn category(&self) -> DatabaseCategory { DatabaseCategory::Analytics }
    async fn execute_raw(&self, _sql: &str) -> Result<QueryResponse, AppError> { Err(AppError::UnsupportedOperation("BigQuery not configured".to_string())) }
    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> { Ok(Vec::new()) }
    async fn get_items(&self, _container: &str) -> Result<Vec<ItemInfo>, AppError> { Ok(Vec::new()) }
    async fn get_item_fields(&self, _container: &str, _item: &str) -> Result<Vec<FieldInfo>, AppError> { Ok(Vec::new()) }
    async fn get_item_data(&self, _container: &str, _item: &str, _limit: i64, _offset: i64) -> Result<QueryResponse, AppError> { Err(AppError::UnsupportedOperation("BigQuery not configured".to_string())) }
    async fn get_item_count(&self, _container: &str, _item: &str) -> Result<i64, AppError> { Ok(0) }
}

#[async_trait]
impl SqlDriver for BigQueryDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> { Ok(Vec::new()) }
    async fn get_tables(&self, _schema: &str) -> Result<Vec<TableInfo>, AppError> { Ok(Vec::new()) }
    async fn get_columns(&self, _schema: &str, _table: &str) -> Result<Vec<ColumnInfo>, AppError> { Ok(Vec::new()) }
    async fn get_indexes(&self, _schema: &str, _table: &str) -> Result<Vec<IndexInfo>, AppError> { Ok(Vec::new()) }
    async fn get_foreign_keys(&self, _schema: &str, _table: &str) -> Result<Vec<ForeignKeyInfo>, AppError> { Ok(Vec::new()) }
    async fn get_table_data(&self, _schema: &str, _table: &str, _limit: i64, _offset: i64) -> Result<QueryResponse, AppError> { Err(AppError::UnsupportedOperation("BigQuery not configured".to_string())) }
    async fn get_row_count(&self, _schema: &str, _table: &str) -> Result<i64, AppError> { Ok(0) }
    async fn update_cell(&self, _schema: &str, _table: &str, _column: &str, _value: &str, _pk_columns: Vec<String>, _pk_values: Vec<String>) -> Result<(), AppError> { Err(AppError::UnsupportedOperation("BigQuery not configured".to_string())) }
    async fn insert_row(&self, _schema: &str, _table: &str, _columns: Vec<String>, _values: Vec<String>) -> Result<(), AppError> { Err(AppError::UnsupportedOperation("BigQuery not configured".to_string())) }
    async fn delete_rows(&self, _schema: &str, _table: &str, _pk_columns: Vec<String>, _pk_values_list: Vec<Vec<String>>) -> Result<u64, AppError> { Err(AppError::UnsupportedOperation("BigQuery not configured".to_string())) }
}
