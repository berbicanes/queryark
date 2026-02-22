import type { SchemaInfo, TableInfo, ColumnInfo, IndexInfo, ForeignKeyInfo, ContainerInfo, ItemInfo, FieldInfo } from '$lib/types/schema';

interface SchemaCache {
  schemas: SchemaInfo[];
  tables: Record<string, TableInfo[]>; // schemaName -> tables
  columns: Record<string, ColumnInfo[]>; // "schema.table" -> columns
  indexes: Record<string, IndexInfo[]>; // "schema.table" -> indexes
  foreignKeys: Record<string, ForeignKeyInfo[]>; // "schema.table" -> fks
}

interface BrowserCache {
  containers: ContainerInfo[];
  items: Record<string, ItemInfo[]>; // container -> items
  fields: Record<string, FieldInfo[]>; // "container.item" -> fields
}

function emptySchemaCache(): SchemaCache {
  return { schemas: [], tables: {}, columns: {}, indexes: {}, foreignKeys: {} };
}

function emptyBrowserCache(): BrowserCache {
  return { containers: [], items: {}, fields: {} };
}

class SchemaStore {
  cache = $state<Record<string, SchemaCache>>({}); // connectionId -> SchemaCache
  browserCache = $state<Record<string, BrowserCache>>({}); // connectionId -> BrowserCache

  // SQL-specific getters
  getSchemas(connectionId: string): SchemaInfo[] {
    return this.cache[connectionId]?.schemas ?? [];
  }

  getTables(connectionId: string, schemaName: string): TableInfo[] {
    return this.cache[connectionId]?.tables[schemaName] ?? [];
  }

  getColumns(connectionId: string, schemaName: string, tableName: string): ColumnInfo[] {
    const key = `${schemaName}.${tableName}`;
    return this.cache[connectionId]?.columns[key] ?? [];
  }

  getIndexes(connectionId: string, schemaName: string, tableName: string): IndexInfo[] {
    const key = `${schemaName}.${tableName}`;
    return this.cache[connectionId]?.indexes[key] ?? [];
  }

  getForeignKeys(connectionId: string, schemaName: string, tableName: string): ForeignKeyInfo[] {
    const key = `${schemaName}.${tableName}`;
    return this.cache[connectionId]?.foreignKeys[key] ?? [];
  }

  // SQL-specific setters
  setSchemas(connectionId: string, schemas: SchemaInfo[]) {
    if (!this.cache[connectionId]) {
      this.cache[connectionId] = emptySchemaCache();
    }
    this.cache[connectionId].schemas = schemas;
  }

  setTables(connectionId: string, schemaName: string, tables: TableInfo[]) {
    if (!this.cache[connectionId]) {
      this.cache[connectionId] = emptySchemaCache();
    }
    this.cache[connectionId].tables[schemaName] = tables;
  }

  setColumns(connectionId: string, schemaName: string, tableName: string, columns: ColumnInfo[]) {
    if (!this.cache[connectionId]) {
      this.cache[connectionId] = emptySchemaCache();
    }
    this.cache[connectionId].columns[`${schemaName}.${tableName}`] = columns;
  }

  setIndexes(connectionId: string, schemaName: string, tableName: string, indexes: IndexInfo[]) {
    if (!this.cache[connectionId]) {
      this.cache[connectionId] = emptySchemaCache();
    }
    this.cache[connectionId].indexes[`${schemaName}.${tableName}`] = indexes;
  }

  setForeignKeys(connectionId: string, schemaName: string, tableName: string, fks: ForeignKeyInfo[]) {
    if (!this.cache[connectionId]) {
      this.cache[connectionId] = emptySchemaCache();
    }
    this.cache[connectionId].foreignKeys[`${schemaName}.${tableName}`] = fks;
  }

  // Generic getters
  getContainers(connectionId: string): ContainerInfo[] {
    return this.browserCache[connectionId]?.containers ?? [];
  }

  getItems(connectionId: string, container: string): ItemInfo[] {
    return this.browserCache[connectionId]?.items[container] ?? [];
  }

  getFields(connectionId: string, container: string, item: string): FieldInfo[] {
    const key = `${container}.${item}`;
    return this.browserCache[connectionId]?.fields[key] ?? [];
  }

  // Generic setters
  setContainers(connectionId: string, containers: ContainerInfo[]) {
    if (!this.browserCache[connectionId]) {
      this.browserCache[connectionId] = emptyBrowserCache();
    }
    this.browserCache[connectionId].containers = containers;
  }

  setItems(connectionId: string, container: string, items: ItemInfo[]) {
    if (!this.browserCache[connectionId]) {
      this.browserCache[connectionId] = emptyBrowserCache();
    }
    this.browserCache[connectionId].items[container] = items;
  }

  setFields(connectionId: string, container: string, item: string, fields: FieldInfo[]) {
    if (!this.browserCache[connectionId]) {
      this.browserCache[connectionId] = emptyBrowserCache();
    }
    this.browserCache[connectionId].fields[`${container}.${item}`] = fields;
  }

  clearConnection(connectionId: string) {
    delete this.cache[connectionId];
    delete this.browserCache[connectionId];
  }
}

export const schemaStore = new SchemaStore();
