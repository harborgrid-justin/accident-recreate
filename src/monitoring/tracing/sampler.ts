/**
 * AccuScene Enterprise v0.2.0
 * Trace Sampling
 *
 * Configurable sampling strategies for distributed tracing
 */

import { SpanContext, SpanKind, SpanAttributes } from '../types';

/**
 * Sampling decision
 */
export enum SamplingDecision {
  DROP = 0,
  RECORD_ONLY = 1,
  RECORD_AND_SAMPLE = 2
}

/**
 * Sampling result
 */
export interface SamplingResult {
  decision: SamplingDecision;
  attributes?: SpanAttributes;
}

/**
 * Sampler interface
 */
export interface Sampler {
  shouldSample(
    context: SpanContext | undefined,
    traceId: string,
    spanName: string,
    spanKind: SpanKind,
    attributes: SpanAttributes
  ): SamplingResult;

  getDescription(): string;
}

/**
 * Always sample all traces
 */
export class AlwaysOnSampler implements Sampler {
  public shouldSample(): SamplingResult {
    return {
      decision: SamplingDecision.RECORD_AND_SAMPLE
    };
  }

  public getDescription(): string {
    return 'AlwaysOnSampler';
  }
}

/**
 * Never sample any traces
 */
export class AlwaysOffSampler implements Sampler {
  public shouldSample(): SamplingResult {
    return {
      decision: SamplingDecision.DROP
    };
  }

  public getDescription(): string {
    return 'AlwaysOffSampler';
  }
}

/**
 * Sample traces based on trace ID with a fixed probability
 */
export class TraceIdRatioBasedSampler implements Sampler {
  private ratio: number;
  private idUpperBound: bigint;

  constructor(ratio: number = 1.0) {
    if (ratio < 0 || ratio > 1) {
      throw new Error('Sampling ratio must be between 0 and 1');
    }

    this.ratio = ratio;

    // Calculate the upper bound for sampling based on ratio
    // We use the first 8 bytes of the trace ID as a random value
    const maxId = BigInt('0xffffffffffffffff');
    this.idUpperBound = BigInt(Math.floor(Number(maxId) * ratio));
  }

  public shouldSample(
    context: SpanContext | undefined,
    traceId: string
  ): SamplingResult {
    // Extract first 16 hex chars (8 bytes) from trace ID
    const traceIdPrefix = traceId.substring(0, 16);
    const traceIdValue = BigInt('0x' + traceIdPrefix);

    const decision = traceIdValue <= this.idUpperBound
      ? SamplingDecision.RECORD_AND_SAMPLE
      : SamplingDecision.DROP;

    return { decision };
  }

  public getDescription(): string {
    return `TraceIdRatioBasedSampler{ratio=${this.ratio}}`;
  }
}

/**
 * Parent-based sampler - respects parent's sampling decision
 */
export class ParentBasedSampler implements Sampler {
  private root: Sampler;
  private remoteParentSampled?: Sampler;
  private remoteParentNotSampled?: Sampler;
  private localParentSampled?: Sampler;
  private localParentNotSampled?: Sampler;

  constructor(config: {
    root: Sampler;
    remoteParentSampled?: Sampler;
    remoteParentNotSampled?: Sampler;
    localParentSampled?: Sampler;
    localParentNotSampled?: Sampler;
  }) {
    this.root = config.root;
    this.remoteParentSampled = config.remoteParentSampled || new AlwaysOnSampler();
    this.remoteParentNotSampled = config.remoteParentNotSampled || new AlwaysOffSampler();
    this.localParentSampled = config.localParentSampled || new AlwaysOnSampler();
    this.localParentNotSampled = config.localParentNotSampled || new AlwaysOffSampler();
  }

  public shouldSample(
    context: SpanContext | undefined,
    traceId: string,
    spanName: string,
    spanKind: SpanKind,
    attributes: SpanAttributes
  ): SamplingResult {
    // No parent, use root sampler
    if (!context) {
      return this.root.shouldSample(context, traceId, spanName, spanKind, attributes);
    }

    // Has parent, check if sampled
    const isSampled = (context.traceFlags & 1) === 1;
    const isRemote = context.isRemote || false;

    let sampler: Sampler;

    if (isRemote) {
      sampler = isSampled ? this.remoteParentSampled! : this.remoteParentNotSampled!;
    } else {
      sampler = isSampled ? this.localParentSampled! : this.localParentNotSampled!;
    }

    return sampler.shouldSample(context, traceId, spanName, spanKind, attributes);
  }

  public getDescription(): string {
    return `ParentBasedSampler{root=${this.root.getDescription()}}`;
  }
}

/**
 * Rate limiting sampler - limits samples per second
 */
export class RateLimitingSampler implements Sampler {
  private maxTracesPerSecond: number;
  private tokens: number;
  private lastRefill: number;

  constructor(maxTracesPerSecond: number = 100) {
    this.maxTracesPerSecond = maxTracesPerSecond;
    this.tokens = maxTracesPerSecond;
    this.lastRefill = Date.now();
  }

  public shouldSample(): SamplingResult {
    this.refillTokens();

    if (this.tokens >= 1) {
      this.tokens -= 1;
      return {
        decision: SamplingDecision.RECORD_AND_SAMPLE
      };
    }

    return {
      decision: SamplingDecision.DROP
    };
  }

  private refillTokens(): void {
    const now = Date.now();
    const elapsed = (now - this.lastRefill) / 1000;

    this.tokens = Math.min(
      this.maxTracesPerSecond,
      this.tokens + elapsed * this.maxTracesPerSecond
    );

    this.lastRefill = now;
  }

  public getDescription(): string {
    return `RateLimitingSampler{maxTracesPerSecond=${this.maxTracesPerSecond}}`;
  }
}

/**
 * Attribute-based sampler - samples based on span attributes
 */
export class AttributeBasedSampler implements Sampler {
  private attributeKey: string;
  private expectedValue: string | number | boolean;
  private fallbackSampler: Sampler;

  constructor(
    attributeKey: string,
    expectedValue: string | number | boolean,
    fallbackSampler: Sampler = new AlwaysOffSampler()
  ) {
    this.attributeKey = attributeKey;
    this.expectedValue = expectedValue;
    this.fallbackSampler = fallbackSampler;
  }

  public shouldSample(
    context: SpanContext | undefined,
    traceId: string,
    spanName: string,
    spanKind: SpanKind,
    attributes: SpanAttributes
  ): SamplingResult {
    const attributeValue = attributes[this.attributeKey];

    if (attributeValue === this.expectedValue) {
      return {
        decision: SamplingDecision.RECORD_AND_SAMPLE
      };
    }

    return this.fallbackSampler.shouldSample(context, traceId, spanName, spanKind, attributes);
  }

  public getDescription(): string {
    return `AttributeBasedSampler{${this.attributeKey}=${this.expectedValue}}`;
  }
}

/**
 * Default sampler configuration
 */
export const defaultSampler = new ParentBasedSampler({
  root: new TraceIdRatioBasedSampler(1.0) // 100% sampling by default
});
