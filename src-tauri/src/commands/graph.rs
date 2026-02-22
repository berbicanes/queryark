use tauri::State;

use crate::db::pool::PoolManager;
use crate::error::AppError;
use crate::models::query::QueryResponse;

#[tauri::command]
pub async fn get_labels(
    connection_id: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<String>, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_graph()?;
    driver.get_labels().await
}

#[tauri::command]
pub async fn get_relationship_types(
    connection_id: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<String>, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_graph()?;
    driver.get_relationship_types().await
}

#[tauri::command]
pub async fn get_node_properties(
    connection_id: String,
    label: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<String>, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_graph()?;
    driver.get_node_properties(&label).await
}

#[tauri::command]
pub async fn get_nodes(
    connection_id: String,
    label: String,
    limit: i64,
    offset: i64,
    pool_manager: State<'_, PoolManager>,
) -> Result<QueryResponse, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_graph()?;
    driver.get_nodes(&label, limit, offset).await
}
