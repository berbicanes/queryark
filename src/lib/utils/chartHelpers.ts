import type { CellValue, ColumnDef } from '$lib/types/query';
import type { ChartDataPoint } from '$lib/types/chart';

/**
 * Extract a numeric value from a CellValue, returning null if not numeric.
 */
export function extractNumericValue(cell: CellValue): number | null {
  switch (cell.type) {
    case 'Int': return cell.value;
    case 'Float': return cell.value;
    case 'Bool': return cell.value ? 1 : 0;
    case 'Text': {
      const n = parseFloat(cell.value);
      return isNaN(n) ? null : n;
    }
    case 'LargeText': {
      const n = parseFloat(cell.value.preview);
      return isNaN(n) ? null : n;
    }
    case 'Null': return null;
    default: return null;
  }
}

/**
 * Extract display text from a CellValue for chart labels.
 */
function extractLabelValue(cell: CellValue): string {
  switch (cell.type) {
    case 'Null': return 'NULL';
    case 'Bool': return cell.value ? 'true' : 'false';
    case 'Int': return String(cell.value);
    case 'Float': return String(cell.value);
    case 'Text': return cell.value.substring(0, 40);
    case 'Timestamp': return cell.value.substring(0, 19);
    case 'Json': return cell.value.substring(0, 40);
    case 'LargeText': return cell.value.preview.substring(0, 40);
    case 'LargeJson': return cell.value.preview.substring(0, 40);
    default: return '';
  }
}

/**
 * Map query result rows into chart data points.
 */
export function prepareChartData(
  columns: ColumnDef[],
  rows: CellValue[][],
  xColumn: string,
  yColumns: string[]
): ChartDataPoint[] {
  const xIdx = columns.findIndex(c => c.name === xColumn);
  const yIndices = yColumns.map(y => columns.findIndex(c => c.name === y)).filter(i => i >= 0);

  if (xIdx < 0 || yIndices.length === 0) return [];

  const points: ChartDataPoint[] = [];
  for (const row of rows) {
    const label = extractLabelValue(row[xIdx]);
    const values = yIndices.map(yi => extractNumericValue(row[yi]) ?? 0);
    points.push({ label, values });
  }
  return points;
}

const CHART_COLORS = [
  '#7aa2f7', // accent blue
  '#9ece6a', // green
  '#e0af68', // yellow/warning
  '#f38ba8', // pink/error
  '#bb9af7', // purple
  '#7dcfff', // cyan
  '#ff9e64', // orange
  '#73daca', // teal
];

/**
 * Return an array of distinct colors for chart series.
 */
export function generateColors(count: number): string[] {
  const colors: string[] = [];
  for (let i = 0; i < count; i++) {
    colors.push(CHART_COLORS[i % CHART_COLORS.length]);
  }
  return colors;
}

/**
 * Smart number formatting (1000 → 1K, etc.)
 */
export function formatAxisLabel(value: number): string {
  const abs = Math.abs(value);
  if (abs >= 1_000_000_000) return (value / 1_000_000_000).toFixed(1) + 'B';
  if (abs >= 1_000_000) return (value / 1_000_000).toFixed(1) + 'M';
  if (abs >= 1_000) return (value / 1_000).toFixed(1) + 'K';
  if (Number.isInteger(value)) return String(value);
  return value.toFixed(2);
}

/**
 * Compute nice axis scale with rounded tick marks.
 */
export function computeScale(values: number[]): { min: number; max: number; ticks: number[] } {
  if (values.length === 0) return { min: 0, max: 1, ticks: [0, 0.5, 1] };

  let min = Math.min(...values);
  let max = Math.max(...values);

  if (min === max) {
    min = min === 0 ? 0 : min * 0.9;
    max = max === 0 ? 1 : max * 1.1;
  }

  // Include 0 if range is close
  if (min > 0 && min < (max - min) * 0.5) min = 0;

  const range = max - min;
  const rawStep = range / 5;
  const magnitude = Math.pow(10, Math.floor(Math.log10(rawStep)));
  const normalized = rawStep / magnitude;

  let step: number;
  if (normalized <= 1) step = magnitude;
  else if (normalized <= 2) step = 2 * magnitude;
  else if (normalized <= 5) step = 5 * magnitude;
  else step = 10 * magnitude;

  const niceMin = Math.floor(min / step) * step;
  const niceMax = Math.ceil(max / step) * step;

  const ticks: number[] = [];
  for (let t = niceMin; t <= niceMax + step * 0.001; t += step) {
    ticks.push(Math.round(t * 1e10) / 1e10);
  }

  return { min: niceMin, max: niceMax, ticks };
}

/**
 * Auto-detect best column defaults for chart configuration.
 */
export function detectDefaults(columns: ColumnDef[]): { xColumn: string; yColumns: string[] } {
  const numericTypes = /^(int|float|double|decimal|numeric|bigint|smallint|real|money|number)/i;
  const textTypes = /^(varchar|text|char|name|timestamp|date|time|string)/i;

  let xColumn = columns[0]?.name ?? '';
  const yColumns: string[] = [];

  // Prefer first text/timestamp column for X axis
  for (const col of columns) {
    if (textTypes.test(col.data_type)) {
      xColumn = col.name;
      break;
    }
  }

  // Collect numeric columns for Y axis
  for (const col of columns) {
    if (col.name === xColumn) continue;
    if (numericTypes.test(col.data_type)) {
      yColumns.push(col.name);
    }
  }

  // If no numeric columns found, try all non-X columns
  if (yColumns.length === 0) {
    for (const col of columns) {
      if (col.name !== xColumn) {
        yColumns.push(col.name);
        break;
      }
    }
  }

  return { xColumn, yColumns: yColumns.slice(0, 3) };
}
