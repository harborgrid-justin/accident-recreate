/**
 * AccuScene Enterprise v0.2.0
 * Prometheus Integration
 *
 * Export metrics in Prometheus format
 */

import { MetricsCollector } from '../core/collector';
import { PrometheusMetric } from '../types';

export class PrometheusExporter {
  private collector: MetricsCollector;
  private prefix: string;

  constructor(collector: MetricsCollector, prefix: string = 'accuscene') {
    this.collector = collector;
    this.prefix = prefix;
  }

  /**
   * Export all metrics in Prometheus text format
   */
  public export(): string {
    const lines: string[] = [];

    for (const metricName of this.collector.getMetricNames()) {
      const metric = this.collector.getMetric(metricName);

      if (metric && typeof (metric as any).toPrometheusFormat === 'function') {
        const prometheusFormat = (metric as any).toPrometheusFormat();
        lines.push(prometheusFormat);
        lines.push('');
      }
    }

    return lines.join('\n');
  }

  /**
   * Export specific metric
   */
  public exportMetric(metricName: string): string | undefined {
    const metric = this.collector.getMetric(metricName);

    if (metric && typeof (metric as any).toPrometheusFormat === 'function') {
      return (metric as any).toPrometheusFormat();
    }

    return undefined;
  }

  /**
   * Get metrics in Prometheus JSON format (for HTTP API)
   */
  public exportJSON(): PrometheusMetric[] {
    const metrics: PrometheusMetric[] = [];

    for (const metricName of this.collector.getMetricNames()) {
      const metric = this.collector.getMetric(metricName);

      if (!metric) continue;

      const values = metric.getValues();
      const metricType = metric.metadata.type;

      metrics.push({
        name: this.prefix ? `${this.prefix}_${metricName}` : metricName,
        type: metricType,
        help: metric.metadata.help,
        metrics: values.map(v => ({
          value: v.value,
          labels: v.labels as Record<string, string>,
          timestamp: v.timestamp
        }))
      });
    }

    return metrics;
  }

  /**
   * Create Prometheus HTTP endpoint handler
   */
  public createHandler(): (req: any, res: any) => void {
    return (req: any, res: any) => {
      const format = req.query?.format || 'text';

      if (format === 'json') {
        const metrics = this.exportJSON();
        res.setHeader('Content-Type', 'application/json');
        res.send(JSON.stringify(metrics, null, 2));
      } else {
        const text = this.export();
        res.setHeader('Content-Type', 'text/plain; version=0.0.4; charset=utf-8');
        res.send(text);
      }
    };
  }

  /**
   * Push metrics to Prometheus Pushgateway
   */
  public async push(pushgatewayUrl: string, job: string): Promise<void> {
    const text = this.export();
    const url = `${pushgatewayUrl}/metrics/job/${encodeURIComponent(job)}`;

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'text/plain; version=0.0.4'
        },
        body: text
      });

      if (!response.ok) {
        throw new Error(`Push failed: ${response.status} ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to push metrics to Prometheus:', error);
      throw error;
    }
  }

  /**
   * Delete metrics from Pushgateway
   */
  public async delete(pushgatewayUrl: string, job: string): Promise<void> {
    const url = `${pushgatewayUrl}/metrics/job/${encodeURIComponent(job)}`;

    try {
      const response = await fetch(url, {
        method: 'DELETE'
      });

      if (!response.ok) {
        throw new Error(`Delete failed: ${response.status} ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to delete metrics from Prometheus:', error);
      throw error;
    }
  }
}

/**
 * Prometheus metric name sanitizer
 */
export class PrometheusNameSanitizer {
  /**
   * Sanitize metric name to Prometheus format
   */
  public static sanitize(name: string): string {
    // Replace invalid characters with underscores
    let sanitized = name.replace(/[^a-zA-Z0-9_:]/g, '_');

    // Ensure it starts with a letter or underscore
    if (!/^[a-zA-Z_:]/.test(sanitized)) {
      sanitized = '_' + sanitized;
    }

    // Remove consecutive underscores
    sanitized = sanitized.replace(/_+/g, '_');

    return sanitized.toLowerCase();
  }

  /**
   * Sanitize label name
   */
  public static sanitizeLabel(label: string): string {
    let sanitized = label.replace(/[^a-zA-Z0-9_]/g, '_');

    if (!/^[a-zA-Z_]/.test(sanitized)) {
      sanitized = '_' + sanitized;
    }

    sanitized = sanitized.replace(/_+/g, '_');

    return sanitized;
  }

  /**
   * Escape label value
   */
  public static escapeLabelValue(value: string): string {
    return value
      .replace(/\\/g, '\\\\')
      .replace(/\n/g, '\\n')
      .replace(/"/g, '\\"');
  }
}

/**
 * Prometheus query builder
 */
export class PrometheusQueryBuilder {
  private query: string = '';

  public metric(name: string): this {
    this.query = name;
    return this;
  }

  public label(key: string, value: string): this {
    if (!this.query) {
      throw new Error('Metric name must be set first');
    }

    if (this.query.includes('{')) {
      this.query = this.query.replace('}', `,${key}="${value}"}`);
    } else {
      this.query += `{${key}="${value}"}`;
    }

    return this;
  }

  public rate(duration: string = '5m'): this {
    this.query = `rate(${this.query}[${duration}])`;
    return this;
  }

  public sum(): this {
    this.query = `sum(${this.query})`;
    return this;
  }

  public avg(): this {
    this.query = `avg(${this.query})`;
    return this;
  }

  public max(): this {
    this.query = `max(${this.query})`;
    return this;
  }

  public min(): this {
    this.query = `min(${this.query})`;
    return this;
  }

  public by(...labels: string[]): this {
    this.query += ` by (${labels.join(', ')})`;
    return this;
  }

  public build(): string {
    return this.query;
  }
}
