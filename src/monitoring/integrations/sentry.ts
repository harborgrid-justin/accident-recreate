/**
 * AccuScene Enterprise v0.2.0
 * Sentry Integration
 *
 * Error tracking and performance monitoring with Sentry
 */

import { Span } from '../tracing/span';
import { SentryEvent } from '../types';

export class SentryIntegration {
  private dsn: string;
  private environment: string;
  private release: string;
  private sampleRate: number;
  private breadcrumbs: any[] = [];
  private maxBreadcrumbs: number = 100;

  constructor(config: {
    dsn: string;
    environment?: string;
    release?: string;
    sampleRate?: number;
    maxBreadcrumbs?: number;
  }) {
    this.dsn = config.dsn;
    this.environment = config.environment || 'production';
    this.release = config.release || 'accuscene-enterprise@0.2.0';
    this.sampleRate = config.sampleRate || 1.0;
    this.maxBreadcrumbs = config.maxBreadcrumbs || 100;
  }

  /**
   * Capture an exception
   */
  public async captureException(error: Error, context?: Record<string, any>): Promise<string> {
    const event = this.createErrorEvent(error, context);
    return await this.sendEvent(event);
  }

  /**
   * Capture a message
   */
  public async captureMessage(
    message: string,
    level: 'fatal' | 'error' | 'warning' | 'info' | 'debug' = 'info',
    context?: Record<string, any>
  ): Promise<string> {
    const event = this.createMessageEvent(message, level, context);
    return await this.sendEvent(event);
  }

  /**
   * Capture a span as a transaction
   */
  public async captureSpan(span: Span): Promise<string> {
    const event = this.createSpanEvent(span);
    return await this.sendEvent(event);
  }

  /**
   * Add breadcrumb
   */
  public addBreadcrumb(breadcrumb: {
    message?: string;
    category?: string;
    level?: string;
    data?: Record<string, any>;
  }): void {
    this.breadcrumbs.push({
      timestamp: Date.now() / 1000,
      ...breadcrumb
    });

    // Limit breadcrumb count
    if (this.breadcrumbs.length > this.maxBreadcrumbs) {
      this.breadcrumbs.shift();
    }
  }

  /**
   * Create error event
   */
  private createErrorEvent(error: Error, context?: Record<string, any>): SentryEvent {
    return {
      event_id: this.generateEventId(),
      timestamp: Date.now() / 1000,
      level: 'error',
      message: error.message,
      transaction: context?.transaction,
      tags: context?.tags,
      extra: {
        ...context,
        stack: error.stack
      },
      breadcrumbs: [...this.breadcrumbs]
    };
  }

  /**
   * Create message event
   */
  private createMessageEvent(
    message: string,
    level: 'fatal' | 'error' | 'warning' | 'info' | 'debug',
    context?: Record<string, any>
  ): SentryEvent {
    return {
      event_id: this.generateEventId(),
      timestamp: Date.now() / 1000,
      level,
      message,
      transaction: context?.transaction,
      tags: context?.tags,
      extra: context,
      breadcrumbs: [...this.breadcrumbs]
    };
  }

  /**
   * Create span event (transaction)
   */
  private createSpanEvent(span: Span): SentryEvent {
    return {
      event_id: this.generateEventId(),
      timestamp: span.startTime / 1000,
      level: span.status === 'error' ? 'error' : 'info',
      transaction: span.name,
      tags: {
        span_id: span.context.spanId,
        trace_id: span.context.traceId
      },
      extra: {
        duration: span.getDuration(),
        attributes: span.attributes,
        events: span.events
      },
      breadcrumbs: [...this.breadcrumbs]
    };
  }

  /**
   * Send event to Sentry
   */
  private async sendEvent(event: SentryEvent): Promise<string> {
    // Check sample rate
    if (Math.random() > this.sampleRate) {
      return event.event_id;
    }

    const envelope = this.createEnvelope(event);

    try {
      const parsedDsn = this.parseDSN(this.dsn);
      const url = `${parsedDsn.protocol}://${parsedDsn.host}/api/${parsedDsn.projectId}/envelope/`;

      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-sentry-envelope',
          'X-Sentry-Auth': this.getAuthHeader(parsedDsn.publicKey)
        },
        body: envelope
      });

      if (!response.ok) {
        console.error(`Sentry event send failed: ${response.status} ${response.statusText}`);
      }

      return event.event_id;
    } catch (error) {
      console.error('Failed to send event to Sentry:', error);
      return event.event_id;
    }
  }

  /**
   * Create Sentry envelope
   */
  private createEnvelope(event: SentryEvent): string {
    const header = {
      event_id: event.event_id,
      sent_at: new Date().toISOString()
    };

    const itemHeader = {
      type: 'event',
      content_type: 'application/json'
    };

    const payload = {
      ...event,
      environment: this.environment,
      release: this.release,
      platform: 'javascript'
    };

    return `${JSON.stringify(header)}\n${JSON.stringify(itemHeader)}\n${JSON.stringify(payload)}`;
  }

  /**
   * Parse Sentry DSN
   */
  private parseDSN(dsn: string): {
    protocol: string;
    publicKey: string;
    host: string;
    projectId: string;
  } {
    const match = dsn.match(/^(https?):\/\/([^@]+)@([^/]+)\/(.+)$/);

    if (!match) {
      throw new Error('Invalid Sentry DSN');
    }

    const [, protocol, publicKey, host, projectId] = match;

    return { protocol, publicKey, host, projectId };
  }

  /**
   * Generate auth header
   */
  private getAuthHeader(publicKey: string): string {
    return `Sentry sentry_version=7, sentry_client=accuscene-monitoring/0.2.0, sentry_key=${publicKey}`;
  }

  /**
   * Generate event ID
   */
  private generateEventId(): string {
    return 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'.replace(/x/g, () =>
      Math.floor(Math.random() * 16).toString(16)
    );
  }

  /**
   * Set user context
   */
  public setUser(user: {
    id?: string;
    username?: string;
    email?: string;
    ip_address?: string;
  }): void {
    this.addBreadcrumb({
      category: 'user',
      message: `User set: ${user.username || user.id}`,
      data: user
    });
  }

  /**
   * Set tags
   */
  public setTags(tags: Record<string, string>): void {
    this.addBreadcrumb({
      category: 'tags',
      message: 'Tags updated',
      data: tags
    });
  }

  /**
   * Set context
   */
  public setContext(name: string, context: Record<string, any>): void {
    this.addBreadcrumb({
      category: 'context',
      message: `Context set: ${name}`,
      data: context
    });
  }

  /**
   * Clear breadcrumbs
   */
  public clearBreadcrumbs(): void {
    this.breadcrumbs = [];
  }
}

/**
 * Sentry error boundary for React
 */
export class SentryErrorBoundary {
  private sentry: SentryIntegration;

  constructor(sentry: SentryIntegration) {
    this.sentry = sentry;
  }

  /**
   * Handle component error
   */
  public handleError(error: Error, errorInfo: any): void {
    this.sentry.captureException(error, {
      tags: {
        error_boundary: 'true'
      },
      extra: {
        componentStack: errorInfo.componentStack
      }
    });
  }
}

/**
 * Global error handler
 */
export function setupGlobalErrorHandler(sentry: SentryIntegration): void {
  // Browser error handler
  if (typeof window !== 'undefined') {
    window.addEventListener('error', (event) => {
      sentry.captureException(event.error || new Error(event.message), {
        tags: {
          type: 'uncaught_error'
        },
        extra: {
          filename: event.filename,
          lineno: event.lineno,
          colno: event.colno
        }
      });
    });

    window.addEventListener('unhandledrejection', (event) => {
      sentry.captureException(
        event.reason instanceof Error ? event.reason : new Error(String(event.reason)),
        {
          tags: {
            type: 'unhandled_rejection'
          }
        }
      );
    });
  }

  // Node.js error handler
  if (typeof process !== 'undefined') {
    process.on('uncaughtException', (error) => {
      sentry.captureException(error, {
        tags: {
          type: 'uncaught_exception'
        }
      });
    });

    process.on('unhandledRejection', (reason) => {
      sentry.captureException(
        reason instanceof Error ? reason : new Error(String(reason)),
        {
          tags: {
            type: 'unhandled_rejection'
          }
        }
      );
    });
  }
}
