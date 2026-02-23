use std::time::Instant;

use async_trait::async_trait;
use neo4rs::{Graph, ConfigBuilder};

use crate::db::traits::{DbDriver, GraphDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{ContainerInfo, FieldInfo, ItemInfo};

pub struct Neo4jDriver {
    graph: Graph,
}

impl Neo4jDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let uri = config.to_connection_url();

        let config_builder = ConfigBuilder::default()
            .uri(&uri)
            .user(config.username_or_default())
            .password(config.password_or_default())
            .build()
            .map_err(|e| AppError::Database(format!("Failed to build Neo4j config: {}", e)))?;

        let graph = Graph::connect(config_builder)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to Neo4j: {}", e)))?;

        Ok(Self { graph })
    }

    fn bolt_value_to_cell(value: &neo4rs::BoltType) -> CellValue {
        use neo4rs::BoltType;
        match value {
            BoltType::Null(_) => CellValue::Null,
            BoltType::Boolean(b) => CellValue::Bool(b.value),
            BoltType::Integer(i) => CellValue::Int(i.value),
            BoltType::Float(f) => CellValue::Float(f.value),
            BoltType::String(s) => CellValue::Text(s.value.clone()),
            BoltType::List(l) => {
                let items: Vec<serde_json::Value> = l.value
                    .iter()
                    .map(|v| bolt_to_json(v))
                    .collect();
                CellValue::Json(serde_json::to_string(&items).unwrap_or_default())
            }
            BoltType::Map(m) => {
                let map: serde_json::Map<String, serde_json::Value> = m.value
                    .iter()
                    .map(|(k, v)| (k.value.clone(), bolt_to_json(v)))
                    .collect();
                CellValue::Json(serde_json::to_string(&map).unwrap_or_default())
            }
            BoltType::Node(n) => {
                let mut map = serde_json::Map::new();
                map.insert("_id".to_string(), serde_json::Value::Number(n.id.value.into()));
                let labels: Vec<String> = n.labels.value.iter().map(|l| format!("{}", l)).collect();
                map.insert("_labels".to_string(), serde_json::to_value(&labels).unwrap_or_default());
                for (k, v) in &n.properties.value {
                    map.insert(k.value.clone(), bolt_to_json(v));
                }
                CellValue::Json(serde_json::to_string(&map).unwrap_or_default())
            }
            BoltType::Relation(r) => {
                let mut map = serde_json::Map::new();
                map.insert("_id".to_string(), serde_json::Value::Number(r.id.value.into()));
                map.insert("_type".to_string(), serde_json::Value::String(r.typ.value.clone()));
                map.insert("_start".to_string(), serde_json::Value::Number(r.start_node_id.value.into()));
                map.insert("_end".to_string(), serde_json::Value::Number(r.end_node_id.value.into()));
                for (k, v) in &r.properties.value {
                    map.insert(k.value.clone(), bolt_to_json(v));
                }
                CellValue::Json(serde_json::to_string(&map).unwrap_or_default())
            }
            _ => CellValue::Text(format!("{:?}", value)),
        }
    }
}

fn bolt_to_json(value: &neo4rs::BoltType) -> serde_json::Value {
    use neo4rs::BoltType;
    match value {
        BoltType::Null(_) => serde_json::Value::Null,
        BoltType::Boolean(b) => serde_json::Value::Bool(b.value),
        BoltType::Integer(i) => serde_json::Value::Number(i.value.into()),
        BoltType::Float(f) => serde_json::Number::from_f64(f.value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        BoltType::String(s) => serde_json::Value::String(s.value.clone()),
        BoltType::List(l) => {
            let items: Vec<serde_json::Value> = l.value.iter().map(|v| bolt_to_json(v)).collect();
            serde_json::Value::Array(items)
        }
        _ => serde_json::Value::String(format!("{:?}", value)),
    }
}

#[async_trait]
impl DbDriver for Neo4jDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Graph
    }

    fn dialect_hint(&self) -> &'static str {
        "neo4j"
    }

    async fn execute_raw(&self, query: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let trimmed = query.trim();

        let mut result = self
            .graph
            .execute(neo4rs::query(trimmed))
            .await
            .map_err(|e| AppError::Database(format!("Neo4j query error: {}", e)))?;

        let mut columns: Vec<ColumnDef> = Vec::new();
        let mut column_keys: Vec<String> = Vec::new();
        let mut rows: Vec<Vec<CellValue>> = Vec::new();
        let mut columns_set = false;

        while let Ok(Some(row)) = result.next().await {
            // Deserialize the row as a BoltMap to access its keys and values
            let bolt_map: neo4rs::BoltMap = match row.to::<neo4rs::BoltMap>() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if !columns_set {
                column_keys = bolt_map.value.keys().map(|k| k.value.clone()).collect::<Vec<_>>();
                column_keys.sort(); // Ensure consistent column ordering
                columns = column_keys
                    .iter()
                    .map(|k| ColumnDef {
                        name: k.clone(),
                        data_type: "mixed".to_string(),
                    })
                    .collect();
                columns_set = true;
            }

            let cells: Vec<CellValue> = column_keys
                .iter()
                .map(|k| {
                    let bolt_key = neo4rs::BoltString::new(k);
                    match bolt_map.value.get(&bolt_key) {
                        Some(val) => Neo4jDriver::bolt_value_to_cell(val),
                        None => CellValue::Null,
                    }
                })
                .collect();

            rows.push(cells);
        }

        let elapsed = start.elapsed().as_millis() as u64;
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
        Ok(vec![ContainerInfo {
            name: "neo4j".to_string(),
            container_type: "database".to_string(),
        }])
    }

    async fn get_items(&self, _container: &str) -> Result<Vec<ItemInfo>, AppError> {
        let labels = self.get_labels().await?;
        Ok(labels
            .into_iter()
            .map(|name| ItemInfo {
                name,
                container: "neo4j".to_string(),
                item_type: "label".to_string(),
                item_count: None,
            })
            .collect())
    }

    async fn get_item_fields(&self, _container: &str, item: &str) -> Result<Vec<FieldInfo>, AppError> {
        let props = self.get_node_properties(item).await?;
        Ok(props
            .into_iter()
            .enumerate()
            .map(|(idx, name)| FieldInfo {
                name,
                data_type: "mixed".to_string(),
                is_nullable: true,
                is_primary: false,
                default_value: None,
                ordinal_position: (idx + 1) as i32,
            })
            .collect())
    }

    async fn get_item_data(&self, _container: &str, item: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        self.get_nodes(item, limit, offset).await
    }

    async fn get_item_count(&self, _container: &str, item: &str) -> Result<i64, AppError> {
        let query = format!("MATCH (n:`{}`) RETURN count(n) as count", item.replace('`', "``"));
        let response = self.execute_raw(&query).await?;
        if let Some(row) = response.rows.first() {
            if let Some(CellValue::Int(count)) = row.first() {
                return Ok(*count);
            }
        }
        Ok(0)
    }

    async fn health_check(&self) -> Result<(), AppError> {
        self.execute_raw("RETURN 1").await.map(|_| ())
    }
}

#[async_trait]
impl GraphDriver for Neo4jDriver {
    async fn get_labels(&self) -> Result<Vec<String>, AppError> {
        let response = self.execute_raw("CALL db.labels()").await?;
        let labels: Vec<String> = response
            .rows
            .iter()
            .filter_map(|row| {
                if let Some(CellValue::Text(name)) = row.first() {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(labels)
    }

    async fn get_relationship_types(&self) -> Result<Vec<String>, AppError> {
        let response = self.execute_raw("CALL db.relationshipTypes()").await?;
        let types: Vec<String> = response
            .rows
            .iter()
            .filter_map(|row| {
                if let Some(CellValue::Text(name)) = row.first() {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(types)
    }

    async fn get_node_properties(&self, label: &str) -> Result<Vec<String>, AppError> {
        let query = format!(
            "MATCH (n:`{}`) WITH keys(n) AS keys UNWIND keys AS key RETURN DISTINCT key ORDER BY key LIMIT 100",
            label.replace('`', "``")
        );
        let response = self.execute_raw(&query).await?;
        let props: Vec<String> = response
            .rows
            .iter()
            .filter_map(|row| {
                if let Some(CellValue::Text(name)) = row.first() {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(props)
    }

    async fn get_nodes(&self, label: &str, limit: i64, offset: i64) -> Result<QueryResponse, AppError> {
        let query = format!(
            "MATCH (n:`{}`) RETURN n SKIP {} LIMIT {}",
            label.replace('`', "``"),
            offset,
            limit
        );
        self.execute_raw(&query).await
    }
}
