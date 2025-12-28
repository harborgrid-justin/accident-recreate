/**
 * AccuScene Enterprise v0.2.0
 * Metrics Reporter
 *
 * Reports metrics to various destinations
 */

import { ReportConfig, PerformanceReport, MetricSnapshot, Metric } from '../types';
import { MetricsCollector } from './collector';
import { MetricsAggregator } from './aggregator';

export class MetricsReporter {
  private config: ReportConfig;
  private collector: MetricsCollector;
  private aggregator: MetricsAggregator;
  private reportInterval?: NodeJS.Timeout;

  constructor(
    collector: MetricsCollector,
    aggregator: MetricsAggregator,
    config: ReportConfig
  ) {
    this.collector = collector;
    this.aggregator = aggregator;
    this.config = config;
  }

  /**
   * Start periodic reporting
   */
  public start(): void {
    if (this.reportInterval || !this.config.interval) {
      return;
    }

    this.reportInterval = setInterval(() => {
      this.report();
    }, this.config.interval);

    // Immediate first report
    this.report();
  }

  /**
   * Stop periodic reporting
   */
  public stop(): void {
    if (this.reportInterval) {
      clearInterval(this.reportInterval);
      this.reportInterval = undefined;
    }
  }

  /**
   * Generate and send a report
   */
  public async report(): Promise<void> {
    const now = Date.now();
    const periodStart = now - (this.config.interval || 60000);

    const report = this.generateReport(periodStart, now);

    switch (this.config.format) {
      case 'json':
        await this.reportJSON(report);
        break;

      case 'prometheus':
        await this.reportPrometheus();
        break;

      case 'opentelemetry':
        await this.reportOpenTelemetry();
        break;

      default:
        await this.reportJSON(report);
    }
  }

  /**
   * Generate performance report
   */
  private generateReport(start: number, end: number): PerformanceReport {
    const collected = this.collector.collect();
    const metricSnapshots: Record<string, MetricSnapshot> = {};

    // Generate snapshots for all metrics
    for (const metricName of this.collector.getMetricNames()) {
      const snapshot = this.aggregator.getSnapshot(metricName, start, end);
      metricSnapshots[metricName] = snapshot;
    }

    // Count traces
    const traces = collected.traces;
    const errorTraces = traces.filter(t => t.status === 'error').length;

    return {
      timestamp: Date.now(),
      period: { start, end },
      metrics: metricSnapshots,
      traces: {
        total: traces.length,
        sampled: traces.length,
        errors: errorTraces
      },
      health: {
        status: 'healthy' as any,
        checks: {},
        timestamp: Date.now()
      },
      alerts: []
    };
  }

  /**
   * Report in JSON format
   */
  private async reportJSON(report: PerformanceReport): Promise<void> {
    const output = JSON.stringify(report, null, 2);

    switch (this.config.destination) {
      case 'console':
        console.log('Performance Report:', output);
        break;

      case 'file':
        // Would write to file in Node.js environment
        console.log('File output not implemented in browser');
        break;

      case 'http':
        if (this.config.endpoint) {
          await this.sendHTTP(output, 'application/json');
        }
        break;
    }
  }

  /**
   * Report in Prometheus format
   */
  private async reportPrometheus(): Promise<void> {
    const lines: string[] = [];

    // Export all metrics in Prometheus format
    for (const metricName of this.collector.getMetricNames()) {
      const metric = this.collector.getMetric(metricName);

      if (metric && typeof (metric as any).toPrometheusFormat === 'function') {
        lines.push((metric as any).toPrometheusFormat());
        lines.push('');
      }
    }

    const output = lines.join('\n');

    switch (this.config.destination) {
      case 'console':
        console.log('Prometheus Metrics:\n', output);
        break;

      case 'http':
        if (this.config.endpoint) {
          await this.sendHTTP(output, 'text/plain; version=0.0.4');
        }
        break;
    }
  }

  /**
   * Report in OpenTelemetry format
   */
  private async reportOpenTelemetry(): Promise<void> {
    const collected = this.collector.collect();

    const payload = {
      resourceMetrics: [
        {
          resource: {
            attributes: [
              {
                key: 'service.name',
                value: { stringValue: 'accuscene-enterprise' }
              },
              {
                key: 'service.version',
                value: { stringValue: '0.2.0' }
              }
            ]
          },
          scopeMetrics: [
            {
              scope: {
                name: 'accuscene-monitoring',
                version: '0.2.0'
              },
              metrics: this.convertToOtelMetrics(collected.metrics)
            }
          ]
        }
      ]
    };

    const output = JSON.stringify(payload);

    if (this.config.endpoint) {
      await this.sendHTTP(output, 'application/json');
    }
  }

  /**
   * Convert metrics to OpenTelemetry format
   */
  private convertToOtelMetrics(metrics: Map<string, any[]>): any[] {
    const otelMetrics: any[] = [];

    for (const [name, values] of metrics.entries()) {
      if (values.length === 0) continue;

      otelMetrics.push({
        name,
        description: `Metric ${name}`,
        unit: '',
        gauge: {
          dataPoints: values.map(v => ({
            asDouble: v.value,
            timeUnixNano: (v.timestamp * 1000000).toString(),
            attributes: v.labels
              ? Object.entries(v.labels).map(([k, val]) => ({
                  key: k,
                  value: { stringValue: String(val) }
                }))
              : []
          }))
        }
      });
    }

    return otelMetrics;
  }

  /**
   * Send data via HTTP
   */
  private async sendHTTP(data: string, contentType: string): Promise<void> {
    try {
      const response = await fetch(this.config.endpoint!, {
        method: 'POST',
        headers: {
          'Content-Type': contentType
        },
        body: data
      });

      if (!response.ok) {
        console.error(`Failed to send report: ${response.status} ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to send report:', error);
    }
  }

  /**
   * Generate summary report
   */
  public generateSummary(): string {
    const now = Date.now();
    const hourAgo = now - 3600000;

    const report = this.generateReport(hourAgo, now);
    const lines: string[] = [];

    lines.push('=== AccuScene Performance Summary ===');
    lines.push(`Generated: ${new Date(report.timestamp).toISOString()}`);
    lines.push(`Period: ${new Date(report.period.start).toISOString()} - ${new Date(report.period.end).toISOString()}`);
    lines.push('');

    lines.push('Metrics:');
    for (const [name, snapshot] of Object.entries(report.metrics)) {
      lines.push(`  ${name}:`);
      lines.push(`    Count: ${snapshot.count}`);
      lines.push(`    Avg: ${snapshot.avg.toFixed(2)}`);
      lines.push(`    Min: ${snapshot.min.toFixed(2)}`);
      lines.push(`    Max: ${snapshot.max.toFixed(2)}`);

      if (snapshot.p95 !== undefined) {
        lines.push(`    P95: ${snapshot.p95.toFixed(2)}`);
      }

      if (snapshot.p99 !== undefined) {
        lines.push(`    P99: ${snapshot.p99.toFixed(2)}`);
      }
    }

    lines.push('');
    lines.push('Traces:');
    lines.push(`  Total: ${report.traces.total}`);
    lines.push(`  Errors: ${report.traces.errors}`);
    lines.push('');

    return lines.join('\n');
  }
}
