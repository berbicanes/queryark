use std::time::Instant;

use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

use crate::db::traits::{DbDriver, KeyValueDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{ContainerInfo, FieldInfo, ItemInfo};

pub struct RedisDriver {
    conn: MultiplexedConnection,
}

impl RedisDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let url = config.to_connection_url();
        let client = redis::Client::open(url.as_str())
            .map_err(|e| AppError::Database(format!("Failed to create Redis client: {}", e)))?;

        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self { conn })
    }
}

#[async_trait]
impl DbDriver for RedisDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::KeyValue
    }

    fn dialect_hint(&self) -> &'static str {
        "redis"
    }

    async fn execute_raw(&self, query: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let trimmed = query.trim();

        // Parse Redis command
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return Err(AppError::InvalidConfig("Empty command".to_string()));
        }

        let mut conn = self.conn.clone();
        let cmd = parts[0].to_uppercase();

        let result: redis::RedisResult<redis::Value> = redis::cmd(&cmd)
            .arg(&parts[1..])
            .query_async(&mut conn)
            .await;

        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(value) => {
                let (columns, rows) = redis_value_to_response(&value);
                let row_count = rows.len();

                Ok(QueryResponse {
                    columns,
                    rows,
                    row_count,
                    execution_time_ms: elapsed,
                    affected_rows: None,
                    truncated: false,
                    max_rows_limit: None,
                })
            }
            Err(e) => Err(AppError::Database(format!("Redis error: {}", e))),
        }
    }

    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> {
        // Redis databases are numbered 0-15 by default
        Ok((0..16)
            .map(|i| ContainerInfo {
                name: format!("db{}", i),
                container_type: "database".to_string(),
            })
            .collect())
    }

    async fn get_items(&self, _container: &str) -> Result<Vec<ItemInfo>, AppError> {
        let keys = self.scan_keys("*", 1000).await?;
        Ok(keys
            .into_iter()
            .map(|name| ItemInfo {
                name,
                container: _container.to_string(),
                item_type: "key".to_string(),
                item_count: None,
            })
            .collect())
    }

    async fn get_item_fields(&self, _container: &str, item: &str) -> Result<Vec<FieldInfo>, AppError> {
        let key_type = self.get_key_type(item).await?;
        Ok(vec![
            FieldInfo {
                name: "key".to_string(),
                data_type: "string".to_string(),
                is_nullable: false,
                is_primary: true,
                default_value: None,
                ordinal_position: 1,
            },
            FieldInfo {
                name: "type".to_string(),
                data_type: key_type,
                is_nullable: false,
                is_primary: false,
                default_value: None,
                ordinal_position: 2,
            },
            FieldInfo {
                name: "value".to_string(),
                data_type: "mixed".to_string(),
                is_nullable: true,
                is_primary: false,
                default_value: None,
                ordinal_position: 3,
            },
        ])
    }

    async fn get_item_data(&self, _container: &str, item: &str, _limit: i64, _offset: i64) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let value = self.get_value(item).await?;
        let elapsed = start.elapsed().as_millis() as u64;

        let columns = vec![
            ColumnDef { name: "key".to_string(), data_type: "string".to_string() },
            ColumnDef { name: "value".to_string(), data_type: "mixed".to_string() },
        ];

        let value_cell = json_value_to_cell(&value);
        let rows = vec![vec![CellValue::Text(item.to_string()), value_cell]];

        Ok(QueryResponse {
            columns,
            rows,
            row_count: 1,
            execution_time_ms: elapsed,
            affected_rows: None,
            truncated: false,
            max_rows_limit: None,
        })
    }

    async fn get_item_count(&self, _container: &str, _item: &str) -> Result<i64, AppError> {
        let mut conn = self.conn.clone();
        let count: i64 = redis::cmd("DBSIZE")
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Redis DBSIZE error: {}", e)))?;
        Ok(count)
    }

    async fn health_check(&self) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Redis PING failed: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl KeyValueDriver for RedisDriver {
    async fn get_value(&self, key: &str) -> Result<serde_json::Value, AppError> {
        let mut conn = self.conn.clone();
        let key_type = self.get_key_type(key).await?;

        match key_type.as_str() {
            "string" => {
                let val: String = conn.get(key).await?;
                Ok(serde_json::Value::String(val))
            }
            "list" => {
                let val: Vec<String> = conn.lrange(key, 0, -1).await?;
                Ok(serde_json::to_value(val).unwrap_or(serde_json::Value::Null))
            }
            "set" => {
                let val: Vec<String> = conn.smembers(key).await?;
                Ok(serde_json::to_value(val).unwrap_or(serde_json::Value::Null))
            }
            "hash" => {
                let val: std::collections::HashMap<String, String> = conn.hgetall(key).await?;
                Ok(serde_json::to_value(val).unwrap_or(serde_json::Value::Null))
            }
            "zset" => {
                let val: Vec<(String, f64)> = conn.zrangebyscore_withscores(key, "-inf", "+inf").await?;
                let arr: Vec<serde_json::Value> = val
                    .into_iter()
                    .map(|(member, score)| {
                        serde_json::json!({"member": member, "score": score})
                    })
                    .collect();
                Ok(serde_json::Value::Array(arr))
            }
            _ => Ok(serde_json::Value::Null),
        }
    }

    async fn set_value(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        if let Some(ttl_secs) = ttl {
            conn.set_ex::<_, _, ()>(key, value, ttl_secs).await?;
        } else {
            conn.set::<_, _, ()>(key, value).await?;
        }
        Ok(())
    }

    async fn delete_keys(&self, keys: Vec<String>) -> Result<u64, AppError> {
        let mut conn = self.conn.clone();
        let count: u64 = conn.del(&keys).await?;
        Ok(count)
    }

    async fn get_key_type(&self, key: &str) -> Result<String, AppError> {
        let mut conn = self.conn.clone();
        let key_type: String = redis::cmd("TYPE")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Redis TYPE error: {}", e)))?;
        Ok(key_type)
    }

    async fn scan_keys(&self, pattern: &str, count: i64) -> Result<Vec<String>, AppError> {
        let mut conn = self.conn.clone();
        let mut keys: Vec<String> = Vec::new();
        let mut cursor: u64 = 0;

        loop {
            let result: (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(count)
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::Database(format!("Redis SCAN error: {}", e)))?;

            cursor = result.0;
            keys.extend(result.1);

            if cursor == 0 || keys.len() >= count as usize {
                break;
            }
        }

        keys.sort();
        Ok(keys)
    }
}

fn json_value_to_cell(value: &serde_json::Value) -> CellValue {
    match value {
        serde_json::Value::Null => CellValue::Null,
        serde_json::Value::Bool(b) => CellValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                CellValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                CellValue::Float(f)
            } else {
                CellValue::Text(n.to_string())
            }
        }
        serde_json::Value::String(s) => CellValue::Text(s.clone()),
        _ => CellValue::Json(value.to_string()),
    }
}

fn redis_value_to_response(value: &redis::Value) -> (Vec<ColumnDef>, Vec<Vec<CellValue>>) {
    let columns = vec![ColumnDef {
        name: "result".to_string(),
        data_type: "mixed".to_string(),
    }];

    match value {
        redis::Value::Nil => (columns, vec![vec![CellValue::Null]]),
        redis::Value::Int(i) => (columns, vec![vec![CellValue::Int(*i)]]),
        redis::Value::BulkString(b) => {
            let text = String::from_utf8_lossy(b).to_string();
            (columns, vec![vec![CellValue::Text(text)]])
        }
        redis::Value::Array(arr) => {
            let rows: Vec<Vec<CellValue>> = arr
                .iter()
                .map(|v| match v {
                    redis::Value::BulkString(b) => {
                        vec![CellValue::Text(String::from_utf8_lossy(b).to_string())]
                    }
                    redis::Value::Int(i) => vec![CellValue::Int(*i)],
                    redis::Value::Nil => vec![CellValue::Null],
                    _ => vec![CellValue::Text(format!("{:?}", v))],
                })
                .collect();
            (columns, rows)
        }
        redis::Value::SimpleString(s) => (columns, vec![vec![CellValue::Text(s.clone())]]),
        redis::Value::Okay => (columns, vec![vec![CellValue::Text("OK".to_string())]]),
        _ => (columns, vec![vec![CellValue::Text(format!("{:?}", value))]]),
    }
}
