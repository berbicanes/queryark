use tauri::State;

use crate::db::pool::PoolManager;
use crate::error::AppError;

#[tauri::command]
pub async fn insert_document(
    connection_id: String,
    container: String,
    collection: String,
    document: serde_json::Value,
    pool_manager: State<'_, PoolManager>,
) -> Result<String, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_document()?;
    driver.insert_document(&container, &collection, document).await
}

#[tauri::command]
pub async fn update_document(
    connection_id: String,
    container: String,
    collection: String,
    filter: serde_json::Value,
    update: serde_json::Value,
    pool_manager: State<'_, PoolManager>,
) -> Result<u64, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_document()?;
    driver.update_document(&container, &collection, filter, update).await
}

#[tauri::command]
pub async fn delete_documents(
    connection_id: String,
    container: String,
    collection: String,
    filter: serde_json::Value,
    pool_manager: State<'_, PoolManager>,
) -> Result<u64, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_document()?;
    driver.delete_documents(&container, &collection, filter).await
}
