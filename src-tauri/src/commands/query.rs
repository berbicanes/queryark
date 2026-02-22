use tauri::State;

use crate::db::pool::PoolManager;
use crate::error::AppError;
use crate::models::query::QueryResponse;

#[tauri::command]
pub async fn execute_query(
    connection_id: String,
    sql: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<QueryResponse, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    handle.base().execute_raw(&sql).await
}
