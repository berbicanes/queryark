export type TabType = 'query' | 'table' | 'document' | 'keyvalue' | 'graph'
  | 'diagram' | 'tablediff' | 'datadiff' | 'visualquery';

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
}
