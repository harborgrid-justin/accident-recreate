/**
 * AccuScene Enterprise v0.2.0
 * Trace Exporter
 *
 * Export traces to various backends
 */

import { Span } from './span';
import { OpenTelemetrySpan } from '../types';

/**
 * Span exporter interface
 */
export interface SpanExporter {
  export(spans: Span[]): Promise<void>;
  shutdown(): Promise<void>;
}

/**
 * Console exporter for debugging
 */
export class ConsoleSpanExporter implements SpanExporter {
  public async export(spans: Span[]): Promise<void> {
    console.group('Exported Spans');
    for (const span of spans) {
      console.log(span.toJSON());
    }
    console.groupEnd();
  }

  public async shutdown(): Promise<void> {
    // No-op for console exporter
  }
}

/**
 * In-memory exporter for testing
 */
export class InMemorySpanExporter implements SpanExporter {
  private spans: Span[] = [];

  public async export(spans: Span[]): Promise<void> {
    this.spans.push(...spans);
  }

  public getSpans(): Span[] {
    return [...this.spans];
  }

  public reset(): void {
    this.spans = [];
  }

  public async shutdown(): Promise<void> {
    this.reset();
  }
}

/**
 * HTTP exporter for sending to collector
 */
export class HttpSpanExporter implements SpanExporter {
  private endpoint: string;
  private headers: Record<string, string>;
  private timeout: number;

  constructor(config: {
    endpoint: string;
    headers?: Record<string, string>;
    timeout?: number;
  }) {
    this.endpoint = config.endpoint;
    this.headers = config.headers || {
      'Content-Type': 'application/json'
    };
    this.timeout = config.timeout || 10000;
  }

  public async export(spans: Span[]): Promise<void> {
    if (spans.length === 0) {
      return;
    }

    const payload = {
      resourceSpans: [
        {
          resource: {
            attributes: [
              {
                key: 'service.name',
                value: { stringValue: 'accuscene-enterprise' }
              }
            ]
          },
          scopeSpans: [
            {
              scope: {
                name: 'accuscene-monitoring',
                version: '0.2.0'
              },
              spans: spans.map(span => span.toOpenTelemetryFormat())
            }
          ]
        }
      ]
    };

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.timeout);

      const response = await fetch(this.endpoint, {
        method: 'POST',
        headers: this.headers,
        body: JSON.stringify(payload),
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`Export failed: ${response.status} ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to export spans:', error);
      throw error;
    }
  }

  public async shutdown(): Promise<void> {
    // No cleanup needed for HTTP exporter
  }
}

/**
 * Batch span processor
 */
export class BatchSpanProcessor {
  private exporter: SpanExporter;
  private spans: Span[] = [];
  private maxBatchSize: number;
  private scheduledDelayMillis: number;
  private maxQueueSize: number;
  private timer?: NodeJS.Timeout;
  private isShutdown: boolean = false;

  constructor(
    exporter: SpanExporter,
    config?: {
      maxBatchSize?: number;
      scheduledDelayMillis?: number;
      maxQueueSize?: number;
    }
  ) {
    this.exporter = exporter;
    this.maxBatchSize = config?.maxBatchSize || 512;
    this.scheduledDelayMillis = config?.scheduledDelayMillis || 5000;
    this.maxQueueSize = config?.maxQueueSize || 2048;

    this.scheduleNextFlush();
  }

  /**
   * Add span to batch
   */
  public onEnd(span: Span): void {
    if (this.isShutdown) {
      return;
    }

    if (this.spans.length >= this.maxQueueSize) {
      console.warn('Span queue full, dropping span');
      return;
    }

    this.spans.push(span);

    if (this.spans.length >= this.maxBatchSize) {
      this.flush();
    }
  }

  /**
   * Force flush all pending spans
   */
  public async flush(): Promise<void> {
    if (this.spans.length === 0) {
      return;
    }

    const toExport = this.spans.splice(0, this.maxBatchSize);

    try {
      await this.exporter.export(toExport);
    } catch (error) {
      console.error('Failed to export batch:', error);
    }
  }

  /**
   * Schedule next automatic flush
   */
  private scheduleNextFlush(): void {
    if (this.timer) {
      clearTimeout(this.timer);
    }

    this.timer = setTimeout(() => {
      this.flush().then(() => {
        if (!this.isShutdown) {
          this.scheduleNextFlush();
        }
      });
    }, this.scheduledDelayMillis);
  }

  /**
   * Shutdown processor
   */
  public async shutdown(): Promise<void> {
    if (this.isShutdown) {
      return;
    }

    this.isShutdown = true;

    if (this.timer) {
      clearTimeout(this.timer);
    }

    // Flush remaining spans
    while (this.spans.length > 0) {
      await this.flush();
    }

    await this.exporter.shutdown();
  }
}

/**
 * Simple span processor (exports immediately)
 */
export class SimpleSpanProcessor {
  private exporter: SpanExporter;

  constructor(exporter: SpanExporter) {
    this.exporter = exporter;
  }

  /**
   * Export span immediately
   */
  public async onEnd(span: Span): Promise<void> {
    try {
      await this.exporter.export([span]);
    } catch (error) {
      console.error('Failed to export span:', error);
    }
  }

  /**
   * Shutdown processor
   */
  public async shutdown(): Promise<void> {
    await this.exporter.shutdown();
  }
}
