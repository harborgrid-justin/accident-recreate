/**
 * AccuScene Enterprise v0.2.0
 * Gauge Metric Implementation
 *
 * Gauge for tracking values that can go up and down
 */

import { Metric, MetricType, MetricMetadata, MetricValue, MetricLabels, MetricSnapshot } from '../types';

export class Gauge implements Metric {
  public readonly metadata: MetricMetadata;
  private value: number = 0;
  private labeledValues: Map<string, number> = new Map();
  private history: MetricValue[] = [];
  private readonly maxHistorySize: number = 1000;

  constructor(
    name: string,
    help: string,
    unit?: string,
    labels?: string[]
  ) {
    this.metadata = {
      name,
      type: MetricType.GAUGE,
      help,
      unit,
      labels
    };
  }

  /**
   * Set gauge to a specific value
   */
  public set(value: number, labels?: MetricLabels): void {
    const timestamp = Date.now();

    if (labels) {
      const key = this.serializeLabels(labels);
      this.labeledValues.set(key, value);
      this.addToHistory({ value, timestamp, labels });
    } else {
      this.value = value;
      this.addToHistory({ value, timestamp });
    }
  }

  /**
   * Increment gauge by a value
   */
  public inc(value: number = 1, labels?: MetricLabels): void {
    if (labels) {
      const key = this.serializeLabels(labels);
      const current = this.labeledValues.get(key) || 0;
      this.set(current + value, labels);
    } else {
      this.set(this.value + value);
    }
  }

  /**
   * Decrement gauge by a value
   */
  public dec(value: number = 1, labels?: MetricLabels): void {
    this.inc(-value, labels);
  }

  /**
   * Set gauge to current timestamp
   */
  public setToCurrentTime(labels?: MetricLabels): void {
    this.set(Date.now() / 1000, labels);
  }

  /**
   * Get current gauge value
   */
  public getValue(): number {
    return this.value;
  }

  /**
   * Get all recorded values
   */
  public getValues(): MetricValue[] {
    return [...this.history];
  }

  /**
   * Get value for specific labels
   */
  public getValueForLabels(labels: MetricLabels): number {
    const key = this.serializeLabels(labels);
    return this.labeledValues.get(key) || 0;
  }

  /**
   * Get all labeled values
   */
  public getLabeledValues(): Map<string, number> {
    return new Map(this.labeledValues);
  }

  /**
   * Get snapshot of gauge metrics
   */
  public getSnapshot(): MetricSnapshot {
    const values = this.history.map(h => h.value);

    return {
      count: values.length,
      sum: values.reduce((a, b) => a + b, 0),
      min: values.length > 0 ? Math.min(...values) : 0,
      max: values.length > 0 ? Math.max(...values) : 0,
      avg: values.length > 0 ? values.reduce((a, b) => a + b, 0) / values.length : 0
    };
  }

  /**
   * Reset gauge to zero
   */
  public reset(): void {
    this.value = 0;
    this.labeledValues.clear();
    this.history = [];
  }

  /**
   * Serialize labels to string key
   */
  private serializeLabels(labels: MetricLabels): string {
    return Object.entries(labels)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([k, v]) => `${k}="${v}"`)
      .join(',');
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
    lines.push(`# TYPE ${this.metadata.name} gauge`);

    if (this.labeledValues.size > 0) {
      for (const [labels, value] of this.labeledValues.entries()) {
        lines.push(`${this.metadata.name}{${labels}} ${value}`);
      }
    } else {
      lines.push(`${this.metadata.name} ${this.value}`);
    }

    return lines.join('\n');
  }
}
