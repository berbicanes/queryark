import type { CellValue } from '$lib/types/query';
import type { DataDiffResult, RowDiff, RowDiffStatus } from '$lib/types/diff';

function cellToString(cell: CellValue): string {
  switch (cell.type) {
    case 'Null': return '<NULL>';
    case 'Bool': return String(cell.value);
    case 'Int': return String(cell.value);
    case 'Float': return String(cell.value);
    case 'Text': return cell.value;
    case 'Timestamp': return cell.value;
    case 'Json': return cell.value;
    case 'Binary': return `[${cell.value.length} bytes]`;
    case 'LargeText': return cell.value.preview;
    case 'LargeJson': return cell.value.preview;
    case 'LargeBinary': return `[${cell.value.full_length} bytes]`;
  }
}

function buildPKKey(row: CellValue[], pkIndices: number[]): string {
  return pkIndices.map(i => cellToString(row[i])).join('|');
}

function cellsEqual(a: CellValue, b: CellValue): boolean {
  if (a.type !== b.type) return false;
  return cellToString(a) === cellToString(b);
}

export function computeDataDiff(
  sourceRows: CellValue[][],
  targetRows: CellValue[][],
  pkColumnIndices: number[],
  columnNames: string[]
): DataDiffResult {
  const targetMap = new Map<string, CellValue[]>();
  for (const row of targetRows) {
    const key = buildPKKey(row, pkColumnIndices);
    targetMap.set(key, row);
  }

  const matchedTargetKeys = new Set<string>();
  const rows: RowDiff[] = [];

  // Process source rows
  for (const srcRow of sourceRows) {
    const key = buildPKKey(srcRow, pkColumnIndices);
    const tgtRow = targetMap.get(key);
    const pkValues = pkColumnIndices.map(i => cellToString(srcRow[i]));

    if (!tgtRow) {
      rows.push({ status: 'removed', pkValues, sourceRow: srcRow, targetRow: null, changedColumns: [] });
    } else {
      matchedTargetKeys.add(key);
      const changedColumns: number[] = [];
      for (let i = 0; i < srcRow.length; i++) {
        if (pkColumnIndices.includes(i)) continue;
        if (i < tgtRow.length && !cellsEqual(srcRow[i], tgtRow[i])) {
          changedColumns.push(i);
        }
      }
      const status: RowDiffStatus = changedColumns.length > 0 ? 'changed' : 'identical';
      rows.push({ status, pkValues, sourceRow: srcRow, targetRow: tgtRow, changedColumns });
    }
  }

  // Process target-only rows (added)
  for (const tgtRow of targetRows) {
    const key = buildPKKey(tgtRow, pkColumnIndices);
    if (!matchedTargetKeys.has(key)) {
      const pkValues = pkColumnIndices.map(i => cellToString(tgtRow[i]));
      rows.push({ status: 'added', pkValues, sourceRow: null, targetRow: tgtRow, changedColumns: [] });
    }
  }

  const summary = {
    added: rows.filter(r => r.status === 'added').length,
    removed: rows.filter(r => r.status === 'removed').length,
    changed: rows.filter(r => r.status === 'changed').length,
    identical: rows.filter(r => r.status === 'identical').length,
  };

  return {
    pkColumns: pkColumnIndices.map(i => columnNames[i]),
    columns: columnNames,
    rows,
    summary,
  };
}
