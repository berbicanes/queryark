use std::fs::File;
use std::io::{BufWriter, Write};

use log::{debug, info};
use tauri::State;

use crate::db::pool::PoolManager;
use crate::error::AppError;
use crate::models::export::ImportResult;
use crate::models::query::{CellValue, ColumnDef};
use crate::models::schema::{ColumnInfo, ForeignKeyInfo, IndexInfo};

// === Helpers ===

fn cell_value_to_string(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => String::new(),
        CellValue::Bool(v) => v.to_string(),
        CellValue::Int(v) => v.to_string(),
        CellValue::Float(v) => v.to_string(),
        CellValue::Text(v) => v.clone(),
        CellValue::Timestamp(v) => v.clone(),
        CellValue::Binary(v) => format!("\\x{}", v.iter().map(|b| format!("{:02x}", b)).collect::<String>()),
        CellValue::Json(v) => v.clone(),
        CellValue::LargeText { preview, .. } => preview.clone(),
        CellValue::LargeJson { preview, .. } => preview.clone(),
        CellValue::LargeBinary { full_length, .. } => format!("[{} bytes]", full_length),
    }
}

fn cell_value_to_sql_literal(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => "NULL".to_string(),
        CellValue::Bool(v) => if *v { "TRUE".to_string() } else { "FALSE".to_string() },
        CellValue::Int(v) => v.to_string(),
        CellValue::Float(v) => v.to_string(),
        CellValue::Text(v) => format!("'{}'", v.replace('\'', "''")),
        CellValue::Timestamp(v) => format!("'{}'", v.replace('\'', "''")),
        CellValue::Binary(v) => format!("'\\x{}'", v.iter().map(|b| format!("{:02x}", b)).collect::<String>()),
        CellValue::Json(v) => format!("'{}'", v.replace('\'', "''")),
        CellValue::LargeText { preview, .. } => format!("'{}'", preview.replace('\'', "''")),
        CellValue::LargeJson { preview, .. } => format!("'{}'", preview.replace('\'', "''")),
        CellValue::LargeBinary { full_length, .. } => format!("'[{} bytes]'", full_length),
    }
}

fn cell_value_to_json(cell: &CellValue) -> serde_json::Value {
    match cell {
        CellValue::Null => serde_json::Value::Null,
        CellValue::Bool(v) => serde_json::Value::Bool(*v),
        CellValue::Int(v) => serde_json::json!(*v),
        CellValue::Float(v) => serde_json::json!(*v),
        CellValue::Text(v) => serde_json::Value::String(v.clone()),
        CellValue::Timestamp(v) => serde_json::Value::String(v.clone()),
        CellValue::Binary(v) => serde_json::Value::String(
            format!("\\x{}", v.iter().map(|b| format!("{:02x}", b)).collect::<String>()),
        ),
        CellValue::Json(v) => {
            serde_json::from_str(v).unwrap_or_else(|_| serde_json::Value::String(v.clone()))
        }
        CellValue::LargeText { preview, .. } => serde_json::Value::String(preview.clone()),
        CellValue::LargeJson { preview, .. } => {
            serde_json::from_str(preview).unwrap_or_else(|_| serde_json::Value::String(preview.clone()))
        }
        CellValue::LargeBinary { full_length, .. } => {
            serde_json::Value::String(format!("[{} bytes]", full_length))
        }
    }
}

fn generate_create_table(
    schema: &str,
    table: &str,
    columns: &[ColumnInfo],
    indexes: &[IndexInfo],
    foreign_keys: &[ForeignKeyInfo],
) -> String {
    let mut ddl = format!("CREATE TABLE \"{}\".\"{}\" (\n", schema, table);

    // Columns
    let col_defs: Vec<String> = columns
        .iter()
        .map(|col| {
            let mut def = format!("  \"{}\" {}", col.name, col.data_type);
            if !col.is_nullable {
                def.push_str(" NOT NULL");
            }
            if let Some(ref default) = col.column_default {
                def.push_str(&format!(" DEFAULT {}", default));
            }
            def
        })
        .collect();
    ddl.push_str(&col_defs.join(",\n"));

    // Primary key constraint
    let pk_columns: Vec<&str> = columns
        .iter()
        .filter(|c| c.is_primary_key)
        .map(|c| c.name.as_str())
        .collect();
    if !pk_columns.is_empty() {
        ddl.push_str(",\n  PRIMARY KEY (");
        ddl.push_str(
            &pk_columns
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(", "),
        );
        ddl.push(')');
    }

    // Foreign key constraints
    for fk in foreign_keys {
        ddl.push_str(&format!(
            ",\n  CONSTRAINT \"{}\" FOREIGN KEY ({}) REFERENCES \"{}\".\"{}\" ({})",
            fk.name,
            fk.columns
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(", "),
            fk.referenced_schema,
            fk.referenced_table,
            fk.referenced_columns
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(", "),
        ));
        if fk.on_update != "NO ACTION" {
            ddl.push_str(&format!(" ON UPDATE {}", fk.on_update));
        }
        if fk.on_delete != "NO ACTION" {
            ddl.push_str(&format!(" ON DELETE {}", fk.on_delete));
        }
    }

    ddl.push_str("\n);\n");

    // Indexes (non-primary)
    for idx in indexes {
        if idx.is_primary {
            continue;
        }
        let unique = if idx.is_unique { "UNIQUE " } else { "" };
        ddl.push_str(&format!(
            "\nCREATE {}INDEX \"{}\" ON \"{}\".\"{}\" ({});",
            unique,
            idx.name,
            schema,
            table,
            idx.columns
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }

    if !indexes.iter().any(|i| !i.is_primary) {
        // No trailing newline needed if no extra indexes
    } else {
        ddl.push('\n');
    }

    ddl
}

/// Stream all rows from a table by paginating, writing each page to the writer.
async fn stream_table_csv<W: Write>(
    writer: &mut csv::Writer<W>,
    pool_manager: &PoolManager,
    connection_id: &str,
    schema: &str,
    table: &str,
) -> Result<u64, AppError> {
    let handle = pool_manager.get(connection_id).await?;
    let driver = handle.as_sql()?;

    let page_size: i64 = 5000;
    let mut offset: i64 = 0;
    let mut total: u64 = 0;

    loop {
        let response = driver.get_table_data(schema, table, page_size, offset).await?;
        if response.rows.is_empty() {
            break;
        }

        // Write header on first page
        if offset == 0 {
            let headers: Vec<String> = response.columns.iter().map(|c| c.name.clone()).collect();
            writer
                .write_record(&headers)
                .map_err(|e| AppError::Database(format!("CSV write error: {}", e)))?;
        }

        for row in &response.rows {
            let fields: Vec<String> = row.iter().map(cell_value_to_string).collect();
            writer
                .write_record(&fields)
                .map_err(|e| AppError::Database(format!("CSV write error: {}", e)))?;
        }

        total += response.rows.len() as u64;
        let count = response.rows.len() as i64;
        if count < page_size {
            break;
        }
        offset += page_size;
    }

    writer
        .flush()
        .map_err(|e| AppError::Database(format!("CSV flush error: {}", e)))?;

    Ok(total)
}

async fn stream_table_json<W: Write>(
    writer: &mut BufWriter<W>,
    pool_manager: &PoolManager,
    connection_id: &str,
    schema: &str,
    table: &str,
) -> Result<u64, AppError> {
    let handle = pool_manager.get(connection_id).await?;
    let driver = handle.as_sql()?;

    let page_size: i64 = 5000;
    let mut offset: i64 = 0;
    let mut total: u64 = 0;
    let mut columns: Vec<ColumnDef> = Vec::new();

    writer
        .write_all(b"[\n")
        .map_err(|e| AppError::Database(format!("JSON write error: {}", e)))?;

    loop {
        let response = driver.get_table_data(schema, table, page_size, offset).await?;
        if response.rows.is_empty() {
            break;
        }

        if offset == 0 {
            columns = response.columns.clone();
        }

        for (i, row) in response.rows.iter().enumerate() {
            if total > 0 || i > 0 {
                writer
                    .write_all(b",\n")
                    .map_err(|e| AppError::Database(format!("JSON write error: {}", e)))?;
            }
            let obj: serde_json::Map<String, serde_json::Value> = columns
                .iter()
                .zip(row.iter())
                .map(|(col, cell)| (col.name.clone(), cell_value_to_json(cell)))
                .collect();
            let json_str = serde_json::to_string_pretty(&serde_json::Value::Object(obj))
                .map_err(|e| AppError::Serialization(e.to_string()))?;
            writer
                .write_all(json_str.as_bytes())
                .map_err(|e| AppError::Database(format!("JSON write error: {}", e)))?;
        }

        total += response.rows.len() as u64;
        let count = response.rows.len() as i64;
        if count < page_size {
            break;
        }
        offset += page_size;
    }

    writer
        .write_all(b"\n]\n")
        .map_err(|e| AppError::Database(format!("JSON write error: {}", e)))?;
    writer
        .flush()
        .map_err(|e| AppError::Database(format!("JSON flush error: {}", e)))?;

    Ok(total)
}

async fn stream_table_sql<W: Write>(
    writer: &mut BufWriter<W>,
    pool_manager: &PoolManager,
    connection_id: &str,
    schema: &str,
    table: &str,
) -> Result<u64, AppError> {
    let handle = pool_manager.get(connection_id).await?;
    let driver = handle.as_sql()?;

    let page_size: i64 = 5000;
    let mut offset: i64 = 0;
    let mut total: u64 = 0;
    let mut columns: Vec<ColumnDef> = Vec::new();

    loop {
        let response = driver.get_table_data(schema, table, page_size, offset).await?;
        if response.rows.is_empty() {
            break;
        }

        if offset == 0 {
            columns = response.columns.clone();
        }

        let col_names = columns
            .iter()
            .map(|c| format!("\"{}\"", c.name))
            .collect::<Vec<_>>()
            .join(", ");
        let table_name = format!("\"{}\".\"{}\"", schema, table);

        for row in &response.rows {
            let values = row
                .iter()
                .map(cell_value_to_sql_literal)
                .collect::<Vec<_>>()
                .join(", ");
            let stmt = format!(
                "INSERT INTO {} ({}) VALUES ({});\n",
                table_name, col_names, values
            );
            writer
                .write_all(stmt.as_bytes())
                .map_err(|e| AppError::Database(format!("SQL write error: {}", e)))?;
        }

        total += response.rows.len() as u64;
        let count = response.rows.len() as i64;
        if count < page_size {
            break;
        }
        offset += page_size;
    }

    writer
        .flush()
        .map_err(|e| AppError::Database(format!("SQL flush error: {}", e)))?;

    Ok(total)
}

// === Tauri Commands ===

#[tauri::command]
pub async fn export_to_csv(
    connection_id: Option<String>,
    schema: Option<String>,
    table: Option<String>,
    file_path: String,
    columns: Vec<ColumnDef>,
    rows: Vec<Vec<CellValue>>,
    export_all: bool,
    pool_manager: State<'_, PoolManager>,
) -> Result<u64, AppError> {
    info!("Exporting to CSV: {}", file_path);

    // Full table export — stream directly from DB
    if export_all {
        let cid = connection_id.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("connection_id required for full table export".into())
        })?;
        let s = schema.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("schema required for full table export".into())
        })?;
        let t = table.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("table required for full table export".into())
        })?;

        let file = File::create(&file_path)
            .map_err(|e| AppError::Database(format!("Failed to create file: {}", e)))?;
        let mut writer = csv::Writer::from_writer(file);

        let count = stream_table_csv(&mut writer, &pool_manager, cid, s, t).await?;
        info!("Exported {} rows to CSV", count);
        return Ok(count);
    }

    // Current result set export
    let file = File::create(&file_path)
        .map_err(|e| AppError::Database(format!("Failed to create file: {}", e)))?;
    let mut writer = csv::Writer::from_writer(file);

    // Header
    let headers: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();
    writer
        .write_record(&headers)
        .map_err(|e| AppError::Database(format!("CSV write error: {}", e)))?;

    // Rows
    for row in &rows {
        let fields: Vec<String> = row.iter().map(cell_value_to_string).collect();
        writer
            .write_record(&fields)
            .map_err(|e| AppError::Database(format!("CSV write error: {}", e)))?;
    }

    writer
        .flush()
        .map_err(|e| AppError::Database(format!("CSV flush error: {}", e)))?;

    let count = rows.len() as u64;
    info!("Exported {} rows to CSV", count);
    Ok(count)
}

#[tauri::command]
pub async fn export_to_json(
    connection_id: Option<String>,
    schema: Option<String>,
    table: Option<String>,
    file_path: String,
    columns: Vec<ColumnDef>,
    rows: Vec<Vec<CellValue>>,
    export_all: bool,
    pool_manager: State<'_, PoolManager>,
) -> Result<u64, AppError> {
    info!("Exporting to JSON: {}", file_path);

    if export_all {
        let cid = connection_id.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("connection_id required for full table export".into())
        })?;
        let s = schema.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("schema required for full table export".into())
        })?;
        let t = table.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("table required for full table export".into())
        })?;

        let file = File::create(&file_path)
            .map_err(|e| AppError::Database(format!("Failed to create file: {}", e)))?;
        let mut writer = BufWriter::new(file);

        let count = stream_table_json(&mut writer, &pool_manager, cid, s, t).await?;
        info!("Exported {} rows to JSON", count);
        return Ok(count);
    }

    // Current result set
    let json_array: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let obj: serde_json::Map<String, serde_json::Value> = columns
                .iter()
                .zip(row.iter())
                .map(|(col, cell)| (col.name.clone(), cell_value_to_json(cell)))
                .collect();
            serde_json::Value::Object(obj)
        })
        .collect();

    let json_str = serde_json::to_string_pretty(&json_array)
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    std::fs::write(&file_path, json_str)
        .map_err(|e| AppError::Database(format!("Failed to write file: {}", e)))?;

    let count = rows.len() as u64;
    info!("Exported {} rows to JSON", count);
    Ok(count)
}

#[tauri::command]
pub async fn export_to_sql(
    connection_id: Option<String>,
    schema: Option<String>,
    table: Option<String>,
    file_path: String,
    columns: Vec<ColumnDef>,
    rows: Vec<Vec<CellValue>>,
    export_all: bool,
    pool_manager: State<'_, PoolManager>,
) -> Result<u64, AppError> {
    info!("Exporting to SQL: {}", file_path);

    let schema_name = schema.as_deref().unwrap_or("public");
    let table_name = table.as_deref().unwrap_or("table");

    if export_all {
        let cid = connection_id.as_deref().ok_or_else(|| {
            AppError::InvalidConfig("connection_id required for full table export".into())
        })?;

        let file = File::create(&file_path)
            .map_err(|e| AppError::Database(format!("Failed to create file: {}", e)))?;
        let mut writer = BufWriter::new(file);

        let count =
            stream_table_sql(&mut writer, &pool_manager, cid, schema_name, table_name).await?;
        info!("Exported {} rows to SQL", count);
        return Ok(count);
    }

    // Current result set
    let file = File::create(&file_path)
        .map_err(|e| AppError::Database(format!("Failed to create file: {}", e)))?;
    let mut writer = BufWriter::new(file);

    let col_names = columns
        .iter()
        .map(|c| format!("\"{}\"", c.name))
        .collect::<Vec<_>>()
        .join(", ");
    let qualified_table = format!("\"{}\".\"{}\"", schema_name, table_name);

    for row in &rows {
        let values = row
            .iter()
            .map(cell_value_to_sql_literal)
            .collect::<Vec<_>>()
            .join(", ");
        let stmt = format!(
            "INSERT INTO {} ({}) VALUES ({});\n",
            qualified_table, col_names, values
        );
        writer
            .write_all(stmt.as_bytes())
            .map_err(|e| AppError::Database(format!("SQL write error: {}", e)))?;
    }

    writer
        .flush()
        .map_err(|e| AppError::Database(format!("SQL flush error: {}", e)))?;

    let count = rows.len() as u64;
    info!("Exported {} rows to SQL", count);
    Ok(count)
}

#[tauri::command]
pub async fn export_ddl(
    connection_id: String,
    schema: String,
    table: String,
    file_path: Option<String>,
    pool_manager: State<'_, PoolManager>,
) -> Result<String, AppError> {
    info!("Generating DDL for '{}'.'{}'.'{}'", connection_id, schema, table);

    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;

    let columns = driver.get_columns(&schema, &table).await?;
    let indexes = driver.get_indexes(&schema, &table).await?;
    let foreign_keys = driver.get_foreign_keys(&schema, &table).await?;

    let ddl = generate_create_table(&schema, &table, &columns, &indexes, &foreign_keys);

    if let Some(ref path) = file_path {
        std::fs::write(path, &ddl)
            .map_err(|e| AppError::Database(format!("Failed to write DDL file: {}", e)))?;
        info!("DDL written to {}", path);
    }

    Ok(ddl)
}

#[tauri::command]
pub async fn import_csv(
    connection_id: String,
    schema: String,
    table: String,
    file_path: String,
    has_header: bool,
    delimiter: Option<String>,
    pool_manager: State<'_, PoolManager>,
) -> Result<ImportResult, AppError> {
    info!("Importing CSV from {} into '{}'.'{}'", file_path, schema, table);

    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;

    let delim = delimiter
        .as_ref()
        .and_then(|d| d.bytes().next())
        .unwrap_or(b',');

    let file = File::open(&file_path)
        .map_err(|e| AppError::Database(format!("Failed to open file: {}", e)))?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(has_header)
        .delimiter(delim)
        .from_reader(file);

    // Determine column names: from header or from the table schema
    let col_names: Vec<String> = if has_header {
        rdr.headers()
            .map_err(|e| AppError::Database(format!("CSV header error: {}", e)))?
            .iter()
            .map(|h| h.to_string())
            .collect()
    } else {
        // Use table columns from schema
        let cols = driver.get_columns(&schema, &table).await?;
        cols.iter().map(|c| c.name.clone()).collect()
    };

    let mut rows_imported: u64 = 0;
    let mut rows_failed: u64 = 0;
    let mut errors: Vec<String> = Vec::new();

    for (i, record) in rdr.records().enumerate() {
        match record {
            Ok(rec) => {
                let values: Vec<String> = rec.iter().map(|f| f.to_string()).collect();
                // Only use as many columns as we have values
                let n = col_names.len().min(values.len());
                let cols = col_names[..n].to_vec();
                let vals = values[..n].to_vec();

                match driver.insert_row(&schema, &table, cols, vals).await {
                    Ok(()) => {
                        rows_imported += 1;
                    }
                    Err(e) => {
                        rows_failed += 1;
                        if errors.len() < 10 {
                            errors.push(format!("Row {}: {}", i + 1, e));
                        }
                    }
                }
            }
            Err(e) => {
                rows_failed += 1;
                if errors.len() < 10 {
                    errors.push(format!("Row {}: CSV parse error: {}", i + 1, e));
                }
            }
        }
    }

    debug!(
        "Import complete: {} imported, {} failed",
        rows_imported, rows_failed
    );
    Ok(ImportResult {
        rows_imported,
        rows_failed,
        errors,
    })
}
