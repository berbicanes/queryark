use std::sync::Arc;

use tauri::State;

use crate::db::drivers;
use crate::db::handle::DriverHandle;
use crate::db::pool::PoolManager;
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseType};

/// Factory function: creates the appropriate driver handle based on database type.
async fn create_driver_handle(config: &ConnectionConfig) -> Result<DriverHandle, AppError> {
    match config.db_type {
        DatabaseType::PostgreSQL => {
            let driver = drivers::postgres::PostgresDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        DatabaseType::MySQL => {
            let driver = drivers::mysql::MySqlDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        DatabaseType::MariaDB => {
            let driver = drivers::mariadb::MariaDbDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        DatabaseType::SQLite => {
            let driver = drivers::sqlite::SqliteDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        DatabaseType::MSSQL => {
            let driver = drivers::mssql::MssqlDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        DatabaseType::CockroachDB => {
            let driver = drivers::cockroachdb::CockroachDbDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        DatabaseType::Redshift => {
            let driver = drivers::redshift::RedshiftDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        DatabaseType::ClickHouse => {
            let driver = drivers::clickhouse::ClickHouseDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        #[cfg(feature = "oracle")]
        DatabaseType::Oracle => {
            let driver = drivers::oracle::OracleDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        #[cfg(not(feature = "oracle"))]
        DatabaseType::Oracle => {
            Err(AppError::UnsupportedOperation("Oracle support requires the 'oracle' feature".into()))
        }
        #[cfg(feature = "snowflake")]
        DatabaseType::Snowflake => {
            let driver = drivers::snowflake::SnowflakeDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        #[cfg(not(feature = "snowflake"))]
        DatabaseType::Snowflake => {
            Err(AppError::UnsupportedOperation("Snowflake support requires the 'snowflake' feature".into()))
        }
        #[cfg(feature = "bigquery")]
        DatabaseType::BigQuery => {
            let driver = drivers::bigquery::BigQueryDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        #[cfg(not(feature = "bigquery"))]
        DatabaseType::BigQuery => {
            Err(AppError::UnsupportedOperation("BigQuery support requires the 'bigquery' feature".into()))
        }
        DatabaseType::Cassandra | DatabaseType::ScyllaDB => {
            let driver = drivers::cassandra::CassandraDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        DatabaseType::MongoDB => {
            let driver = drivers::mongodb::MongoDbDriver::connect(config).await?;
            Ok(DriverHandle::Document(Arc::new(driver)))
        }
        DatabaseType::DynamoDB => {
            let driver = drivers::dynamodb::DynamoDbDriver::connect(config).await?;
            Ok(DriverHandle::Document(Arc::new(driver)))
        }
        DatabaseType::Redis => {
            let driver = drivers::redis::RedisDriver::connect(config).await?;
            Ok(DriverHandle::KeyValue(Arc::new(driver)))
        }
        DatabaseType::Neo4j => {
            let driver = drivers::neo4j::Neo4jDriver::connect(config).await?;
            Ok(DriverHandle::Graph(Arc::new(driver)))
        }
    }
}

#[tauri::command]
pub async fn connect_db(
    config: ConnectionConfig,
    pool_manager: State<'_, PoolManager>,
) -> Result<String, AppError> {
    let id = config.id.clone();
    let handle = create_driver_handle(&config).await?;
    pool_manager.add(id.clone(), handle).await;
    Ok(id)
}

#[tauri::command]
pub async fn disconnect_db(
    connection_id: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<(), AppError> {
    pool_manager.remove(&connection_id).await
}

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<bool, AppError> {
    match create_driver_handle(&config).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
