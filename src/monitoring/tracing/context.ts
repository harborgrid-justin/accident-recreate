/**
 * AccuScene Enterprise v0.2.0
 * Trace Context Management
 *
 * Context propagation for distributed tracing
 */

import { Span } from './span';
import { SpanContext, TraceContext } from '../types';

/**
 * Trace context holder using AsyncLocalStorage pattern
 */
class TraceContextManager {
  private contextStack: Span[] = [];
  private contextMap: Map<string, Span> = new Map();

  /**
   * Get the current active span
   */
  public getCurrentSpan(): Span | undefined {
    return this.contextStack[this.contextStack.length - 1];
  }

  /**
   * Set the current active span
   */
  public setCurrentSpan(span: Span): void {
    this.contextStack.push(span);
    this.contextMap.set(span.context.spanId, span);
  }

  /**
   * Remove current span from context
   */
  public removeCurrentSpan(): Span | undefined {
    const span = this.contextStack.pop();
    if (span) {
      this.contextMap.delete(span.context.spanId);
    }
    return span;
  }

  /**
   * Get span by span ID
   */
  public getSpan(spanId: string): Span | undefined {
    return this.contextMap.get(spanId);
  }

  /**
   * Get current trace context
   */
  public getCurrentContext(): TraceContext | undefined {
    const span = this.getCurrentSpan();
    if (!span) return undefined;

    return {
      traceId: span.context.traceId,
      spanId: span.context.spanId,
      parentSpanId: span.parentSpanId
    };
  }

  /**
   * Run function with a specific span as current
   */
  public withSpan<T>(span: Span, fn: () => T): T {
    this.setCurrentSpan(span);
    try {
      return fn();
    } finally {
      this.removeCurrentSpan();
    }
  }

  /**
   * Run async function with a specific span as current
   */
  public async withSpanAsync<T>(span: Span, fn: () => Promise<T>): Promise<T> {
    this.setCurrentSpan(span);
    try {
      return await fn();
    } finally {
      this.removeCurrentSpan();
    }
  }

  /**
   * Clear all contexts
   */
  public clear(): void {
    this.contextStack = [];
    this.contextMap.clear();
  }

  /**
   * Get all active spans
   */
  public getAllSpans(): Span[] {
    return Array.from(this.contextMap.values());
  }

  /**
   * Get context stack depth
   */
  public getDepth(): number {
    return this.contextStack.length;
  }
}

// Global context manager instance
export const traceContext = new TraceContextManager();

/**
 * Extract trace context from headers (W3C Trace Context format)
 */
export function extractTraceContext(headers: Record<string, string>): SpanContext | undefined {
  const traceparent = headers['traceparent'] || headers['Traceparent'];

  if (!traceparent) {
    return undefined;
  }

  // Parse W3C traceparent: version-traceId-spanId-flags
  const parts = traceparent.split('-');
  if (parts.length !== 4) {
    return undefined;
  }

  const [version, traceId, spanId, flags] = parts;

  // Only support version 00
  if (version !== '00') {
    return undefined;
  }

  return {
    traceId,
    spanId,
    traceFlags: parseInt(flags, 16),
    traceState: headers['tracestate'] || headers['Tracestate'],
    isRemote: true
  };
}

/**
 * Inject trace context into headers (W3C Trace Context format)
 */
export function injectTraceContext(
  context: SpanContext,
  headers: Record<string, string> = {}
): Record<string, string> {
  // Format: version-traceId-spanId-flags
  const flags = context.traceFlags.toString(16).padStart(2, '0');
  headers['traceparent'] = `00-${context.traceId}-${context.spanId}-${flags}`;

  if (context.traceState) {
    headers['tracestate'] = context.traceState;
  }

  return headers;
}

/**
 * Create a child span from current context
 */
export function createChildSpan(name: string, kind?: any): Span {
  const parent = traceContext.getCurrentSpan();

  if (!parent) {
    return new Span(name, kind);
  }

  return new Span(
    name,
    kind,
    parent.context.traceId,
    parent.context.spanId
  );
}

/**
 * Decorator to automatically trace method execution
 */
export function Trace(operationName?: string) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    const originalMethod = descriptor.value;
    const spanName = operationName || `${target.constructor.name}.${propertyKey}`;

    descriptor.value = async function (...args: any[]) {
      const span = createChildSpan(spanName);
      traceContext.setCurrentSpan(span);

      try {
        const result = await Promise.resolve(originalMethod.apply(this, args));
        span.setStatus('ok' as any);
        return result;
      } catch (error: any) {
        span.recordException(error);
        throw error;
      } finally {
        span.end();
        traceContext.removeCurrentSpan();
      }
    };

    return descriptor;
  };
}
