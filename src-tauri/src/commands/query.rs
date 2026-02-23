use std::time::Duration;

use log::{debug, error, info, warn};
use tauri::State;
use tokio::time::timeout;

use crate::commands::schema::{build_order_by, quote_ident};
use crate::db::cancel::CancellationRegistry;
use crate::db::pool::PoolManager;
use crate::error::AppError;
use crate::models::query::{CellValue, QueryResponse, SortColumn};

const DEFAULT_QUERY_TIMEOUT: Duration = Duration::from_secs(30);

const DEFAULT_MAX_ROWS: usize = 10_000;

// === Helpers ===

/// Check if a SQL query is paginatable (SELECT, WITH, TABLE, VALUES).
fn is_paginatable_query(sql: &str) -> bool {
    let trimmed = sql.trim();
    let upper = trimmed.to_uppercase();
    upper.starts_with("SELECT")
        || upper.starts_with("WITH")
        || upper.starts_with("TABLE")
        || upper.starts_with("VALUES")
}

/// Wrap a user SQL query with LIMIT/OFFSET for pagination.
/// MSSQL uses OFFSET...FETCH NEXT syntax; others use LIMIT...OFFSET.
fn wrap_paginated(sql: &str, limit: i64, offset: i64, dialect: &str) -> String {
    let trimmed = sql.trim().trim_end_matches(';');
    if dialect == "mssql" {
        // MSSQL requires ORDER BY for OFFSET...FETCH. If none present, add ORDER BY (SELECT NULL).
        let upper = trimmed.to_uppercase();
        if upper.contains("ORDER BY") {
            format!(
                "{} OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
                trimmed, offset, limit
            )
        } else {
            format!(
                "{} ORDER BY (SELECT NULL) OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
                trimmed, offset, limit
            )
        }
    } else {
        format!(
            "SELECT * FROM ({}) AS _df_page LIMIT {} OFFSET {}",
            trimmed, limit, offset
        )
    }
}

/// Truncate large cell values in a response to save bandwidth.
fn truncate_large_values(response: &mut QueryResponse, max_cell_size: usize) {
    for row in response.rows.iter_mut() {
        for cell in row.iter_mut() {
            match cell {
                CellValue::Text(ref v) if v.len() > max_cell_size => {
                    let preview = v.chars().take(max_cell_size).collect::<String>();
                    let full_length = v.len();
                    *cell = CellValue::LargeText {
                        preview,
                        full_length,
                    };
                }
                CellValue::Json(ref v) if v.len() > max_cell_size => {
                    let preview = v.chars().take(max_cell_size).collect::<String>();
                    let full_length = v.len();
                    *cell = CellValue::LargeJson {
                        preview,
                        full_length,
                    };
                }
                CellValue::Binary(ref v) if v.len() > max_cell_size => {
                    let full_length = v.len();
                    let preview_length = max_cell_size.min(full_length);
                    *cell = CellValue::LargeBinary {
                        preview_length,
                        full_length,
                    };
                }
                _ => {}
            }
        }
    }
}

// === Commands ===

#[tauri::command]
pub async fn execute_query(
    connection_id: String,
    sql: String,
    timeout_secs: Option<u64>,
    query_id: Option<String>,
    max_rows: Option<usize>,
    max_cell_size: Option<usize>,
    pool_manager: State<'_, PoolManager>,
    cancel_registry: State<'_, CancellationRegistry>,
) -> Result<QueryResponse, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let duration = timeout_secs
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_QUERY_TIMEOUT);

    let log_sql = if sql.len() > 200 {
        format!("{}...", &sql[..200])
    } else {
        sql.clone()
    };
    debug!("Executing query on '{}': {}", connection_id, log_sql);

    let query_future = timeout(duration, handle.base().execute_raw(&sql));

    let result = if let Some(ref qid) = query_id {
        let cancel_rx = cancel_registry.register(qid.clone());

        let outcome = tokio::select! {
            res = query_future => {
                cancel_registry.remove(qid);
                res.map_err(|_| {
                    error!("Query timed out after {}s on '{}'", duration.as_secs(), connection_id);
                    AppError::QueryTimeout(duration.as_secs())
                })?
            }
            _ = cancel_rx => {
                warn!("Query '{}' cancelled on '{}'", qid, connection_id);
                return Err(AppError::QueryCancelled);
            }
        };
        outcome
    } else {
        query_future
            .await
            .map_err(|_| {
                error!("Query timed out after {}s on '{}'", duration.as_secs(), connection_id);
                AppError::QueryTimeout(duration.as_secs())
            })?
    };

    match result {
        Ok(mut response) => {
            let limit = max_rows.unwrap_or(DEFAULT_MAX_ROWS);
            if response.rows.len() > limit {
                info!(
                    "Query on '{}' returned {} rows, truncating to {} (limit)",
                    connection_id, response.rows.len(), limit
                );
                response.rows.truncate(limit);
                response.truncated = true;
                response.max_rows_limit = Some(limit);
                response.row_count = limit;
            }
            if let Some(mcs) = max_cell_size {
                truncate_large_values(&mut response, mcs);
            }
            info!(
                "Query on '{}' completed in {}ms ({} rows)",
                connection_id, response.execution_time_ms, response.row_count
            );
            Ok(response)
        }
        Err(e) => {
            error!("Query failed on '{}': {}", connection_id, e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn execute_query_page(
    connection_id: String,
    sql: String,
    limit: i64,
    offset: i64,
    timeout_secs: Option<u64>,
    query_id: Option<String>,
    max_cell_size: Option<usize>,
    sort_columns: Option<Vec<SortColumn>>,
    pool_manager: State<'_, PoolManager>,
    cancel_registry: State<'_, CancellationRegistry>,
) -> Result<QueryResponse, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let duration = timeout_secs
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_QUERY_TIMEOUT);

    let dialect = handle.base().dialect_hint();
    let category = handle.base().category();

    // Build the paginated SQL
    let paginated_sql = if is_paginatable_query(&sql) {
        // If sort columns are provided, append ORDER BY before wrapping
        if let Some(ref sorts) = sort_columns {
            if !sorts.is_empty() {
                let order_clause = build_order_by(sorts, &category);
                let trimmed = sql.trim().trim_end_matches(';');
                let with_order = format!("{}{}", trimmed, order_clause);
                wrap_paginated(&with_order, limit, offset, dialect)
            } else {
                wrap_paginated(&sql, limit, offset, dialect)
            }
        } else {
            wrap_paginated(&sql, limit, offset, dialect)
        }
    } else {
        // Not paginatable — just execute as-is
        sql.clone()
    };

    debug!(
        "Executing paginated query on '{}': LIMIT={} OFFSET={}",
        connection_id, limit, offset
    );

    let query_future = timeout(duration, handle.base().execute_raw(&paginated_sql));

    let result = if let Some(ref qid) = query_id {
        let cancel_rx = cancel_registry.register(qid.clone());

        let outcome = tokio::select! {
            res = query_future => {
                cancel_registry.remove(qid);
                res.map_err(|_| {
                    error!("Paginated query timed out after {}s on '{}'", duration.as_secs(), connection_id);
                    AppError::QueryTimeout(duration.as_secs())
                })?
            }
            _ = cancel_rx => {
                warn!("Query '{}' cancelled on '{}'", qid, connection_id);
                return Err(AppError::QueryCancelled);
            }
        };
        outcome
    } else {
        query_future
            .await
            .map_err(|_| {
                error!("Paginated query timed out after {}s on '{}'", duration.as_secs(), connection_id);
                AppError::QueryTimeout(duration.as_secs())
            })?
    };

    match result {
        Ok(mut response) => {
            if let Some(mcs) = max_cell_size {
                truncate_large_values(&mut response, mcs);
            }
            info!(
                "Paginated query on '{}' completed in {}ms ({} rows)",
                connection_id, response.execution_time_ms, response.row_count
            );
            Ok(response)
        }
        Err(e) => {
            error!("Paginated query failed on '{}': {}", connection_id, e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn count_query_rows(
    connection_id: String,
    sql: String,
    pool_manager: State<'_, PoolManager>,
) -> Result<i64, AppError> {
    let handle = pool_manager.get(&connection_id).await?;

    if !is_paginatable_query(&sql) {
        return Err(AppError::InvalidConfig(
            "Cannot count rows for non-SELECT queries".to_string(),
        ));
    }

    let trimmed = sql.trim().trim_end_matches(';');
    let count_sql = format!("SELECT COUNT(*) AS _df_count FROM ({}) AS _df_cnt", trimmed);

    debug!("Counting query rows on '{}'", connection_id);

    let count_timeout = Duration::from_secs(5);
    let result = timeout(count_timeout, handle.base().execute_raw(&count_sql))
        .await
        .map_err(|_| {
            debug!("Count query timed out on '{}'", connection_id);
            AppError::QueryTimeout(5)
        })??;

    if let Some(first_row) = result.rows.first() {
        if let Some(cell) = first_row.first() {
            return match cell {
                CellValue::Int(v) => Ok(*v),
                CellValue::Float(v) => Ok(*v as i64),
                CellValue::Text(v) => v
                    .parse::<i64>()
                    .map_err(|_| AppError::Database("Invalid count value".to_string())),
                _ => Ok(0),
            };
        }
    }
    Ok(0)
}

#[tauri::command]
pub async fn fetch_full_cell(
    connection_id: String,
    sql: String,
    column: String,
    row_offset: i64,
    pool_manager: State<'_, PoolManager>,
) -> Result<CellValue, AppError> {
    let handle = pool_manager.get(&connection_id).await?;
    let dialect = handle.base().dialect_hint();
    let category = handle.base().category();

    let col_ident = quote_ident(&column, &category);
    let trimmed = sql.trim().trim_end_matches(';');

    let fetch_sql = if dialect == "mssql" {
        format!(
            "SELECT {} FROM ({}) AS _df_cell ORDER BY (SELECT NULL) OFFSET {} ROWS FETCH NEXT 1 ROWS ONLY",
            col_ident, trimmed, row_offset
        )
    } else {
        format!(
            "SELECT {} FROM ({}) AS _df_cell LIMIT 1 OFFSET {}",
            col_ident, trimmed, row_offset
        )
    };

    debug!(
        "Fetching full cell on '{}': column='{}', offset={}",
        connection_id, column, row_offset
    );

    let fetch_timeout = Duration::from_secs(10);
    let result = timeout(fetch_timeout, handle.base().execute_raw(&fetch_sql))
        .await
        .map_err(|_| {
            error!("Fetch full cell timed out on '{}'", connection_id);
            AppError::QueryTimeout(10)
        })??;

    if let Some(first_row) = result.rows.into_iter().next() {
        if let Some(cell) = first_row.into_iter().next() {
            return Ok(cell);
        }
    }

    Ok(CellValue::Null)
}

#[tauri::command]
pub async fn cancel_query(
    query_id: String,
    cancel_registry: State<'_, CancellationRegistry>,
) -> Result<bool, AppError> {
    info!("Cancelling query '{}'", query_id);
    Ok(cancel_registry.cancel(&query_id))
}
