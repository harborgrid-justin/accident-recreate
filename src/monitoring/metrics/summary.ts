/**
 * AccuScene Enterprise v0.2.0
 * Summary Metric Implementation
 *
 * Summary for tracking size and number of observations with configurable quantiles
 */

import { Metric, MetricType, MetricMetadata, MetricValue, MetricLabels, MetricSnapshot } from '../types';

interface Quantile {
  quantile: number;
  value: number;
}

export class Summary implements Metric {
  public readonly metadata: MetricMetadata;
  private observations: number[] = [];
  private quantiles: number[];
  private sum: number = 0;
  private count: number = 0;
  private history: MetricValue[] = [];
  private readonly maxObservations: number = 10000;
  private readonly maxHistorySize: number = 10000;
  private readonly maxAge: number;

  constructor(
    name: string,
    help: string,
    quantiles: number[] = [0.5, 0.9, 0.95, 0.99],
    maxAge: number = 600000, // 10 minutes
    unit?: string,
    labels?: string[]
  ) {
    this.metadata = {
      name,
      type: MetricType.SUMMARY,
      help,
      unit,
      labels
    };

    this.quantiles = quantiles.sort((a, b) => a - b);
    this.maxAge = maxAge;
  }

  /**
   * Observe a value
   */
  public observe(value: number, labels?: MetricLabels): void {
    const timestamp = Date.now();

    this.observations.push(value);
    this.sum += value;
    this.count++;

    // Trim old observations
    this.trimObservations();

    // Limit observation size
    if (this.observations.length > this.maxObservations) {
      this.observations.shift();
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
   * Get snapshot of summary metrics with quantiles
   */
  public getSnapshot(): MetricSnapshot {
    const sorted = [...this.observations].sort((a, b) => a - b);

    return {
      count: this.count,
      sum: this.sum,
      min: sorted.length > 0 ? sorted[0] : 0,
      max: sorted.length > 0 ? sorted[sorted.length - 1] : 0,
      avg: this.count > 0 ? this.sum / this.count : 0,
      p50: this.calculateQuantile(sorted, 0.50),
      p95: this.calculateQuantile(sorted, 0.95),
      p99: this.calculateQuantile(sorted, 0.99)
    };
  }

  /**
   * Get quantile values
   */
  public getQuantiles(): Quantile[] {
    const sorted = [...this.observations].sort((a, b) => a - b);

    return this.quantiles.map(q => ({
      quantile: q,
      value: this.calculateQuantile(sorted, q)
    }));
  }

  /**
   * Calculate quantile value
   */
  private calculateQuantile(sorted: number[], quantile: number): number {
    if (sorted.length === 0) return 0;

    const index = Math.ceil(sorted.length * quantile) - 1;
    return sorted[Math.max(0, index)];
  }

  /**
   * Trim observations older than maxAge
   */
  private trimObservations(): void {
    const now = Date.now();
    const cutoff = now - this.maxAge;

    // Remove observations older than cutoff
    const validHistory = this.history.filter(h => h.timestamp >= cutoff);

    if (validHistory.length < this.history.length) {
      this.observations = validHistory.map(h => h.value);
      this.history = validHistory;

      // Recalculate sum and count
      this.sum = this.observations.reduce((a, b) => a + b, 0);
      this.count = this.observations.length;
    }
  }

  /**
   * Reset summary
   */
  public reset(): void {
    this.observations = [];
    this.sum = 0;
    this.count = 0;
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
    lines.push(`# TYPE ${this.metadata.name} summary`);

    const quantiles = this.getQuantiles();
    for (const q of quantiles) {
      lines.push(`${this.metadata.name}{quantile="${q.quantile}"} ${q.value}`);
    }

    lines.push(`${this.metadata.name}_sum ${this.sum}`);
    lines.push(`${this.metadata.name}_count ${this.count}`);

    return lines.join('\n');
  }
}
