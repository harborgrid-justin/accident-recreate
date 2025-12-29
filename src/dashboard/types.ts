/**
 * TypeScript type definitions for AccuScene Enterprise Dashboard v0.2.5
 *
 * Provides comprehensive type safety for the dashboard system
 */

/**
 * Responsive breakpoint definitions
 */
export enum Breakpoint {
  Mobile = 'mobile',
  MobileLandscape = 'mobile-landscape',
  Tablet = 'tablet',
  Desktop = 'desktop',
  DesktopLarge = 'desktop-large',
  DesktopXL = 'desktop-xl',
}

/**
 * Breakpoint width thresholds in pixels
 */
export const BREAKPOINT_WIDTHS: Record<Breakpoint, { min: number; max?: number }> = {
  [Breakpoint.Mobile]: { min: 0, max: 575 },
  [Breakpoint.MobileLandscape]: { min: 576, max: 767 },
  [Breakpoint.Tablet]: { min: 768, max: 991 },
  [Breakpoint.Desktop]: { min: 992, max: 1199 },
  [Breakpoint.DesktopLarge]: { min: 1200, max: 1599 },
  [Breakpoint.DesktopXL]: { min: 1600 },
};

/**
 * Theme configuration
 */
export interface ThemeConfig {
  primaryColor: string;
  secondaryColor: string;
  backgroundColor: string;
  surfaceColor: string;
  textColor: string;
  darkMode: boolean;
  borderRadius: number;
  spacingUnit: number;
}

/**
 * Grid configuration for a specific breakpoint
 */
export interface GridConfig {
  columns: number;
  rowHeight: number;
  horizontalGap: number;
  verticalGap: number;
  containerPadding: number;
}

/**
 * Refresh configuration
 */
export interface RefreshConfig {
  enabled: boolean;
  intervalSeconds: number;
  refreshOnFocus: boolean;
  staleThresholdSeconds: number;
}

/**
 * Main dashboard configuration
 */
export interface DashboardConfig {
  id: string;
  name: string;
  description?: string;
  theme: ThemeConfig;
  gridConfigs: Record<Breakpoint, GridConfig>;
  refresh: RefreshConfig;
  animationsEnabled: boolean;
  dragDropEnabled: boolean;
  resizeEnabled: boolean;
  maxWidgets: number;
  persistenceEnabled: boolean;
  version: number;
}

/**
 * Widget position and size in the grid
 */
export interface WidgetLayout {
  widgetId: string;
  x: number;
  y: number;
  width: number;
  height: number;
  minWidth?: number;
  maxWidth?: number;
  minHeight?: number;
  maxHeight?: number;
  isStatic: boolean;
  isResizable: boolean;
  zIndex: number;
}

/**
 * Layout for a specific breakpoint
 */
export interface BreakpointLayout {
  breakpoint: Breakpoint;
  widgets: WidgetLayout[];
  columns: number;
  autoCompact: boolean;
}

/**
 * Responsive layout
 */
export interface ResponsiveLayout {
  dashboardId: string;
  breakpointLayouts: Record<Breakpoint, BreakpointLayout>;
  currentBreakpoint: Breakpoint;
}

/**
 * Widget types
 */
export enum WidgetType {
  Metrics = 'metrics',
  Chart = 'chart',
  Table = 'table',
  Text = 'text',
  Image = 'image',
  Custom = 'custom',
}

/**
 * Refresh strategy
 */
export type RefreshStrategy =
  | { type: 'manual' }
  | { type: 'interval'; seconds: number }
  | { type: 'realtime' }
  | { type: 'onDemand' };

/**
 * Data source configuration
 */
export interface DataSource {
  sourceType: string;
  endpoint: string;
  params: Record<string, string>;
  refresh: RefreshStrategy;
  requiresAuth: boolean;
  cacheTtl?: number;
}

/**
 * Widget metadata
 */
export interface WidgetMetadata {
  id: string;
  widgetType: WidgetType;
  title: string;
  description?: string;
  tags: string[];
  createdAt: string;
  updatedAt: string;
  version: number;
}

/**
 * Display options
 */
export interface DisplayOptions {
  showTitle: boolean;
  showBorder: boolean;
  backgroundColor?: string;
  textColor?: string;
  padding: number;
  cssClasses: string[];
}

/**
 * Interaction options
 */
export interface InteractionOptions {
  clickable: boolean;
  hoverable: boolean;
  drilldownEnabled: boolean;
  exportEnabled: boolean;
  fullscreenEnabled: boolean;
}

/**
 * Widget configuration
 */
export interface WidgetConfig {
  metadata: WidgetMetadata;
  dataSource?: DataSource;
  config: Record<string, any>;
  display: DisplayOptions;
  interaction: InteractionOptions;
}

/**
 * Widget data
 */
export interface WidgetData<T = any> {
  data: T;
  timestamp: string;
  isStale: boolean;
  error?: string;
  metadata: Record<string, string>;
}

/**
 * Widget state
 */
export interface WidgetState {
  config: WidgetConfig;
  data?: WidgetData;
  isLoading: boolean;
  error?: string;
  lastRefresh?: string;
  uiState: Record<string, any>;
}

/**
 * User preferences
 */
export interface UserPreferences {
  theme: string;
  autoRefresh: boolean;
  defaultPageSize: number;
  collapsedWidgets: string[];
  custom: Record<string, any>;
}

/**
 * Dashboard state
 */
export interface DashboardState {
  config: DashboardConfig;
  layout: ResponsiveLayout;
  widgets: Record<string, WidgetState>;
  preferences: UserPreferences;
  lastUpdated: string;
  version: number;
}

/**
 * Metric format types
 */
export enum MetricFormat {
  Number = 'number',
  Percentage = 'percentage',
  Currency = 'currency',
  Duration = 'duration',
  Bytes = 'bytes',
}

/**
 * Trend direction
 */
export enum TrendDirection {
  Up = 'up',
  Down = 'down',
  Stable = 'stable',
}

/**
 * Metric value
 */
export interface MetricValue {
  value: number;
  previousValue?: number;
  format: MetricFormat;
  trend?: TrendDirection;
  changePercent?: number;
  label: string;
  unit?: string;
  target?: number;
  color?: string;
}

/**
 * Metrics layout mode
 */
export enum MetricsLayout {
  Single = 'single',
  Grid = 'grid',
  Horizontal = 'horizontal',
  Vertical = 'vertical',
}

/**
 * Metrics configuration
 */
export interface MetricsConfig {
  metrics: MetricValue[];
  layout: MetricsLayout;
  showTrend: boolean;
  showSparkline: boolean;
  comparisonPeriod?: string;
}

/**
 * Chart types
 */
export enum ChartType {
  Line = 'line',
  Bar = 'bar',
  Area = 'area',
  Pie = 'pie',
  Donut = 'donut',
  Scatter = 'scatter',
  Heatmap = 'heatmap',
  Radar = 'radar',
}

/**
 * Data point for charts
 */
export interface DataPoint {
  x: any;
  y: number;
  metadata?: Record<string, any>;
}

/**
 * Data series
 */
export interface DataSeries {
  name: string;
  data: DataPoint[];
  color?: string;
  seriesType?: ChartType;
  visible: boolean;
}

/**
 * Axis configuration
 */
export interface AxisConfig {
  label?: string;
  axisType: 'linear' | 'logarithmic' | 'time' | 'category';
  showGrid: boolean;
  min?: number;
  max?: number;
  showAxis: boolean;
}

/**
 * Legend configuration
 */
export interface LegendConfig {
  show: boolean;
  position: 'top' | 'bottom' | 'left' | 'right';
  align: 'start' | 'center' | 'end';
}

/**
 * Tooltip configuration
 */
export interface TooltipConfig {
  enabled: boolean;
  format?: string;
  shared: boolean;
  sorted: boolean;
}

/**
 * Chart configuration
 */
export interface ChartConfig {
  chartType: ChartType;
  series: DataSeries[];
  xAxis: AxisConfig;
  yAxis: AxisConfig;
  legend: LegendConfig;
  tooltip: TooltipConfig;
  zoomEnabled: boolean;
  panEnabled: boolean;
  animationsEnabled: boolean;
  stacked: boolean;
  colorPalette: string[];
}

/**
 * Column data types
 */
export enum ColumnType {
  String = 'string',
  Number = 'number',
  Boolean = 'boolean',
  Date = 'date',
  DateTime = 'datetime',
  Currency = 'currency',
  Percentage = 'percentage',
  Custom = 'custom',
}

/**
 * Column alignment
 */
export enum ColumnAlign {
  Left = 'left',
  Center = 'center',
  Right = 'right',
}

/**
 * Column definition
 */
export interface ColumnDef {
  id: string;
  label: string;
  dataType: ColumnType;
  field: string;
  align: ColumnAlign;
  width?: number;
  minWidth?: number;
  sortable: boolean;
  filterable: boolean;
  visible: boolean;
  frozen: boolean;
  renderer?: string;
  format?: string;
}

/**
 * Sort configuration
 */
export interface SortConfig {
  columnId: string;
  direction: 'asc' | 'desc';
}

/**
 * Filter operator
 */
export enum FilterOperator {
  Equals = 'equals',
  NotEquals = 'notEquals',
  Contains = 'contains',
  StartsWith = 'startsWith',
  EndsWith = 'endsWith',
  GreaterThan = 'greaterThan',
  GreaterThanOrEqual = 'greaterThanOrEqual',
  LessThan = 'lessThan',
  LessThanOrEqual = 'lessThanOrEqual',
  In = 'in',
  NotIn = 'notIn',
}

/**
 * Filter configuration
 */
export interface FilterConfig {
  columnId: string;
  operator: FilterOperator;
  value: any;
}

/**
 * Pagination configuration
 */
export interface PaginationConfig {
  page: number;
  pageSize: number;
  totalRows: number;
  totalPages: number;
}

/**
 * Table row
 */
export interface TableRow {
  id: string;
  data: Record<string, any>;
  metadata?: any;
  selected: boolean;
}

/**
 * Table configuration
 */
export interface TableConfig {
  columns: ColumnDef[];
  sortingEnabled: boolean;
  filteringEnabled: boolean;
  paginationEnabled: boolean;
  defaultPageSize: number;
  selectionEnabled: boolean;
  multiSelect: boolean;
  columnReorderEnabled: boolean;
  columnResizeEnabled: boolean;
  virtualizationEnabled: boolean;
  showRowNumbers: boolean;
  striped: boolean;
  dense: boolean;
}

/**
 * Interaction event
 */
export interface InteractionEvent {
  eventType: string;
  data: any;
  timestamp: string;
}

/**
 * Export format
 */
export enum ExportFormat {
  Json = 'json',
  Csv = 'csv',
  Png = 'png',
  Pdf = 'pdf',
}
