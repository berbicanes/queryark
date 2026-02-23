use std::time::Instant;

use async_trait::async_trait;
use scylla::frame::response::result::CqlValue;
use scylla::Session;
use scylla::SessionBuilder;

use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo, SchemaInfo, TableInfo,
};

pub struct CassandraDriver {
    session: Session,
}

impl CassandraDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let contact_point = format!("{}:{}", config.host_or_default(), config.port_or_default());

        let mut builder = SessionBuilder::new().known_node(&contact_point);

        if !config.username_or_default().is_empty() {
            builder = builder.user(config.username_or_default(), config.password_or_default());
        }

        let session = builder
            .build()
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to Cassandra: {}", e)))?;

        Ok(Self { session })
    }

    fn cql_value_to_cell(value: &CqlValue) -> CellValue {
        match value {
            CqlValue::Boolean(b) => CellValue::Bool(*b),
            CqlValue::TinyInt(i) => CellValue::Int(*i as i64),
            CqlValue::SmallInt(i) => CellValue::Int(*i as i64),
            CqlValue::Int(i) => CellValue::Int(*i as i64),
            CqlValue::BigInt(i) => CellValue::Int(*i),
            CqlValue::Float(f) => CellValue::Float(*f as f64),
            CqlValue::Double(f) => CellValue::Float(*f),
            CqlValue::Text(s) | CqlValue::Ascii(s) => CellValue::Text(s.clone()),
            CqlValue::Blob(b) => CellValue::Binary(b.clone()),
            CqlValue::Uuid(u) => CellValue::Text(u.to_string()),
            CqlValue::Timeuuid(u) => CellValue::Text(u.to_string()),
            CqlValue::Timestamp(ts) => CellValue::Timestamp(format!("{:?}", ts)),
            CqlValue::Date(d) => CellValue::Timestamp(format!("{:?}", d)),
            CqlValue::Time(t) => CellValue::Timestamp(format!("{:?}", t)),
            CqlValue::Inet(addr) => CellValue::Text(addr.to_string()),
            CqlValue::Counter(c) => CellValue::Int(c.0),
            CqlValue::Varint(v) => CellValue::Text(format!("{:?}", v)),
            CqlValue::Decimal(d) => CellValue::Text(format!("{:?}", d)),
            CqlValue::Empty => CellValue::Null,
            _ => CellValue::Text(format!("{:?}", value)),
        }
    }
}

#[async_trait]
impl DbDriver for CassandraDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::WideColumn
    }

    fn dialect_hint(&self) -> &'static str {
        "cassandra"
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let trimmed = sql.trim();

        let result = self
            .session
            .query_unpaged(trimmed, &[])
            .await
            .map_err(|e| AppError::Database(format!("Cassandra query error: {}", e)))?;

        let elapsed = start.elapsed().as_millis() as u64;

        // Extract column specs before consuming rows, since col_specs() borrows result
        let columns: Vec<ColumnDef> = result
            .col_specs()
            .iter()
            .map(|spec| ColumnDef {
                name: spec.name.clone(),
                data_type: format!("{:?}", spec.typ),
            })
            .collect();
        let num_columns = columns.len();

        if let Some(rows) = result.rows {
            let mut data_rows: Vec<Vec<CellValue>> = Vec::new();

            for row in &rows {
                let mut cells = Vec::new();
                for i in 0..num_columns {
                    let cell = match row.columns.get(i).and_then(|c| c.as_ref()) {
                        Some(val) => Self::cql_value_to_cell(val),
                        None => CellValue::Null,
                    };
                    cells.push(cell);
                }
                data_rows.push(cells);
            }

            let row_count = data_rows.len();
            Ok(QueryResponse {
                columns,
                rows: data_rows,
                row_count,
                execution_time_ms: elapsed,
                affected_rows: None,
                truncated: false,
                max_rows_limit: None,
            })
        } else {
            Ok(QueryResponse {
                columns: Vec::new(),
                rows: Vec::new(),
                row_count: 0,
                execution_time_ms: elapsed,
                affected_rows: Some(0),
                truncated: false,
                max_rows_limit: None,
            })
        }
    }

    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> {
        let schemas = self.get_schemas().await?;
        Ok(schemas
            .iter()
            .map(|s| ContainerInfo {
                name: s.name.clone(),
                container_type: "keyspace".to_string(),
            })
            .collect())
    }

    async fn get_items(&self, container: &str) -> Result<Vec<ItemInfo>, AppError> {
        let tables = self.get_tables(container).await?;
        Ok(tables.iter().map(ItemInfo::from).collect())
    }

    async fn get_item_fields(&self, container: &str, item: &str) -> Result<Vec<FieldInfo>, AppError> {
        let columns = self.get_columns(container, item).await?;
        Ok(columns.iter().map(FieldInfo::from).collect())
    }

    async fn get_item_data(&self, container: &str, item: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        SqlDriver::get_table_data(self, container, item, limit, offset).await
    }

    async fn get_item_count(&self, container: &str, item: &str) -> Result<i64, AppError> {
        SqlDriver::get_row_count(self, container, item).await
    }
}

#[async_trait]
impl SqlDriver for CassandraDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        let result = self
            .session
            .query_unpaged(
                "SELECT keyspace_name FROM system_schema.keyspaces",
                &[],
            )
            .await
            .map_err(|e| AppError::Database(format!("Cassandra query error: {}", e)))?;

        let mut schemas = Vec::new();
        if let Some(rows) = result.rows {
            for row in &rows {
                if let Some(Some(CqlValue::Text(name))) = row.columns.first() {
                    if !name.starts_with("system") {
                        schemas.push(SchemaInfo { name: name.clone() });
                    }
                }
            }
        }

        schemas.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(schemas)
    }

    async fn get_tables(&self, schema: &str) -> Result<Vec<TableInfo>, AppError> {
        let sql = format!(
            "SELECT table_name FROM system_schema.tables WHERE keyspace_name = '{}'",
            schema.replace('\'', "''")
        );

        let result = self
            .session
            .query_unpaged(sql.as_str(), &[])
            .await
            .map_err(|e| AppError::Database(format!("Cassandra query error: {}", e)))?;

        let mut tables = Vec::new();
        if let Some(rows) = result.rows {
            for row in &rows {
                if let Some(Some(CqlValue::Text(name))) = row.columns.first() {
                    tables.push(TableInfo {
                        name: name.clone(),
                        schema: schema.to_string(),
                        table_type: "TABLE".to_string(),
                        row_count: None,
                    });
                }
            }
        }

        tables.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tables)
    }

    async fn get_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        let sql = format!(
            "SELECT column_name, type, kind, position \
             FROM system_schema.columns \
             WHERE keyspace_name = '{}' AND table_name = '{}'",
            schema.replace('\'', "''"),
            table.replace('\'', "''")
        );

        let result = self
            .session
            .query_unpaged(sql.as_str(), &[])
            .await
            .map_err(|e| AppError::Database(format!("Cassandra query error: {}", e)))?;

        let mut columns = Vec::new();
        if let Some(rows) = result.rows {
            for row in &rows {
                let name = match &row.columns[0] {
                    Some(CqlValue::Text(s)) => s.clone(),
                    _ => continue,
                };
                let data_type = match &row.columns[1] {
                    Some(CqlValue::Text(s)) => s.clone(),
                    _ => "unknown".to_string(),
                };
                let kind = match &row.columns[2] {
                    Some(CqlValue::Text(s)) => s.clone(),
                    _ => String::new(),
                };
                let position = match &row.columns[3] {
                    Some(CqlValue::Int(i)) => *i,
                    _ => 0,
                };

                columns.push(ColumnInfo {
                    name,
                    data_type,
                    is_nullable: kind != "partition_key" && kind != "clustering",
                    column_default: None,
                    is_primary_key: kind == "partition_key" || kind == "clustering",
                    ordinal_position: position,
                });
            }
        }

        columns.sort_by(|a, b| a.ordinal_position.cmp(&b.ordinal_position));
        Ok(columns)
    }

    async fn get_indexes(&self, _schema: &str, _table: &str) -> Result<Vec<IndexInfo>, AppError> {
        Ok(Vec::new())
    }

    async fn get_foreign_keys(&self, _schema: &str, _table: &str) -> Result<Vec<ForeignKeyInfo>, AppError> {
        Ok(Vec::new())
    }

    async fn get_table_data(&self, schema: &str, table: &str, limit: i64, _offset: i64) -> Result<QueryResponse, AppError> {
        // Cassandra doesn't support OFFSET natively
        let sql = format!("SELECT * FROM {}.{} LIMIT {}", schema, table, limit);
        self.execute_raw(&sql).await
    }

    async fn get_row_count(&self, schema: &str, table: &str) -> Result<i64, AppError> {
        let sql = format!("SELECT COUNT(*) FROM {}.{}", schema, table);
        let response = self.execute_raw(&sql).await?;
        if let Some(row) = response.rows.first() {
            if let Some(CellValue::Int(count)) = row.first() {
                return Ok(*count);
            }
        }
        Ok(0)
    }

    async fn update_cell(&self, schema: &str, table: &str, column: &str, value: &str, pk_columns: Vec<String>, pk_values: Vec<String>) -> Result<(), AppError> {
        if pk_columns.len() != pk_values.len() || pk_columns.is_empty() {
            return Err(AppError::InvalidConfig("Invalid primary key specification".to_string()));
        }

        let where_clauses: Vec<String> = pk_columns
            .iter()
            .map(|col| format!("{} = ?", col))
            .collect();

        let sql = format!(
            "UPDATE {}.{} SET {} = ? WHERE {}",
            schema, table, column, where_clauses.join(" AND ")
        );

        let mut cql_values: Vec<CqlValue> = vec![CqlValue::Text(value.to_string())];
        for pk_val in &pk_values {
            cql_values.push(CqlValue::Text(pk_val.clone()));
        }

        self.session
            .query_unpaged(sql.as_str(), &cql_values)
            .await
            .map_err(|e| AppError::Database(format!("Cassandra update error: {}", e)))?;
        Ok(())
    }

    async fn insert_row(&self, schema: &str, table: &str, columns: Vec<String>, values: Vec<String>) -> Result<(), AppError> {
        if columns.len() != values.len() {
            return Err(AppError::InvalidConfig("Columns and values must have the same length".to_string()));
        }

        let cols = columns.join(", ");
        let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();

        let sql = format!(
            "INSERT INTO {}.{} ({}) VALUES ({})",
            schema, table, cols, placeholders.join(", ")
        );

        let cql_values: Vec<CqlValue> = values.iter().map(|v| CqlValue::Text(v.clone())).collect();

        self.session
            .query_unpaged(sql.as_str(), &cql_values)
            .await
            .map_err(|e| AppError::Database(format!("Cassandra insert error: {}", e)))?;
        Ok(())
    }

    async fn delete_rows(&self, schema: &str, table: &str, pk_columns: Vec<String>, pk_values_list: Vec<Vec<String>>) -> Result<u64, AppError> {
        if pk_columns.is_empty() {
            return Err(AppError::InvalidConfig("At least one primary key column is required".to_string()));
        }

        let mut total: u64 = 0;
        for pk_values in &pk_values_list {
            if pk_columns.len() != pk_values.len() {
                return Err(AppError::InvalidConfig("Primary key columns and values must have the same length".to_string()));
            }

            let where_clauses: Vec<String> = pk_columns
                .iter()
                .map(|col| format!("{} = ?", col))
                .collect();

            let sql = format!(
                "DELETE FROM {}.{} WHERE {}",
                schema, table, where_clauses.join(" AND ")
            );

            let cql_values: Vec<CqlValue> = pk_values.iter().map(|v| CqlValue::Text(v.clone())).collect();

            self.session
                .query_unpaged(sql.as_str(), &cql_values)
                .await
                .map_err(|e| AppError::Database(format!("Cassandra delete error: {}", e)))?;
            total += 1;
        }

        Ok(total)
    }
}
