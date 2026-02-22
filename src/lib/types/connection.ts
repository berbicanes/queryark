export type DatabaseType =
  | 'PostgreSQL'
  | 'MySQL'
  | 'MariaDB'
  | 'SQLite'
  | 'MSSQL'
  | 'Oracle'
  | 'CockroachDB'
  | 'Redshift'
  | 'ClickHouse'
  | 'Snowflake'
  | 'BigQuery'
  | 'MongoDB'
  | 'Cassandra'
  | 'Redis'
  | 'Neo4j'
  | 'DynamoDB'
  | 'ScyllaDB';

export type DatabaseCategory =
  | 'Relational'
  | 'Analytics'
  | 'Document'
  | 'KeyValue'
  | 'Graph'
  | 'WideColumn';

export interface CloudAuth {
  GcpServiceAccount?: { credentials_json: string };
  AwsCredentials?: { access_key: string; secret_key: string; region: string };
}

export interface ConnectionConfig {
  id: string;
  name: string;
  db_type: DatabaseType;
  host?: string;
  port?: number;
  username?: string;
  password?: string;
  database?: string;
  use_ssl: boolean;
  // SQLite
  file_path?: string;
  // Oracle
  oracle_sid?: string;
  oracle_service_name?: string;
  // Snowflake
  snowflake_account?: string;
  snowflake_warehouse?: string;
  snowflake_role?: string;
  // Neo4j
  bolt_url?: string;
  // Cloud auth
  cloud_auth?: CloudAuth;
  // AWS
  aws_region?: string;
}

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface ConnectionState {
  config: ConnectionConfig;
  status: ConnectionStatus;
  error?: string;
}
