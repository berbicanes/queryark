use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpResult {
    pub tables_dumped: u32,
    pub rows_dumped: u64,
    pub file_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpProgress {
    pub schema: String,
    pub table: String,
    pub tables_done: u32,
    pub tables_total: u32,
    pub rows_dumped: u64,
}
