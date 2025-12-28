/**
 * AccuScene Enterprise v0.2.0
 * Counter Metric Implementation
 *
 * Monotonically increasing counter for tracking cumulative values
 */

import { Metric, MetricType, MetricMetadata, MetricValue, MetricLabels, MetricSnapshot } from '../types';

export class Counter implements Metric {
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
      type: MetricType.COUNTER,
      help,
      unit,
      labels
    };
  }

  /**
   * Increment counter by a value
   */
  public inc(value: number = 1, labels?: MetricLabels): void {
    if (value < 0) {
      throw new Error('Counter can only be incremented by non-negative values');
    }

    const timestamp = Date.now();

    if (labels) {
      const key = this.serializeLabels(labels);
      const current = this.labeledValues.get(key) || 0;
      this.labeledValues.set(key, current + value);

      this.addToHistory({ value: current + value, timestamp, labels });
    } else {
      this.value += value;
      this.addToHistory({ value: this.value, timestamp });
    }
  }

  /**
   * Get current counter value
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
   * Get snapshot of counter metrics
   */
  public getSnapshot(): MetricSnapshot {
    return {
      count: this.history.length,
      sum: this.value,
      min: 0,
      max: this.value,
      avg: this.value
    };
  }

  /**
   * Reset counter to zero
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
    lines.push(`# TYPE ${this.metadata.name} counter`);

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
