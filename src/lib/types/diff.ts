import type { ColumnInfo, IndexInfo, ForeignKeyInfo } from './schema';
import type { CellValue } from './query';

export type DiffStatus = 'added' | 'removed' | 'changed' | 'unchanged';

export interface ColumnDiff {
  name: string;
  status: DiffStatus;
  source: ColumnInfo | null;
  target: ColumnInfo | null;
  changes?: string[];
}

export interface IndexDiff {
  name: string;
  status: DiffStatus;
  source: IndexInfo | null;
  target: IndexInfo | null;
  changes?: string[];
}

export interface ForeignKeyDiff {
  name: string;
  status: DiffStatus;
  source: ForeignKeyInfo | null;
  target: ForeignKeyInfo | null;
  changes?: string[];
}

export interface TableDiffResult {
  sourceTable: string;
  targetTable: string;
  columns: ColumnDiff[];
  indexes: IndexDiff[];
  foreignKeys: ForeignKeyDiff[];
  summary: { added: number; removed: number; changed: number; unchanged: number };
}

// Data diff types
export type RowDiffStatus = 'added' | 'removed' | 'changed' | 'identical';

export interface RowDiff {
  status: RowDiffStatus;
  pkValues: string[];
  sourceRow: CellValue[] | null;
  targetRow: CellValue[] | null;
  changedColumns: number[];
}

export interface DataDiffResult {
  pkColumns: string[];
  columns: string[];
  rows: RowDiff[];
  summary: { added: number; removed: number; changed: number; identical: number };
}
