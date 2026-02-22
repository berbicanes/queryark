use crate::db::keychain;
use crate::error::AppError;

#[tauri::command]
pub async fn store_keychain_password(
    connection_id: String,
    password: String,
) -> Result<(), AppError> {
    keychain::store_password(&connection_id, &password)
}

#[tauri::command]
pub async fn get_keychain_password(connection_id: String) -> Result<Option<String>, AppError> {
    Ok(keychain::get_password(&connection_id))
}

#[tauri::command]
pub async fn delete_keychain_password(connection_id: String) -> Result<(), AppError> {
    keychain::delete_password(&connection_id)
}

#[tauri::command]
pub async fn check_keychain_available() -> Result<bool, AppError> {
    Ok(keychain::is_keychain_available())
}
