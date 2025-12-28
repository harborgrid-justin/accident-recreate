/**
 * AccuScene Enterprise v0.2.0
 * Context Propagation
 *
 * Propagation of trace context across service boundaries
 */

import { SpanContext } from '../types';

/**
 * Text map propagator interface
 */
export interface TextMapPropagator {
  inject(context: SpanContext, carrier: Record<string, string>): void;
  extract(carrier: Record<string, string>): SpanContext | undefined;
}

/**
 * W3C Trace Context Propagator
 * Implements https://www.w3.org/TR/trace-context/
 */
export class W3CTraceContextPropagator implements TextMapPropagator {
  private readonly TRACEPARENT = 'traceparent';
  private readonly TRACESTATE = 'tracestate';

  /**
   * Inject trace context into carrier
   */
  public inject(context: SpanContext, carrier: Record<string, string>): void {
    const flags = (context.traceFlags || 0).toString(16).padStart(2, '0');
    carrier[this.TRACEPARENT] = `00-${context.traceId}-${context.spanId}-${flags}`;

    if (context.traceState) {
      carrier[this.TRACESTATE] = context.traceState;
    }
  }

  /**
   * Extract trace context from carrier
   */
  public extract(carrier: Record<string, string>): SpanContext | undefined {
    const traceparent = this.getHeader(carrier, this.TRACEPARENT);

    if (!traceparent) {
      return undefined;
    }

    const match = traceparent.match(/^([0-9a-f]{2})-([0-9a-f]{32})-([0-9a-f]{16})-([0-9a-f]{2})$/);

    if (!match) {
      return undefined;
    }

    const [, version, traceId, spanId, flags] = match;

    // Only support version 00
    if (version !== '00') {
      return undefined;
    }

    return {
      traceId,
      spanId,
      traceFlags: parseInt(flags, 16),
      traceState: this.getHeader(carrier, this.TRACESTATE),
      isRemote: true
    };
  }

  /**
   * Get header value (case-insensitive)
   */
  private getHeader(carrier: Record<string, string>, key: string): string | undefined {
    return carrier[key] || carrier[key.toLowerCase()] || carrier[key.toUpperCase()];
  }
}

/**
 * B3 Propagator (Zipkin format)
 * Single header format
 */
export class B3Propagator implements TextMapPropagator {
  private readonly B3_HEADER = 'b3';

  /**
   * Inject trace context into carrier
   */
  public inject(context: SpanContext, carrier: Record<string, string>): void {
    const sampled = (context.traceFlags & 1) === 1 ? '1' : '0';
    carrier[this.B3_HEADER] = `${context.traceId}-${context.spanId}-${sampled}`;
  }

  /**
   * Extract trace context from carrier
   */
  public extract(carrier: Record<string, string>): SpanContext | undefined {
    const b3 = this.getHeader(carrier, this.B3_HEADER);

    if (!b3) {
      return undefined;
    }

    const parts = b3.split('-');

    if (parts.length < 2) {
      return undefined;
    }

    const [traceId, spanId, sampled] = parts;

    return {
      traceId,
      spanId,
      traceFlags: sampled === '1' ? 1 : 0,
      isRemote: true
    };
  }

  /**
   * Get header value (case-insensitive)
   */
  private getHeader(carrier: Record<string, string>, key: string): string | undefined {
    return carrier[key] || carrier[key.toLowerCase()] || carrier[key.toUpperCase()];
  }
}

/**
 * Composite propagator that tries multiple propagators
 */
export class CompositePropagator implements TextMapPropagator {
  private propagators: TextMapPropagator[];

  constructor(propagators: TextMapPropagator[]) {
    this.propagators = propagators;
  }

  /**
   * Inject using all propagators
   */
  public inject(context: SpanContext, carrier: Record<string, string>): void {
    for (const propagator of this.propagators) {
      propagator.inject(context, carrier);
    }
  }

  /**
   * Extract using first successful propagator
   */
  public extract(carrier: Record<string, string>): SpanContext | undefined {
    for (const propagator of this.propagators) {
      const context = propagator.extract(carrier);
      if (context) {
        return context;
      }
    }

    return undefined;
  }
}

/**
 * Default propagator (W3C Trace Context)
 */
export const defaultPropagator = new W3CTraceContextPropagator();

/**
 * Multi-format propagator (W3C + B3)
 */
export const multiFormatPropagator = new CompositePropagator([
  new W3CTraceContextPropagator(),
  new B3Propagator()
]);
