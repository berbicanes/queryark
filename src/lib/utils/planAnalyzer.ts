import type { QueryResponse } from '$lib/types/query';
import { extractCellValue } from '$lib/utils/formatters';

export interface PlanNode {
  type: string;
  table?: string;
  cost: number;
  startupCost?: number;
  actualTime?: number;
  startupTime?: number;
  rows: number;
  actualRows?: number;
  loops?: number;
  width?: number;
  children: PlanNode[];
  extra: Record<string, string>;
}

export interface ProfilingHint {
  severity: 'info' | 'warning' | 'critical';
  message: string;
  node: string;
  suggestion: string;
}

export interface TimelineEntry {
  node: string;
  start: number;
  duration: number;
  cost: number;
}

/**
 * Parse a query plan response into a PlanNode tree.
 */
export function parsePlanTree(planData: QueryResponse, dialect: string): PlanNode | null {
  if (!planData.rows.length) return null;

  const firstCell = extractCellValue(planData.rows[0][0]);

  if (firstCell.startsWith('[') || firstCell.startsWith('{')) {
    try {
      const json = JSON.parse(firstCell);

      if (dialect === 'MySQL' || dialect === 'MariaDB') {
        return parseMySqlJsonNode(json);
      }

      // PostgreSQL EXPLAIN (FORMAT JSON)
      const plan = Array.isArray(json) ? json[0]?.Plan ?? json[0] : json?.Plan ?? json;
      return parsePgJsonNode(plan);
    } catch {
      return null;
    }
  }

  // SQLite: EXPLAIN QUERY PLAN
  if (dialect === 'SQLite' && planData.columns.length >= 3) {
    return parseSQLitePlan(planData);
  }

  return null;
}

function parsePgJsonNode(node: Record<string, unknown>): PlanNode {
  const children: PlanNode[] = [];
  if (Array.isArray(node.Plans)) {
    for (const child of node.Plans) {
      children.push(parsePgJsonNode(child as Record<string, unknown>));
    }
  }

  const extra: Record<string, string> = {};
  const skip = new Set([
    'Node Type', 'Relation Name', 'Total Cost', 'Plan Rows',
    'Actual Total Time', 'Plan Width', 'Plans', 'Startup Cost',
    'Actual Startup Time', 'Actual Rows', 'Actual Loops',
  ]);

  for (const [k, v] of Object.entries(node)) {
    if (!skip.has(k) && v !== undefined && v !== null) {
      extra[k] = String(v);
    }
  }

  return {
    type: String(node['Node Type'] ?? 'Unknown'),
    table: node['Relation Name'] as string | undefined,
    cost: (node['Total Cost'] as number) ?? 0,
    startupCost: node['Startup Cost'] as number | undefined,
    actualTime: node['Actual Total Time'] as number | undefined,
    startupTime: node['Actual Startup Time'] as number | undefined,
    rows: (node['Plan Rows'] as number) ?? 0,
    actualRows: node['Actual Rows'] as number | undefined,
    loops: node['Actual Loops'] as number | undefined,
    width: node['Plan Width'] as number | undefined,
    children,
    extra,
  };
}

function parseMySqlJsonNode(json: Record<string, unknown>): PlanNode {
  const qb = json.query_block as Record<string, unknown> | undefined;
  if (!qb) return { type: 'Query', cost: 0, rows: 0, children: [], extra: {} };

  const children: PlanNode[] = [];
  const nested = qb.nested_loop as Array<Record<string, unknown>> | undefined;
  if (Array.isArray(nested)) {
    for (const item of nested) {
      const tbl = item.table as Record<string, unknown> | undefined;
      if (tbl) {
        children.push({
          type: String(tbl.access_type ?? 'scan'),
          table: String(tbl.table_name ?? ''),
          rows: (tbl.rows_examined_per_scan as number) ?? 0,
          cost: tbl.read_cost ? Number(tbl.read_cost) : 0,
          children: [],
          extra: { key: String(tbl.key ?? 'none'), used_columns: String(tbl.used_columns ?? '') },
        });
      }
    }
  }

  return {
    type: 'Query Block',
    cost: qb.cost_info ? Number((qb.cost_info as Record<string, unknown>).query_cost) : 0,
    rows: 0,
    children,
    extra: {},
  };
}

function parseSQLitePlan(data: QueryResponse): PlanNode {
  const root: PlanNode = { type: 'Query Plan', cost: 0, rows: 0, children: [], extra: {} };
  const nodeMap = new Map<number, PlanNode>();
  nodeMap.set(-1, root);

  for (const row of data.rows) {
    const id = Number(extractCellValue(row[0]));
    const parent = Number(extractCellValue(row[1]));
    const detail = extractCellValue(row[row.length - 1]);

    const node: PlanNode = { type: detail, cost: 0, rows: 0, children: [], extra: {} };
    nodeMap.set(id, node);
    const parentNode = nodeMap.get(parent) ?? root;
    parentNode.children.push(node);
  }

  return root;
}

/**
 * Analyze a plan tree and produce optimization hints.
 */
export function analyzePlan(root: PlanNode): ProfilingHint[] {
  const hints: ProfilingHint[] = [];
  visitNode(root, hints);
  return hints;
}

function visitNode(node: PlanNode, hints: ProfilingHint[]) {
  const typeLower = node.type.toLowerCase();

  // Sequential/full table scan on a table
  if ((typeLower.includes('seq scan') || typeLower.includes('full') || typeLower === 'scan') && node.table) {
    if (node.rows > 1000 || (node.actualRows !== undefined && node.actualRows > 1000)) {
      hints.push({
        severity: 'warning',
        message: `Sequential scan on "${node.table}" reading ${node.actualRows ?? node.rows} rows`,
        node: node.type,
        suggestion: `Consider adding an index on the filtered columns of "${node.table}"`,
      });
    }
  }

  // Nested loop with high row count
  if (typeLower.includes('nested loop')) {
    const totalRows = (node.actualRows ?? node.rows) * (node.loops ?? 1);
    if (totalRows > 10000) {
      hints.push({
        severity: 'warning',
        message: `Nested loop join producing ${totalRows.toLocaleString()} rows`,
        node: node.type,
        suggestion: 'Consider adding indexes on join columns or rewriting as a hash/merge join',
      });
    }
  }

  // Sort without index
  if (typeLower.includes('sort') && !typeLower.includes('index')) {
    if (node.actualTime !== undefined && node.actualTime > 100) {
      hints.push({
        severity: 'warning',
        message: `Sort operation taking ${node.actualTime.toFixed(1)}ms`,
        node: node.type,
        suggestion: 'Consider adding an index to support the ORDER BY clause',
      });
    }
  }

  // Row estimate drift
  if (node.actualRows !== undefined && node.rows > 0) {
    const ratio = node.actualRows / node.rows;
    if (ratio > 10 || ratio < 0.1) {
      hints.push({
        severity: 'info',
        message: `Estimated ${node.rows} rows but got ${node.actualRows} (${ratio.toFixed(1)}x drift)`,
        node: node.type,
        suggestion: 'Table statistics may be outdated — run ANALYZE on the table',
      });
    }
  }

  // High cost node
  if (node.cost > 10000) {
    hints.push({
      severity: node.cost > 100000 ? 'critical' : 'info',
      message: `High cost operation: ${node.cost.toFixed(0)}`,
      node: node.type,
      suggestion: 'Review this operation for optimization opportunities',
    });
  }

  for (const child of node.children) {
    visitNode(child, hints);
  }
}

/**
 * Flatten a plan tree into timeline entries for visualization.
 */
export function computeTimeline(root: PlanNode): TimelineEntry[] {
  const entries: TimelineEntry[] = [];
  flattenTimeline(root, entries);
  return entries;
}

function flattenTimeline(node: PlanNode, entries: TimelineEntry[]) {
  if (node.actualTime !== undefined) {
    const start = node.startupTime ?? 0;
    const duration = node.actualTime - start;
    entries.push({
      node: node.table ? `${node.type} on ${node.table}` : node.type,
      start,
      duration: Math.max(0, duration),
      cost: node.cost,
    });
  }

  for (const child of node.children) {
    flattenTimeline(child, entries);
  }
}

/**
 * Get the maximum cost from a plan tree.
 */
export function getMaxCost(node: PlanNode): number {
  let max = node.cost;
  for (const child of node.children) {
    max = Math.max(max, getMaxCost(child));
  }
  return max || 1;
}

/**
 * Get total execution time from a plan tree.
 */
export function getTotalTime(node: PlanNode): number {
  return node.actualTime ?? 0;
}

/**
 * Sum up all costs in the plan tree.
 */
export function getTotalCost(node: PlanNode): number {
  let total = node.cost;
  for (const child of node.children) {
    total += getTotalCost(child);
  }
  return total;
}
