/**
 * AccuScene Enterprise v0.2.0
 * Metrics Aggregator
 *
 * Aggregates metrics over time windows
 */

import { MetricValue, MetricSnapshot, TimeSeries, TimeSeriesPoint } from '../types';

export class MetricsAggregator {
  private timeSeriesData: Map<string, TimeSeriesPoint[]> = new Map();
  private readonly maxDataPoints: number = 10000;

  /**
   * Add metric value to time series
   */
  public addValue(metric: string, value: MetricValue): void {
    let series = this.timeSeriesData.get(metric);

    if (!series) {
      series = [];
      this.timeSeriesData.set(metric, series);
    }

    series.push({
      timestamp: value.timestamp,
      value: value.value,
      labels: value.labels
    });

    // Limit data points
    if (series.length > this.maxDataPoints) {
      series.shift();
    }
  }

  /**
   * Add multiple values
   */
  public addValues(metric: string, values: MetricValue[]): void {
    for (const value of values) {
      this.addValue(metric, value);
    }
  }

  /**
   * Get time series for a metric
   */
  public getTimeSeries(metric: string): TimeSeries | undefined {
    const points = this.timeSeriesData.get(metric);

    if (!points) {
      return undefined;
    }

    return {
      metric,
      points: [...points]
    };
  }

  /**
   * Get time series for time range
   */
  public getTimeSeriesRange(
    metric: string,
    start: number,
    end: number
  ): TimeSeries | undefined {
    const allPoints = this.timeSeriesData.get(metric);

    if (!allPoints) {
      return undefined;
    }

    const points = allPoints.filter(
      point => point.timestamp >= start && point.timestamp <= end
    );

    return {
      metric,
      points
    };
  }

  /**
   * Aggregate values over a time window
   */
  public aggregate(
    metric: string,
    start: number,
    end: number,
    method: 'avg' | 'sum' | 'min' | 'max' | 'count' = 'avg'
  ): number {
    const series = this.getTimeSeriesRange(metric, start, end);

    if (!series || series.points.length === 0) {
      return 0;
    }

    const values = series.points.map(p => p.value);

    switch (method) {
      case 'sum':
        return values.reduce((a, b) => a + b, 0);

      case 'avg':
        return values.reduce((a, b) => a + b, 0) / values.length;

      case 'min':
        return Math.min(...values);

      case 'max':
        return Math.max(...values);

      case 'count':
        return values.length;

      default:
        return 0;
    }
  }

  /**
   * Calculate rate of change
   */
  public rate(metric: string, windowMs: number = 60000): number {
    const now = Date.now();
    const start = now - windowMs;

    const series = this.getTimeSeriesRange(metric, start, now);

    if (!series || series.points.length < 2) {
      return 0;
    }

    const first = series.points[0];
    const last = series.points[series.points.length - 1];

    const valueDelta = last.value - first.value;
    const timeDelta = (last.timestamp - first.timestamp) / 1000; // Convert to seconds

    return timeDelta > 0 ? valueDelta / timeDelta : 0;
  }

  /**
   * Downsample time series to fewer data points
   */
  public downsample(
    metric: string,
    targetPoints: number,
    method: 'avg' | 'max' | 'min' | 'last' = 'avg'
  ): TimeSeries | undefined {
    const series = this.getTimeSeries(metric);

    if (!series || series.points.length <= targetPoints) {
      return series;
    }

    const bucketSize = Math.ceil(series.points.length / targetPoints);
    const downsampled: TimeSeriesPoint[] = [];

    for (let i = 0; i < series.points.length; i += bucketSize) {
      const bucket = series.points.slice(i, i + bucketSize);

      if (bucket.length === 0) continue;

      let value: number;
      const timestamp = bucket[Math.floor(bucket.length / 2)].timestamp;

      switch (method) {
        case 'avg':
          value = bucket.reduce((sum, p) => sum + p.value, 0) / bucket.length;
          break;

        case 'max':
          value = Math.max(...bucket.map(p => p.value));
          break;

        case 'min':
          value = Math.min(...bucket.map(p => p.value));
          break;

        case 'last':
          value = bucket[bucket.length - 1].value;
          break;
      }

      downsampled.push({ timestamp, value });
    }

    return {
      metric,
      points: downsampled
    };
  }

  /**
   * Calculate moving average
   */
  public movingAverage(metric: string, windowSize: number): TimeSeries | undefined {
    const series = this.getTimeSeries(metric);

    if (!series || series.points.length < windowSize) {
      return undefined;
    }

    const smoothed: TimeSeriesPoint[] = [];

    for (let i = windowSize - 1; i < series.points.length; i++) {
      const window = series.points.slice(i - windowSize + 1, i + 1);
      const avg = window.reduce((sum, p) => sum + p.value, 0) / window.length;

      smoothed.push({
        timestamp: series.points[i].timestamp,
        value: avg
      });
    }

    return {
      metric,
      points: smoothed
    };
  }

  /**
   * Calculate percentiles for a metric
   */
  public percentiles(
    metric: string,
    start: number,
    end: number,
    percentiles: number[] = [0.5, 0.95, 0.99]
  ): Record<string, number> {
    const series = this.getTimeSeriesRange(metric, start, end);

    if (!series || series.points.length === 0) {
      return {};
    }

    const sorted = series.points.map(p => p.value).sort((a, b) => a - b);
    const result: Record<string, number> = {};

    for (const p of percentiles) {
      const index = Math.ceil(sorted.length * p) - 1;
      result[`p${Math.floor(p * 100)}`] = sorted[Math.max(0, index)];
    }

    return result;
  }

  /**
   * Get snapshot statistics
   */
  public getSnapshot(metric: string, start: number, end: number): MetricSnapshot {
    const series = this.getTimeSeriesRange(metric, start, end);

    if (!series || series.points.length === 0) {
      return {
        count: 0,
        sum: 0,
        min: 0,
        max: 0,
        avg: 0
      };
    }

    const values = series.points.map(p => p.value);
    const sorted = [...values].sort((a, b) => a - b);

    return {
      count: values.length,
      sum: values.reduce((a, b) => a + b, 0),
      min: sorted[0],
      max: sorted[sorted.length - 1],
      avg: values.reduce((a, b) => a + b, 0) / values.length,
      p50: this.calculatePercentile(sorted, 0.5),
      p95: this.calculatePercentile(sorted, 0.95),
      p99: this.calculatePercentile(sorted, 0.99)
    };
  }

  /**
   * Calculate percentile value
   */
  private calculatePercentile(sorted: number[], percentile: number): number {
    if (sorted.length === 0) return 0;

    const index = Math.ceil(sorted.length * percentile) - 1;
    return sorted[Math.max(0, index)];
  }

  /**
   * Clear old data points
   */
  public clearOldData(cutoffTimestamp: number): void {
    for (const [metric, points] of this.timeSeriesData.entries()) {
      const filtered = points.filter(p => p.timestamp >= cutoffTimestamp);
      this.timeSeriesData.set(metric, filtered);
    }
  }

  /**
   * Clear all data
   */
  public clear(): void {
    this.timeSeriesData.clear();
  }

  /**
   * Get all metric names
   */
  public getMetricNames(): string[] {
    return Array.from(this.timeSeriesData.keys());
  }

  /**
   * Get data point count for metric
   */
  public getDataPointCount(metric: string): number {
    const points = this.timeSeriesData.get(metric);
    return points ? points.length : 0;
  }
}

// Global aggregator instance
export const globalAggregator = new MetricsAggregator();
