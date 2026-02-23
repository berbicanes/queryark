use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BackupEntry {
    pub filename: String,
    pub created_at: String,
    pub size_bytes: u64,
}
