use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CellValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Timestamp(String),
    Binary(Vec<u8>),
    Json(String),
    LargeText { preview: String, full_length: usize },
    LargeJson { preview: String, full_length: usize },
    LargeBinary { preview_length: usize, full_length: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Vec<CellValue>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
    pub affected_rows: Option<u64>,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default)]
    pub max_rows_limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SortColumn {
    pub column: String,
    pub direction: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterCondition {
    pub column: String,
    pub operator: String,
    pub value: String,
}
