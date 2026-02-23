export interface IndexSuggestion {
  table: string;
  columns: string[];
  reason: string;
  sql: string;
}

/**
 * Analyze a SQL query and suggest indexes that could improve performance.
 */
export function analyzeQueryForIndexes(sql: string, dialect: string): IndexSuggestion[] {
  const suggestions: IndexSuggestion[] = [];
  const seen = new Set<string>();

  const normalized = stripComments(sql);

  // Extract tables referenced in FROM/JOIN clauses
  const tables = extractReferencedTables(normalized);

  // WHERE clause columns
  const whereColumns = extractWhereColumns(normalized);
  for (const { table, columns } of whereColumns) {
    const resolvedTable = resolveTable(table, tables);
    if (!resolvedTable) continue;
    const key = `${resolvedTable}:${columns.join(',')}`;
    if (seen.has(key)) continue;
    seen.add(key);
    suggestions.push({
      table: resolvedTable,
      columns,
      reason: `Column${columns.length > 1 ? 's' : ''} used in WHERE clause`,
      sql: buildCreateIndex(resolvedTable, columns, dialect),
    });
  }

  // JOIN columns
  const joinColumns = extractJoinColumns(normalized);
  for (const { table, columns } of joinColumns) {
    const resolvedTable = resolveTable(table, tables);
    if (!resolvedTable) continue;
    const key = `${resolvedTable}:${columns.join(',')}`;
    if (seen.has(key)) continue;
    seen.add(key);
    suggestions.push({
      table: resolvedTable,
      columns,
      reason: 'Column used in JOIN condition',
      sql: buildCreateIndex(resolvedTable, columns, dialect),
    });
  }

  // ORDER BY columns
  const orderByColumns = extractOrderByColumns(normalized);
  for (const { table, columns } of orderByColumns) {
    const resolvedTable = resolveTable(table, tables) ?? tables[0]?.name;
    if (!resolvedTable) continue;
    const key = `${resolvedTable}:${columns.join(',')}`;
    if (seen.has(key)) continue;
    seen.add(key);
    suggestions.push({
      table: resolvedTable,
      columns,
      reason: 'Column used in ORDER BY — index can avoid sort',
      sql: buildCreateIndex(resolvedTable, columns, dialect),
    });
  }

  // GROUP BY columns
  const groupByColumns = extractGroupByColumns(normalized);
  for (const { table, columns } of groupByColumns) {
    const resolvedTable = resolveTable(table, tables) ?? tables[0]?.name;
    if (!resolvedTable) continue;
    const key = `${resolvedTable}:${columns.join(',')}`;
    if (seen.has(key)) continue;
    seen.add(key);
    suggestions.push({
      table: resolvedTable,
      columns,
      reason: 'Column used in GROUP BY — index can speed up grouping',
      sql: buildCreateIndex(resolvedTable, columns, dialect),
    });
  }

  return suggestions;
}

interface TableRef {
  name: string;
  alias?: string;
}

interface ColumnRef {
  table: string;
  columns: string[];
}

function stripComments(sql: string): string {
  return sql
    .replace(/--[^\n]*/g, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .trim();
}

function extractReferencedTables(sql: string): TableRef[] {
  const tables: TableRef[] = [];
  // FROM table [AS alias], JOIN table [AS alias]
  const pattern = /\b(?:FROM|JOIN)\s+(?:(\w+)\.)?(\w+)(?:\s+(?:AS\s+)?(\w+))?/gi;
  let match;
  while ((match = pattern.exec(sql)) !== null) {
    const name = match[2];
    const alias = match[3];
    if (!isKeyword(name)) {
      tables.push({ name, alias });
    }
  }
  return tables;
}

function isKeyword(word: string): boolean {
  const kw = new Set([
    'select', 'from', 'where', 'join', 'inner', 'left', 'right', 'outer',
    'on', 'and', 'or', 'not', 'in', 'between', 'like', 'order', 'group',
    'having', 'limit', 'offset', 'union', 'intersect', 'except', 'as',
    'set', 'update', 'insert', 'delete', 'create', 'alter', 'drop',
    'index', 'table', 'view', 'into', 'values', 'null', 'true', 'false',
  ]);
  return kw.has(word.toLowerCase());
}

function extractWhereColumns(sql: string): ColumnRef[] {
  const results: ColumnRef[] = [];
  const whereMatch = sql.match(/\bWHERE\b([\s\S]*?)(?:\bGROUP\b|\bORDER\b|\bLIMIT\b|\bHAVING\b|\bUNION\b|$)/i);
  if (!whereMatch) return results;

  const whereClause = whereMatch[1];
  // Match table.column or just column patterns in conditions
  const colPattern = /(?:(\w+)\.)?(\w+)\s*(?:=|!=|<>|>=?|<=?|LIKE|IN|IS|BETWEEN)\b/gi;
  let match;
  while ((match = colPattern.exec(whereClause)) !== null) {
    const table = match[1] ?? '';
    const col = match[2];
    if (!isKeyword(col) && col.toLowerCase() !== 'null') {
      results.push({ table, columns: [col] });
    }
  }

  return results;
}

function extractJoinColumns(sql: string): ColumnRef[] {
  const results: ColumnRef[] = [];
  const onPattern = /\bON\s+([\s\S]*?)(?=\bJOIN\b|\bWHERE\b|\bGROUP\b|\bORDER\b|\bLIMIT\b|$)/gi;
  let onMatch;
  while ((onMatch = onPattern.exec(sql)) !== null) {
    const onClause = onMatch[1];
    const colPattern = /(?:(\w+)\.)?(\w+)\s*=\s*(?:(\w+)\.)?(\w+)/g;
    let match;
    while ((match = colPattern.exec(onClause)) !== null) {
      const table1 = match[1] ?? '';
      const col1 = match[2];
      const table2 = match[3] ?? '';
      const col2 = match[4];
      if (!isKeyword(col1)) results.push({ table: table1, columns: [col1] });
      if (!isKeyword(col2)) results.push({ table: table2, columns: [col2] });
    }
  }
  return results;
}

function extractOrderByColumns(sql: string): ColumnRef[] {
  const results: ColumnRef[] = [];
  const orderMatch = sql.match(/\bORDER\s+BY\b([\s\S]*?)(?:\bLIMIT\b|\bOFFSET\b|\bFETCH\b|\bUNION\b|$)/i);
  if (!orderMatch) return results;

  const parts = orderMatch[1].split(',');
  const columns: string[] = [];
  let table = '';
  for (const part of parts) {
    const m = part.trim().match(/^(?:(\w+)\.)?(\w+)(?:\s+(?:ASC|DESC))?$/i);
    if (m && !isKeyword(m[2])) {
      if (m[1]) table = m[1];
      columns.push(m[2]);
    }
  }
  if (columns.length > 0) {
    results.push({ table, columns });
  }
  return results;
}

function extractGroupByColumns(sql: string): ColumnRef[] {
  const results: ColumnRef[] = [];
  const groupMatch = sql.match(/\bGROUP\s+BY\b([\s\S]*?)(?:\bHAVING\b|\bORDER\b|\bLIMIT\b|\bUNION\b|$)/i);
  if (!groupMatch) return results;

  const parts = groupMatch[1].split(',');
  const columns: string[] = [];
  let table = '';
  for (const part of parts) {
    const m = part.trim().match(/^(?:(\w+)\.)?(\w+)$/i);
    if (m && !isKeyword(m[2])) {
      if (m[1]) table = m[1];
      columns.push(m[2]);
    }
  }
  if (columns.length > 0) {
    results.push({ table, columns });
  }
  return results;
}

function resolveTable(ref: string, tables: TableRef[]): string | undefined {
  if (!ref) return tables[0]?.name;
  // Check if it matches a table name
  const direct = tables.find(t => t.name.toLowerCase() === ref.toLowerCase());
  if (direct) return direct.name;
  // Check if it matches an alias
  const aliased = tables.find(t => t.alias?.toLowerCase() === ref.toLowerCase());
  if (aliased) return aliased.name;
  return ref;
}

function buildCreateIndex(table: string, columns: string[], dialect: string): string {
  const indexName = `idx_${table}_${columns.join('_')}`.toLowerCase();
  const colList = columns.map(c => quoteIdentifier(c, dialect)).join(', ');
  const quotedTable = quoteIdentifier(table, dialect);

  return `CREATE INDEX ${quoteIdentifier(indexName, dialect)} ON ${quotedTable} (${colList});`;
}

function quoteIdentifier(name: string, dialect: string): string {
  if (/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(name)) return name;
  if (dialect === 'MySQL' || dialect === 'MariaDB') return '`' + name + '`';
  if (dialect === 'MSSQL') return '[' + name + ']';
  return '"' + name + '"';
}
