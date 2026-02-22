use std::time::Instant;

use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::{AuthMethod, Config, EncryptionLevel};

use crate::db::traits::{DbDriver, SqlDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{
    ColumnInfo, ContainerInfo, FieldInfo, ForeignKeyInfo, IndexInfo, ItemInfo, SchemaInfo, TableInfo,
};

pub struct MssqlDriver {
    pool: Pool<ConnectionManager>,
}

impl MssqlDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let mut tib_config = Config::new();
        tib_config.host(config.host_or_default());
        tib_config.port(config.port_or_default());
        tib_config.database(config.database_or_default());
        tib_config.authentication(AuthMethod::sql_server(
            config.username_or_default(),
            config.password_or_default(),
        ));
        tib_config.encryption(if config.use_ssl {
            EncryptionLevel::Required
        } else {
            EncryptionLevel::NotSupported
        });
        tib_config.trust_cert();

        let mgr = ConnectionManager::build(tib_config)
            .map_err(|e| AppError::Database(format!("Failed to create MSSQL connection manager: {}", e)))?;

        let pool = Pool::builder()
            .max_size(5)
            .build(mgr)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to MSSQL: {}", e)))?;

        Ok(Self { pool })
    }

    async fn query_rows(&self, sql: &str) -> Result<(Vec<ColumnDef>, Vec<Vec<CellValue>>), AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database(format!("Failed to get MSSQL connection: {}", e)))?;

        let stream = conn.simple_query(sql).await
            .map_err(|e| AppError::Database(format!("MSSQL query error: {}", e)))?;

        let results = stream.into_results().await
            .map_err(|e| AppError::Database(format!("MSSQL result error: {}", e)))?;

        if results.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        let result_set = &results[0];
        if result_set.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        // Extract columns from first row
        let columns: Vec<ColumnDef> = result_set[0]
            .columns()
            .iter()
            .map(|col| ColumnDef {
                name: col.name().to_string(),
                data_type: format!("{:?}", col.column_type()),
            })
            .collect();

        let rows: Vec<Vec<CellValue>> = result_set
            .iter()
            .map(|row| {
                (0..columns.len())
                    .map(|i| {
                        // Try bool (BIT)
                        if let Ok(Some(v)) = row.try_get::<bool, _>(i) {
                            return CellValue::Bool(v);
                        }
                        // Try i16 (SMALLINT)
                        if let Ok(Some(v)) = row.try_get::<i16, _>(i) {
                            return CellValue::Int(v as i64);
                        }
                        // Try i32 (INT)
                        if let Ok(Some(v)) = row.try_get::<i32, _>(i) {
                            return CellValue::Int(v as i64);
                        }
                        // Try i64 (BIGINT)
                        if let Ok(Some(v)) = row.try_get::<i64, _>(i) {
                            return CellValue::Int(v);
                        }
                        // Try f32 (REAL)
                        if let Ok(Some(v)) = row.try_get::<f32, _>(i) {
                            return CellValue::Float(v as f64);
                        }
                        // Try f64 (FLOAT)
                        if let Ok(Some(v)) = row.try_get::<f64, _>(i) {
                            return CellValue::Float(v);
                        }
                        // Try &str (VARCHAR, NVARCHAR, etc.)
                        if let Ok(Some(v)) = row.try_get::<&str, _>(i) {
                            return CellValue::Text(v.to_string());
                        }
                        // Try Numeric (DECIMAL, NUMERIC)
                        if let Ok(Some(v)) = row.try_get::<tiberius::numeric::Numeric, _>(i) {
                            return CellValue::Text(v.to_string());
                        }
                        // Try binary (VARBINARY, IMAGE, etc.)
                        if let Ok(Some(v)) = row.try_get::<&[u8], _>(i) {
                            return CellValue::Binary(v.to_vec());
                        }
                        CellValue::Null
                    })
                    .collect()
            })
            .collect();

        Ok((columns, rows))
    }
}

#[async_trait]
impl DbDriver for MssqlDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Relational
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let trimmed = sql.trim();
        let upper = trimmed.to_uppercase();

        let is_select = upper.starts_with("SELECT")
            || upper.starts_with("WITH")
            || upper.starts_with("EXEC")
            || upper.starts_with("SP_");

        if is_select {
            let (columns, rows) = self.query_rows(trimmed).await?;
            let elapsed = start.elapsed().as_millis() as u64;
            let row_count = rows.len();

            Ok(QueryResponse {
                columns,
                rows,
                row_count,
                execution_time_ms: elapsed,
                affected_rows: None,
            })
        } else {
            let mut conn = self.pool.get().await
                .map_err(|e| AppError::Database(format!("Failed to get MSSQL connection: {}", e)))?;
            let result = conn.execute(trimmed, &[]).await
                .map_err(|e| AppError::Database(format!("MSSQL execute error: {}", e)))?;
            let elapsed = start.elapsed().as_millis() as u64;

            Ok(QueryResponse {
                columns: Vec::new(),
                rows: Vec::new(),
                row_count: 0,
                execution_time_ms: elapsed,
                affected_rows: Some(result.rows_affected().iter().sum::<u64>()),
            })
        }
    }

    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> {
        let schemas = self.get_schemas().await?;
        Ok(schemas.iter().map(ContainerInfo::from).collect())
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
impl SqlDriver for MssqlDriver {
    async fn get_schemas(&self) -> Result<Vec<SchemaInfo>, AppError> {
        let (_, rows) = self.query_rows(
            "SELECT s.name FROM sys.schemas s \
             WHERE s.name NOT IN ('sys', 'INFORMATION_SCHEMA', 'guest', 'db_owner', 'db_accessadmin', \
             'db_securityadmin', 'db_ddladmin', 'db_backupoperator', 'db_datareader', 'db_datawriter', \
             'db_denydatareader', 'db_denydatawriter') \
             ORDER BY s.name"
        ).await?;

        let schemas = rows
            .iter()
            .filter_map(|row| {
                if let Some(CellValue::Text(name)) = row.first() {
                    Some(SchemaInfo { name: name.clone() })
                } else {
                    None
                }
            })
            .collect();

        Ok(schemas)
    }

    async fn get_tables(&self, schema: &str) -> Result<Vec<TableInfo>, AppError> {
        let sql = format!(
            "SELECT o.name, o.type_desc \
             FROM sys.objects o \
             JOIN sys.schemas s ON o.schema_id = s.schema_id \
             WHERE s.name = '{}' AND o.type IN ('U', 'V') \
             ORDER BY o.name",
            schema.replace('\'', "''")
        );
        let (_, rows) = self.query_rows(&sql).await?;

        let tables = rows
            .iter()
            .filter_map(|row| {
                let name = match row.get(0) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => return None,
                };
                let table_type = match row.get(1) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => "TABLE".to_string(),
                };
                Some(TableInfo {
                    name,
                    schema: schema.to_string(),
                    table_type,
                    row_count: None,
                })
            })
            .collect();

        Ok(tables)
    }

    async fn get_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        let sql = format!(
            "SELECT c.name, t.name as type_name, c.is_nullable, \
                    OBJECT_DEFINITION(c.default_object_id) as column_default, \
                    c.column_id, \
                    CASE WHEN ic.column_id IS NOT NULL THEN 1 ELSE 0 END as is_pk \
             FROM sys.columns c \
             JOIN sys.types t ON c.user_type_id = t.user_type_id \
             JOIN sys.objects o ON c.object_id = o.object_id \
             JOIN sys.schemas s ON o.schema_id = s.schema_id \
             LEFT JOIN sys.indexes i ON i.object_id = o.object_id AND i.is_primary_key = 1 \
             LEFT JOIN sys.index_columns ic ON ic.object_id = i.object_id AND ic.index_id = i.index_id AND ic.column_id = c.column_id \
             WHERE s.name = '{}' AND o.name = '{}' \
             ORDER BY c.column_id",
            schema.replace('\'', "''"),
            table.replace('\'', "''")
        );
        let (_, rows) = self.query_rows(&sql).await?;

        let columns = rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| {
                let name = match row.get(0) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => return None,
                };
                let data_type = match row.get(1) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => "unknown".to_string(),
                };
                let is_nullable = match row.get(2) {
                    Some(CellValue::Bool(v)) => *v,
                    Some(CellValue::Int(v)) => *v != 0,
                    _ => true,
                };
                let column_default = match row.get(3) {
                    Some(CellValue::Text(v)) => Some(v.clone()),
                    _ => None,
                };
                let is_pk = match row.get(5) {
                    Some(CellValue::Int(v)) => *v != 0,
                    Some(CellValue::Bool(v)) => *v,
                    _ => false,
                };

                Some(ColumnInfo {
                    name,
                    data_type,
                    is_nullable,
                    column_default,
                    is_primary_key: is_pk,
                    ordinal_position: (idx + 1) as i32,
                })
            })
            .collect();

        Ok(columns)
    }

    async fn get_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        let sql = format!(
            "SELECT i.name, i.is_unique, i.is_primary_key, i.type_desc, \
                    STRING_AGG(c.name, ',') WITHIN GROUP (ORDER BY ic.key_ordinal) as columns \
             FROM sys.indexes i \
             JOIN sys.index_columns ic ON i.object_id = ic.object_id AND i.index_id = ic.index_id \
             JOIN sys.columns c ON ic.object_id = c.object_id AND ic.column_id = c.column_id \
             JOIN sys.objects o ON i.object_id = o.object_id \
             JOIN sys.schemas s ON o.schema_id = s.schema_id \
             WHERE s.name = '{}' AND o.name = '{}' AND i.name IS NOT NULL \
             GROUP BY i.name, i.is_unique, i.is_primary_key, i.type_desc \
             ORDER BY i.name",
            schema.replace('\'', "''"),
            table.replace('\'', "''")
        );
        let (_, rows) = self.query_rows(&sql).await?;

        let indexes = rows
            .iter()
            .filter_map(|row| {
                let name = match row.get(0) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => return None,
                };
                let is_unique = match row.get(1) {
                    Some(CellValue::Bool(v)) => *v,
                    Some(CellValue::Int(v)) => *v != 0,
                    _ => false,
                };
                let is_primary = match row.get(2) {
                    Some(CellValue::Bool(v)) => *v,
                    Some(CellValue::Int(v)) => *v != 0,
                    _ => false,
                };
                let index_type = match row.get(3) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => "NONCLUSTERED".to_string(),
                };
                let columns_str = match row.get(4) {
                    Some(CellValue::Text(v)) => v.clone(),
                    _ => String::new(),
                };

                Some(IndexInfo {
                    name,
                    columns: columns_str.split(',').map(|s| s.to_string()).collect(),
                    is_unique,
                    is_primary,
                    index_type,
                })
            })
            .collect();

        Ok(indexes)
    }

    async fn get_foreign_keys(&self, schema: &str, table: &str) -> Result<Vec<ForeignKeyInfo>, AppError> {
        let sql = format!(
            "SELECT fk.name, \
                    COL_NAME(fkc.parent_object_id, fkc.parent_column_id) as column_name, \
                    OBJECT_NAME(fkc.referenced_object_id) as referenced_table, \
                    SCHEMA_NAME(o.schema_id) as referenced_schema, \
                    COL_NAME(fkc.referenced_object_id, fkc.referenced_column_id) as referenced_column, \
                    fk.update_referential_action_desc, \
                    fk.delete_referential_action_desc \
             FROM sys.foreign_keys fk \
             JOIN sys.foreign_key_columns fkc ON fk.object_id = fkc.constraint_object_id \
             JOIN sys.objects o ON fk.referenced_object_id = o.object_id \
             JOIN sys.objects po ON fk.parent_object_id = po.object_id \
             JOIN sys.schemas ps ON po.schema_id = ps.schema_id \
             WHERE ps.name = '{}' AND po.name = '{}' \
             ORDER BY fk.name",
            schema.replace('\'', "''"),
            table.replace('\'', "''")
        );
        let (_, rows) = self.query_rows(&sql).await?;

        use std::collections::HashMap;
        let mut fk_map: HashMap<String, ForeignKeyInfo> = HashMap::new();

        for row in &rows {
            let name = match row.get(0) {
                Some(CellValue::Text(v)) => v.clone(),
                _ => continue,
            };
            let column = match row.get(1) { Some(CellValue::Text(v)) => v.clone(), _ => continue };
            let ref_table = match row.get(2) { Some(CellValue::Text(v)) => v.clone(), _ => continue };
            let ref_schema = match row.get(3) { Some(CellValue::Text(v)) => v.clone(), _ => String::new() };
            let ref_column = match row.get(4) { Some(CellValue::Text(v)) => v.clone(), _ => continue };
            let on_update = match row.get(5) { Some(CellValue::Text(v)) => v.clone(), _ => "NO_ACTION".to_string() };
            let on_delete = match row.get(6) { Some(CellValue::Text(v)) => v.clone(), _ => "NO_ACTION".to_string() };

            let entry = fk_map.entry(name.clone()).or_insert_with(|| ForeignKeyInfo {
                name,
                columns: Vec::new(),
                referenced_table: ref_table,
                referenced_schema: ref_schema,
                referenced_columns: Vec::new(),
                on_update,
                on_delete,
            });

            if !entry.columns.contains(&column) {
                entry.columns.push(column);
            }
            if !entry.referenced_columns.contains(&ref_column) {
                entry.referenced_columns.push(ref_column);
            }
        }

        let mut foreign_keys: Vec<ForeignKeyInfo> = fk_map.into_values().collect();
        foreign_keys.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(foreign_keys)
    }

    async fn get_table_data(&self, schema: &str, table: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        let sql = format!(
            "SELECT * FROM [{}].[{}] ORDER BY (SELECT NULL) OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
            schema, table, offset, limit
        );
        self.execute_raw(&sql).await
    }

    async fn get_row_count(&self, schema: &str, table: &str) -> Result<i64, AppError> {
        let sql = format!("SELECT COUNT(*) as count FROM [{}].[{}]", schema, table);
        let (_, rows) = self.query_rows(&sql).await?;

        if let Some(row) = rows.first() {
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
            .zip(pk_values.iter())
            .map(|(col, val)| format!("[{}] = '{}'", col, val.replace('\'', "''")))
            .collect();

        let escaped_value = value.replace('\'', "''");
        let sql = format!(
            "UPDATE [{}].[{}] SET [{}] = '{}' WHERE {}",
            schema, table, column, escaped_value, where_clauses.join(" AND ")
        );

        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database(format!("Failed to get MSSQL connection: {}", e)))?;
        conn.execute(&sql[..], &[]).await
            .map_err(|e| AppError::Database(format!("MSSQL update error: {}", e)))?;
        Ok(())
    }

    async fn insert_row(&self, schema: &str, table: &str, columns: Vec<String>, values: Vec<String>) -> Result<(), AppError> {
        if columns.len() != values.len() {
            return Err(AppError::InvalidConfig("Columns and values must have the same length".to_string()));
        }

        let cols: Vec<String> = columns.iter().map(|c| format!("[{}]", c)).collect();
        let vals: Vec<String> = values.iter().map(|v| format!("'{}'", v.replace('\'', "''"))).collect();

        let sql = format!(
            "INSERT INTO [{}].[{}] ({}) VALUES ({})",
            schema, table, cols.join(", "), vals.join(", ")
        );

        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database(format!("Failed to get MSSQL connection: {}", e)))?;
        conn.execute(&sql[..], &[]).await
            .map_err(|e| AppError::Database(format!("MSSQL insert error: {}", e)))?;
        Ok(())
    }

    async fn delete_rows(&self, schema: &str, table: &str, pk_columns: Vec<String>, pk_values_list: Vec<Vec<String>>) -> Result<u64, AppError> {
        if pk_columns.is_empty() {
            return Err(AppError::InvalidConfig("At least one primary key column is required".to_string()));
        }

        let mut total_affected: u64 = 0;
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database(format!("Failed to get MSSQL connection: {}", e)))?;

        for pk_values in &pk_values_list {
            if pk_columns.len() != pk_values.len() {
                return Err(AppError::InvalidConfig("Primary key columns and values must have the same length".to_string()));
            }

            let where_clauses: Vec<String> = pk_columns
                .iter()
                .zip(pk_values.iter())
                .map(|(col, val)| format!("[{}] = '{}'", col, val.replace('\'', "''")))
                .collect();

            let sql = format!(
                "DELETE FROM [{}].[{}] WHERE {}",
                schema, table, where_clauses.join(" AND ")
            );

            let result = conn.execute(&sql[..], &[]).await
                .map_err(|e| AppError::Database(format!("MSSQL delete error: {}", e)))?;
            total_affected += result.rows_affected().iter().sum::<u64>();
        }

        Ok(total_affected)
    }
}
