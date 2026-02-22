import type { DatabaseType } from '$lib/types/connection';

export function quoteIdentifier(name: string, dbType: DatabaseType): string {
  switch (dbType) {
    case 'MySQL':
    case 'MariaDB':
      return `\`${name.replace(/`/g, '``')}\``;
    case 'MSSQL':
      return `[${name.replace(/\]/g, ']]')}]`;
    default:
      // PostgreSQL, SQLite, Oracle, CockroachDB, Redshift, ClickHouse, Snowflake, BigQuery, Cassandra, ScyllaDB
      return `"${name.replace(/"/g, '""')}"`;
  }
}
