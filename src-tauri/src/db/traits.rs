use async_trait::async_trait;

use crate::error::AppError;
use crate::models::connection::DatabaseCategory;
use crate::models::query::QueryResponse;
use crate::models::schema::{
    ColumnInfo, ContainerInfo, EnumInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo,
    RoutineInfo, SchemaInfo, SequenceInfo, TableInfo, TableStats,
};

/// Base trait implemented by all 17 database drivers.
#[async_trait]
pub trait DbDriver: Send + Sync {
    /// Execute a raw query/command string.
    fn category(&self) -> DatabaseCategory;

    async fn execute_raw(&self, query: &str) -> Result<QueryResponse, AppError>;

    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError>;

    async fn get_items(&self, container: &str) -> Result<Vec<ItemInfo>, AppError>;

    async fn get_item_fields(
        &self,
        container: &str,
        item: &str,
    ) -> Result<Vec<FieldInfo>, AppError>;

    async fn get_item_data(
        &self,
        container: &str,
        item: &str,
        limit: i64,
        offset: i64,
    ) -> Result<QueryResponse, AppError>;

    async fn get_item_count(&self, container: &str, item: &str) -> Result<i64, AppError>;

    /// Return the SQL dialect hint for pagination wrapping.
    fn dialect_hint(&self) -> &'static str {
        "generic"
    }

    /// Check if the connection is still alive. Default uses SELECT 1.
    async fn health_check(&self) -> Result<(), AppError> {
        self.execute_raw("SELECT 1").await.map(|_| ())
    }
}

/// Extended trait for SQL-compatible databases (relational + analytics + CQL).
#[async_trait]
pub trait SqlDriver: DbDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError>;

    async fn get_tables(&self, schema: &str) -> Result<Vec<TableInfo>, AppError>;

    async fn get_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError>;

    async fn get_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError>;

    async fn get_foreign_keys(
        &self,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ForeignKeyInfo>, AppError>;

    async fn get_table_data(
        &self,
        schema: &str,
        table: &str,
        limit: i64,
        offset: i64,
    ) -> Result<QueryResponse, AppError>;

    async fn get_row_count(&self, schema: &str, table: &str) -> Result<i64, AppError>;

    async fn update_cell(
        &self,
        schema: &str,
        table: &str,
        column: &str,
        value: &str,
        pk_columns: Vec<String>,
        pk_values: Vec<String>,
    ) -> Result<(), AppError>;

    async fn insert_row(
        &self,
        schema: &str,
        table: &str,
        columns: Vec<String>,
        values: Vec<String>,
    ) -> Result<(), AppError>;

    async fn delete_rows(
        &self,
        schema: &str,
        table: &str,
        pk_columns: Vec<String>,
        pk_values_list: Vec<Vec<String>>,
    ) -> Result<u64, AppError>;

    async fn get_table_stats(&self, schema: &str, table: &str) -> Result<TableStats, AppError> {
        let count = self.get_row_count(schema, table).await?;
        Ok(TableStats {
            row_count: count,
            size_bytes: None,
            size_display: None,
        })
    }

    async fn get_routines(&self, _schema: &str) -> Result<Vec<RoutineInfo>, AppError> {
        Ok(Vec::new())
    }

    async fn get_sequences(&self, _schema: &str) -> Result<Vec<SequenceInfo>, AppError> {
        Ok(Vec::new())
    }

    async fn get_enums(&self, _schema: &str) -> Result<Vec<EnumInfo>, AppError> {
        Ok(Vec::new())
    }

    /// Begin an explicit transaction. Holds a connection from the pool.
    async fn begin_transaction(&self) -> Result<(), AppError> {
        Err(AppError::UnsupportedOperation(
            "Transactions not supported by this driver".to_string(),
        ))
    }

    /// Commit the current transaction.
    async fn commit_transaction(&self) -> Result<(), AppError> {
        Err(AppError::UnsupportedOperation(
            "Transactions not supported by this driver".to_string(),
        ))
    }

    /// Rollback the current transaction.
    async fn rollback_transaction(&self) -> Result<(), AppError> {
        Err(AppError::UnsupportedOperation(
            "Transactions not supported by this driver".to_string(),
        ))
    }

    /// Check if a transaction is currently active.
    #[allow(dead_code)]
    async fn in_transaction(&self) -> Result<bool, AppError> {
        Ok(false)
    }
}

/// Trait for document databases (MongoDB, DynamoDB).
#[async_trait]
pub trait DocumentDriver: DbDriver {
    async fn insert_document(
        &self,
        container: &str,
        collection: &str,
        document: serde_json::Value,
    ) -> Result<String, AppError>;

    async fn update_document(
        &self,
        container: &str,
        collection: &str,
        filter: serde_json::Value,
        update: serde_json::Value,
    ) -> Result<u64, AppError>;

    async fn delete_documents(
        &self,
        container: &str,
        collection: &str,
        filter: serde_json::Value,
    ) -> Result<u64, AppError>;
}

/// Trait for key-value stores (Redis).
#[async_trait]
pub trait KeyValueDriver: DbDriver {
    async fn get_value(&self, key: &str) -> Result<serde_json::Value, AppError>;

    async fn set_value(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<(), AppError>;

    async fn delete_keys(&self, keys: Vec<String>) -> Result<u64, AppError>;

    async fn get_key_type(&self, key: &str) -> Result<String, AppError>;

    async fn scan_keys(&self, pattern: &str, count: i64) -> Result<Vec<String>, AppError>;
}

/// Trait for graph databases (Neo4j).
#[async_trait]
pub trait GraphDriver: DbDriver {
    async fn get_labels(&self) -> Result<Vec<String>, AppError>;

    async fn get_relationship_types(&self) -> Result<Vec<String>, AppError>;

    async fn get_node_properties(&self, label: &str) -> Result<Vec<String>, AppError>;

    async fn get_nodes(
        &self,
        label: &str,
        limit: i64,
        offset: i64,
    ) -> Result<QueryResponse, AppError>;
}
