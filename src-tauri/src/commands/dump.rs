use std::fs::{self, File};
use std::io::{BufWriter, Write};

use chrono::Local;
use log::info;
use tauri::{AppHandle, Emitter, State};

use crate::commands::export::{cell_value_to_sql_literal, generate_create_table};
use crate::db::pool::PoolManager;
use crate::error::AppError;
use crate::models::dump::{DumpProgress, DumpResult};
use crate::models::query::ColumnDef;

#[tauri::command]
pub async fn dump_database(
    app: AppHandle,
    connection_id: String,
    file_path: String,
    schemas: Vec<String>,
    include_data: bool,
    pool_manager: State<'_, PoolManager>,
) -> Result<DumpResult, AppError> {
    info!(
        "Starting database dump for connection '{}' to '{}'",
        connection_id, file_path
    );

    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;

    // Count total tables across all schemas
    let mut schema_tables: Vec<(String, Vec<String>)> = Vec::new();
    let mut tables_total: u32 = 0;

    for schema in &schemas {
        let tables = driver.get_tables(schema).await?;
        let table_names: Vec<String> = tables.into_iter().map(|t| t.name).collect();
        tables_total += table_names.len() as u32;
        schema_tables.push((schema.clone(), table_names));
    }

    let file = File::create(&file_path)
        .map_err(|e| AppError::Database(format!("Failed to create dump file: {}", e)))?;
    let mut writer = BufWriter::new(file);

    // Write header
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    writeln!(writer, "-- QueryArk Database Dump").ok();
    writeln!(writer, "-- Generated: {}", timestamp).ok();
    writeln!(
        writer,
        "-- Schemas: {}",
        schemas.join(", ")
    )
    .ok();
    writeln!(
        writer,
        "-- Mode: {}",
        if include_data {
            "Schema + Data"
        } else {
            "Schema Only"
        }
    )
    .ok();
    writeln!(writer, "--").ok();
    writeln!(writer).ok();

    let mut tables_done: u32 = 0;
    let mut total_rows: u64 = 0;

    // First pass: DDL for all schemas and tables
    for (schema, table_names) in &schema_tables {
        // CREATE SCHEMA
        writeln!(
            writer,
            "CREATE SCHEMA IF NOT EXISTS \"{}\";",
            schema
        )
        .map_err(|e| AppError::Database(format!("Write error: {}", e)))?;
        writeln!(writer)
            .map_err(|e| AppError::Database(format!("Write error: {}", e)))?;

        for table_name in table_names {
            let columns = driver.get_columns(schema, table_name).await?;
            let indexes = driver.get_indexes(schema, table_name).await?;
            let foreign_keys = driver.get_foreign_keys(schema, table_name).await?;

            let ddl = generate_create_table(schema, table_name, &columns, &indexes, &foreign_keys);
            writer
                .write_all(ddl.as_bytes())
                .map_err(|e| AppError::Database(format!("Write error: {}", e)))?;
            writeln!(writer)
                .map_err(|e| AppError::Database(format!("Write error: {}", e)))?;

            if !include_data {
                tables_done += 1;
                let _ = app.emit(
                    "dump-progress",
                    DumpProgress {
                        schema: schema.clone(),
                        table: table_name.clone(),
                        tables_done,
                        tables_total,
                        rows_dumped: 0,
                    },
                );
            }
        }
    }

    // Second pass: Data (if requested)
    if include_data {
        tables_done = 0;

        for (schema, table_names) in &schema_tables {
            for table_name in table_names {
                let rows_for_table = stream_insert_statements(
                    &mut writer,
                    &pool_manager,
                    &connection_id,
                    schema,
                    table_name,
                )
                .await?;

                total_rows += rows_for_table;
                tables_done += 1;

                let _ = app.emit(
                    "dump-progress",
                    DumpProgress {
                        schema: schema.clone(),
                        table: table_name.clone(),
                        tables_done,
                        tables_total,
                        rows_dumped: total_rows,
                    },
                );
            }
        }
    }

    writer
        .flush()
        .map_err(|e| AppError::Database(format!("Flush error: {}", e)))?;

    let file_size_bytes = fs::metadata(&file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    info!(
        "Dump complete: {} tables, {} rows, {} bytes",
        tables_total, total_rows, file_size_bytes
    );

    Ok(DumpResult {
        tables_dumped: tables_total,
        rows_dumped: total_rows,
        file_size_bytes,
    })
}

/// Stream INSERT statements for a single table using paginated reads.
async fn stream_insert_statements<W: Write>(
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
        let table_ref = format!("\"{}\".\"{}\"", schema, table);

        for row in &response.rows {
            let values = row
                .iter()
                .map(cell_value_to_sql_literal)
                .collect::<Vec<_>>()
                .join(", ");
            let stmt = format!(
                "INSERT INTO {} ({}) VALUES ({});\n",
                table_ref, col_names, values
            );
            writer
                .write_all(stmt.as_bytes())
                .map_err(|e| AppError::Database(format!("Write error: {}", e)))?;
        }

        total += response.rows.len() as u64;
        let count = response.rows.len() as i64;
        if count < page_size {
            break;
        }
        offset += page_size;
    }

    if total > 0 {
        writeln!(writer)
            .map_err(|e| AppError::Database(format!("Write error: {}", e)))?;
    }

    Ok(total)
}
