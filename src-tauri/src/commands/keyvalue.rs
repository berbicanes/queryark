use tauri::State;

use crate::db::pool::PoolManager;
use crate::error::AppError;

#[tauri::command]
pub async fn get_value(
    connection_id: String,
    key: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<serde_json::Value, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_keyvalue()?;
    driver.get_value(&key).await
}

#[tauri::command]
pub async fn set_value(
    connection_id: String,
    key: String,
    value: String,
    ttl: Option<u64>,
    pool_manager: State<'_, PoolManager>,
) -> Result<(), AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_keyvalue()?;
    driver.set_value(&key, &value, ttl).await
}

#[tauri::command]
pub async fn delete_keys(
    connection_id: String,
    keys: Vec<String>,
    pool_manager: State<'_, PoolManager>,
) -> Result<u64, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_keyvalue()?;
    driver.delete_keys(keys).await
}

#[tauri::command]
pub async fn get_key_type(
    connection_id: String,
    key: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<String, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_keyvalue()?;
    driver.get_key_type(&key).await
}

#[tauri::command]
pub async fn scan_keys(
    connection_id: String,
    pattern: String,
    count: i64,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<String>, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_keyvalue()?;
    driver.scan_keys(&pattern, count).await
}
