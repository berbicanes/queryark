import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { captureError } from '$lib/services/sentryService';
import type { ConnectionConfig, DatabaseCategory } from '$lib/types/connection';
import type { QueryResponse, SortColumn, FilterCondition, CellValue, ColumnDef } from '$lib/types/query';
import type {
  SchemaInfo, TableInfo, ColumnInfo, IndexInfo, ForeignKeyInfo,
  ContainerInfo, ItemInfo, FieldInfo,
  TableStats, RoutineInfo, SequenceInfo, EnumInfo
} from '$lib/types/schema';
import type { ImportResult } from '$lib/types/export';

// Wrapper that captures IPC errors to Sentry
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (error) {
    captureError(error, { command: cmd, args });
    throw error;
  }
}

// Connection management
export async function connectDb(config: ConnectionConfig): Promise<string> {
  return invoke<string>('connect_db', { config });
}

export async function disconnectDb(connectionId: string): Promise<void> {
  return invoke<void>('disconnect_db', { connectionId });
}

export async function testConnection(config: ConnectionConfig): Promise<boolean> {
  return invoke<boolean>('test_connection', { config });
}

export async function pingConnection(connectionId: string): Promise<boolean> {
  return invoke<boolean>('ping_connection', { connectionId });
}

// Query execution
export async function executeQuery(connectionId: string, sql: string, timeoutSecs?: number, queryId?: string, maxRows?: number, maxCellSize?: number): Promise<QueryResponse> {
  return invoke<QueryResponse>('execute_query', { connectionId, sql, timeoutSecs: timeoutSecs ?? null, queryId: queryId ?? null, maxRows: maxRows ?? null, maxCellSize: maxCellSize ?? null });
}

export async function executeQueryPage(connectionId: string, sql: string, limit: number, offset: number, timeoutSecs?: number, queryId?: string, maxCellSize?: number, sortColumns?: SortColumn[]): Promise<QueryResponse> {
  return invoke<QueryResponse>('execute_query_page', {
    connectionId, sql, limit, offset,
    timeoutSecs: timeoutSecs ?? null,
    queryId: queryId ?? null,
    maxCellSize: maxCellSize ?? null,
    sortColumns: sortColumns && sortColumns.length > 0 ? sortColumns : null,
  });
}

export async function countQueryRows(connectionId: string, sql: string): Promise<number> {
  return invoke<number>('count_query_rows', { connectionId, sql });
}

export async function fetchFullCell(connectionId: string, sql: string, column: string, rowOffset: number): Promise<CellValue> {
  return invoke<CellValue>('fetch_full_cell', { connectionId, sql, column, rowOffset });
}

export async function cancelQuery(queryId: string): Promise<boolean> {
  return invoke<boolean>('cancel_query', { queryId });
}

// Generic schema browsing (all databases)
export async function getDatabaseCategory(connectionId: string): Promise<DatabaseCategory> {
  return invoke<DatabaseCategory>('get_database_category', { connectionId });
}

export async function getContainers(connectionId: string): Promise<ContainerInfo[]> {
  return invoke<ContainerInfo[]>('get_containers', { connectionId });
}

export async function getItems(connectionId: string, container: string): Promise<ItemInfo[]> {
  return invoke<ItemInfo[]>('get_items', { connectionId, container });
}

export async function getItemFields(connectionId: string, container: string, item: string): Promise<FieldInfo[]> {
  return invoke<FieldInfo[]>('get_item_fields', { connectionId, container, item });
}

export async function getItemData(connectionId: string, container: string, item: string, limit: number, offset: number): Promise<QueryResponse> {
  return invoke<QueryResponse>('get_item_data', { connectionId, container, item, limit, offset });
}

export async function getItemCount(connectionId: string, container: string, item: string): Promise<number> {
  return invoke<number>('get_item_count', { connectionId, container, item });
}

// SQL-specific schema (relational + analytics + CQL)
export async function getSchemas(connectionId: string): Promise<SchemaInfo[]> {
  return invoke<SchemaInfo[]>('get_schemas', { connectionId });
}

export async function getTables(connectionId: string, schema: string): Promise<TableInfo[]> {
  return invoke<TableInfo[]>('get_tables', { connectionId, schema });
}

export async function getColumns(connectionId: string, schema: string, table: string): Promise<ColumnInfo[]> {
  return invoke<ColumnInfo[]>('get_columns', { connectionId, schema, table });
}

export async function getIndexes(connectionId: string, schema: string, table: string): Promise<IndexInfo[]> {
  return invoke<IndexInfo[]>('get_indexes', { connectionId, schema, table });
}

export async function getForeignKeys(connectionId: string, schema: string, table: string): Promise<ForeignKeyInfo[]> {
  return invoke<ForeignKeyInfo[]>('get_foreign_keys', { connectionId, schema, table });
}

export async function getTableData(connectionId: string, schema: string, table: string, limit: number, offset: number, sortColumns?: SortColumn[], filters?: FilterCondition[]): Promise<QueryResponse> {
  return invoke<QueryResponse>('get_table_data', {
    connectionId, schema, table, limit, offset,
    sortColumns: sortColumns && sortColumns.length > 0 ? sortColumns : null,
    filters: filters && filters.length > 0 ? filters : null,
  });
}

export async function getRowCount(connectionId: string, schema: string, table: string, filters?: FilterCondition[]): Promise<number> {
  return invoke<number>('get_row_count', {
    connectionId, schema, table,
    filters: filters && filters.length > 0 ? filters : null,
  });
}

export async function updateCell(connectionId: string, schema: string, table: string, column: string, value: string, pkColumns: string[], pkValues: string[], isNull?: boolean): Promise<void> {
  return invoke<void>('update_cell', { connectionId, schema, table, column, value, pkColumns, pkValues, isNull: isNull ?? null });
}

export async function insertRow(connectionId: string, schema: string, table: string, columns: string[], values: string[]): Promise<void> {
  return invoke<void>('insert_row', { connectionId, schema, table, columns, values });
}

export async function deleteRows(connectionId: string, schema: string, table: string, pkColumns: string[], pkValuesList: string[][]): Promise<number> {
  return invoke<number>('delete_rows', { connectionId, schema, table, pkColumns, pkValuesList });
}

// Phase 5: Schema browser additions
export async function getTableStats(connectionId: string, schema: string, table: string): Promise<TableStats> {
  return invoke<TableStats>('get_table_stats', { connectionId, schema, table });
}

export async function getRoutines(connectionId: string, schema: string): Promise<RoutineInfo[]> {
  return invoke<RoutineInfo[]>('get_routines', { connectionId, schema });
}

export async function getSequences(connectionId: string, schema: string): Promise<SequenceInfo[]> {
  return invoke<SequenceInfo[]>('get_sequences', { connectionId, schema });
}

export async function getEnums(connectionId: string, schema: string): Promise<EnumInfo[]> {
  return invoke<EnumInfo[]>('get_enums', { connectionId, schema });
}

// Transaction management
export async function beginTransaction(connectionId: string): Promise<void> {
  return invoke<void>('begin_transaction', { connectionId });
}

export async function commitTransaction(connectionId: string): Promise<void> {
  return invoke<void>('commit_transaction', { connectionId });
}

export async function rollbackTransaction(connectionId: string): Promise<void> {
  return invoke<void>('rollback_transaction', { connectionId });
}

// Document operations (MongoDB, DynamoDB)
export async function insertDocument(connectionId: string, container: string, item: string, document: string): Promise<string> {
  return invoke<string>('insert_document', { connectionId, container, item, document });
}

export async function updateDocument(connectionId: string, container: string, item: string, filter: string, update: string): Promise<number> {
  return invoke<number>('update_document', { connectionId, container, item, filter, update });
}

export async function deleteDocuments(connectionId: string, container: string, item: string, filter: string): Promise<number> {
  return invoke<number>('delete_documents', { connectionId, container, item, filter });
}

// Key-value operations (Redis)
export async function getValue(connectionId: string, key: string): Promise<string> {
  return invoke<string>('get_value', { connectionId, key });
}

export async function setValue(connectionId: string, key: string, value: string, ttl: number | null = null): Promise<void> {
  return invoke<void>('set_value', { connectionId, key, value, ttl });
}

export async function deleteKeys(connectionId: string, keys: string[]): Promise<number> {
  return invoke<number>('delete_keys', { connectionId, keys });
}

export async function getKeyType(connectionId: string, key: string): Promise<string> {
  return invoke<string>('get_key_type', { connectionId, key });
}

export async function scanKeys(connectionId: string, pattern: string, count: number): Promise<string[]> {
  return invoke<string[]>('scan_keys', { connectionId, pattern, count });
}

// Graph operations (Neo4j)
export async function getLabels(connectionId: string): Promise<string[]> {
  return invoke<string[]>('get_labels', { connectionId });
}

export async function getRelationshipTypes(connectionId: string): Promise<string[]> {
  return invoke<string[]>('get_relationship_types', { connectionId });
}

export async function getNodeProperties(connectionId: string, label: string): Promise<string[]> {
  return invoke<string[]>('get_node_properties', { connectionId, label });
}

export async function getNodes(connectionId: string, label: string, limit: number, skip: number): Promise<QueryResponse> {
  return invoke<QueryResponse>('get_nodes', { connectionId, label, limit, skip });
}

// Keychain operations
export async function storeKeychainPassword(connectionId: string, password: string): Promise<void> {
  return invoke<void>('store_keychain_password', { connectionId, password });
}

export async function getKeychainPassword(connectionId: string): Promise<string | null> {
  return invoke<string | null>('get_keychain_password', { connectionId });
}

export async function deleteKeychainPassword(connectionId: string): Promise<void> {
  return invoke<void>('delete_keychain_password', { connectionId });
}

export async function checkKeychainAvailable(): Promise<boolean> {
  return invoke<boolean>('check_keychain_available');
}

export async function storeKeychainSecret(connectionId: string, key: string, value: string): Promise<void> {
  return invoke<void>('store_keychain_secret', { connectionId, key, value });
}

export async function getKeychainSecret(connectionId: string, key: string): Promise<string | null> {
  return invoke<string | null>('get_keychain_secret', { connectionId, key });
}

// Export/Import operations
export async function exportToCsv(
  filePath: string, columns: ColumnDef[], rows: CellValue[][],
  connectionId?: string, schema?: string, table?: string, exportAll = false
): Promise<number> {
  return invoke<number>('export_to_csv', {
    connectionId: connectionId ?? null, schema: schema ?? null, table: table ?? null,
    filePath, columns, rows, exportAll,
  });
}

export async function exportToJson(
  filePath: string, columns: ColumnDef[], rows: CellValue[][],
  connectionId?: string, schema?: string, table?: string, exportAll = false
): Promise<number> {
  return invoke<number>('export_to_json', {
    connectionId: connectionId ?? null, schema: schema ?? null, table: table ?? null,
    filePath, columns, rows, exportAll,
  });
}

export async function exportToSql(
  filePath: string, columns: ColumnDef[], rows: CellValue[][],
  connectionId?: string, schema?: string, table?: string, exportAll = false
): Promise<number> {
  return invoke<number>('export_to_sql', {
    connectionId: connectionId ?? null, schema: schema ?? null, table: table ?? null,
    filePath, columns, rows, exportAll,
  });
}

export async function exportDdl(
  connectionId: string, schema: string, table: string, filePath?: string
): Promise<string> {
  return invoke<string>('export_ddl', {
    connectionId, schema, table, filePath: filePath ?? null,
  });
}

export async function importCsv(
  connectionId: string, schema: string, table: string,
  filePath: string, hasHeader = true, delimiter?: string
): Promise<ImportResult> {
  return invoke<ImportResult>('import_csv', {
    connectionId, schema, table, filePath, hasHeader, delimiter: delimiter ?? null,
  });
}
