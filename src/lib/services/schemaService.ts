import * as tauri from '$lib/services/tauri';
import { schemaStore } from '$lib/stores/schema.svelte';
import { uiStore } from '$lib/stores/ui.svelte';
import type { SchemaInfo, TableInfo, ColumnInfo, IndexInfo, ForeignKeyInfo, ContainerInfo, ItemInfo, FieldInfo } from '$lib/types/schema';

// SQL-specific loaders
export async function loadSchemas(connectionId: string): Promise<SchemaInfo[]> {
  try {
    const schemas = await tauri.getSchemas(connectionId);
    schemaStore.setSchemas(connectionId, schemas);
    return schemas;
  } catch (err) {
    uiStore.showError(`Failed to load schemas: ${err}`);
    return [];
  }
}

export async function loadTables(connectionId: string, schemaName: string): Promise<TableInfo[]> {
  const cached = schemaStore.getTables(connectionId, schemaName);
  if (cached.length > 0) return cached;

  try {
    const tables = await tauri.getTables(connectionId, schemaName);
    schemaStore.setTables(connectionId, schemaName, tables);
    return tables;
  } catch (err) {
    uiStore.showError(`Failed to load tables: ${err}`);
    return [];
  }
}

export async function loadColumns(connectionId: string, schemaName: string, tableName: string): Promise<ColumnInfo[]> {
  const cached = schemaStore.getColumns(connectionId, schemaName, tableName);
  if (cached.length > 0) return cached;

  try {
    const columns = await tauri.getColumns(connectionId, schemaName, tableName);
    schemaStore.setColumns(connectionId, schemaName, tableName, columns);
    return columns;
  } catch (err) {
    uiStore.showError(`Failed to load columns: ${err}`);
    return [];
  }
}

export async function loadIndexes(connectionId: string, schemaName: string, tableName: string): Promise<IndexInfo[]> {
  const cached = schemaStore.getIndexes(connectionId, schemaName, tableName);
  if (cached.length > 0) return cached;

  try {
    const indexes = await tauri.getIndexes(connectionId, schemaName, tableName);
    schemaStore.setIndexes(connectionId, schemaName, tableName, indexes);
    return indexes;
  } catch (err) {
    uiStore.showError(`Failed to load indexes: ${err}`);
    return [];
  }
}

export async function loadForeignKeys(connectionId: string, schemaName: string, tableName: string): Promise<ForeignKeyInfo[]> {
  const cached = schemaStore.getForeignKeys(connectionId, schemaName, tableName);
  if (cached.length > 0) return cached;

  try {
    const fks = await tauri.getForeignKeys(connectionId, schemaName, tableName);
    schemaStore.setForeignKeys(connectionId, schemaName, tableName, fks);
    return fks;
  } catch (err) {
    uiStore.showError(`Failed to load foreign keys: ${err}`);
    return [];
  }
}

// Generic loaders (all database types)
export async function loadContainers(connectionId: string): Promise<ContainerInfo[]> {
  try {
    const containers = await tauri.getContainers(connectionId);
    schemaStore.setContainers(connectionId, containers);
    return containers;
  } catch (err) {
    uiStore.showError(`Failed to load containers: ${err}`);
    return [];
  }
}

export async function loadItems(connectionId: string, container: string): Promise<ItemInfo[]> {
  const cached = schemaStore.getItems(connectionId, container);
  if (cached.length > 0) return cached;

  try {
    const items = await tauri.getItems(connectionId, container);
    schemaStore.setItems(connectionId, container, items);
    return items;
  } catch (err) {
    uiStore.showError(`Failed to load items: ${err}`);
    return [];
  }
}

export async function loadFields(connectionId: string, container: string, item: string): Promise<FieldInfo[]> {
  const cached = schemaStore.getFields(connectionId, container, item);
  if (cached.length > 0) return cached;

  try {
    const fields = await tauri.getItemFields(connectionId, container, item);
    schemaStore.setFields(connectionId, container, item, fields);
    return fields;
  } catch (err) {
    uiStore.showError(`Failed to load fields: ${err}`);
    return [];
  }
}

export function refreshSchema(connectionId: string) {
  schemaStore.clearConnection(connectionId);
  return loadSchemas(connectionId);
}

export function refreshContainers(connectionId: string) {
  schemaStore.clearConnection(connectionId);
  return loadContainers(connectionId);
}
