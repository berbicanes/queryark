use std::time::Instant;

use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{Client, options::ClientOptions};

use crate::db::traits::{DbDriver, DocumentDriver};
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, DatabaseCategory};
use crate::models::query::{CellValue, ColumnDef, QueryResponse};
use crate::models::schema::{ContainerInfo, FieldInfo, ItemInfo};

pub struct MongoDbDriver {
    client: Client,
}

impl MongoDbDriver {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, AppError> {
        let url = config.to_connection_url();
        let mut options = ClientOptions::parse(&url)
            .await
            .map_err(|e| AppError::Database(format!("Failed to parse MongoDB URL: {}", e)))?;

        // Configure TLS with custom certificates if provided
        if config.ssl_ca_cert.is_some() || config.ssl_client_cert.is_some() {
            let mut tls_options = mongodb::options::TlsOptions::default();

            if let Some(ref ca_path) = config.ssl_ca_cert {
                tls_options.ca_file_path = Some(std::path::PathBuf::from(ca_path));
            }
            if let Some(ref cert_path) = config.ssl_client_cert {
                tls_options.cert_key_file_path = Some(std::path::PathBuf::from(cert_path));
            }

            options.tls = Some(mongodb::options::Tls::Enabled(tls_options));
        } else if config.use_ssl {
            options.tls = Some(mongodb::options::Tls::Enabled(
                mongodb::options::TlsOptions::default(),
            ));
        }

        let client = Client::with_options(options)
            .map_err(|e| AppError::Database(format!("Failed to create MongoDB client: {}", e)))?;

        // Test connection
        client
            .list_database_names()
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to MongoDB: {}", e)))?;

        Ok(Self { client })
    }

    fn bson_to_cell(value: &mongodb::bson::Bson) -> CellValue {
        use mongodb::bson::Bson;
        match value {
            Bson::Null => CellValue::Null,
            Bson::Boolean(b) => CellValue::Bool(*b),
            Bson::Int32(i) => CellValue::Int(*i as i64),
            Bson::Int64(i) => CellValue::Int(*i),
            Bson::Double(f) => CellValue::Float(*f),
            Bson::String(s) => CellValue::Text(s.clone()),
            Bson::ObjectId(oid) => CellValue::Text(oid.to_hex()),
            Bson::DateTime(dt) => CellValue::Timestamp(dt.to_string()),
            Bson::Binary(bin) => CellValue::Binary(bin.bytes.clone()),
            Bson::Document(doc) => CellValue::Json(serde_json::to_string(doc).unwrap_or_default()),
            Bson::Array(arr) => CellValue::Json(serde_json::to_string(arr).unwrap_or_default()),
            _ => CellValue::Text(value.to_string()),
        }
    }
}

#[async_trait]
impl DbDriver for MongoDbDriver {
    fn category(&self) -> DatabaseCategory {
        DatabaseCategory::Document
    }

    async fn execute_raw(&self, query: &str) -> Result<QueryResponse, AppError> {
        let start = Instant::now();

        // Parse JSON query: { "database": "db", "collection": "col", "operation": "find", "filter": {} }
        let cmd: serde_json::Value = serde_json::from_str(query)
            .map_err(|e| AppError::InvalidConfig(format!("Invalid JSON query: {}", e)))?;

        let db_name = cmd["database"].as_str().unwrap_or("test");
        let coll_name = cmd["collection"].as_str().unwrap_or("");
        let operation = cmd["operation"].as_str().unwrap_or("find");

        let db = self.client.database(db_name);

        match operation {
            "find" => {
                if coll_name.is_empty() {
                    return Err(AppError::InvalidConfig("Collection name required".to_string()));
                }
                let collection = db.collection::<mongodb::bson::Document>(coll_name);
                let filter = if let Some(f) = cmd.get("filter") {
                    mongodb::bson::to_document(f)
                        .map_err(|e| AppError::InvalidConfig(format!("Invalid filter: {}", e)))?
                } else {
                    mongodb::bson::doc! {}
                };

                let limit = cmd["limit"].as_i64().unwrap_or(50);

                let cursor = collection
                    .find(filter)
                    .limit(limit)
                    .await
                    .map_err(|e| AppError::Database(format!("MongoDB find error: {}", e)))?;

                let docs: Vec<mongodb::bson::Document> = cursor
                    .try_collect()
                    .await
                    .map_err(|e| AppError::Database(format!("MongoDB cursor error: {}", e)))?;

                // Collect all unique keys from all documents
                let mut all_keys: Vec<String> = Vec::new();
                for doc in &docs {
                    for key in doc.keys() {
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

                let rows: Vec<Vec<CellValue>> = docs
                    .iter()
                    .map(|doc| {
                        all_keys
                            .iter()
                            .map(|key| {
                                doc.get(key)
                                    .map(|v| Self::bson_to_cell(v))
                                    .unwrap_or(CellValue::Null)
                            })
                            .collect()
                    })
                    .collect();

                let elapsed = start.elapsed().as_millis() as u64;
                let row_count = rows.len();

                Ok(QueryResponse {
                    columns,
                    rows,
                    row_count,
                    execution_time_ms: elapsed,
                    affected_rows: None,
                })
            }
            _ => Err(AppError::UnsupportedOperation(format!(
                "Unsupported MongoDB operation: {}",
                operation
            ))),
        }
    }

    async fn get_containers(&self) -> Result<Vec<ContainerInfo>, AppError> {
        let db_names = self
            .client
            .list_database_names()
            .await
            .map_err(|e| AppError::Database(format!("Failed to list databases: {}", e)))?;

        Ok(db_names
            .into_iter()
            .filter(|name| name != "admin" && name != "local" && name != "config")
            .map(|name| ContainerInfo {
                name,
                container_type: "database".to_string(),
            })
            .collect())
    }

    async fn get_items(&self, container: &str) -> Result<Vec<ItemInfo>, AppError> {
        let db = self.client.database(container);
        let collections = db
            .list_collection_names()
            .await
            .map_err(|e| AppError::Database(format!("Failed to list collections: {}", e)))?;

        Ok(collections
            .into_iter()
            .map(|name| ItemInfo {
                name,
                container: container.to_string(),
                item_type: "collection".to_string(),
                item_count: None,
            })
            .collect())
    }

    async fn get_item_fields(
        &self,
        container: &str,
        item: &str,
    ) -> Result<Vec<FieldInfo>, AppError> {
        let db = self.client.database(container);
        let collection = db.collection::<mongodb::bson::Document>(item);

        // Sample documents to discover fields
        let cursor = collection
            .find(mongodb::bson::doc! {})
            .limit(100)
            .await
            .map_err(|e| AppError::Database(format!("MongoDB find error: {}", e)))?;

        let docs: Vec<mongodb::bson::Document> = cursor
            .try_collect()
            .await
            .map_err(|e| AppError::Database(format!("MongoDB cursor error: {}", e)))?;

        let mut field_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for doc in &docs {
            for (key, value) in doc {
                field_map.entry(key.clone()).or_insert_with(|| {
                    use mongodb::bson::Bson;
                    match value {
                        Bson::Null => "null",
                        Bson::Boolean(_) => "boolean",
                        Bson::Int32(_) => "int32",
                        Bson::Int64(_) => "int64",
                        Bson::Double(_) => "double",
                        Bson::String(_) => "string",
                        Bson::ObjectId(_) => "objectId",
                        Bson::DateTime(_) => "date",
                        Bson::Binary(_) => "binary",
                        Bson::Document(_) => "document",
                        Bson::Array(_) => "array",
                        _ => "mixed",
                    }
                    .to_string()
                });
            }
        }

        let mut fields: Vec<FieldInfo> = field_map
            .into_iter()
            .enumerate()
            .map(|(idx, (name, data_type))| FieldInfo {
                name,
                data_type,
                is_nullable: true,
                is_primary: false,
                default_value: None,
                ordinal_position: (idx + 1) as i32,
            })
            .collect();

        // Ensure _id is first and marked as primary
        fields.sort_by(|a, b| {
            if a.name == "_id" {
                std::cmp::Ordering::Less
            } else if b.name == "_id" {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        for (idx, field) in fields.iter_mut().enumerate() {
            field.ordinal_position = (idx + 1) as i32;
            if field.name == "_id" {
                field.is_primary = true;
                field.is_nullable = false;
            }
        }

        Ok(fields)
    }

    async fn get_item_data(
        &self,
        container: &str,
        item: &str,
        limit: i64,
        offset: i64,
    ) -> Result<QueryResponse, AppError> {
        let start = Instant::now();
        let db = self.client.database(container);
        let collection = db.collection::<mongodb::bson::Document>(item);

        let cursor = collection
            .find(mongodb::bson::doc! {})
            .skip(offset as u64)
            .limit(limit)
            .await
            .map_err(|e| AppError::Database(format!("MongoDB find error: {}", e)))?;

        let docs: Vec<mongodb::bson::Document> = cursor
            .try_collect()
            .await
            .map_err(|e| AppError::Database(format!("MongoDB cursor error: {}", e)))?;

        let mut all_keys: Vec<String> = Vec::new();
        for doc in &docs {
            for key in doc.keys() {
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

        let rows: Vec<Vec<CellValue>> = docs
            .iter()
            .map(|doc| {
                all_keys
                    .iter()
                    .map(|key| {
                        doc.get(key)
                            .map(|v| Self::bson_to_cell(v))
                            .unwrap_or(CellValue::Null)
                    })
                    .collect()
            })
            .collect();

        let elapsed = start.elapsed().as_millis() as u64;
        let row_count = rows.len();

        Ok(QueryResponse {
            columns,
            rows,
            row_count,
            execution_time_ms: elapsed,
            affected_rows: None,
        })
    }

    async fn get_item_count(&self, container: &str, item: &str) -> Result<i64, AppError> {
        let db = self.client.database(container);
        let collection = db.collection::<mongodb::bson::Document>(item);
        let count = collection
            .count_documents(mongodb::bson::doc! {})
            .await
            .map_err(|e| AppError::Database(format!("MongoDB count error: {}", e)))?;
        Ok(count as i64)
    }

    async fn health_check(&self) -> Result<(), AppError> {
        let db = self.client.database("admin");
        db.run_command(mongodb::bson::doc! { "ping": 1 })
            .await
            .map_err(|e| AppError::Database(format!("MongoDB ping failed: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl DocumentDriver for MongoDbDriver {
    async fn insert_document(
        &self,
        container: &str,
        collection: &str,
        document: serde_json::Value,
    ) -> Result<String, AppError> {
        let db = self.client.database(container);
        let coll = db.collection::<mongodb::bson::Document>(collection);

        let doc = mongodb::bson::to_document(&document)
            .map_err(|e| AppError::InvalidConfig(format!("Invalid document: {}", e)))?;

        let result = coll
            .insert_one(doc)
            .await
            .map_err(|e| AppError::Database(format!("MongoDB insert error: {}", e)))?;

        Ok(result.inserted_id.to_string())
    }

    async fn update_document(
        &self,
        container: &str,
        collection: &str,
        filter: serde_json::Value,
        update: serde_json::Value,
    ) -> Result<u64, AppError> {
        let db = self.client.database(container);
        let coll = db.collection::<mongodb::bson::Document>(collection);

        let filter_doc = mongodb::bson::to_document(&filter)
            .map_err(|e| AppError::InvalidConfig(format!("Invalid filter: {}", e)))?;
        let update_doc = mongodb::bson::to_document(&update)
            .map_err(|e| AppError::InvalidConfig(format!("Invalid update: {}", e)))?;

        let result = coll
            .update_many(filter_doc, update_doc)
            .await
            .map_err(|e| AppError::Database(format!("MongoDB update error: {}", e)))?;

        Ok(result.modified_count)
    }

    async fn delete_documents(
        &self,
        container: &str,
        collection: &str,
        filter: serde_json::Value,
    ) -> Result<u64, AppError> {
        let db = self.client.database(container);
        let coll = db.collection::<mongodb::bson::Document>(collection);

        let filter_doc = mongodb::bson::to_document(&filter)
            .map_err(|e| AppError::InvalidConfig(format!("Invalid filter: {}", e)))?;

        let result = coll
            .delete_many(filter_doc)
            .await
            .map_err(|e| AppError::Database(format!("MongoDB delete error: {}", e)))?;

        Ok(result.deleted_count)
    }
}
