use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Query timed out after {0} seconds")]
    QueryTimeout(u64),

    #[error("Query cancelled")]
    QueryCancelled,

    #[allow(dead_code)]
    #[error("Failed to connect to {db_type} at {host}: {cause}")]
    ConnectionFailed {
        db_type: String,
        host: String,
        cause: String,
    },

    #[error("SSH tunnel error: {0}")]
    SshTunnel(String),

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[allow(dead_code)]
    #[error("Connection lost: {0}")]
    ConnectionLost(String),
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::ConnectionNotFound(_) => "CONNECTION_NOT_FOUND",
            AppError::InvalidConfig(_) => "INVALID_CONFIG",
            AppError::Serialization(_) => "SERIALIZATION_ERROR",
            AppError::UnsupportedOperation(_) => "UNSUPPORTED_OPERATION",
            AppError::QueryTimeout(_) => "QUERY_TIMEOUT",
            AppError::QueryCancelled => "QUERY_CANCELLED",
            AppError::ConnectionFailed { .. } => "CONNECTION_FAILED",
            AppError::SshTunnel(_) => "SSH_TUNNEL_ERROR",
            AppError::Keychain(_) => "KEYCHAIN_ERROR",
            AppError::ConnectionLost(_) => "CONNECTION_LOST",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("code", self.error_code())?;
        map.serialize_entry("message", &self.to_string())?;
        map.end()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl From<tiberius::error::Error> for AppError {
    fn from(err: tiberius::error::Error) -> Self {
        AppError::Database(format!("MSSQL error: {}", err))
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        AppError::Database(format!("MongoDB error: {}", err))
    }
}

impl From<scylla::transport::errors::NewSessionError> for AppError {
    fn from(err: scylla::transport::errors::NewSessionError) -> Self {
        AppError::Database(format!("Cassandra error: {}", err))
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::Database(format!("Redis error: {}", err))
    }
}

impl From<neo4rs::Error> for AppError {
    fn from(err: neo4rs::Error) -> Self {
        AppError::Database(format!("Neo4j error: {}", err))
    }
}

impl<E: std::fmt::Display> From<aws_sdk_dynamodb::error::SdkError<E>> for AppError {
    fn from(err: aws_sdk_dynamodb::error::SdkError<E>) -> Self {
        AppError::Database(format!("DynamoDB error: {}", err))
    }
}

impl From<clickhouse::error::Error> for AppError {
    fn from(err: clickhouse::error::Error) -> Self {
        AppError::Database(format!("ClickHouse error: {}", err))
    }
}
