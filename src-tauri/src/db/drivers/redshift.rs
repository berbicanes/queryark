use async_trait::async_trait;

use crate::db::drivers::postgres::PostgresDriver;
use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::QueryResponse;
use crate::models::schema::{
    ColumnInfo, ContainerInfo, EnumInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo,
    RoutineInfo, SchemaInfo, SequenceInfo, TableInfo, TableStats,
};

/// Amazon Redshift driver — wrapper around PostgresDriver with Redshift-specific metadata queries.
pub struct RedshiftDriver {
    inner: PostgresDriver,
}

impl RedshiftDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let driver = PostgresDriver::connect(config).await?;
        Ok(Self { inner: driver })
    }
}

#[async_trait]
impl DbDriver for RedshiftDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Analytics
    }

    fn dialect_hint(&self) -> &'static str {
        self.inner.dialect_hint()
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
impl SqlDriver for RedshiftDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        let mut schemas = self.inner.get_schemas().await?;
        // Filter out Redshift system schemas
        schemas.retain(|s| {
            !s.name.starts_with("pg_temp_")
                && s.name != "pg_internal"
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
        // Redshift doesn't support traditional indexes, return empty
        let _ = (schema, table);
        Ok(Vec::new())
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

    async fn get_sequences(&self, schema: &str) -> Result<Vec<SequenceInfo>, AppError> {
        self.inner.get_sequences(schema).await
    }

    async fn get_enums(&self, schema: &str) -> Result<Vec<EnumInfo>, AppError> {
        self.inner.get_enums(schema).await
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
