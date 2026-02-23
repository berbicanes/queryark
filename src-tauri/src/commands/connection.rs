use std::sync::Arc;

use log::{error, info, warn};
use tauri::State;

use crate::db::drivers;
use crate::db::handle::DriverHandle;
use crate::db::keychain;
use crate::db::pool::PoolManager;
use crate::db::tunnel::TunnelManager;
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseType};

/// Resolve secrets from OS keychain if use_keychain is enabled.
/// Resolves: password, SSH password/passphrase, AWS secret key, GCP credentials JSON.
fn resolve_keychain_password(config: &mut ConnectionConfig) {
    if !config.use_keychain {
        return;
    }

    // Main password
    if config.password.as_deref().unwrap_or("").is_empty() {
        if let Some(pw) = keychain::get_secret(&config.id, "password") {
            config.password = Some(pw);
        }
    }

    // SSH password
    if config.ssh_enabled {
        if config.ssh_password.as_deref().unwrap_or("").is_empty() {
            if let Some(pw) = keychain::get_secret(&config.id, "ssh_password") {
                config.ssh_password = Some(pw);
            }
        }
        if config.ssh_passphrase.as_deref().unwrap_or("").is_empty() {
            if let Some(pp) = keychain::get_secret(&config.id, "ssh_passphrase") {
                config.ssh_passphrase = Some(pp);
            }
        }
    }

    // Cloud credentials
    if let Some(ref mut cloud_auth) = config.cloud_auth {
        match cloud_auth {
            crate::models::connection::CloudAuth::AwsCredentials { secret_key, .. } => {
                if secret_key.is_empty() {
                    if let Some(sk) = keychain::get_secret(&config.id, "aws_secret_key") {
                        *secret_key = sk;
                    }
                }
            }
            crate::models::connection::CloudAuth::GcpServiceAccount { credentials_json } => {
                if credentials_json.is_empty() {
                    if let Some(cj) = keychain::get_secret(&config.id, "credentials_json") {
                        *credentials_json = cj;
                    }
                }
            }
        }
    }
}

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
        DatabaseType::Snowflake => {
            let driver = drivers::snowflake::SnowflakeDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
        }
        DatabaseType::BigQuery => {
            let driver = drivers::bigquery::BigQueryDriver::connect(config).await?;
            Ok(DriverHandle::Sql(Arc::new(driver)))
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
    tunnel_manager: State<'_, TunnelManager>,
) -> Result<String, AppError> {
    let id = config.id.clone();
    info!("Connecting to {:?} '{}'", config.db_type, id);

    let mut config = config;
    resolve_keychain_password(&mut config);

    let config = tunnel_manager.ensure_tunnel(&config).await.map_err(|e| {
        error!("SSH tunnel failed for '{}': {}", id, e);
        e
    })?;

    let handle = create_driver_handle(&config).await.map_err(|e| {
        error!("Connection failed for '{}': {}", id, e);
        e
    })?;

    pool_manager.add(id.clone(), handle).await;
    info!("Connected to '{}'", id);
    Ok(id)
}

#[tauri::command]
pub async fn disconnect_db(
    connection_id: String,
    pool_manager: State<'_, PoolManager>,
    tunnel_manager: State<'_, TunnelManager>,
) -> Result<(), AppError> {
    info!("Disconnecting '{}'", connection_id);
    pool_manager.remove(&connection_id).await?;
    tunnel_manager.remove_tunnel(&connection_id).await;
    info!("Disconnected '{}'", connection_id);
    Ok(())
}

#[tauri::command]
pub async fn test_connection(
    config: ConnectionConfig,
    tunnel_manager: State<'_, TunnelManager>,
) -> Result<bool, AppError> {
    info!("Testing connection to {:?}", config.db_type);

    let mut config = config;
    resolve_keychain_password(&mut config);

    let tunneled_config = tunnel_manager.ensure_tunnel(&config).await.map_err(|e| {
        warn!("SSH tunnel failed during test for {:?}: {}", config.db_type, e);
        e
    })?;

    match create_driver_handle(&tunneled_config).await {
        Ok(_) => {
            // Clean up test tunnel
            if config.ssh_enabled {
                tunnel_manager.remove_tunnel(&config.id).await;
            }
            info!("Connection test successful for {:?}", config.db_type);
            Ok(true)
        }
        Err(e) => {
            // Clean up test tunnel
            if config.ssh_enabled {
                tunnel_manager.remove_tunnel(&config.id).await;
            }
            warn!("Connection test failed for {:?}: {}", config.db_type, e);
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn ping_connection(
    connection_id: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<bool, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    match handle.base().health_check().await {
        Ok(_) => Ok(true),
        Err(e) => {
            warn!("Health check failed for '{}': {}", connection_id, e);
            Ok(false)
        }
    }
}
