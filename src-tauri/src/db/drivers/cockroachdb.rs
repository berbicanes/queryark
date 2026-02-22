use async_trait::async_trait;

use crate::db::drivers::postgres::PostgresDriver;
use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::QueryResponse;
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo, SchemaInfo, TableInfo,
};

/// CockroachDB driver — wrapper around PostgresDriver, filters out crdb_internal schemas.
pub struct CockroachDbDriver {
    inner: PostgresDriver,
}

impl CockroachDbDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let driver = PostgresDriver::connect(config).await?;
        Ok(Self { inner: driver })
    }
}

#[async_trait]
impl DbDriver for CockroachDbDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Relational
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResponse, AppError> {
        self.inner.execute_raw(sql).await
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
impl SqlDriver for CockroachDbDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        let mut schemas = self.inner.get_schemas().await?;
        // Filter out CockroachDB internal schemas
        schemas.retain(|s| {
            !s.name.starts_with("crdb_internal")
                && s.name != "pg_extension"
        });
        Ok(schemas)
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
}
