export interface DatabaseInfo {
  name: string;
  badge: string;
  color: string;
  bgColor: string;
  group: 'SQL' | 'Analytics' | 'NoSQL' | 'Cloud';
}

export const databases: DatabaseInfo[] = [
  { name: 'PostgreSQL', badge: 'PG', color: '#4a9eff', bgColor: 'rgba(74, 158, 255, 0.2)', group: 'SQL' },
  { name: 'MySQL', badge: 'MY', color: '#fbbf24', bgColor: 'rgba(251, 191, 36, 0.2)', group: 'SQL' },
  { name: 'MariaDB', badge: 'MA', color: '#cba6f7', bgColor: 'rgba(203, 166, 247, 0.2)', group: 'SQL' },
  { name: 'SQLite', badge: 'SL', color: '#94e2d5', bgColor: 'rgba(148, 226, 213, 0.2)', group: 'SQL' },
  { name: 'SQL Server', badge: 'MS', color: '#f87171', bgColor: 'rgba(248, 113, 113, 0.2)', group: 'SQL' },
  { name: 'CockroachDB', badge: 'CR', color: '#4ade80', bgColor: 'rgba(74, 222, 128, 0.2)', group: 'SQL' },
  { name: 'Redshift', badge: 'RS', color: '#f87171', bgColor: 'rgba(248, 113, 113, 0.2)', group: 'Analytics' },
  { name: 'ClickHouse', badge: 'CH', color: '#fbbf24', bgColor: 'rgba(251, 191, 36, 0.2)', group: 'Analytics' },
  { name: 'Snowflake', badge: 'SF', color: '#89dceb', bgColor: 'rgba(137, 220, 235, 0.2)', group: 'Analytics' },
  { name: 'BigQuery', badge: 'BQ', color: '#74c7ec', bgColor: 'rgba(116, 199, 236, 0.2)', group: 'Analytics' },
  { name: 'MongoDB', badge: 'MO', color: '#4ade80', bgColor: 'rgba(74, 222, 128, 0.2)', group: 'NoSQL' },
  { name: 'Redis', badge: 'RD', color: '#f87171', bgColor: 'rgba(248, 113, 113, 0.2)', group: 'NoSQL' },
  { name: 'Cassandra', badge: 'CA', color: '#89b4fa', bgColor: 'rgba(137, 180, 250, 0.2)', group: 'NoSQL' },
  { name: 'ScyllaDB', badge: 'SC', color: '#fab387', bgColor: 'rgba(250, 179, 135, 0.2)', group: 'NoSQL' },
  { name: 'Neo4j', badge: 'NJ', color: '#94e2d5', bgColor: 'rgba(148, 226, 213, 0.2)', group: 'NoSQL' },
  { name: 'DynamoDB', badge: 'DY', color: '#fbbf24', bgColor: 'rgba(251, 191, 36, 0.2)', group: 'Cloud' },
];

export const dbGroups = ['SQL', 'Analytics', 'NoSQL', 'Cloud'] as const;
