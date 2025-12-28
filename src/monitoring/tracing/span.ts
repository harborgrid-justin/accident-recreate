/**
 * AccuScene Enterprise v0.2.0
 * Span Implementation
 *
 * OpenTelemetry-compatible span for distributed tracing
 */

import { v4 as uuidv4 } from 'uuid';
import {
  Span as ISpan,
  SpanContext,
  SpanKind,
  SpanStatus,
  SpanAttributes,
  SpanEvent,
  SpanLink
} from '../types';

export class Span implements ISpan {
  public readonly context: SpanContext;
  public name: string;
  public kind: SpanKind;
  public parentSpanId?: string;
  public readonly startTime: number;
  public endTime?: number;
  public status: SpanStatus = SpanStatus.UNSET;
  public attributes: SpanAttributes = {};
  public events: SpanEvent[] = [];
  public links: SpanLink[] = [];

  private statusMessage?: string;
  private ended: boolean = false;

  constructor(
    name: string,
    kind: SpanKind = SpanKind.INTERNAL,
    traceId?: string,
    parentSpanId?: string
  ) {
    this.name = name;
    this.kind = kind;
    this.parentSpanId = parentSpanId;
    this.startTime = Date.now();

    this.context = {
      traceId: traceId || this.generateTraceId(),
      spanId: this.generateSpanId(),
      traceFlags: 1, // Sampled
      isRemote: false
    };
  }

  /**
   * Set a single attribute
   */
  public setAttribute(key: string, value: string | number | boolean): void {
    if (this.ended) {
      console.warn('Cannot set attribute on ended span');
      return;
    }

    this.attributes[key] = value;
  }

  /**
   * Set multiple attributes
   */
  public setAttributes(attributes: SpanAttributes): void {
    if (this.ended) {
      console.warn('Cannot set attributes on ended span');
      return;
    }

    Object.assign(this.attributes, attributes);
  }

  /**
   * Add an event to the span
   */
  public addEvent(name: string, attributes?: SpanAttributes): void {
    if (this.ended) {
      console.warn('Cannot add event to ended span');
      return;
    }

    this.events.push({
      name,
      timestamp: Date.now(),
      attributes
    });
  }

  /**
   * Add a link to another span
   */
  public addLink(context: SpanContext, attributes?: SpanAttributes): void {
    if (this.ended) {
      console.warn('Cannot add link to ended span');
      return;
    }

    this.links.push({ context, attributes });
  }

  /**
   * Set the span status
   */
  public setStatus(status: SpanStatus, message?: string): void {
    if (this.ended) {
      console.warn('Cannot set status on ended span');
      return;
    }

    this.status = status;
    this.statusMessage = message;
  }

  /**
   * Record an exception
   */
  public recordException(error: Error): void {
    this.addEvent('exception', {
      'exception.type': error.name,
      'exception.message': error.message,
      'exception.stacktrace': error.stack || ''
    });

    this.setStatus(SpanStatus.ERROR, error.message);
  }

  /**
   * End the span
   */
  public end(timestamp?: number): void {
    if (this.ended) {
      console.warn('Span already ended');
      return;
    }

    this.endTime = timestamp || Date.now();
    this.ended = true;
  }

  /**
   * Check if span has ended
   */
  public isEnded(): boolean {
    return this.ended;
  }

  /**
   * Get span duration in milliseconds
   */
  public getDuration(): number {
    if (!this.endTime) {
      return Date.now() - this.startTime;
    }
    return this.endTime - this.startTime;
  }

  /**
   * Get status message
   */
  public getStatusMessage(): string | undefined {
    return this.statusMessage;
  }

  /**
   * Generate a unique trace ID (128-bit)
   */
  private generateTraceId(): string {
    return uuidv4().replace(/-/g, '') + uuidv4().replace(/-/g, '').substring(0, 16);
  }

  /**
   * Generate a unique span ID (64-bit)
   */
  private generateSpanId(): string {
    return uuidv4().replace(/-/g, '').substring(0, 16);
  }

  /**
   * Convert to JSON for export
   */
  public toJSON(): any {
    return {
      traceId: this.context.traceId,
      spanId: this.context.spanId,
      parentSpanId: this.parentSpanId,
      name: this.name,
      kind: this.kind,
      startTime: this.startTime,
      endTime: this.endTime,
      duration: this.getDuration(),
      status: this.status,
      statusMessage: this.statusMessage,
      attributes: this.attributes,
      events: this.events,
      links: this.links
    };
  }

  /**
   * Convert to OpenTelemetry format
   */
  public toOpenTelemetryFormat(): any {
    return {
      traceId: this.context.traceId,
      spanId: this.context.spanId,
      parentSpanId: this.parentSpanId,
      name: this.name,
      kind: this.getOtelKind(),
      startTimeUnixNano: (this.startTime * 1000000).toString(),
      endTimeUnixNano: this.endTime ? (this.endTime * 1000000).toString() : undefined,
      attributes: this.attributesToOtelFormat(),
      status: {
        code: this.status === SpanStatus.OK ? 1 : this.status === SpanStatus.ERROR ? 2 : 0,
        message: this.statusMessage
      },
      events: this.events.map(event => ({
        timeUnixNano: (event.timestamp * 1000000).toString(),
        name: event.name,
        attributes: event.attributes ? this.attributesToOtelFormat(event.attributes) : []
      }))
    };
  }

  /**
   * Convert SpanKind to OpenTelemetry numeric kind
   */
  private getOtelKind(): number {
    const kindMap: Record<SpanKind, number> = {
      [SpanKind.INTERNAL]: 1,
      [SpanKind.SERVER]: 2,
      [SpanKind.CLIENT]: 3,
      [SpanKind.PRODUCER]: 4,
      [SpanKind.CONSUMER]: 5
    };

    return kindMap[this.kind] || 0;
  }

  /**
   * Convert attributes to OpenTelemetry format
   */
  private attributesToOtelFormat(attrs: SpanAttributes = this.attributes): any[] {
    return Object.entries(attrs).map(([key, value]) => {
      let valueType: string;
      let valueKey: string;

      if (typeof value === 'string') {
        valueType = 'stringValue';
        valueKey = 'stringValue';
      } else if (typeof value === 'number') {
        valueType = Number.isInteger(value) ? 'intValue' : 'doubleValue';
        valueKey = valueType;
      } else if (typeof value === 'boolean') {
        valueType = 'boolValue';
        valueKey = 'boolValue';
      } else {
        valueType = 'stringValue';
        valueKey = 'stringValue';
        value = JSON.stringify(value);
      }

      return {
        key,
        value: {
          [valueKey]: value
        }
      };
    });
  }
}
