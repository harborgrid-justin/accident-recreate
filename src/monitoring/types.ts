/**
 * AccuScene Enterprise v0.2.0
 * Performance Monitoring System - Type Definitions
 *
 * Comprehensive type definitions for the monitoring system
 */

// ===== Metric Types =====

export enum MetricType {
  COUNTER = 'counter',
  GAUGE = 'gauge',
  HISTOGRAM = 'histogram',
  SUMMARY = 'summary',
  TIMER = 'timer'
}

export interface MetricLabels {
  [key: string]: string | number;
}

export interface MetricValue {
  value: number;
  timestamp: number;
  labels?: MetricLabels;
}

export interface MetricMetadata {
  name: string;
  type: MetricType;
  help: string;
  unit?: string;
  labels?: string[];
}

export interface Metric {
  metadata: MetricMetadata;
  getValue(): number | MetricSnapshot;
  getValues(): MetricValue[];
  reset(): void;
}

export interface MetricSnapshot {
  count: number;
  sum: number;
  min: number;
  max: number;
  avg: number;
  p50?: number;
  p95?: number;
  p99?: number;
}

// ===== Tracing Types =====

export enum SpanKind {
  INTERNAL = 'internal',
  SERVER = 'server',
  CLIENT = 'client',
  PRODUCER = 'producer',
  CONSUMER = 'consumer'
}

export enum SpanStatus {
  UNSET = 'unset',
  OK = 'ok',
  ERROR = 'error'
}

export interface SpanContext {
  traceId: string;
  spanId: string;
  traceFlags: number;
  traceState?: string;
  isRemote?: boolean;
}

export interface SpanAttributes {
  [key: string]: string | number | boolean | string[] | number[] | boolean[];
}

export interface SpanEvent {
  name: string;
  timestamp: number;
  attributes?: SpanAttributes;
}

export interface SpanLink {
  context: SpanContext;
  attributes?: SpanAttributes;
}

export interface Span {
  context: SpanContext;
  name: string;
  kind: SpanKind;
  parentSpanId?: string;
  startTime: number;
  endTime?: number;
  status: SpanStatus;
  attributes: SpanAttributes;
  events: SpanEvent[];
  links: SpanLink[];

  setAttribute(key: string, value: string | number | boolean): void;
  setAttributes(attributes: SpanAttributes): void;
  addEvent(name: string, attributes?: SpanAttributes): void;
  setStatus(status: SpanStatus, message?: string): void;
  end(timestamp?: number): void;
}

export interface TraceContext {
  traceId: string;
  spanId: string;
  parentSpanId?: string;
}

export interface TracingConfig {
  serviceName: string;
  sampleRate: number;
  maxSpansPerTrace: number;
  exporterEndpoint?: string;
  exportInterval?: number;
}

// ===== Profiling Types =====

export interface CPUProfile {
  timestamp: number;
  duration: number;
  samples: number;
  nodes: ProfileNode[];
}

export interface ProfileNode {
  id: number;
  callFrame: {
    functionName: string;
    scriptId: string;
    url: string;
    lineNumber: number;
    columnNumber: number;
  };
  hitCount: number;
  children?: number[];
  positionTicks?: {
    line: number;
    ticks: number;
  }[];
}

export interface MemoryProfile {
  timestamp: number;
  heapUsed: number;
  heapTotal: number;
  external: number;
  arrayBuffers: number;
  rss?: number;
  allocations?: AllocationProfile[];
}

export interface AllocationProfile {
  type: string;
  size: number;
  count: number;
  stack?: string[];
}

export interface RenderProfile {
  timestamp: number;
  fps: number;
  frameTime: number;
  longFrames: number;
  droppedFrames: number;
  components?: ComponentProfile[];
}

export interface ComponentProfile {
  name: string;
  renderCount: number;
  renderTime: number;
  avgRenderTime: number;
  maxRenderTime: number;
}

export interface NetworkProfile {
  timestamp: number;
  requests: NetworkRequest[];
  totalRequests: number;
  totalBytes: number;
  avgLatency: number;
}

export interface NetworkRequest {
  id: string;
  method: string;
  url: string;
  status: number;
  startTime: number;
  endTime: number;
  duration: number;
  size: number;
  cached?: boolean;
}

// ===== Health Check Types =====

export enum HealthStatus {
  HEALTHY = 'healthy',
  DEGRADED = 'degraded',
  UNHEALTHY = 'unhealthy'
}

export interface HealthCheck {
  name: string;
  check(): Promise<HealthCheckResult>;
}

export interface HealthCheckResult {
  status: HealthStatus;
  message?: string;
  timestamp: number;
  details?: Record<string, any>;
}

export interface HealthReport {
  status: HealthStatus;
  checks: Record<string, HealthCheckResult>;
  timestamp: number;
}

// ===== Alerting Types =====

export enum AlertSeverity {
  INFO = 'info',
  WARNING = 'warning',
  ERROR = 'error',
  CRITICAL = 'critical'
}

export enum AlertState {
  PENDING = 'pending',
  FIRING = 'firing',
  RESOLVED = 'resolved'
}

export interface AlertRule {
  id: string;
  name: string;
  description: string;
  severity: AlertSeverity;
  condition: AlertCondition;
  duration?: number;
  labels?: MetricLabels;
  annotations?: Record<string, string>;
}

export interface AlertCondition {
  metric: string;
  operator: 'gt' | 'gte' | 'lt' | 'lte' | 'eq' | 'neq';
  threshold: number;
  aggregation?: 'avg' | 'sum' | 'min' | 'max' | 'count';
  window?: number;
}

export interface Alert {
  id: string;
  rule: AlertRule;
  state: AlertState;
  value: number;
  startsAt: number;
  endsAt?: number;
  labels: MetricLabels;
  annotations: Record<string, string>;
}

export interface AlertNotification {
  alert: Alert;
  timestamp: number;
}

export type AlertHandler = (notification: AlertNotification) => Promise<void>;

// ===== Dashboard Types =====

export interface DashboardMetric {
  name: string;
  value: number;
  unit?: string;
  trend?: number;
  sparkline?: number[];
}

export interface DashboardPanel {
  id: string;
  title: string;
  type: 'metric' | 'chart' | 'table' | 'log';
  data: any;
  config?: Record<string, any>;
}

export interface TimeSeriesPoint {
  timestamp: number;
  value: number;
  labels?: MetricLabels;
}

export interface TimeSeries {
  metric: string;
  points: TimeSeriesPoint[];
}

// ===== Storage Types =====

export interface MetricStorage {
  store(metric: string, value: MetricValue): Promise<void>;
  query(metric: string, start: number, end: number, labels?: MetricLabels): Promise<MetricValue[]>;
  queryRange(metric: string, start: number, end: number, step: number): Promise<TimeSeries[]>;
  delete(metric: string): Promise<void>;
  clear(): Promise<void>;
}

export interface TraceStorage {
  store(span: Span): Promise<void>;
  query(traceId: string): Promise<Span[]>;
  queryByTime(start: number, end: number): Promise<Span[]>;
  delete(traceId: string): Promise<void>;
  clear(): Promise<void>;
}

// ===== Integration Types =====

export interface PrometheusMetric {
  name: string;
  type: string;
  help: string;
  metrics: Array<{
    value: number;
    labels?: Record<string, string>;
    timestamp?: number;
  }>;
}

export interface OpenTelemetrySpan {
  traceId: string;
  spanId: string;
  parentSpanId?: string;
  name: string;
  kind: number;
  startTimeUnixNano: string;
  endTimeUnixNano?: string;
  attributes: Array<{
    key: string;
    value: {
      stringValue?: string;
      intValue?: string;
      doubleValue?: number;
      boolValue?: boolean;
    };
  }>;
  status: {
    code: number;
    message?: string;
  };
  events: Array<{
    timeUnixNano: string;
    name: string;
    attributes?: any[];
  }>;
}

export interface SentryEvent {
  event_id: string;
  timestamp: number;
  level: 'fatal' | 'error' | 'warning' | 'info' | 'debug';
  message?: string;
  transaction?: string;
  tags?: Record<string, string>;
  extra?: Record<string, any>;
  breadcrumbs?: Array<{
    timestamp: number;
    message?: string;
    category?: string;
    level?: string;
    data?: Record<string, any>;
  }>;
}

// ===== Collector Types =====

export interface CollectorConfig {
  interval: number;
  batchSize: number;
  maxBufferSize: number;
  flushInterval: number;
}

export interface CollectedMetrics {
  timestamp: number;
  metrics: Map<string, MetricValue[]>;
  traces: Span[];
  profiles: {
    cpu?: CPUProfile;
    memory?: MemoryProfile;
    render?: RenderProfile;
    network?: NetworkProfile;
  };
}

// ===== Reporter Types =====

export interface ReportConfig {
  format: 'json' | 'prometheus' | 'opentelemetry';
  destination: 'console' | 'file' | 'http';
  endpoint?: string;
  interval?: number;
}

export interface PerformanceReport {
  timestamp: number;
  period: {
    start: number;
    end: number;
  };
  metrics: Record<string, MetricSnapshot>;
  traces: {
    total: number;
    sampled: number;
    errors: number;
  };
  health: HealthReport;
  alerts: Alert[];
}
