export type TabType = 'query' | 'table' | 'document' | 'keyvalue' | 'graph';

export interface Tab {
  id: string;
  type: TabType;
  title: string;
  connectionId: string;
  // For query tabs
  sql?: string;
  // For table/document/keyvalue/graph tabs
  schema?: string;
  table?: string;
  // For generic browsing
  container?: string;
  item?: string;
}
