export type TabType = 'query' | 'table' | 'document' | 'keyvalue' | 'graph'
  | 'diagram' | 'tablediff' | 'datadiff' | 'visualquery' | 'bookmark' | 'resultcompare';

export interface Tab {
  id: string;
  type: TabType;
  title: string;
  connectionId: string;
  pinned?: boolean;
  // For query tabs
  sql?: string;
  // For table/document/keyvalue/graph tabs
  schema?: string;
  table?: string;
  // For generic browsing
  container?: string;
  item?: string;
  // Phase 20: Visual database tools
  diagramSchemas?: string[];
  diffTargetConnectionId?: string;
  diffTargetSchema?: string;
  diffTargetTable?: string;
  // Phase 21: Bookmarks
  bookmarkId?: string;
  // Phase 22: Result comparison
  compareSourceResult?: { columns: import('./query').ColumnDef[]; rows: import('./query').CellValue[][]; sql: string };
  compareTargetResult?: { columns: import('./query').ColumnDef[]; rows: import('./query').CellValue[][]; sql: string };
}
