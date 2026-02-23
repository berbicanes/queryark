use async_trait::async_trait;

use crate::db::drivers::mysql::MySqlDriver;
use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::QueryResponse;
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo,
    RoutineInfo, SchemaInfo, TableInfo, TableStats,
};

/// MariaDB driver — thin wrapper around MySqlDriver since MariaDB is MySQL-compatible.
pub struct MariaDbDriver {
    inner: MySqlDriver,
}

impl MariaDbDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let driver = MySqlDriver::connect(config).await?;
        Ok(Self { inner: driver })
    }
}

#[async_trait]
impl DbDriver for MariaDbDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Relational
    }

    fn dialect_hint(&self) -> &'static str {
        self.inner.dialect_hint()
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResponse, AppError> {
        self.inner.execute_raw(sql).await
    }

    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> {
        self.inner.get_containers().await
    }

    async fn get_items(&self, container: &str) -> Result<Vec<ItemInfo>, AppError> {
        self.inner.get_items(container).await
    }

    async fn get_item_fields(&self, container: &str, item: &str) -> Result<Vec<FieldInfo>, AppError> {
        self.inner.get_item_fields(container, item).await
    }

    async fn get_item_data(&self, container: &str, item: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        self.inner.get_item_data(container, item, limit, offset).await
    }

    async fn get_item_count(&self, container: &str, item: &str) -> Result<i64, AppError> {
        self.inner.get_item_count(container, item).await
    }
}

#[async_trait]
impl SqlDriver for MariaDbDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        self.inner.get_schemas().await
    }

    async fn get_tables(&self, schema: &str) -> Result<Vec<TableInfo>, AppError> {
        self.inner.get_tables(schema).await
    }

    async fn get_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        self.inner.get_columns(schema, table).await
    }

    async fn get_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        self.inner.get_indexes(schema, table).await
    }

    async fn get_foreign_keys(&self, schema: &str, table: &str) -> Result<Vec<ForeignKeyInfo>, AppError> {
        self.inner.get_foreign_keys(schema, table).await
    }

    async fn get_table_data(&self, schema: &str, table: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        self.inner.get_table_data(schema, table, limit, offset).await
    }

    async fn get_row_count(&self, schema: &str, table: &str) -> Result<i64, AppError> {
        self.inner.get_row_count(schema, table).await
    }

    async fn update_cell(&self, schema: &str, table: &str, column: &str, value: &str, pk_columns: Vec<String>, pk_values: Vec<String>) -> Result<(), AppError> {
        self.inner.update_cell(schema, table, column, value, pk_columns, pk_values).await
    }

    async fn insert_row(&self, schema: &str, table: &str, columns: Vec<String>, values: Vec<String>) -> Result<(), AppError> {
        self.inner.insert_row(schema, table, columns, values).await
    }

    async fn delete_rows(&self, schema: &str, table: &str, pk_columns: Vec<String>, pk_values_list: Vec<Vec<String>>) -> Result<u64, AppError> {
        self.inner.delete_rows(schema, table, pk_columns, pk_values_list).await
    }

    async fn get_table_stats(&self, schema: &str, table: &str) -> Result<TableStats, AppError> {
        self.inner.get_table_stats(schema, table).await
    }

    async fn get_routines(&self, schema: &str) -> Result<Vec<RoutineInfo>, AppError> {
        self.inner.get_routines(schema).await
    }

    async fn begin_transaction(&self) -> Result<(), AppError> {
        self.inner.begin_transaction().await
    }

    async fn commit_transaction(&self) -> Result<(), AppError> {
        self.inner.commit_transaction().await
    }

    async fn rollback_transaction(&self) -> Result<(), AppError> {
        self.inner.rollback_transaction().await
    }

    async fn in_transaction(&self) -> Result<bool, AppError> {
        self.inner.in_transaction().await
    }
}
