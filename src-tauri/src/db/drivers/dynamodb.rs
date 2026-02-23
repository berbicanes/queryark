use std::collections::HashMap;
use std::time::Instant;

use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;

use crate::db::traits::{DbDriver, DocumentDriver};
use crate::error::AppError;
use crate::models::connection::{CloudAuth, ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{ContainerInfo, FieldInfo, ItemInfo};

pub struct DynamoDbDriver {
    client: Client,
}

impl DynamoDbDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let region = config
            .aws_region
            .as_deref()
            .unwrap_or("us-east-1");

        let mut aws_config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region.to_string()));

        // Use cloud auth credentials if provided
        if let Some(CloudAuth::AwsCredentials {
            access_key,
            secret_key,
            ..
        }) = &config.cloud_auth
        {
            aws_config_loader = aws_config_loader.credentials_provider(
                aws_sdk_dynamodb::config::Credentials::new(
                    access_key,
                    secret_key,
                    None,
                    None,
                    "dataforge",
                ),
            );
        }

        let aws_config = aws_config_loader.load().await;
        let client = Client::new(&aws_config);

        // Test connection
        client
            .list_tables()
            .limit(1)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to DynamoDB: {}", e)))?;

        Ok(Self { client })
    }

    fn attribute_to_cell(value: &AttributeValue) -> CellValue {
        match value {
            AttributeValue::S(s) => CellValue::Text(s.clone()),
            AttributeValue::N(n) => {
                if let Ok(i) = n.parse::<i64>() {
                    CellValue::Int(i)
                } else if let Ok(f) = n.parse::<f64>() {
                    CellValue::Float(f)
                } else {
                    CellValue::Text(n.clone())
                }
            }
            AttributeValue::Bool(b) => CellValue::Bool(*b),
            AttributeValue::B(b) => CellValue::Binary(b.as_ref().to_vec()),
            AttributeValue::Null(n) => {
                if *n {
                    CellValue::Null
                } else {
                    CellValue::Bool(false)
                }
            }
            AttributeValue::L(list) => {
                let items: Vec<serde_json::Value> = list
                    .iter()
                    .map(|v| Self::attribute_to_json(v))
                    .collect();
                CellValue::Json(serde_json::to_string(&items).unwrap_or_default())
            }
            AttributeValue::M(map) => {
                let obj: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::attribute_to_json(v)))
                    .collect();
                CellValue::Json(serde_json::to_string(&obj).unwrap_or_default())
            }
            AttributeValue::Ss(set) => CellValue::Json(serde_json::to_string(set).unwrap_or_default()),
            AttributeValue::Ns(set) => CellValue::Json(serde_json::to_string(set).unwrap_or_default()),
            _ => CellValue::Text(format!("{:?}", value)),
        }
    }

    fn attribute_to_json(value: &AttributeValue) -> serde_json::Value {
        match value {
            AttributeValue::S(s) => serde_json::Value::String(s.clone()),
            AttributeValue::N(n) => {
                if let Ok(i) = n.parse::<i64>() {
                    serde_json::Value::Number(i.into())
                } else if let Ok(f) = n.parse::<f64>() {
                    serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                } else {
                    serde_json::Value::String(n.clone())
                }
            }
            AttributeValue::Bool(b) => serde_json::Value::Bool(*b),
            AttributeValue::Null(_) => serde_json::Value::Null,
            AttributeValue::L(list) => {
                let items: Vec<serde_json::Value> = list.iter().map(|v| Self::attribute_to_json(v)).collect();
                serde_json::Value::Array(items)
            }
            AttributeValue::M(map) => {
                let obj: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::attribute_to_json(v)))
                    .collect();
                serde_json::Value::Object(obj)
            }
            _ => serde_json::Value::String(format!("{:?}", value)),
        }
    }

    fn json_to_attribute(value: &serde_json::Value) -> AttributeValue {
        match value {
            serde_json::Value::Null => AttributeValue::Null(true),
            serde_json::Value::Bool(b) => AttributeValue::Bool(*b),
            serde_json::Value::Number(n) => AttributeValue::N(n.to_string()),
            serde_json::Value::String(s) => AttributeValue::S(s.clone()),
            serde_json::Value::Array(arr) => {
                let items: Vec<AttributeValue> = arr.iter().map(|v| Self::json_to_attribute(v)).collect();
                AttributeValue::L(items)
            }
            serde_json::Value::Object(map) => {
                let items: HashMap<String, AttributeValue> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::json_to_attribute(v)))
                    .collect();
                AttributeValue::M(items)
            }
        }
    }
}

#[async_trait]
impl DbDriver for DynamoDbDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Document
    }

    fn dialect_hint(&self) -> &'static str {
        "dynamodb"
    }

    async fn execute_raw(&self, query: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();

        // Parse JSON command
        let cmd: serde_json::Value = serde_json::from_str(query)
            .map_err(|e| AppError::InvalidConfig(format!("Invalid JSON query: {}", e)))?;

        let table_name = cmd["table"]
            .as_str()
            .ok_or_else(|| AppError::InvalidConfig("'table' field required".to_string()))?;

        let result = self
            .client
            .scan()
            .table_name(table_name)
            .limit(50)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("DynamoDB scan error: {}", e)))?;

        let elapsed = start.elapsed().as_millis() as u64;

        let items = result.items();

        // Collect all unique keys
        let mut all_keys: Vec<String> = Vec::new();
        for item in items {
            for key in item.keys() {
                if !all_keys.contains(key) {
                    all_keys.push(key.clone());
                }
            }
        }

        let columns: Vec<ColumnDef> = all_keys
            .iter()
            .map(|k| ColumnDef {
                name: k.clone(),
                data_type: "mixed".to_string(),
            })
            .collect();

        let rows: Vec<Vec<CellValue>> = items
            .iter()
            .map(|item| {
                all_keys
                    .iter()
                    .map(|key| {
                        item.get(key)
                            .map(|v| Self::attribute_to_cell(v))
                            .unwrap_or(CellValue::Null)
                    })
                    .collect()
            })
            .collect();

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

    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> {
        // DynamoDB doesn't have databases/schemas, just tables at the top level
        Ok(vec![ContainerInfo {
            name: "default".to_string(),
            container_type: "region".to_string(),
        }])
    }

    async fn get_items(&self, _container: &str) -> Result<Vec<ItemInfo>, AppError> {
        let mut table_names = Vec::new();
        let mut exclusive_start = None;

        loop {
            let mut req = self.client.list_tables().limit(100);
            if let Some(start) = &exclusive_start {
                req = req.exclusive_start_table_name(start);
            }

            let result = req
                .send()
                .await
                .map_err(|e| AppError::Database(format!("DynamoDB list tables error: {}", e)))?;

            let names = result.table_names();
            table_names.extend(names.iter().cloned());

            exclusive_start = result.last_evaluated_table_name().map(|s| s.to_string());
            if exclusive_start.is_none() {
                break;
            }
        }

        Ok(table_names
            .into_iter()
            .map(|name| ItemInfo {
                name,
                container: "default".to_string(),
                item_type: "table".to_string(),
                item_count: None,
            })
            .collect())
    }

    async fn get_item_fields(&self, _container: &str, item: &str) -> Result<Vec<FieldInfo>, AppError> {
        let result = self
            .client
            .describe_table()
            .table_name(item)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("DynamoDB describe table error: {}", e)))?;

        let table = result.table()
            .ok_or_else(|| AppError::Database("Table not found".to_string()))?;

        let mut fields = Vec::new();

        let key_schema = table.key_schema();
        let attr_defs = table.attribute_definitions();

        for (idx, key) in key_schema.iter().enumerate() {
            let key_attr_name: &str = key.attribute_name();
            let data_type = attr_defs
                .iter()
                .find(|a| a.attribute_name() == key_attr_name)
                .map(|a| format!("{:?}", a.attribute_type()))
                .unwrap_or_else(|| "S".to_string());

            fields.push(FieldInfo {
                name: key_attr_name.to_string(),
                data_type,
                is_nullable: false,
                is_primary: true,
                default_value: None,
                ordinal_position: (idx + 1) as i32,
            });
        }

        // Also scan a few items to discover additional fields
        let scan_result = self
            .client
            .scan()
            .table_name(item)
            .limit(10)
            .send()
            .await
            .ok();

        if let Some(scan) = scan_result {
            let existing_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
            let mut extra_idx = fields.len();

            for scanned_item in scan.items() {
                for key in scanned_item.keys() {
                    if !existing_names.contains(key) && !fields.iter().any(|f| f.name == *key) {
                        extra_idx += 1;
                        fields.push(FieldInfo {
                            name: key.clone(),
                            data_type: "mixed".to_string(),
                            is_nullable: true,
                            is_primary: false,
                            default_value: None,
                            ordinal_position: extra_idx as i32,
                        });
                    }
                }
            }
        }

        Ok(fields)
    }

    async fn get_item_data(&self, _container: &str, item: &str, limit: i64, _offset: i64) -> Result<QueryResponse, AppError> {
        let start = Instant::now();

        let result = self
            .client
            .scan()
            .table_name(item)
            .limit(limit as i32)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("DynamoDB scan error: {}", e)))?;

        let elapsed = start.elapsed().as_millis() as u64;
        let items = result.items();

        let mut all_keys: Vec<String> = Vec::new();
        for scan_item in items {
            for key in scan_item.keys() {
                if !all_keys.contains(key) {
                    all_keys.push(key.clone());
                }
            }
        }

        let columns: Vec<ColumnDef> = all_keys
            .iter()
            .map(|k| ColumnDef {
                name: k.clone(),
                data_type: "mixed".to_string(),
            })
            .collect();

        let rows: Vec<Vec<CellValue>> = items
            .iter()
            .map(|scan_item| {
                all_keys
                    .iter()
                    .map(|key| {
                        scan_item
                            .get(key)
                            .map(|v| Self::attribute_to_cell(v))
                            .unwrap_or(CellValue::Null)
                    })
                    .collect()
            })
            .collect();

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

    async fn get_item_count(&self, _container: &str, item: &str) -> Result<i64, AppError> {
        let result = self
            .client
            .describe_table()
            .table_name(item)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("DynamoDB describe error: {}", e)))?;

        Ok(result
            .table()
            .and_then(|t| t.item_count())
            .unwrap_or(0))
    }

    async fn health_check(&self) -> Result<(), AppError> {
        self.client
            .list_tables()
            .limit(1)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("DynamoDB health check failed: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl DocumentDriver for DynamoDbDriver {
    async fn insert_document(
        &self,
        _container: &str,
        collection: &str,
        document: serde_json::Value,
    ) -> Result<String, AppError> {
        let item: HashMap<String, AttributeValue> = if let serde_json::Value::Object(map) = document {
            map.iter()
                .map(|(k, v)| (k.clone(), Self::json_to_attribute(v)))
                .collect()
        } else {
            return Err(AppError::InvalidConfig("Document must be a JSON object".to_string()));
        };

        self.client
            .put_item()
            .table_name(collection)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| AppError::Database(format!("DynamoDB put item error: {}", e)))?;

        Ok("inserted".to_string())
    }

    async fn update_document(
        &self,
        _container: &str,
        collection: &str,
        filter: serde_json::Value,
        update: serde_json::Value,
    ) -> Result<u64, AppError> {
        // Convert filter to key
        let key: HashMap<String, AttributeValue> = if let serde_json::Value::Object(map) = filter {
            map.iter()
                .map(|(k, v)| (k.clone(), Self::json_to_attribute(v)))
                .collect()
        } else {
            return Err(AppError::InvalidConfig("Filter must be a JSON object".to_string()));
        };

        // Build update expression
        let updates: HashMap<String, AttributeValue> = if let serde_json::Value::Object(map) = update {
            map.iter()
                .map(|(k, v)| (k.clone(), Self::json_to_attribute(v)))
                .collect()
        } else {
            return Err(AppError::InvalidConfig("Update must be a JSON object".to_string()));
        };

        let mut update_expr_parts = Vec::new();
        let mut expr_attr_values = HashMap::new();
        let mut expr_attr_names = HashMap::new();

        for (idx, (k, v)) in updates.iter().enumerate() {
            let name_placeholder = format!("#attr{}", idx);
            let value_placeholder = format!(":val{}", idx);
            update_expr_parts.push(format!("{} = {}", name_placeholder, value_placeholder));
            expr_attr_names.insert(name_placeholder, k.clone());
            expr_attr_values.insert(value_placeholder, v.clone());
        }

        let update_expression = format!("SET {}", update_expr_parts.join(", "));

        let mut req = self.client
            .update_item()
            .table_name(collection)
            .set_key(Some(key))
            .update_expression(update_expression);

        for (k, v) in expr_attr_names {
            req = req.expression_attribute_names(k, v);
        }
        for (k, v) in expr_attr_values {
            req = req.expression_attribute_values(k, v);
        }

        req.send()
            .await
            .map_err(|e| AppError::Database(format!("DynamoDB update error: {}", e)))?;

        Ok(1)
    }

    async fn delete_documents(
        &self,
        _container: &str,
        collection: &str,
        filter: serde_json::Value,
    ) -> Result<u64, AppError> {
        let key: HashMap<String, AttributeValue> = if let serde_json::Value::Object(map) = filter {
            map.iter()
                .map(|(k, v)| (k.clone(), Self::json_to_attribute(v)))
                .collect()
        } else {
            return Err(AppError::InvalidConfig("Filter must be a JSON object".to_string()));
        };

        self.client
            .delete_item()
            .table_name(collection)
            .set_key(Some(key))
            .send()
            .await
            .map_err(|e| AppError::Database(format!("DynamoDB delete error: {}", e)))?;

        Ok(1)
    }
}
