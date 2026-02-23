use crate::error::AppError;
use crate::models::backup::BackupEntry;
use chrono::Local;
use serde_json::Value;
use std::fs;
use tauri::{AppHandle, Manager};

fn backup_dir(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Database(format!("Failed to get app data dir: {}", e)))?;
    let dir = data_dir.join("backups");
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| AppError::Database(format!("Failed to create backup dir: {}", e)))?;
    }
    Ok(dir)
}

fn store_dir(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::Database(format!("Failed to get app data dir: {}", e)))
}

#[tauri::command]
pub async fn backup_configs(app: AppHandle) -> Result<String, AppError> {
    let store_path = store_dir(&app)?;
    let backup_path = backup_dir(&app)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("backup_{}.json", timestamp);

    let mut combined = serde_json::Map::new();

    // Read connections.json
    let connections_path = store_path.join("connections.json");
    if connections_path.exists() {
        let contents = fs::read_to_string(&connections_path)
            .map_err(|e| AppError::Database(format!("Failed to read connections.json: {}", e)))?;
        if let Ok(val) = serde_json::from_str::<Value>(&contents) {
            combined.insert("connections".to_string(), val);
        }
    }

    // Read settings.json
    let settings_path = store_path.join("settings.json");
    if settings_path.exists() {
        let contents = fs::read_to_string(&settings_path)
            .map_err(|e| AppError::Database(format!("Failed to read settings.json: {}", e)))?;
        if let Ok(val) = serde_json::from_str::<Value>(&contents) {
            combined.insert("settings".to_string(), val);
        }
    }

    let backup_content = serde_json::to_string_pretty(&Value::Object(combined))
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    fs::write(backup_path.join(&filename), backup_content)
        .map_err(|e| AppError::Database(format!("Failed to write backup file: {}", e)))?;

    Ok(filename)
}

#[tauri::command]
pub async fn list_backups(app: AppHandle) -> Result<Vec<BackupEntry>, AppError> {
    let backup_path = backup_dir(&app)?;
    let mut entries = Vec::new();

    let dir_entries = fs::read_dir(&backup_path)
        .map_err(|e| AppError::Database(format!("Failed to read backup dir: {}", e)))?;

    for entry in dir_entries {
        let entry =
            entry.map_err(|e| AppError::Database(format!("Failed to read dir entry: {}", e)))?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let metadata = fs::metadata(&path)
                .map_err(|e| AppError::Database(format!("Failed to read file metadata: {}", e)))?;

            let created_at = metadata
                .modified()
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| {
                            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                                .unwrap_or_default()
                        })
                })
                .unwrap_or_default();

            entries.push(BackupEntry {
                filename,
                created_at,
                size_bytes: metadata.len(),
            });
        }
    }

    // Sort by filename descending (newest first)
    entries.sort_by(|a, b| b.filename.cmp(&a.filename));

    Ok(entries)
}

#[tauri::command]
pub async fn restore_backup(app: AppHandle, filename: String) -> Result<(), AppError> {
    let backup_path = backup_dir(&app)?;
    let store_path = store_dir(&app)?;
    let file_path = backup_path.join(&filename);

    if !file_path.exists() {
        return Err(AppError::InvalidConfig(format!(
            "Backup file not found: {}",
            filename
        )));
    }

    let contents = fs::read_to_string(&file_path)
        .map_err(|e| AppError::Database(format!("Failed to read backup file: {}", e)))?;

    let combined: Value = serde_json::from_str(&contents)
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    // Restore connections.json
    if let Some(connections) = combined.get("connections") {
        let connections_json = serde_json::to_string_pretty(connections)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        fs::write(store_path.join("connections.json"), connections_json)
            .map_err(|e| AppError::Database(format!("Failed to write connections.json: {}", e)))?;
    }

    // Restore settings.json
    if let Some(settings) = combined.get("settings") {
        let settings_json = serde_json::to_string_pretty(settings)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        fs::write(store_path.join("settings.json"), settings_json)
            .map_err(|e| AppError::Database(format!("Failed to write settings.json: {}", e)))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_backup(app: AppHandle, filename: String) -> Result<(), AppError> {
    let backup_path = backup_dir(&app)?;
    let file_path = backup_path.join(&filename);

    if !file_path.exists() {
        return Err(AppError::InvalidConfig(format!(
            "Backup file not found: {}",
            filename
        )));
    }

    fs::remove_file(&file_path)
        .map_err(|e| AppError::Database(format!("Failed to delete backup file: {}", e)))?;

    Ok(())
}
