import type { DatabaseType } from '$lib/types/connection';
import type { VQState, VQTable } from '$lib/types/visualQuery';
import { quoteIdentifier } from '$lib/utils/sqlHelpers';

function resolveTableRef(table: VQTable, dbType: DatabaseType): string {
  const qualified = `${quoteIdentifier(table.schema, dbType)}.${quoteIdentifier(table.name, dbType)}`;
  if (table.alias && table.alias !== table.name) {
    return `${qualified} AS ${quoteIdentifier(table.alias, dbType)}`;
  }
  return qualified;
}

function tableAlias(table: VQTable, dbType: DatabaseType): string {
  return quoteIdentifier(table.alias || table.name, dbType);
}

export function generateSQL(state: VQState, dbType: DatabaseType): string {
  if (state.tables.length === 0) return '';

  const tableMap = new Map(state.tables.map(t => [t.id, t]));
  const parts: string[] = [];

  // SELECT
  const selectCols: string[] = [];
  for (const table of state.tables) {
    for (const colName of table.selectedColumns) {
      selectCols.push(`${tableAlias(table, dbType)}.${quoteIdentifier(colName, dbType)}`);
    }
  }
  if (selectCols.length === 0) {
    selectCols.push('*');
  }
  parts.push(`SELECT${state.distinct ? ' DISTINCT' : ''} ${selectCols.join(', ')}`);

  // FROM
  const firstTable = state.tables[0];
  parts.push(`FROM ${resolveTableRef(firstTable, dbType)}`);

  // JOINs
  for (const join of state.joins) {
    const srcTable = tableMap.get(join.sourceTableId);
    const tgtTable = tableMap.get(join.targetTableId);
    if (!srcTable || !tgtTable) continue;

    parts.push(`${join.joinType} ${resolveTableRef(tgtTable, dbType)} ON ${tableAlias(srcTable, dbType)}.${quoteIdentifier(join.sourceColumn, dbType)} = ${tableAlias(tgtTable, dbType)}.${quoteIdentifier(join.targetColumn, dbType)}`);
  }

  // WHERE
  if (state.where.length > 0) {
    const conditions: string[] = [];
    for (let i = 0; i < state.where.length; i++) {
      const w = state.where[i];
      const table = tableMap.get(w.tableId);
      if (!table) continue;
      const col = `${tableAlias(table, dbType)}.${quoteIdentifier(w.column, dbType)}`;
      let condition: string;

      if (w.operator === 'IS NULL' || w.operator === 'IS NOT NULL') {
        condition = `${col} ${w.operator}`;
      } else if (w.operator === 'IN' || w.operator === 'NOT IN') {
        condition = `${col} ${w.operator} (${w.value})`;
      } else if (w.operator === 'LIKE' || w.operator === 'NOT LIKE') {
        condition = `${col} ${w.operator} '${w.value.replace(/'/g, "''")}'`;
      } else {
        // For numeric operators, try to use numeric value; otherwise quote
        const isNumeric = /^-?\d+(\.\d+)?$/.test(w.value);
        condition = isNumeric
          ? `${col} ${w.operator} ${w.value}`
          : `${col} ${w.operator} '${w.value.replace(/'/g, "''")}'`;
      }

      if (i > 0) {
        conditions.push(`${w.connector} ${condition}`);
      } else {
        conditions.push(condition);
      }
    }
    if (conditions.length > 0) {
      parts.push(`WHERE ${conditions.join(' ')}`);
    }
  }

  // GROUP BY
  if (state.groupBy.length > 0) {
    const cols = state.groupBy
      .map(g => {
        const table = tableMap.get(g.tableId);
        if (!table) return null;
        return `${tableAlias(table, dbType)}.${quoteIdentifier(g.column, dbType)}`;
      })
      .filter(Boolean);
    if (cols.length > 0) {
      parts.push(`GROUP BY ${cols.join(', ')}`);
    }
  }

  // ORDER BY
  if (state.orderBy.length > 0) {
    const cols = state.orderBy
      .map(o => {
        const table = tableMap.get(o.tableId);
        if (!table) return null;
        return `${tableAlias(table, dbType)}.${quoteIdentifier(o.column, dbType)} ${o.direction}`;
      })
      .filter(Boolean);
    if (cols.length > 0) {
      parts.push(`ORDER BY ${cols.join(', ')}`);
    }
  }

  // LIMIT
  if (state.limit !== null && state.limit > 0) {
    if (dbType === 'MSSQL') {
      // MSSQL uses OFFSET...FETCH
      if (state.orderBy.length === 0) {
        parts.push('ORDER BY (SELECT NULL)');
      }
      parts.push(`OFFSET 0 ROWS FETCH NEXT ${state.limit} ROWS ONLY`);
    } else {
      parts.push(`LIMIT ${state.limit}`);
    }
  }

  return parts.join('\n');
}
