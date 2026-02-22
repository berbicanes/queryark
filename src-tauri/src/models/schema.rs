use serde::{Deserialize, Serialize};

// === Generic models (all database types) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub container_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemInfo {
    pub name: String,
    pub container: String,
    pub item_type: String,
    pub item_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary: bool,
    pub default_value: Option<String>,
    pub ordinal_position: i32,
}

// === SQL-specific models (kept for backward compatibility) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: String,
    pub table_type: String,
    pub row_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub is_primary_key: bool,
    pub ordinal_position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_schema: String,
    pub referenced_columns: Vec<String>,
    pub on_update: String,
    pub on_delete: String,
}

// === Phase 5: Schema browser additions ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStats {
    pub row_count: i64,
    pub size_bytes: Option<i64>,
    pub size_display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineInfo {
    pub name: String,
    pub schema: String,
    pub routine_type: String,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceInfo {
    pub name: String,
    pub schema: String,
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumInfo {
    pub name: String,
    pub schema: String,
    pub variants: Vec<String>,
}

// Conversion helpers
impl From<&SchemaInfo> for ContainerInfo {
    fn from(s: &SchemaInfo) -> Self {
        ContainerInfo {
            name: s.name.clone(),
            container_type: "schema".to_string(),
        }
    }
}

impl From<&TableInfo> for ItemInfo {
    fn from(t: &TableInfo) -> Self {
        ItemInfo {
            name: t.name.clone(),
            container: t.schema.clone(),
            item_type: t.table_type.clone(),
            item_count: t.row_count,
        }
    }
}

impl From<&ColumnInfo> for FieldInfo {
    fn from(c: &ColumnInfo) -> Self {
        FieldInfo {
            name: c.name.clone(),
            data_type: c.data_type.clone(),
            is_nullable: c.is_nullable,
            is_primary: c.is_primary_key,
            default_value: c.column_default.clone(),
            ordinal_position: c.ordinal_position,
        }
    }
}

