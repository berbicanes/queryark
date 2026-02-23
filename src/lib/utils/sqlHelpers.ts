import type { DatabaseType } from '$lib/types/connection';
import { schemaStore } from '$lib/stores/schema.svelte';

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

/**
 * Returns a schema-qualified table name if the schema differs from the active schema,
 * otherwise returns just the table name.
 */
export function qualifyTableName(
  schemaName: string,
  tableName: string,
  activeSchema: string | null,
  dbType: DatabaseType
): string {
  if (activeSchema && schemaName === activeSchema) {
    return tableName;
  }
  return `${quoteIdentifier(schemaName, dbType)}.${quoteIdentifier(tableName, dbType)}`;
}

/**
 * Build a SQLNamespace object from the schema cache for CodeMirror autocompletion.
 * Returns { "schema.table": ["col1", "col2"], ... } format.
 */
export function buildSqlNamespace(connectionId: string): Record<string, readonly string[]> {
  const namespace: Record<string, string[]> = {};
  const schemas = schemaStore.getSchemas(connectionId);

  for (const schema of schemas) {
    const tables = schemaStore.getTables(connectionId, schema.name);
    for (const table of tables) {
      const columns = schemaStore.getColumns(connectionId, schema.name, table.name);
      const colNames = columns.map(c => c.name);
      // Add as "schema.table" for qualified references
      namespace[`${schema.name}.${table.name}`] = colNames;
      // Also add as just "table" for unqualified references
      if (!namespace[table.name]) {
        namespace[table.name] = colNames;
      }
    }
  }

  return namespace;
}

/**
 * Split a SQL string into individual statements on semicolons,
 * respecting string literals and comments.
 */
export function splitStatements(sql: string): string[] {
  const statements: string[] = [];
  let current = '';
  let inSingleQuote = false;
  let inDoubleQuote = false;
  let inLineComment = false;
  let inBlockComment = false;

  for (let i = 0; i < sql.length; i++) {
    const ch = sql[i];
    const next = sql[i + 1];

    if (inLineComment) {
      current += ch;
      if (ch === '\n') inLineComment = false;
      continue;
    }

    if (inBlockComment) {
      current += ch;
      if (ch === '*' && next === '/') {
        current += '/';
        i++;
        inBlockComment = false;
      }
      continue;
    }

    if (inSingleQuote) {
      current += ch;
      if (ch === "'" && next === "'") {
        current += "'";
        i++; // escaped quote
      } else if (ch === "'") {
        inSingleQuote = false;
      }
      continue;
    }

    if (inDoubleQuote) {
      current += ch;
      if (ch === '"' && next === '"') {
        current += '"';
        i++; // escaped quote
      } else if (ch === '"') {
        inDoubleQuote = false;
      }
      continue;
    }

    // Not inside any special context
    if (ch === "'") {
      inSingleQuote = true;
      current += ch;
    } else if (ch === '"') {
      inDoubleQuote = true;
      current += ch;
    } else if (ch === '-' && next === '-') {
      inLineComment = true;
      current += ch;
    } else if (ch === '/' && next === '*') {
      inBlockComment = true;
      current += ch;
    } else if (ch === ';') {
      const trimmed = current.trim();
      if (trimmed) statements.push(trimmed);
      current = '';
    } else {
      current += ch;
    }
  }

  const trimmed = current.trim();
  if (trimmed) statements.push(trimmed);

  return statements;
}

/**
 * Parse error position from database error messages.
 * Returns the character offset (0-based) or null if not parseable.
 */
export function parseErrorPosition(errorMessage: string, sql: string): { from: number; to: number } | null {
  // PostgreSQL: "... at character 42"
  const pgMatch = errorMessage.match(/at character (\d+)/i);
  if (pgMatch) {
    const pos = parseInt(pgMatch[1], 10) - 1; // PG uses 1-based
    if (pos >= 0 && pos < sql.length) {
      // Highlight from error position to end of the current word/token
      let end = pos + 1;
      while (end < sql.length && /\S/.test(sql[end])) end++;
      return { from: pos, to: Math.max(end, pos + 1) };
    }
  }

  // MySQL: "... at line N"
  const mysqlMatch = errorMessage.match(/at line (\d+)/i);
  if (mysqlMatch) {
    const lineNum = parseInt(mysqlMatch[1], 10) - 1; // Convert to 0-based
    const lines = sql.split('\n');
    if (lineNum >= 0 && lineNum < lines.length) {
      let offset = 0;
      for (let i = 0; i < lineNum; i++) offset += lines[i].length + 1;
      return { from: offset, to: offset + lines[lineNum].length };
    }
  }

  // MSSQL: "Incorrect syntax near '...'" — try to find the token
  const mssqlMatch = errorMessage.match(/near ['"](.*?)['"]/i);
  if (mssqlMatch) {
    const token = mssqlMatch[1];
    const idx = sql.lastIndexOf(token);
    if (idx >= 0) {
      return { from: idx, to: idx + token.length };
    }
  }

  return null;
}

/**
 * Build a dialect-aware EXPLAIN query.
 */
export function buildExplainQuery(sql: string, dbType: DatabaseType): string {
  const trimmed = sql.replace(/;\s*$/, '');
  switch (dbType) {
    case 'PostgreSQL':
    case 'CockroachDB':
    case 'Redshift':
      return `EXPLAIN (ANALYZE, FORMAT JSON) ${trimmed}`;
    case 'MySQL':
    case 'MariaDB':
      return `EXPLAIN FORMAT=JSON ${trimmed}`;
    case 'SQLite':
      return `EXPLAIN QUERY PLAN ${trimmed}`;
    case 'MSSQL':
      return `SET SHOWPLAN_TEXT ON;\n${trimmed};\nSET SHOWPLAN_TEXT OFF`;
    case 'ClickHouse':
      return `EXPLAIN ${trimmed}`;
    default:
      return `EXPLAIN ${trimmed}`;
  }
}
