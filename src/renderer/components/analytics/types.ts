/**
 * AccuScene Enterprise v0.3.0 - Analytics Dashboard Types
 * Complete TypeScript type definitions for analytics system
 */

import { ReactNode } from 'react';

// ============================================================================
// Core Data Types
// ============================================================================

export interface Vector3D {
  x: number;
  y: number;
  z: number;
}

export interface Vector2D {
  x: number;
  y: number;
}

export interface TimeSeriesDataPoint {
  timestamp: number;
  value: number;
  label?: string;
  metadata?: Record<string, any>;
}

export interface PhysicsDataPoint {
  time: number;
  position: Vector3D;
  velocity: Vector3D;
  acceleration: Vector3D;
  force: Vector3D;
  energy: number;
  metadata?: Record<string, any>;
}

// ============================================================================
// Vehicle & Impact Data
// ============================================================================

export interface VehicleData {
  id: string;
  name: string;
  mass: number;
  dimensions: {
    length: number;
    width: number;
    height: number;
  };
  damageProfile: DamagePoint[];
  trajectory: TrajectoryPoint[];
  speed: TimeSeriesDataPoint[];
  forces: Vector3D[];
}

export interface DamagePoint {
  id: string;
  location: Vector3D;
  severity: number; // 0-1
  type: 'crush' | 'scratch' | 'dent' | 'break' | 'deformation';
  area: number;
  depth: number;
  timestamp?: number;
}

export interface TrajectoryPoint {
  time: number;
  position: Vector3D;
  velocity: Vector3D;
  heading: number;
  roll: number;
  pitch: number;
  yaw: number;
}

export interface ImpactEvent {
  id: string;
  timestamp: number;
  location: Vector3D;
  force: Vector3D;
  energy: number;
  duration: number;
  vehicles: string[];
  severity: number;
  type: 'primary' | 'secondary' | 'rollover' | 'collision';
}

// ============================================================================
// Energy & Force Analysis
// ============================================================================

export interface EnergyTransfer {
  source: string;
  target: string;
  amount: number;
  type: 'kinetic' | 'potential' | 'heat' | 'deformation';
  efficiency: number;
}

export interface ForceVector {
  id: string;
  origin: Vector3D;
  direction: Vector3D;
  magnitude: number;
  timestamp: number;
  type: 'impact' | 'friction' | 'normal' | 'drag';
  color?: string;
}

// ============================================================================
// Widget System
// ============================================================================

export type WidgetType =
  | 'speed-chart'
  | 'force-vector'
  | 'impact-analysis'
  | 'energy-flow'
  | 'trajectory'
  | 'damage-heatmap'
  | 'timeline'
  | 'statistics'
  | 'comparison'
  | 'data-table'
  | 'report-summary'
  | 'line-chart'
  | 'bar-chart'
  | 'pie-chart'
  | 'scatter-plot'
  | 'radar-chart'
  | 'gauge-chart';

export interface WidgetConfig {
  id: string;
  type: WidgetType;
  title: string;
  description?: string;
  position: {
    x: number;
    y: number;
    w: number;
    h: number;
  };
  settings: Record<string, any>;
  dataSource?: string;
  refreshRate?: number; // ms
  isVisible: boolean;
  isLocked?: boolean;
}

export interface WidgetProps<T = any> {
  config: WidgetConfig;
  data: T;
  onUpdate?: (config: WidgetConfig) => void;
  onRemove?: (id: string) => void;
  onResize?: (id: string, width: number, height: number) => void;
  isEditing?: boolean;
}

export interface WidgetRegistryEntry {
  type: WidgetType;
  name: string;
  description: string;
  icon: string;
  component: React.ComponentType<WidgetProps<any>>;
  defaultSize: { w: number; h: number };
  minSize: { w: number; h: number };
  maxSize?: { w: number; h: number };
  defaultSettings: Record<string, any>;
  category: 'analysis' | 'visualization' | 'data' | 'summary';
}

// ============================================================================
// Dashboard Layout
// ============================================================================

export interface DashboardLayout {
  id: string;
  name: string;
  description?: string;
  widgets: WidgetConfig[];
  gridSettings: GridSettings;
  theme?: DashboardTheme;
  createdAt: number;
  updatedAt: number;
  author?: string;
  isDefault?: boolean;
}

export interface GridSettings {
  cols: number;
  rowHeight: number;
  margin: [number, number];
  containerPadding: [number, number];
  compactType: 'vertical' | 'horizontal' | null;
  preventCollision: boolean;
  isDraggable: boolean;
  isResizable: boolean;
}

export interface DashboardTheme {
  primary: string;
  secondary: string;
  background: string;
  surface: string;
  text: string;
  textSecondary: string;
  border: string;
  success: string;
  warning: string;
  error: string;
  info: string;
}

// ============================================================================
// Chart Configuration
// ============================================================================

export interface ChartConfig {
  type: 'line' | 'bar' | 'pie' | 'scatter' | 'radar' | 'gauge';
  title?: string;
  subtitle?: string;
  xAxis?: AxisConfig;
  yAxis?: AxisConfig;
  legend?: LegendConfig;
  tooltip?: TooltipConfig;
  colors?: string[];
  animations?: AnimationConfig;
  responsive?: boolean;
}

export interface AxisConfig {
  label?: string;
  min?: number;
  max?: number;
  tickCount?: number;
  format?: (value: number) => string;
  scale?: 'linear' | 'log' | 'time';
  grid?: boolean;
}

export interface LegendConfig {
  show: boolean;
  position: 'top' | 'bottom' | 'left' | 'right';
  align?: 'start' | 'center' | 'end';
}

export interface TooltipConfig {
  show: boolean;
  format?: (value: any) => string;
  shared?: boolean;
}

export interface AnimationConfig {
  duration: number;
  easing: 'linear' | 'ease' | 'ease-in' | 'ease-out' | 'ease-in-out';
  delay?: number;
}

// ============================================================================
// Data Series
// ============================================================================

export interface DataSeries {
  id: string;
  name: string;
  data: any[];
  type?: 'line' | 'bar' | 'area' | 'scatter';
  color?: string;
  visible?: boolean;
  yAxis?: number;
}

export interface ChartData {
  series: DataSeries[];
  categories?: string[];
  metadata?: Record<string, any>;
}

// ============================================================================
// Export & Report Types
// ============================================================================

export type ExportFormat = 'csv' | 'xlsx' | 'pdf' | 'json' | 'png' | 'svg';

export interface ExportOptions {
  format: ExportFormat;
  filename?: string;
  includeMetadata?: boolean;
  includeCharts?: boolean;
  includeTables?: boolean;
  pageSize?: 'letter' | 'a4' | 'legal';
  orientation?: 'portrait' | 'landscape';
  compression?: boolean;
}

export interface ReportSection {
  id: string;
  type: 'text' | 'chart' | 'table' | 'image' | 'summary';
  title?: string;
  content: any;
  order: number;
}

export interface ReportTemplate {
  id: string;
  name: string;
  description?: string;
  sections: ReportSection[];
  metadata?: Record<string, any>;
}

export interface GeneratedReport {
  id: string;
  template: string;
  data: any;
  generatedAt: number;
  author?: string;
  sections: ReportSection[];
  metadata?: Record<string, any>;
}

// ============================================================================
// Statistics & KPIs
// ============================================================================

export interface StatisticValue {
  label: string;
  value: number | string;
  unit?: string;
  change?: number;
  changeType?: 'increase' | 'decrease' | 'neutral';
  trend?: number[];
  format?: 'number' | 'percentage' | 'currency' | 'time' | 'distance';
  precision?: number;
  icon?: string;
  color?: string;
}

export interface KPICard {
  id: string;
  title: string;
  statistics: StatisticValue[];
  threshold?: {
    warning: number;
    critical: number;
  };
  status?: 'normal' | 'warning' | 'critical';
}

// ============================================================================
// Timeline & Events
// ============================================================================

export interface TimelineEvent {
  id: string;
  timestamp: number;
  title: string;
  description?: string;
  type: 'impact' | 'vehicle-action' | 'environment' | 'system' | 'annotation';
  severity?: 'low' | 'medium' | 'high' | 'critical';
  data?: any;
  icon?: string;
  color?: string;
}

export interface TimelineRange {
  start: number;
  end: number;
  step?: number;
}

// ============================================================================
// Comparison Types
// ============================================================================

export interface ComparisonData {
  before: any;
  after: any;
  differences?: ComparisonDifference[];
  metadata?: Record<string, any>;
}

export interface ComparisonDifference {
  field: string;
  before: any;
  after: any;
  change: number | string;
  changePercent?: number;
  significance: 'major' | 'minor' | 'negligible';
}

// ============================================================================
// Data Table Types
// ============================================================================

export interface TableColumn<T = any> {
  id: string;
  header: string;
  accessor: keyof T | ((row: T) => any);
  sortable?: boolean;
  filterable?: boolean;
  width?: number;
  format?: (value: any, row: T) => ReactNode;
  align?: 'left' | 'center' | 'right';
}

export interface TableConfig<T = any> {
  columns: TableColumn<T>[];
  data: T[];
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  filters?: Record<string, any>;
  pageSize?: number;
  currentPage?: number;
  selectable?: boolean;
  expandable?: boolean;
}

// ============================================================================
// Real-time Data Streaming
// ============================================================================

export interface StreamConfig {
  endpoint: string;
  updateInterval: number;
  bufferSize?: number;
  reconnect?: boolean;
  reconnectInterval?: number;
}

export interface DataStream<T = any> {
  id: string;
  config: StreamConfig;
  data: T[];
  isConnected: boolean;
  lastUpdate: number;
  error?: Error;
}

// ============================================================================
// Analytics Hooks Return Types
// ============================================================================

export interface AnalyticsData {
  vehicles: VehicleData[];
  impacts: ImpactEvent[];
  energyTransfers: EnergyTransfer[];
  forceVectors: ForceVector[];
  timeline: TimelineEvent[];
  metadata: Record<string, any>;
  timestamp: number;
}

export interface AnalyticsState {
  data: AnalyticsData | null;
  loading: boolean;
  error: Error | null;
  refresh: () => Promise<void>;
  updateData: (data: Partial<AnalyticsData>) => void;
}

export interface WidgetState {
  config: WidgetConfig;
  data: any;
  loading: boolean;
  error: Error | null;
  update: (config: Partial<WidgetConfig>) => void;
  refresh: () => Promise<void>;
}

export interface ExportState {
  exporting: boolean;
  progress: number;
  error: Error | null;
  exportData: (options: ExportOptions) => Promise<void>;
  cancel: () => void;
}

// ============================================================================
// Heatmap Types
// ============================================================================

export interface HeatmapPoint {
  x: number;
  y: number;
  value: number;
  label?: string;
}

export interface HeatmapConfig {
  width: number;
  height: number;
  colorScale?: string[];
  min?: number;
  max?: number;
  radius?: number;
  blur?: number;
  opacity?: number;
}

// ============================================================================
// 3D Visualization Types
// ============================================================================

export interface Camera3DConfig {
  position: Vector3D;
  target: Vector3D;
  fov: number;
  near: number;
  far: number;
}

export interface Scene3DConfig {
  background: string;
  lighting: {
    ambient: number;
    directional: {
      color: string;
      intensity: number;
      position: Vector3D;
    }[];
  };
  camera: Camera3DConfig;
  grid?: boolean;
  axes?: boolean;
}

// ============================================================================
// Utility Types
// ============================================================================

export type DataPoint = TimeSeriesDataPoint | PhysicsDataPoint | any;

export type ChartType = 'line' | 'bar' | 'pie' | 'scatter' | 'radar' | 'gauge';

export type SeriesType = 'line' | 'bar' | 'area' | 'scatter';

export interface Bounds {
  min: number;
  max: number;
}

export interface Range {
  start: number;
  end: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface Position {
  x: number;
  y: number;
}
