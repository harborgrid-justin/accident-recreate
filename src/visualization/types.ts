/**
 * AccuScene Visualization Types
 * TypeScript interfaces for advanced data visualization
 */

export interface DataPoint {
  x: number;
  y: number;
  label?: string;
  metadata?: Record<string, any>;
}

export interface TimeSeriesPoint {
  timestamp: number;
  value: number;
}

export interface SeriesData {
  name: string;
  data: DataPoint[];
  color?: string;
  chartType: ChartType;
  lineStyle?: LineStyle;
}

export type ChartType =
  | 'line'
  | 'area'
  | 'bar'
  | 'scatter'
  | 'heatmap'
  | 'radar'
  | 'sankey';

export type LineStyle = 'solid' | 'dashed' | 'dotted' | 'dashdot';

export interface AxisConfig {
  label: string;
  min?: number;
  max?: number;
  grid: boolean;
  tickFormat?: string;
}

export interface ChartData {
  title: string;
  xAxis: AxisConfig;
  yAxis: AxisConfig;
  series: SeriesData[];
}

export interface ChartConfig {
  width: number;
  height: number;
  margin: Margin;
  colors: ColorScheme;
  font: FontConfig;
  animation: AnimationConfig;
  responsive?: boolean;
}

export interface Margin {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export interface ColorScheme {
  primary: string[];
  background: string;
  text: string;
  grid: string;
  axis: string;
}

export interface FontConfig {
  family: string;
  size: number;
  weight: string;
}

export interface AnimationConfig {
  enabled: boolean;
  duration: number;
  easing: string;
}

export interface TooltipData {
  x: number;
  y: number;
  label: string;
  value: number | string;
  color: string;
  series: string;
}

export interface LegendItem {
  name: string;
  color: string;
  visible: boolean;
  chartType: ChartType;
}

export interface ZoomState {
  xDomain: [number, number] | null;
  yDomain: [number, number] | null;
}

export interface ChartInteraction {
  zoom: boolean;
  pan: boolean;
  brush: boolean;
  tooltip: boolean;
  legend: boolean;
}

// Statistical types
export interface StatisticalSummary {
  count: number;
  mean: number;
  median: number;
  stdDev: number;
  variance: number;
  min: number;
  max: number;
  quartiles: Quartiles;
}

export interface Quartiles {
  q1: number;
  q2: number;
  q3: number;
}

export interface HistogramBin {
  start: number;
  end: number;
  count: number;
  frequency: number;
}

export interface BoxPlotData {
  min: number;
  q1: number;
  median: number;
  q3: number;
  max: number;
  outliers: number[];
}

export interface CorrelationEntry {
  variable1: string;
  variable2: string;
  correlation: number;
  pValue?: number;
}

export interface CorrelationMatrix {
  variables: string[];
  matrix: number[][];
}

export interface LinearRegression {
  slope: number;
  intercept: number;
  rSquared: number;
}

// Time series types
export interface TimeSeriesAnalysis {
  trend: DataPoint[];
  seasonal: DataPoint[];
  residual: DataPoint[];
  statistics: TimeSeriesStats;
}

export interface TimeSeriesStats {
  mean: number;
  stdDev: number;
  autocorrelation: number[];
  trendSlope: number;
}

// Heatmap types
export interface HeatmapCell {
  x: number;
  y: number;
  value: number;
  xLabel?: string;
  yLabel?: string;
}

export interface HeatmapData {
  data: HeatmapCell[];
  xLabels: string[];
  yLabels: string[];
  colorScale: 'sequential' | 'diverging' | 'categorical';
}

// Radar chart types
export interface RadarDataPoint {
  axis: string;
  value: number;
}

export interface RadarSeries {
  name: string;
  data: RadarDataPoint[];
  color?: string;
}

// Sankey diagram types
export interface SankeyNode {
  id: string;
  label: string;
  color?: string;
}

export interface SankeyLink {
  source: string;
  target: string;
  value: number;
  color?: string;
}

export interface SankeyData {
  nodes: SankeyNode[];
  links: SankeyLink[];
}

// Export types
export type ExportFormat = 'svg' | 'png' | 'json' | 'csv';

export interface ExportConfig {
  format: ExportFormat;
  width: number;
  height: number;
  dpi: number;
  backgroundColor: string;
}

export interface ExportResult {
  format: ExportFormat;
  data: string;
  mimeType: string;
}

// Chart component props
export interface BaseChartProps {
  data: ChartData;
  config?: Partial<ChartConfig>;
  interaction?: Partial<ChartInteraction>;
  onPointClick?: (point: DataPoint, series: SeriesData) => void;
  onZoomChange?: (zoom: ZoomState) => void;
  className?: string;
  style?: React.CSSProperties;
}

export interface LineChartProps extends BaseChartProps {
  showPoints?: boolean;
  curve?: 'linear' | 'monotone' | 'step' | 'basis';
}

export interface AreaChartProps extends BaseChartProps {
  stacked?: boolean;
  curve?: 'linear' | 'monotone' | 'step' | 'basis';
  fillOpacity?: number;
}

export interface BarChartProps extends BaseChartProps {
  horizontal?: boolean;
  stacked?: boolean;
  grouped?: boolean;
  barWidth?: number;
}

export interface ScatterPlotProps extends BaseChartProps {
  showRegression?: boolean;
  pointSize?: number;
}

export interface HeatmapChartProps {
  data: HeatmapData;
  config?: Partial<ChartConfig>;
  colorScheme?: string[];
  showValues?: boolean;
  className?: string;
  style?: React.CSSProperties;
}

export interface RadarChartProps {
  data: RadarSeries[];
  config?: Partial<ChartConfig>;
  maxValue?: number;
  levels?: number;
  className?: string;
  style?: React.CSSProperties;
}

export interface SankeyDiagramProps {
  data: SankeyData;
  config?: Partial<ChartConfig>;
  nodeWidth?: number;
  nodePadding?: number;
  className?: string;
  style?: React.CSSProperties;
}

// Utility types
export type Scale = d3.ScaleLinear<number, number> | d3.ScaleTime<number, number>;

export interface Scales {
  x: Scale;
  y: Scale;
}

export interface Dimensions {
  width: number;
  height: number;
  boundedWidth: number;
  boundedHeight: number;
}
