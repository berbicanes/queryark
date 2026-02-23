export type ChartType = 'bar' | 'line' | 'pie';

export interface ChartConfig {
  type: ChartType;
  xColumn: string;
  yColumns: string[];
  title?: string;
  showLegend: boolean;
  showGrid: boolean;
}

export interface ChartDataPoint {
  label: string;
  values: number[];
}
