/**
 * AccuScene Enterprise v0.2.0
 * OpenTelemetry Integration
 *
 * OpenTelemetry protocol (OTLP) support
 */

import { Span } from '../tracing/span';
import { MetricsCollector } from '../core/collector';
import { OpenTelemetrySpan } from '../types';

/**
 * OpenTelemetry trace exporter
 */
export class OpenTelemetryTraceExporter {
  private endpoint: string;
  private headers: Record<string, string>;
  private serviceName: string;
  private serviceVersion: string;

  constructor(config: {
    endpoint: string;
    serviceName?: string;
    serviceVersion?: string;
    headers?: Record<string, string>;
  }) {
    this.endpoint = config.endpoint;
    this.serviceName = config.serviceName || 'accuscene-enterprise';
    this.serviceVersion = config.serviceVersion || '0.2.0';
    this.headers = config.headers || {};
  }

  /**
   * Export spans in OTLP format
   */
  public async export(spans: Span[]): Promise<void> {
    if (spans.length === 0) {
      return;
    }

    const payload = this.createOTLPPayload(spans);

    try {
      const response = await fetch(`${this.endpoint}/v1/traces`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...this.headers
        },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        throw new Error(`OTLP export failed: ${response.status} ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to export traces to OpenTelemetry:', error);
      throw error;
    }
  }

  /**
   * Create OTLP payload
   */
  private createOTLPPayload(spans: Span[]): any {
    return {
      resourceSpans: [
        {
          resource: {
            attributes: [
              {
                key: 'service.name',
                value: { stringValue: this.serviceName }
              },
              {
                key: 'service.version',
                value: { stringValue: this.serviceVersion }
              }
            ]
          },
          scopeSpans: [
            {
              scope: {
                name: 'accuscene-monitoring',
                version: '0.2.0'
              },
              spans: spans.map(span => this.convertSpan(span))
            }
          ]
        }
      ]
    };
  }

  /**
   * Convert span to OTLP format
   */
  private convertSpan(span: Span): OpenTelemetrySpan {
    return span.toOpenTelemetryFormat();
  }
}

/**
 * OpenTelemetry metrics exporter
 */
export class OpenTelemetryMetricsExporter {
  private endpoint: string;
  private headers: Record<string, string>;
  private collector: MetricsCollector;
  private serviceName: string;
  private serviceVersion: string;

  constructor(
    collector: MetricsCollector,
    config: {
      endpoint: string;
      serviceName?: string;
      serviceVersion?: string;
      headers?: Record<string, string>;
    }
  ) {
    this.collector = collector;
    this.endpoint = config.endpoint;
    this.serviceName = config.serviceName || 'accuscene-enterprise';
    this.serviceVersion = config.serviceVersion || '0.2.0';
    this.headers = config.headers || {};
  }

  /**
   * Export metrics in OTLP format
   */
  public async export(): Promise<void> {
    const collected = this.collector.collect();
    const payload = this.createOTLPPayload(collected.metrics);

    try {
      const response = await fetch(`${this.endpoint}/v1/metrics`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...this.headers
        },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        throw new Error(`OTLP export failed: ${response.status} ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to export metrics to OpenTelemetry:', error);
      throw error;
    }
  }

  /**
   * Create OTLP metrics payload
   */
  private createOTLPPayload(metrics: Map<string, any[]>): any {
    const otlpMetrics: any[] = [];

    for (const [name, values] of metrics.entries()) {
      if (values.length === 0) continue;

      const metric = this.collector.getMetric(name);

      if (!metric) continue;

      otlpMetrics.push({
        name,
        description: metric.metadata.help,
        unit: metric.metadata.unit || '',
        gauge: {
          dataPoints: values.map(v => ({
            asDouble: v.value,
            timeUnixNano: (v.timestamp * 1000000).toString(),
            attributes: v.labels
              ? Object.entries(v.labels).map(([key, val]) => ({
                  key,
                  value: { stringValue: String(val) }
                }))
              : []
          }))
        }
      });
    }

    return {
      resourceMetrics: [
        {
          resource: {
            attributes: [
              {
                key: 'service.name',
                value: { stringValue: this.serviceName }
              },
              {
                key: 'service.version',
                value: { stringValue: this.serviceVersion }
              }
            ]
          },
          scopeMetrics: [
            {
              scope: {
                name: 'accuscene-monitoring',
                version: '0.2.0'
              },
              metrics: otlpMetrics
            }
          ]
        }
      ]
    };
  }
}

/**
 * OpenTelemetry SDK configuration
 */
export class OpenTelemetrySDK {
  private traceExporter?: OpenTelemetryTraceExporter;
  private metricsExporter?: OpenTelemetryMetricsExporter;
  private isStarted: boolean = false;
  private exportInterval?: NodeJS.Timeout;

  constructor(config: {
    endpoint: string;
    serviceName?: string;
    serviceVersion?: string;
    headers?: Record<string, string>;
    exportIntervalMs?: number;
  }) {
    const exportConfig = {
      endpoint: config.endpoint,
      serviceName: config.serviceName,
      serviceVersion: config.serviceVersion,
      headers: config.headers
    };

    this.traceExporter = new OpenTelemetryTraceExporter(exportConfig);
  }

  /**
   * Start SDK
   */
  public start(): void {
    if (this.isStarted) {
      return;
    }

    this.isStarted = true;
    console.log('OpenTelemetry SDK started');
  }

  /**
   * Stop SDK
   */
  public stop(): void {
    if (!this.isStarted) {
      return;
    }

    this.isStarted = false;

    if (this.exportInterval) {
      clearInterval(this.exportInterval);
      this.exportInterval = undefined;
    }

    console.log('OpenTelemetry SDK stopped');
  }

  /**
   * Export traces
   */
  public async exportTraces(spans: Span[]): Promise<void> {
    if (!this.traceExporter) {
      throw new Error('Trace exporter not configured');
    }

    await this.traceExporter.export(spans);
  }

  /**
   * Export metrics
   */
  public async exportMetrics(): Promise<void> {
    if (!this.metricsExporter) {
      throw new Error('Metrics exporter not configured');
    }

    await this.metricsExporter.export();
  }

  /**
   * Flush all pending data
   */
  public async flush(): Promise<void> {
    // Flush any pending exports
    if (this.metricsExporter) {
      await this.metricsExporter.export();
    }
  }
}

/**
 * OpenTelemetry context propagation utilities
 */
export class OTelContextPropagation {
  /**
   * Extract W3C traceparent header
   */
  public static extractTraceparent(headers: Record<string, string>): {
    traceId: string;
    spanId: string;
    traceFlags: number;
  } | undefined {
    const traceparent = headers['traceparent'] || headers['Traceparent'];

    if (!traceparent) {
      return undefined;
    }

    const parts = traceparent.split('-');

    if (parts.length !== 4) {
      return undefined;
    }

    const [version, traceId, spanId, flags] = parts;

    if (version !== '00') {
      return undefined;
    }

    return {
      traceId,
      spanId,
      traceFlags: parseInt(flags, 16)
    };
  }

  /**
   * Inject W3C traceparent header
   */
  public static injectTraceparent(
    headers: Record<string, string>,
    traceId: string,
    spanId: string,
    traceFlags: number = 1
  ): Record<string, string> {
    const flags = traceFlags.toString(16).padStart(2, '0');
    headers['traceparent'] = `00-${traceId}-${spanId}-${flags}`;

    return headers;
  }
}
