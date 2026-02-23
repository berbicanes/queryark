use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MariaDB,
    SQLite,
    MSSQL,
    Oracle,
    CockroachDB,
    Redshift,
    ClickHouse,
    Snowflake,
    BigQuery,
    MongoDB,
    Cassandra,
    Redis,
    Neo4j,
    DynamoDB,
    ScyllaDB,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseCategory {
    Relational,
    Analytics,
    Document,
    KeyValue,
    Graph,
    WideColumn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudAuth {
    GcpServiceAccount { credentials_json: String },
    AwsCredentials { access_key: String, secret_key: String, region: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub db_type: DatabaseType,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub use_ssl: bool,
    // SQLite
    #[serde(default)]
    pub file_path: Option<String>,
    // Oracle
    #[serde(default)]
    pub oracle_sid: Option<String>,
    #[serde(default)]
    pub oracle_service_name: Option<String>,
    // Snowflake
    #[serde(default)]
    pub snowflake_account: Option<String>,
    #[serde(default)]
    pub snowflake_warehouse: Option<String>,
    #[serde(default)]
    pub snowflake_role: Option<String>,
    // Neo4j
    #[serde(default)]
    pub bolt_url: Option<String>,
    // Cloud auth (BigQuery, DynamoDB)
    #[serde(default)]
    pub cloud_auth: Option<CloudAuth>,
    // AWS region for DynamoDB
    #[serde(default)]
    pub aws_region: Option<String>,
    // SSH tunneling
    #[serde(default)]
    pub ssh_enabled: bool,
    #[serde(default)]
    pub ssh_host: Option<String>,
    #[serde(default)]
    pub ssh_port: Option<u16>,
    #[serde(default)]
    pub ssh_user: Option<String>,
    #[serde(default)]
    pub ssh_password: Option<String>,
    #[serde(default)]
    pub ssh_key_path: Option<String>,
    #[serde(default)]
    pub ssh_passphrase: Option<String>,
    // SSL certificates
    #[serde(default)]
    pub ssl_ca_cert: Option<String>,
    #[serde(default)]
    pub ssl_client_cert: Option<String>,
    #[serde(default)]
    pub ssl_client_key: Option<String>,
    // OS keychain
    #[serde(default)]
    pub use_keychain: bool,
    // Connection pool tuning
    #[serde(default = "default_pool_size")]
    pub pool_max_connections: u32,
    #[serde(default = "default_idle_timeout")]
    pub pool_idle_timeout_secs: u64,
    #[serde(default = "default_acquire_timeout")]
    pub pool_acquire_timeout_secs: u64,
}

fn default_pool_size() -> u32 {
    5
}
fn default_idle_timeout() -> u64 {
    300
}
fn default_acquire_timeout() -> u64 {
    10
}

impl DatabaseType {
    pub fn default_port(&self) -> Option<u16> {
        match self {
            DatabaseType::PostgreSQL | DatabaseType::CockroachDB => Some(5432),
            DatabaseType::Redshift => Some(5439),
            DatabaseType::MySQL | DatabaseType::MariaDB => Some(3306),
            DatabaseType::MSSQL => Some(1433),
            DatabaseType::Oracle => Some(1521),
            DatabaseType::ClickHouse => Some(8123),
            DatabaseType::MongoDB => Some(27017),
            DatabaseType::Cassandra | DatabaseType::ScyllaDB => Some(9042),
            DatabaseType::Redis => Some(6379),
            DatabaseType::Neo4j => Some(7687),
            DatabaseType::Snowflake | DatabaseType::BigQuery | DatabaseType::DynamoDB => None,
            DatabaseType::SQLite => None,
        }
    }
}

impl ConnectionConfig {
    pub fn host_or_default(&self) -> &str {
        self.host.as_deref().unwrap_or("localhost")
    }

    pub fn port_or_default(&self) -> u16 {
        self.port.unwrap_or_else(|| self.db_type.default_port().unwrap_or(0))
    }

    pub fn username_or_default(&self) -> &str {
        self.username.as_deref().unwrap_or("")
    }

    pub fn password_or_default(&self) -> &str {
        self.password.as_deref().unwrap_or("")
    }

    pub fn database_or_default(&self) -> &str {
        self.database.as_deref().unwrap_or("")
    }

    pub fn to_connection_url(&self) -> String {
        match self.db_type {
            DatabaseType::PostgreSQL | DatabaseType::CockroachDB | DatabaseType::Redshift => {
                let ssl_mode = if self.use_ssl { "require" } else { "disable" };
                let mut url = format!(
                    "postgres://{}:{}@{}:{}/{}?sslmode={}",
                    self.username_or_default(),
                    self.password_or_default(),
                    self.host_or_default(),
                    self.port_or_default(),
                    self.database_or_default(),
                    ssl_mode
                );
                if let Some(ref ca) = self.ssl_ca_cert {
                    url.push_str(&format!("&sslrootcert={}", ca));
                }
                if let Some(ref cert) = self.ssl_client_cert {
                    url.push_str(&format!("&sslcert={}", cert));
                }
                if let Some(ref key) = self.ssl_client_key {
                    url.push_str(&format!("&sslkey={}", key));
                }
                url
            }
            DatabaseType::MySQL | DatabaseType::MariaDB => {
                let mut url = format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username_or_default(),
                    self.password_or_default(),
                    self.host_or_default(),
                    self.port_or_default(),
                    self.database_or_default(),
                );
                let mut params: Vec<String> = Vec::new();
                if self.use_ssl {
                    params.push("ssl-mode=REQUIRED".to_string());
                }
                if let Some(ref ca) = self.ssl_ca_cert {
                    params.push(format!("ssl-ca={}", ca));
                }
                if let Some(ref cert) = self.ssl_client_cert {
                    params.push(format!("ssl-cert={}", cert));
                }
                if let Some(ref key) = self.ssl_client_key {
                    params.push(format!("ssl-key={}", key));
                }
                if !params.is_empty() {
                    url.push('?');
                    url.push_str(&params.join("&"));
                }
                url
            }
            DatabaseType::SQLite => {
                if let Some(ref path) = self.file_path {
                    format!("sqlite:{}", path)
                } else {
                    "sqlite::memory:".to_string()
                }
            }
            DatabaseType::MSSQL => {
                format!(
                    "server=tcp:{},{};database={};user={};password={};TrustServerCertificate=true",
                    self.host_or_default(),
                    self.port_or_default(),
                    self.database_or_default(),
                    self.username_or_default(),
                    self.password_or_default()
                )
            }
            DatabaseType::ClickHouse => {
                format!(
                    "http://{}:{}",
                    self.host_or_default(),
                    self.port_or_default()
                )
            }
            DatabaseType::MongoDB => {
                let auth = if !self.username_or_default().is_empty() {
                    format!(
                        "{}:{}@",
                        self.username_or_default(),
                        self.password_or_default()
                    )
                } else {
                    String::new()
                };
                format!(
                    "mongodb://{}{}:{}",
                    auth,
                    self.host_or_default(),
                    self.port_or_default()
                )
            }
            DatabaseType::Redis => {
                if !self.password_or_default().is_empty() {
                    format!(
                        "redis://:{}@{}:{}/{}",
                        self.password_or_default(),
                        self.host_or_default(),
                        self.port_or_default(),
                        self.database_or_default()
                    )
                } else {
                    format!(
                        "redis://{}:{}/{}",
                        self.host_or_default(),
                        self.port_or_default(),
                        self.database_or_default()
                    )
                }
            }
            DatabaseType::Neo4j => {
                if let Some(ref bolt) = self.bolt_url {
                    bolt.clone()
                } else {
                    format!("bolt://{}:{}", self.host_or_default(), self.port_or_default())
                }
            }
            DatabaseType::Cassandra | DatabaseType::ScyllaDB => {
                format!("{}:{}", self.host_or_default(), self.port_or_default())
            }
            DatabaseType::Oracle => {
                if let Some(ref service) = self.oracle_service_name {
                    format!(
                        "//{}:{}/{}",
                        self.host_or_default(),
                        self.port_or_default(),
                        service
                    )
                } else if let Some(ref sid) = self.oracle_sid {
                    format!(
                        "(DESCRIPTION=(ADDRESS=(PROTOCOL=TCP)(HOST={})(PORT={}))(CONNECT_DATA=(SID={})))",
                        self.host_or_default(),
                        self.port_or_default(),
                        sid
                    )
                } else {
                    format!(
                        "//{}:{}/{}",
                        self.host_or_default(),
                        self.port_or_default(),
                        self.database_or_default()
                    )
                }
            }
            DatabaseType::Snowflake | DatabaseType::BigQuery | DatabaseType::DynamoDB => {
                String::new()
            }
        }
    }
}
