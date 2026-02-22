use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::db::handle::DriverHandle;
use crate::error::AppError;

pub struct PoolManager {
    pools: RwLock<HashMap<String, Arc<DriverHandle>>>,
}

impl PoolManager {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add(&self, id: String, handle: DriverHandle) {
        let mut pools = self.pools.write().await;
        pools.insert(id, Arc::new(handle));
    }

    pub async fn get(&self, id: &str) -> Result<Arc<DriverHandle>, AppError> {
        let pools = self.pools.read().await;
        pools
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::ConnectionNotFound(format!("Connection '{}' not found", id)))
    }

    pub async fn remove(&self, id: &str) -> Result<(), AppError> {
        let mut pools = self.pools.write().await;
        pools
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| AppError::ConnectionNotFound(format!("Connection '{}' not found", id)))
    }
}
