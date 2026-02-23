import type { CellValue, ColumnDef } from '$lib/types/query';

export interface ResultCompareRow {
  sourceRow: CellValue[] | null;
  targetRow: CellValue[] | null;
  status: 'identical' | 'changed' | 'source-only' | 'target-only';
  changedColumns: Set<number>;
}

export interface ResultCompareResult {
  columns: ColumnDef[];
  rows: ResultCompareRow[];
  stats: { identical: number; changed: number; sourceOnly: number; targetOnly: number };
}

function cellToString(cell: CellValue): string {
  switch (cell.type) {
    case 'Null': return '\x00NULL\x00';
    case 'Bool': return `bool:${cell.value}`;
    case 'Int': return `int:${cell.value}`;
    case 'Float': return `float:${cell.value}`;
    case 'Text': return `text:${cell.value}`;
    case 'Timestamp': return `ts:${cell.value}`;
    case 'Json': return `json:${cell.value}`;
    case 'Binary': return `bin:${cell.value.join(',')}`;
    case 'LargeText': return `lt:${cell.value.preview}`;
    case 'LargeJson': return `lj:${cell.value.preview}`;
    case 'LargeBinary': return `lb:${cell.value.full_length}`;
  }
}

function cellsEqual(a: CellValue, b: CellValue): boolean {
  return cellToString(a) === cellToString(b);
}

function buildRowKey(row: CellValue[], keyColumns: number[]): string {
  return keyColumns.map(i => cellToString(row[i])).join('|');
}

/**
 * Compare two query result sets.
 * If matchByColumns given: match rows by those column values (like PK matching).
 * If not given: compare row-by-row (positional matching).
 */
export function compareResults(
  source: { columns: ColumnDef[]; rows: CellValue[][] },
  target: { columns: ColumnDef[]; rows: CellValue[][] },
  matchByColumns?: number[]
): ResultCompareResult {
  // Build unified column list (use source columns as base)
  const columns = source.columns;
  const stats = { identical: 0, changed: 0, sourceOnly: 0, targetOnly: 0 };
  const rows: ResultCompareRow[] = [];

  if (matchByColumns && matchByColumns.length > 0) {
    // Key-based matching
    const targetMap = new Map<string, CellValue[]>();
    const targetMatched = new Set<string>();

    for (const row of target.rows) {
      const key = buildRowKey(row, matchByColumns);
      targetMap.set(key, row);
    }

    for (const srcRow of source.rows) {
      const key = buildRowKey(srcRow, matchByColumns);
      const tgtRow = targetMap.get(key);

      if (!tgtRow) {
        rows.push({ sourceRow: srcRow, targetRow: null, status: 'source-only', changedColumns: new Set() });
        stats.sourceOnly++;
      } else {
        targetMatched.add(key);
        const changedColumns = new Set<number>();
        let isIdentical = true;

        const maxCols = Math.max(srcRow.length, tgtRow.length);
        for (let c = 0; c < maxCols; c++) {
          if (matchByColumns.includes(c)) continue;
          const srcCell = srcRow[c];
          const tgtCell = tgtRow[c];
          if (!srcCell || !tgtCell || !cellsEqual(srcCell, tgtCell)) {
            changedColumns.add(c);
            isIdentical = false;
          }
        }

        if (isIdentical) {
          rows.push({ sourceRow: srcRow, targetRow: tgtRow, status: 'identical', changedColumns });
          stats.identical++;
        } else {
          rows.push({ sourceRow: srcRow, targetRow: tgtRow, status: 'changed', changedColumns });
          stats.changed++;
        }
      }
    }

    // Remaining target rows not matched
    for (const tgtRow of target.rows) {
      const key = buildRowKey(tgtRow, matchByColumns);
      if (!targetMatched.has(key)) {
        rows.push({ sourceRow: null, targetRow: tgtRow, status: 'target-only', changedColumns: new Set() });
        stats.targetOnly++;
      }
    }
  } else {
    // Positional matching
    const maxLen = Math.max(source.rows.length, target.rows.length);
    for (let r = 0; r < maxLen; r++) {
      const srcRow = r < source.rows.length ? source.rows[r] : null;
      const tgtRow = r < target.rows.length ? target.rows[r] : null;

      if (!srcRow) {
        rows.push({ sourceRow: null, targetRow: tgtRow, status: 'target-only', changedColumns: new Set() });
        stats.targetOnly++;
      } else if (!tgtRow) {
        rows.push({ sourceRow: srcRow, targetRow: null, status: 'source-only', changedColumns: new Set() });
        stats.sourceOnly++;
      } else {
        const changedColumns = new Set<number>();
        let isIdentical = true;

        const maxCols = Math.max(srcRow.length, tgtRow.length);
        for (let c = 0; c < maxCols; c++) {
          const srcCell = srcRow[c];
          const tgtCell = tgtRow[c];
          if (!srcCell || !tgtCell || !cellsEqual(srcCell, tgtCell)) {
            changedColumns.add(c);
            isIdentical = false;
          }
        }

        if (isIdentical) {
          rows.push({ sourceRow: srcRow, targetRow: tgtRow, status: 'identical', changedColumns });
          stats.identical++;
        } else {
          rows.push({ sourceRow: srcRow, targetRow: tgtRow, status: 'changed', changedColumns });
          stats.changed++;
        }
      }
    }
  }

  return { columns, rows, stats };
}
