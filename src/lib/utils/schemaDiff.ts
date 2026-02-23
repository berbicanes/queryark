import type { ColumnInfo, IndexInfo, ForeignKeyInfo } from '$lib/types/schema';
import type { ColumnDiff, IndexDiff, ForeignKeyDiff, TableDiffResult, DiffStatus } from '$lib/types/diff';

export function diffColumns(source: ColumnInfo[], target: ColumnInfo[]): ColumnDiff[] {
  const results: ColumnDiff[] = [];
  const sourceMap = new Map(source.map(c => [c.name, c]));
  const targetMap = new Map(target.map(c => [c.name, c]));

  // Check source columns
  for (const [name, srcCol] of sourceMap) {
    const tgtCol = targetMap.get(name);
    if (!tgtCol) {
      results.push({ name, status: 'removed', source: srcCol, target: null });
    } else {
      const changes: string[] = [];
      if (srcCol.data_type !== tgtCol.data_type) changes.push(`type: ${srcCol.data_type} → ${tgtCol.data_type}`);
      if (srcCol.is_nullable !== tgtCol.is_nullable) changes.push(`nullable: ${srcCol.is_nullable} → ${tgtCol.is_nullable}`);
      if (srcCol.column_default !== tgtCol.column_default) changes.push(`default: ${srcCol.column_default ?? 'null'} → ${tgtCol.column_default ?? 'null'}`);
      if (srcCol.is_primary_key !== tgtCol.is_primary_key) changes.push(`pk: ${srcCol.is_primary_key} → ${tgtCol.is_primary_key}`);

      const status: DiffStatus = changes.length > 0 ? 'changed' : 'unchanged';
      results.push({ name, status, source: srcCol, target: tgtCol, changes: changes.length > 0 ? changes : undefined });
    }
  }

  // Check target-only columns (added)
  for (const [name, tgtCol] of targetMap) {
    if (!sourceMap.has(name)) {
      results.push({ name, status: 'added', source: null, target: tgtCol });
    }
  }

  // Sort: removed first, then changed, then added, then unchanged
  const order: Record<DiffStatus, number> = { removed: 0, changed: 1, added: 2, unchanged: 3 };
  results.sort((a, b) => order[a.status] - order[b.status]);

  return results;
}

export function diffIndexes(source: IndexInfo[], target: IndexInfo[]): IndexDiff[] {
  const results: IndexDiff[] = [];
  const sourceMap = new Map(source.map(i => [i.name, i]));
  const targetMap = new Map(target.map(i => [i.name, i]));

  for (const [name, srcIdx] of sourceMap) {
    const tgtIdx = targetMap.get(name);
    if (!tgtIdx) {
      results.push({ name, status: 'removed', source: srcIdx, target: null });
    } else {
      const changes: string[] = [];
      if (JSON.stringify(srcIdx.columns) !== JSON.stringify(tgtIdx.columns)) changes.push(`columns: [${srcIdx.columns.join(', ')}] → [${tgtIdx.columns.join(', ')}]`);
      if (srcIdx.is_unique !== tgtIdx.is_unique) changes.push(`unique: ${srcIdx.is_unique} → ${tgtIdx.is_unique}`);
      if (srcIdx.index_type !== tgtIdx.index_type) changes.push(`type: ${srcIdx.index_type} → ${tgtIdx.index_type}`);

      const status: DiffStatus = changes.length > 0 ? 'changed' : 'unchanged';
      results.push({ name, status, source: srcIdx, target: tgtIdx, changes: changes.length > 0 ? changes : undefined });
    }
  }

  for (const [name, tgtIdx] of targetMap) {
    if (!sourceMap.has(name)) {
      results.push({ name, status: 'added', source: null, target: tgtIdx });
    }
  }

  const order: Record<DiffStatus, number> = { removed: 0, changed: 1, added: 2, unchanged: 3 };
  results.sort((a, b) => order[a.status] - order[b.status]);

  return results;
}

export function diffForeignKeys(source: ForeignKeyInfo[], target: ForeignKeyInfo[]): ForeignKeyDiff[] {
  const results: ForeignKeyDiff[] = [];
  const sourceMap = new Map(source.map(f => [f.name, f]));
  const targetMap = new Map(target.map(f => [f.name, f]));

  for (const [name, srcFK] of sourceMap) {
    const tgtFK = targetMap.get(name);
    if (!tgtFK) {
      results.push({ name, status: 'removed', source: srcFK, target: null });
    } else {
      const changes: string[] = [];
      if (JSON.stringify(srcFK.columns) !== JSON.stringify(tgtFK.columns)) changes.push(`columns: [${srcFK.columns.join(', ')}] → [${tgtFK.columns.join(', ')}]`);
      if (srcFK.referenced_table !== tgtFK.referenced_table) changes.push(`ref table: ${srcFK.referenced_table} → ${tgtFK.referenced_table}`);
      if (JSON.stringify(srcFK.referenced_columns) !== JSON.stringify(tgtFK.referenced_columns)) changes.push(`ref columns: [${srcFK.referenced_columns.join(', ')}] → [${tgtFK.referenced_columns.join(', ')}]`);
      if (srcFK.on_update !== tgtFK.on_update) changes.push(`on_update: ${srcFK.on_update} → ${tgtFK.on_update}`);
      if (srcFK.on_delete !== tgtFK.on_delete) changes.push(`on_delete: ${srcFK.on_delete} → ${tgtFK.on_delete}`);

      const status: DiffStatus = changes.length > 0 ? 'changed' : 'unchanged';
      results.push({ name, status, source: srcFK, target: tgtFK, changes: changes.length > 0 ? changes : undefined });
    }
  }

  for (const [name, tgtFK] of targetMap) {
    if (!sourceMap.has(name)) {
      results.push({ name, status: 'added', source: null, target: tgtFK });
    }
  }

  const order: Record<DiffStatus, number> = { removed: 0, changed: 1, added: 2, unchanged: 3 };
  results.sort((a, b) => order[a.status] - order[b.status]);

  return results;
}

export function computeTableDiff(
  sourceColumns: ColumnInfo[],
  targetColumns: ColumnInfo[],
  sourceIndexes: IndexInfo[],
  targetIndexes: IndexInfo[],
  sourceFKs: ForeignKeyInfo[],
  targetFKs: ForeignKeyInfo[],
  sourceTable: string,
  targetTable: string
): TableDiffResult {
  const columns = diffColumns(sourceColumns, targetColumns);
  const indexes = diffIndexes(sourceIndexes, targetIndexes);
  const foreignKeys = diffForeignKeys(sourceFKs, targetFKs);

  const allDiffs = [...columns, ...indexes, ...foreignKeys];
  const summary = {
    added: allDiffs.filter(d => d.status === 'added').length,
    removed: allDiffs.filter(d => d.status === 'removed').length,
    changed: allDiffs.filter(d => d.status === 'changed').length,
    unchanged: allDiffs.filter(d => d.status === 'unchanged').length,
  };

  return { sourceTable, targetTable, columns, indexes, foreignKeys, summary };
}
