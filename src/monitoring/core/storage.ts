/**
 * AccuScene Enterprise v0.2.0
 * Metrics Storage
 *
 * In-memory time-series storage for metrics and traces
 */

import {
  MetricStorage,
  TraceStorage,
  MetricValue,
  MetricLabels,
  TimeSeries,
  TimeSeriesPoint,
  Span
} from '../types';

/**
 * In-memory metric storage
 */
export class InMemoryMetricStorage implements MetricStorage {
  private data: Map<string, MetricValue[]> = new Map();
  private readonly maxValuesPerMetric: number = 100000;

  /**
   * Store a metric value
   */
  public async store(metric: string, value: MetricValue): Promise<void> {
    let values = this.data.get(metric);

    if (!values) {
      values = [];
      this.data.set(metric, values);
    }

    values.push(value);

    // Limit storage size
    if (values.length > this.maxValuesPerMetric) {
      values.shift();
    }
  }

  /**
   * Query metric values in time range
   */
  public async query(
    metric: string,
    start: number,
    end: number,
    labels?: MetricLabels
  ): Promise<MetricValue[]> {
    const values = this.data.get(metric) || [];

    return values.filter(v => {
      // Time range filter
      if (v.timestamp < start || v.timestamp > end) {
        return false;
      }

      // Label filter
      if (labels && v.labels) {
        for (const [key, value] of Object.entries(labels)) {
          if (v.labels[key] !== value) {
            return false;
          }
        }
      }

      return true;
    });
  }

  /**
   * Query metric values with aggregation
   */
  public async queryRange(
    metric: string,
    start: number,
    end: number,
    step: number
  ): Promise<TimeSeries[]> {
    const values = await this.query(metric, start, end);

    if (values.length === 0) {
      return [];
    }

    // Group by labels
    const seriesMap = new Map<string, TimeSeriesPoint[]>();

    for (const value of values) {
      const labelKey = value.labels
        ? JSON.stringify(value.labels)
        : 'default';

      let points = seriesMap.get(labelKey);

      if (!points) {
        points = [];
        seriesMap.set(labelKey, points);
      }

      points.push({
        timestamp: value.timestamp,
        value: value.value,
        labels: value.labels
      });
    }

    // Convert to time series
    return Array.from(seriesMap.values()).map(points => ({
      metric,
      points: this.downsample(points, step)
    }));
  }

  /**
   * Downsample points to match step interval
   */
  private downsample(points: TimeSeriesPoint[], step: number): TimeSeriesPoint[] {
    if (points.length === 0 || step === 0) {
      return points;
    }

    const result: TimeSeriesPoint[] = [];
    const sorted = points.sort((a, b) => a.timestamp - b.timestamp);

    let bucket: TimeSeriesPoint[] = [];
    let bucketStart = sorted[0].timestamp;

    for (const point of sorted) {
      if (point.timestamp >= bucketStart + step) {
        // Aggregate bucket
        if (bucket.length > 0) {
          const avg = bucket.reduce((sum, p) => sum + p.value, 0) / bucket.length;

          result.push({
            timestamp: bucketStart,
            value: avg,
            labels: bucket[0].labels
          });
        }

        bucket = [point];
        bucketStart = point.timestamp;
      } else {
        bucket.push(point);
      }
    }

    // Add final bucket
    if (bucket.length > 0) {
      const avg = bucket.reduce((sum, p) => sum + p.value, 0) / bucket.length;

      result.push({
        timestamp: bucketStart,
        value: avg,
        labels: bucket[0].labels
      });
    }

    return result;
  }

  /**
   * Delete metric
   */
  public async delete(metric: string): Promise<void> {
    this.data.delete(metric);
  }

  /**
   * Clear all data
   */
  public async clear(): Promise<void> {
    this.data.clear();
  }

  /**
   * Get all metric names
   */
  public getMetricNames(): string[] {
    return Array.from(this.data.keys());
  }

  /**
   * Get storage size
   */
  public getSize(): number {
    let total = 0;

    for (const values of this.data.values()) {
      total += values.length;
    }

    return total;
  }
}

/**
 * In-memory trace storage
 */
export class InMemoryTraceStorage implements TraceStorage {
  private traces: Map<string, Span[]> = new Map();
  private readonly maxTracesPerTraceId: number = 1000;
  private readonly maxTraceIds: number = 10000;

  /**
   * Store a span
   */
  public async store(span: Span): Promise<void> {
    const traceId = span.context.traceId;
    let spans = this.traces.get(traceId);

    if (!spans) {
      spans = [];
      this.traces.set(traceId, spans);

      // Limit number of trace IDs
      if (this.traces.size > this.maxTraceIds) {
        const firstKey = this.traces.keys().next().value;
        this.traces.delete(firstKey);
      }
    }

    spans.push(span);

    // Limit spans per trace
    if (spans.length > this.maxTracesPerTraceId) {
      spans.shift();
    }
  }

  /**
   * Query all spans for a trace ID
   */
  public async query(traceId: string): Promise<Span[]> {
    return this.traces.get(traceId) || [];
  }

  /**
   * Query spans by time range
   */
  public async queryByTime(start: number, end: number): Promise<Span[]> {
    const result: Span[] = [];

    for (const spans of this.traces.values()) {
      for (const span of spans) {
        if (span.startTime >= start && span.startTime <= end) {
          result.push(span);
        }
      }
    }

    return result.sort((a, b) => a.startTime - b.startTime);
  }

  /**
   * Delete traces by trace ID
   */
  public async delete(traceId: string): Promise<void> {
    this.traces.delete(traceId);
  }

  /**
   * Clear all traces
   */
  public async clear(): Promise<void> {
    this.traces.clear();
  }

  /**
   * Get trace count
   */
  public getTraceCount(): number {
    return this.traces.size;
  }

  /**
   * Get total span count
   */
  public getSpanCount(): number {
    let total = 0;

    for (const spans of this.traces.values()) {
      total += spans.length;
    }

    return total;
  }

  /**
   * Get all trace IDs
   */
  public getTraceIds(): string[] {
    return Array.from(this.traces.keys());
  }
}

// Global storage instances
export const globalMetricStorage = new InMemoryMetricStorage();
export const globalTraceStorage = new InMemoryTraceStorage();
