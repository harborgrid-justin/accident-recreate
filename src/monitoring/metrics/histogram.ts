/**
 * AccuScene Enterprise v0.2.0
 * Histogram Metric Implementation
 *
 * Histogram for sampling observations and counting them in configurable buckets
 */

import { Metric, MetricType, MetricMetadata, MetricValue, MetricLabels, MetricSnapshot } from '../types';

interface Bucket {
  upperBound: number;
  count: number;
}

export class Histogram implements Metric {
  public readonly metadata: MetricMetadata;
  private observations: number[] = [];
  private buckets: Bucket[];
  private sum: number = 0;
  private count: number = 0;
  private history: MetricValue[] = [];
  private readonly maxHistorySize: number = 10000;

  constructor(
    name: string,
    help: string,
    buckets: number[] = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
    unit?: string,
    labels?: string[]
  ) {
    this.metadata = {
      name,
      type: MetricType.HISTOGRAM,
      help,
      unit,
      labels
    };

    this.buckets = buckets
      .sort((a, b) => a - b)
      .map(upperBound => ({ upperBound, count: 0 }));

    // Always add +Inf bucket
    this.buckets.push({ upperBound: Infinity, count: 0 });
  }

  /**
   * Observe a value and update buckets
   */
  public observe(value: number, labels?: MetricLabels): void {
    if (value < 0) {
      throw new Error('Histogram values must be non-negative');
    }

    const timestamp = Date.now();

    this.observations.push(value);
    this.sum += value;
    this.count++;

    // Update buckets
    for (const bucket of this.buckets) {
      if (value <= bucket.upperBound) {
        bucket.count++;
      }
    }

    this.addToHistory({ value, timestamp, labels });
  }

  /**
   * Get current value (returns snapshot)
   */
  public getValue(): MetricSnapshot {
    return this.getSnapshot();
  }

  /**
   * Get all recorded values
   */
  public getValues(): MetricValue[] {
    return [...this.history];
  }

  /**
   * Get snapshot of histogram metrics with percentiles
   */
  public getSnapshot(): MetricSnapshot {
    const sorted = [...this.observations].sort((a, b) => a - b);

    return {
      count: this.count,
      sum: this.sum,
      min: sorted.length > 0 ? sorted[0] : 0,
      max: sorted.length > 0 ? sorted[sorted.length - 1] : 0,
      avg: this.count > 0 ? this.sum / this.count : 0,
      p50: this.calculatePercentile(sorted, 0.50),
      p95: this.calculatePercentile(sorted, 0.95),
      p99: this.calculatePercentile(sorted, 0.99)
    };
  }

  /**
   * Get bucket counts
   */
  public getBuckets(): Bucket[] {
    return [...this.buckets];
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
   * Reset histogram
   */
  public reset(): void {
    this.observations = [];
    this.sum = 0;
    this.count = 0;
    this.buckets.forEach(bucket => bucket.count = 0);
    this.history = [];
  }

  /**
   * Add value to history with size limit
   */
  private addToHistory(value: MetricValue): void {
    this.history.push(value);
    if (this.history.length > this.maxHistorySize) {
      this.history.shift();
    }
  }

  /**
   * Export in Prometheus format
   */
  public toPrometheusFormat(): string {
    const lines: string[] = [];
    lines.push(`# HELP ${this.metadata.name} ${this.metadata.help}`);
    lines.push(`# TYPE ${this.metadata.name} histogram`);

    for (const bucket of this.buckets) {
      const le = bucket.upperBound === Infinity ? '+Inf' : bucket.upperBound.toString();
      lines.push(`${this.metadata.name}_bucket{le="${le}"} ${bucket.count}`);
    }

    lines.push(`${this.metadata.name}_sum ${this.sum}`);
    lines.push(`${this.metadata.name}_count ${this.count}`);

    return lines.join('\n');
  }
}
