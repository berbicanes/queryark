import type { DiagramTable, DiagramRelationship } from '$lib/types/diagram';

export const TABLE_WIDTH = 220;
export const HEADER_HEIGHT = 28;
export const COL_ROW_HEIGHT = 20;
export const GAP_X = 80;
export const GAP_Y = 60;
export const COLS_PER_ROW = 5;

/**
 * Compute a grid layout for tables, ordering by FK dependencies (tables with fewer
 * outgoing FKs placed first). Then arrange in rows of COLS_PER_ROW columns.
 */
export function computeGridLayout(
  tables: DiagramTable[],
  relationships: DiagramRelationship[]
): DiagramTable[] {
  // Build adjacency: for each table, count outgoing FK references
  const outDegree = new Map<string, number>();
  const inDegree = new Map<string, number>();
  for (const t of tables) {
    outDegree.set(t.id, 0);
    inDegree.set(t.id, 0);
  }
  for (const rel of relationships) {
    outDegree.set(rel.sourceTable, (outDegree.get(rel.sourceTable) ?? 0) + 1);
    inDegree.set(rel.targetTable, (inDegree.get(rel.targetTable) ?? 0) + 1);
  }

  // Topological-ish sort: tables referenced by many go first (they are "root" tables)
  const sorted = [...tables].sort((a, b) => {
    const aScore = (inDegree.get(a.id) ?? 0) - (outDegree.get(a.id) ?? 0);
    const bScore = (inDegree.get(b.id) ?? 0) - (outDegree.get(b.id) ?? 0);
    if (bScore !== aScore) return bScore - aScore;
    return a.name.localeCompare(b.name);
  });

  // Place in grid
  for (let i = 0; i < sorted.length; i++) {
    const col = i % COLS_PER_ROW;
    const row = Math.floor(i / COLS_PER_ROW);
    const tableHeight = HEADER_HEIGHT + sorted[i].columns.length * COL_ROW_HEIGHT;
    sorted[i].x = col * (TABLE_WIDTH + GAP_X);
    sorted[i].y = row * (Math.max(tableHeight, 150) + GAP_Y);
  }

  return sorted;
}

/**
 * Compute an SVG cubic bezier path from a source table/column to a target table/column.
 * Exits the right side of the source table, enters the left side of the target table.
 */
export function computeRelationshipPath(
  sourceTable: DiagramTable,
  sourceColIndex: number,
  targetTable: DiagramTable,
  targetColIndex: number
): string {
  const sy = sourceTable.y + HEADER_HEIGHT + sourceColIndex * COL_ROW_HEIGHT + COL_ROW_HEIGHT / 2;
  const ty = targetTable.y + HEADER_HEIGHT + targetColIndex * COL_ROW_HEIGHT + COL_ROW_HEIGHT / 2;

  // Determine which side to exit/enter based on relative positions
  let sx: number, tx: number, cp1x: number, cp2x: number;

  if (sourceTable.x + TABLE_WIDTH < targetTable.x) {
    // Source is to the left of target
    sx = sourceTable.x + TABLE_WIDTH;
    tx = targetTable.x;
    const midX = (sx + tx) / 2;
    cp1x = midX;
    cp2x = midX;
  } else if (targetTable.x + TABLE_WIDTH < sourceTable.x) {
    // Source is to the right of target
    sx = sourceTable.x;
    tx = targetTable.x + TABLE_WIDTH;
    const midX = (sx + tx) / 2;
    cp1x = midX;
    cp2x = midX;
  } else {
    // Overlapping horizontally — route around the right
    sx = sourceTable.x + TABLE_WIDTH;
    tx = targetTable.x + TABLE_WIDTH;
    cp1x = Math.max(sx, tx) + 40;
    cp2x = cp1x;
  }

  return `M ${sx} ${sy} C ${cp1x} ${sy}, ${cp2x} ${ty}, ${tx} ${ty}`;
}

/**
 * Compute the bounding box of all tables.
 */
export function computeBoundingBox(tables: DiagramTable[]): { x: number; y: number; width: number; height: number } {
  if (tables.length === 0) return { x: 0, y: 0, width: 800, height: 600 };

  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const t of tables) {
    const h = HEADER_HEIGHT + t.columns.length * COL_ROW_HEIGHT;
    minX = Math.min(minX, t.x);
    minY = Math.min(minY, t.y);
    maxX = Math.max(maxX, t.x + TABLE_WIDTH);
    maxY = Math.max(maxY, t.y + h);
  }

  const padding = 40;
  return {
    x: minX - padding,
    y: minY - padding,
    width: maxX - minX + padding * 2,
    height: maxY - minY + padding * 2
  };
}
