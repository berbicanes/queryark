use std::time::Duration;

use log::{debug, error, info};
use tauri::State;
use tokio::time::timeout;

use crate::db::escape::escape_sql_literal;
use crate::db::pool::PoolManager;
use crate::error::AppError;
use crate::models::connection::DatabaseCategory;
use crate::models::query::{FilterCondition, QueryResponse, SortColumn};
use crate::models::schema::{
    ColumnInfo, ContainerInfo, EnumInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo,
    RoutineInfo, SchemaInfo, SequenceInfo, TableInfo, TableStats,
};

const DEFAULT_DATA_TIMEOUT: Duration = Duration::from_secs(30);

// === Helpers ===

/// Quote an identifier based on database category.
/// MSSQL uses [name], MySQL uses `name`, everything else uses "name".
pub(crate) fn quote_ident(name: &str, category: &DatabaseCategory) -> String {
    match category {
        DatabaseCategory::Relational => {
            // We can't distinguish MSSQL from PG here by category alone,
            // so we use double quotes which work for PG, SQLite, CockroachDB, Redshift.
            // MSSQL also supports double-quoted identifiers when QUOTED_IDENTIFIER is ON (default).
            format!("\"{}\"", name.replace('"', "\"\""))
        }
        DatabaseCategory::Analytics => {
            // ClickHouse uses double quotes
            format!("\"{}\"", name.replace('"', "\"\""))
        }
        DatabaseCategory::WideColumn => {
            // Cassandra/ScyllaDB use double quotes
            format!("\"{}\"", name.replace('"', "\"\""))
        }
        _ => format!("\"{}\"", name.replace('"', "\"\"")),
    }
}

/// Build a WHERE clause from filter conditions.
fn build_where_clause(filters: &[FilterCondition], category: &DatabaseCategory) -> String {
    if filters.is_empty() {
        return String::new();
    }

    let conditions: Vec<String> = filters
        .iter()
        .filter_map(|f| {
            let col = quote_ident(&f.column, category);
            match f.operator.as_str() {
                "eq" => Some(format!("{} = '{}'", col, escape_sql_literal(&f.value))),
                "neq" => Some(format!("{} != '{}'", col, escape_sql_literal(&f.value))),
                "gt" => Some(format!("{} > '{}'", col, escape_sql_literal(&f.value))),
                "gte" => Some(format!("{} >= '{}'", col, escape_sql_literal(&f.value))),
                "lt" => Some(format!("{} < '{}'", col, escape_sql_literal(&f.value))),
                "lte" => Some(format!("{} <= '{}'", col, escape_sql_literal(&f.value))),
                "contains" => Some(format!(
                    "{} LIKE '%{}%'",
                    col,
                    escape_sql_literal(&f.value).replace('%', "\\%")
                )),
                "starts_with" => Some(format!(
                    "{} LIKE '{}%'",
                    col,
                    escape_sql_literal(&f.value).replace('%', "\\%")
                )),
                "is_null" => Some(format!("{} IS NULL", col)),
                "is_not_null" => Some(format!("{} IS NOT NULL", col)),
                _ => None,
            }
        })
        .collect();

    if conditions.is_empty() {
        return String::new();
    }

    format!(" WHERE {}", conditions.join(" AND "))
}

/// Build an ORDER BY clause from sort columns.
pub(crate) fn build_order_by(sorts: &[SortColumn], category: &DatabaseCategory) -> String {
    if sorts.is_empty() {
        return String::new();
    }

    let clauses: Vec<String> = sorts
        .iter()
        .map(|s| {
            let dir = if s.direction == "DESC" { "DESC" } else { "ASC" };
            format!("{} {}", quote_ident(&s.column, category), dir)
        })
        .collect();

    format!(" ORDER BY {}", clauses.join(", "))
}

/// Build a WHERE clause from pk_columns and pk_values.
fn build_pk_where(pk_columns: &[String], pk_values: &[String], category: &DatabaseCategory) -> String {
    pk_columns
        .iter()
        .zip(pk_values.iter())
        .map(|(col, val)| {
            format!(
                "{} = '{}'",
                quote_ident(col, category),
                escape_sql_literal(val)
            )
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

// === Generic commands (all database types) ===

#[tauri::command]
pub async fn get_database_category(
    connection_id: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<DatabaseCategory, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    Ok(handle.base().category())
}

#[tauri::command]
pub async fn get_containers(
    connection_id: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<ContainerInfo>, AppError> {
    debug!("Loading containers for '{}'", connection_id);
    let handle = pool_manager.get(&connection_id).await?;
    handle.base().get_containers().await
}

#[tauri::command]
pub async fn get_items(
    connection_id: String,
    container: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<ItemInfo>, AppError> {
    debug!("Loading items for '{}'.'{}'", connection_id, container);
    let handle = pool_manager.get(&connection_id).await?;
    handle.base().get_items(&container).await
}

#[tauri::command]
pub async fn get_item_fields(
    connection_id: String,
    container: String,
    item: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<FieldInfo>, AppError> {
    debug!("Loading fields for '{}'.'{}'.'{}'", connection_id, container, item);
    let handle = pool_manager.get(&connection_id).await?;
    handle.base().get_item_fields(&container, &item).await
}

#[tauri::command]
pub async fn get_item_data(
    connection_id: String,
    container: String,
    item: String,
    limit: i64,
    offset: i64,
    pool_manager: State<'_, PoolManager>,
) -> Result<QueryResponse, AppError> {
    debug!("Loading item data for '{}'.'{}'.'{}'", connection_id, container, item);
    let handle = pool_manager.get(&connection_id).await?;

    timeout(
        DEFAULT_DATA_TIMEOUT,
        handle.base().get_item_data(&container, &item, limit, offset),
    )
    .await
    .map_err(|_| {
        error!("get_item_data timed out for '{}'.'{}'.'{}'", connection_id, container, item);
        AppError::QueryTimeout(DEFAULT_DATA_TIMEOUT.as_secs())
    })?
}

#[tauri::command]
pub async fn get_item_count(
    connection_id: String,
    container: String,
    item: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<i64, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    handle.base().get_item_count(&container, &item).await
}

// === SQL-specific commands (gated on SqlDriver) ===

#[tauri::command]
pub async fn get_schemas(
    connection_id: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<SchemaInfo>, AppError> {
    debug!("Loading schemas for '{}'", connection_id);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.get_schemas().await
}

#[tauri::command]
pub async fn get_tables(
    connection_id: String,
    schema: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<TableInfo>, AppError> {
    debug!("Loading tables for '{}'.'{}'", connection_id, schema);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.get_tables(&schema).await
}

#[tauri::command]
pub async fn get_columns(
    connection_id: String,
    schema: String,
    table: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<ColumnInfo>, AppError> {
    debug!("Loading columns for '{}'.'{}'.'{}'", connection_id, schema, table);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.get_columns(&schema, &table).await
}

#[tauri::command]
pub async fn get_indexes(
    connection_id: String,
    schema: String,
    table: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<IndexInfo>, AppError> {
    debug!("Loading indexes for '{}'.'{}'.'{}'", connection_id, schema, table);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.get_indexes(&schema, &table).await
}

#[tauri::command]
pub async fn get_foreign_keys(
    connection_id: String,
    schema: String,
    table: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<ForeignKeyInfo>, AppError> {
    debug!("Loading foreign keys for '{}'.'{}'.'{}'", connection_id, schema, table);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.get_foreign_keys(&schema, &table).await
}

#[tauri::command]
pub async fn get_table_data(
    connection_id: String,
    schema: String,
    table: String,
    limit: i64,
    offset: i64,
    sort_columns: Option<Vec<SortColumn>>,
    filters: Option<Vec<FilterCondition>>,
    pool_manager: State<'_, PoolManager>,
) -> Result<QueryResponse, AppError> {
    debug!("Loading table data for '{}'.'{}'.'{}'", connection_id, schema, table);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;

    let has_sorts = sort_columns.as_ref().map_or(false, |s| !s.is_empty());
    let has_filters = filters.as_ref().map_or(false, |f| !f.is_empty());

    if has_sorts || has_filters {
        let category = handle.base().category();
        let qualified_table = format!(
            "{}.{}",
            quote_ident(&schema, &category),
            quote_ident(&table, &category)
        );

        let where_clause = if has_filters {
            build_where_clause(filters.as_ref().unwrap(), &category)
        } else {
            String::new()
        };

        let order_clause = if has_sorts {
            build_order_by(sort_columns.as_ref().unwrap(), &category)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT * FROM {}{}{} LIMIT {} OFFSET {}",
            qualified_table, where_clause, order_clause, limit, offset
        );

        return timeout(DEFAULT_DATA_TIMEOUT, handle.base().execute_raw(&sql))
            .await
            .map_err(|_| {
                error!("get_table_data timed out for '{}'.'{}'.'{}'", connection_id, schema, table);
                AppError::QueryTimeout(DEFAULT_DATA_TIMEOUT.as_secs())
            })?;
    }

    // Fallback to driver method (no sort/filter)
    timeout(
        DEFAULT_DATA_TIMEOUT,
        driver.get_table_data(&schema, &table, limit, offset),
    )
    .await
    .map_err(|_| {
        error!("get_table_data timed out for '{}'.'{}'.'{}'", connection_id, schema, table);
        AppError::QueryTimeout(DEFAULT_DATA_TIMEOUT.as_secs())
    })?
}

#[tauri::command]
pub async fn get_row_count(
    connection_id: String,
    schema: String,
    table: String,
    filters: Option<Vec<FilterCondition>>,
    pool_manager: State<'_, PoolManager>,
) -> Result<i64, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;

    let has_filters = filters.as_ref().map_or(false, |f| !f.is_empty());

    if has_filters {
        let category = handle.base().category();
        let qualified_table = format!(
            "{}.{}",
            quote_ident(&schema, &category),
            quote_ident(&table, &category)
        );

        let where_clause = build_where_clause(filters.as_ref().unwrap(), &category);
        let sql = format!("SELECT COUNT(*) as count FROM {}{}", qualified_table, where_clause);

        let result = handle.base().execute_raw(&sql).await?;
        if let Some(first_row) = result.rows.first() {
            if let Some(cell) = first_row.first() {
                return match cell {
                    crate::models::query::CellValue::Int(v) => Ok(*v),
                    crate::models::query::CellValue::Text(v) => {
                        v.parse::<i64>().map_err(|_| AppError::Database("Invalid count value".to_string()))
                    }
                    _ => Ok(0),
                };
            }
        }
        return Ok(0);
    }

    driver.get_row_count(&schema, &table).await
}

#[tauri::command]
pub async fn update_cell(
    connection_id: String,
    schema: String,
    table: String,
    column: String,
    value: String,
    pk_columns: Vec<String>,
    pk_values: Vec<String>,
    is_null: Option<bool>,
    pool_manager: State<'_, PoolManager>,
) -> Result<(), AppError> {
    info!("Updating cell in '{}'.'{}'.'{}'.'{}'", connection_id, schema, table, column);
    let handle = pool_manager.get(&connection_id).await?;

    if is_null.unwrap_or(false) {
        let category = handle.base().category();
        let where_clause = build_pk_where(&pk_columns, &pk_values, &category);
        let sql = format!(
            "UPDATE {}.{} SET {} = NULL WHERE {}",
            quote_ident(&schema, &category),
            quote_ident(&table, &category),
            quote_ident(&column, &category),
            where_clause
        );
        handle.base().execute_raw(&sql).await?;
        return Ok(());
    }

    let driver = handle.as_sql()?;
    driver
        .update_cell(&schema, &table, &column, &value, pk_columns, pk_values)
        .await
}

#[tauri::command]
pub async fn insert_row(
    connection_id: String,
    schema: String,
    table: String,
    columns: Vec<String>,
    values: Vec<String>,
    pool_manager: State<'_, PoolManager>,
) -> Result<(), AppError> {
    info!("Inserting row into '{}'.'{}'.'{}'", connection_id, schema, table);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.insert_row(&schema, &table, columns, values).await
}

#[tauri::command]
pub async fn delete_rows(
    connection_id: String,
    schema: String,
    table: String,
    pk_columns: Vec<String>,
    pk_values_list: Vec<Vec<String>>,
    pool_manager: State<'_, PoolManager>,
) -> Result<u64, AppError> {
    info!(
        "Deleting {} row(s) from '{}'.'{}'.'{}'",
        pk_values_list.len(), connection_id, schema, table
    );
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver
        .delete_rows(&schema, &table, pk_columns, pk_values_list)
        .await
}

// === Phase 5: Schema browser commands ===

#[tauri::command]
pub async fn get_table_stats(
    connection_id: String,
    schema: String,
    table: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<TableStats, AppError> {
    debug!("Loading table stats for '{}'.'{}'.'{}'", connection_id, schema, table);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.get_table_stats(&schema, &table).await
}

#[tauri::command]
pub async fn get_routines(
    connection_id: String,
    schema: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<RoutineInfo>, AppError> {
    debug!("Loading routines for '{}'.'{}'", connection_id, schema);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.get_routines(&schema).await
}

#[tauri::command]
pub async fn get_sequences(
    connection_id: String,
    schema: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<SequenceInfo>, AppError> {
    debug!("Loading sequences for '{}'.'{}'", connection_id, schema);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.get_sequences(&schema).await
}

#[tauri::command]
pub async fn get_enums(
    connection_id: String,
    schema: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<Vec<EnumInfo>, AppError> {
    debug!("Loading enums for '{}'.'{}'", connection_id, schema);
    let handle = pool_manager.get(&connection_id).await?;
    let driver = handle.as_sql()?;
    driver.get_enums(&schema).await
}
