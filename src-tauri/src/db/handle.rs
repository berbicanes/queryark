use std::sync::Arc;

use crate::error::AppError;

use super::traits::{DbDriver, DocumentDriver, GraphDriver, KeyValueDriver, SqlDriver};

/// Typed wrapper that knows which trait category a driver supports.
pub enum DriverHandle {
    Sql(Arc<dyn SqlDriver>),
    Document(Arc<dyn DocumentDriver>),
    KeyValue(Arc<dyn KeyValueDriver>),
    Graph(Arc<dyn GraphDriver>),
}

impl DriverHandle {
    /// Access the base DbDriver trait object.
    pub fn base(&self) -> &dyn DbDriver {
        match self {
            DriverHandle::Sql(d) => d.as_ref() as &dyn DbDriver,
            DriverHandle::Document(d) => d.as_ref() as &dyn DbDriver,
            DriverHandle::KeyValue(d) => d.as_ref() as &dyn DbDriver,
            DriverHandle::Graph(d) => d.as_ref() as &dyn DbDriver,
        }
    }

    pub fn as_sql(&self) -> Result<&dyn SqlDriver, AppError> {
        match self {
            DriverHandle::Sql(d) => Ok(d.as_ref()),
            _ => Err(AppError::UnsupportedOperation(
                "This database does not support SQL operations".to_string(),
            )),
        }
    }

    pub fn as_document(&self) -> Result<&dyn DocumentDriver, AppError> {
        match self {
            DriverHandle::Document(d) => Ok(d.as_ref()),
            _ => Err(AppError::UnsupportedOperation(
                "This database does not support document operations".to_string(),
            )),
        }
    }

    pub fn as_keyvalue(&self) -> Result<&dyn KeyValueDriver, AppError> {
        match self {
            DriverHandle::KeyValue(d) => Ok(d.as_ref()),
            _ => Err(AppError::UnsupportedOperation(
                "This database does not support key-value operations".to_string(),
            )),
        }
    }

    pub fn as_graph(&self) -> Result<&dyn GraphDriver, AppError> {
        match self {
            DriverHandle::Graph(d) => Ok(d.as_ref()),
            _ => Err(AppError::UnsupportedOperation(
                "This database does not support graph operations".to_string(),
            )),
        }
    }

    pub async fn begin_transaction(&self) -> Result<(), AppError> {
        self.as_sql()?.begin_transaction().await
    }

    pub async fn commit_transaction(&self) -> Result<(), AppError> {
        self.as_sql()?.commit_transaction().await
    }

    pub async fn rollback_transaction(&self) -> Result<(), AppError> {
        self.as_sql()?.rollback_transaction().await
    }
}
