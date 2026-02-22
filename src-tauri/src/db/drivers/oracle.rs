// Oracle driver — requires Oracle Instant Client on the system.
// Enabled via the "oracle" feature flag.

use async_trait::async_trait;

use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo, SchemaInfo, TableInfo,
};

pub struct OracleDriver {
    // oracle::Connection is not Send, so we wrap it in a blocking approach
    _config: ConnectionConfig,
}

impl OracleDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        // Oracle connections require OCI client libraries
        // This is a placeholder that returns an error if not properly configured
        let _url = config.to_connection_url();
        Err(AppError::Database(
            "Oracle driver requires Oracle Instant Client. Enable the 'oracle' feature and install OCI libraries.".to_string(),
        ))
    }
}

// Stub implementations — these will be replaced with real OCI calls when the feature is enabled
#[async_trait]
impl DbDriver for OracleDriver {
    fn category(&self) -> DatabaseCategory { DatabaseCategory::Relational }
    async fn execute_raw(&self, _sql: &str) -> Result<QueryResponse, AppError> { Err(AppError::UnsupportedOperation("Oracle not configured".to_string())) }
    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> { Ok(Vec::new()) }
    async fn get_items(&self, _container: &str) -> Result<Vec<ItemInfo>, AppError> { Ok(Vec::new()) }
    async fn get_item_fields(&self, _container: &str, _item: &str) -> Result<Vec<FieldInfo>, AppError> { Ok(Vec::new()) }
    async fn get_item_data(&self, _container: &str, _item: &str, _limit: i64, _offset: i64) -> Result<QueryResponse, AppError> { Err(AppError::UnsupportedOperation("Oracle not configured".to_string())) }
    async fn get_item_count(&self, _container: &str, _item: &str) -> Result<i64, AppError> { Ok(0) }
}

#[async_trait]
impl SqlDriver for OracleDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> { Ok(Vec::new()) }
    async fn get_tables(&self, _schema: &str) -> Result<Vec<TableInfo>, AppError> { Ok(Vec::new()) }
    async fn get_columns(&self, _schema: &str, _table: &str) -> Result<Vec<ColumnInfo>, AppError> { Ok(Vec::new()) }
    async fn get_indexes(&self, _schema: &str, _table: &str) -> Result<Vec<IndexInfo>, AppError> { Ok(Vec::new()) }
    async fn get_foreign_keys(&self, _schema: &str, _table: &str) -> Result<Vec<ForeignKeyInfo>, AppError> { Ok(Vec::new()) }
    async fn get_table_data(&self, _schema: &str, _table: &str, _limit: i64, _offset: i64) -> Result<QueryResponse, AppError> { Err(AppError::UnsupportedOperation("Oracle not configured".to_string())) }
    async fn get_row_count(&self, _schema: &str, _table: &str) -> Result<i64, AppError> { Ok(0) }
    async fn update_cell(&self, _schema: &str, _table: &str, _column: &str, _value: &str, _pk_columns: Vec<String>, _pk_values: Vec<String>) -> Result<(), AppError> { Err(AppError::UnsupportedOperation("Oracle not configured".to_string())) }
    async fn insert_row(&self, _schema: &str, _table: &str, _columns: Vec<String>, _values: Vec<String>) -> Result<(), AppError> { Err(AppError::UnsupportedOperation("Oracle not configured".to_string())) }
    async fn delete_rows(&self, _schema: &str, _table: &str, _pk_columns: Vec<String>, _pk_values_list: Vec<Vec<String>>) -> Result<u64, AppError> { Err(AppError::UnsupportedOperation("Oracle not configured".to_string())) }
}
