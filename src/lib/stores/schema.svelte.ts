import type {
  SchemaInfo, TableInfo, ColumnInfo, IndexInfo, ForeignKeyInfo,
  ContainerInfo, ItemInfo, FieldInfo,
  TableStats, RoutineInfo, SequenceInfo, EnumInfo
} from '$lib/types/schema';

interface SchemaCache {
  schemas: SchemaInfo[];
  tables: Record<string, TableInfo[]>; // schemaName -> tables
  columns: Record<string, ColumnInfo[]>; // "schema.table" -> columns
  indexes: Record<string, IndexInfo[]>; // "schema.table" -> indexes
  foreignKeys: Record<string, ForeignKeyInfo[]>; // "schema.table" -> fks
  tableStats: Record<string, TableStats>; // "schema.table" -> stats
  routines: Record<string, RoutineInfo[]>; // schemaName -> routines
  sequences: Record<string, SequenceInfo[]>; // schemaName -> sequences
  enums: Record<string, EnumInfo[]>; // schemaName -> enums
}

interface BrowserCache {
  containers: ContainerInfo[];
  items: Record<string, ItemInfo[]>; // container -> items
  fields: Record<string, FieldInfo[]>; // "container.item" -> fields
}

function emptySchemaCache(): SchemaCache {
  return {
    schemas: [], tables: {}, columns: {}, indexes: {}, foreignKeys: {},
    tableStats: {}, routines: {}, sequences: {}, enums: {},
  };
}

function emptyBrowserCache(): BrowserCache {
  return { containers: [], items: {}, fields: {} };
}

class SchemaStore {
  cache = $state<Record<string, SchemaCache>>({}); // connectionId -> SchemaCache
  browserCache = $state<Record<string, BrowserCache>>({}); // connectionId -> BrowserCache
  lastRefreshed = $state<Record<string, number>>({}); // connectionId -> timestamp
  visibleSchemas = $state<Record<string, string[] | null>>({}); // connectionId -> visible schema names (null = show all)

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

  getTableStats(connectionId: string, schemaName: string, tableName: string): TableStats | null {
    const key = `${schemaName}.${tableName}`;
    return this.cache[connectionId]?.tableStats[key] ?? null;
  }

  getRoutines(connectionId: string, schemaName: string): RoutineInfo[] {
    return this.cache[connectionId]?.routines[schemaName] ?? [];
  }

  getSequences(connectionId: string, schemaName: string): SequenceInfo[] {
    return this.cache[connectionId]?.sequences[schemaName] ?? [];
  }

  getEnums(connectionId: string, schemaName: string): EnumInfo[] {
    return this.cache[connectionId]?.enums[schemaName] ?? [];
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

  setTableStats(connectionId: string, schemaName: string, tableName: string, stats: TableStats) {
    if (!this.cache[connectionId]) {
      this.cache[connectionId] = emptySchemaCache();
    }
    this.cache[connectionId].tableStats[`${schemaName}.${tableName}`] = stats;
  }

  setRoutines(connectionId: string, schemaName: string, routines: RoutineInfo[]) {
    if (!this.cache[connectionId]) {
      this.cache[connectionId] = emptySchemaCache();
    }
    this.cache[connectionId].routines[schemaName] = routines;
  }

  setSequences(connectionId: string, schemaName: string, sequences: SequenceInfo[]) {
    if (!this.cache[connectionId]) {
      this.cache[connectionId] = emptySchemaCache();
    }
    this.cache[connectionId].sequences[schemaName] = sequences;
  }

  setEnums(connectionId: string, schemaName: string, enums: EnumInfo[]) {
    if (!this.cache[connectionId]) {
      this.cache[connectionId] = emptySchemaCache();
    }
    this.cache[connectionId].enums[schemaName] = enums;
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

  setLastRefreshed(connectionId: string) {
    this.lastRefreshed[connectionId] = Date.now();
  }

  // Schema visibility
  getVisibleSchemas(connectionId: string): string[] | null {
    return this.visibleSchemas[connectionId] ?? null;
  }

  setVisibleSchemas(connectionId: string, schemas: string[] | null) {
    this.visibleSchemas[connectionId] = schemas;
  }

  isSchemaVisible(connectionId: string, schemaName: string): boolean {
    const visible = this.visibleSchemas[connectionId];
    if (visible === null || visible === undefined) return true;
    return visible.includes(schemaName);
  }

  toggleSchemaVisibility(connectionId: string, schemaName: string, allSchemas: string[]) {
    const current = this.visibleSchemas[connectionId];
    if (current === null || current === undefined) {
      // Currently showing all — toggle off means show all except this one
      this.visibleSchemas[connectionId] = allSchemas.filter(s => s !== schemaName);
    } else if (current.includes(schemaName)) {
      // Remove it
      const next = current.filter(s => s !== schemaName);
      // If none visible, keep at least one (don't allow empty)
      if (next.length === 0) return;
      this.visibleSchemas[connectionId] = next;
    } else {
      // Add it
      const next = [...current, schemaName];
      // If all are now visible, set to null (show all)
      if (next.length === allSchemas.length) {
        this.visibleSchemas[connectionId] = null;
      } else {
        this.visibleSchemas[connectionId] = next;
      }
    }
  }

  /**
   * Returns the "active" (default) schema for a connection based on visibility state.
   * - If 1 schema visible → that's the active schema
   * - If multiple visible → first visible is treated as default
   * - If all visible (null) → returns the conventional default for the db type
   */
  getActiveSchema(connectionId: string, dbType: string): string | null {
    const visible = this.visibleSchemas[connectionId];
    if (visible && visible.length > 0) {
      return visible[0];
    }
    // All schemas visible — return conventional default
    switch (dbType) {
      case 'PostgreSQL':
      case 'CockroachDB':
      case 'Redshift':
        return 'public';
      case 'MySQL':
      case 'MariaDB': {
        const schemas = this.cache[connectionId]?.schemas ?? [];
        return schemas.length > 0 ? schemas[0].name : null;
      }
      case 'MSSQL':
        return 'dbo';
      default:
        return null;
    }
  }

  clearTableStats(connectionId: string, schemaName: string, tableName: string) {
    const key = `${schemaName}.${tableName}`;
    if (this.cache[connectionId]?.tableStats) {
      delete this.cache[connectionId].tableStats[key];
    }
  }

  clearConnection(connectionId: string) {
    delete this.cache[connectionId];
    delete this.browserCache[connectionId];
    delete this.lastRefreshed[connectionId];
    delete this.visibleSchemas[connectionId];
  }
}

export const schemaStore = new SchemaStore();
