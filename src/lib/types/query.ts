export type CellValue =
  | { type: 'Null' }
  | { type: 'Bool'; value: boolean }
  | { type: 'Int'; value: number }
  | { type: 'Float'; value: number }
  | { type: 'Text'; value: string }
  | { type: 'Timestamp'; value: string }
  | { type: 'Binary'; value: number[] }
  | { type: 'Json'; value: string }
  | { type: 'LargeText'; value: { preview: string; full_length: number } }
  | { type: 'LargeJson'; value: { preview: string; full_length: number } }
  | { type: 'LargeBinary'; value: { preview_length: number; full_length: number } };

export interface ColumnDef {
  name: string;
  data_type: string;
}

export interface QueryResponse {
  columns: ColumnDef[];
  rows: CellValue[][];
  row_count: number;
  execution_time_ms: number;
  affected_rows: number | null;
  truncated?: boolean;
  max_rows_limit?: number;
}

export interface SortColumn {
  column: string;
  direction: 'ASC' | 'DESC';
}

export interface FilterCondition {
  column: string;
  operator: 'eq' | 'neq' | 'gt' | 'gte' | 'lt' | 'lte' | 'contains' | 'starts_with' | 'is_null' | 'is_not_null';
  value: string;
}

export interface MultiStatementResult {
  results: QueryResponse[];
  error?: {
    index: number;
    message: string;
  };
}

export interface QueryHistoryEntry {
  id: string;
  connectionId: string;
  sql: string;
  executedAt: number;
  executionTimeMs: number;
  rowCount: number;
  error?: string;
}

export interface SavedQuery {
  id: string;
  name: string;
  connectionId: string;
  sql: string;
  createdAt: number;
  updatedAt: number;
}
