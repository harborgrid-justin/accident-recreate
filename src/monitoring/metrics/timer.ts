/**
 * AccuScene Enterprise v0.2.0
 * Timer Metric Implementation
 *
 * Timer for measuring duration of operations
 */

import { Histogram } from './histogram';
import { MetricLabels, MetricValue, MetricSnapshot } from '../types';

export class Timer {
  private histogram: Histogram;

  constructor(
    name: string,
    help: string,
    buckets?: number[],
    labels?: string[]
  ) {
    // Default buckets for duration in seconds
    const defaultBuckets = [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10];

    this.histogram = new Histogram(
      name,
      help,
      buckets || defaultBuckets,
      'seconds',
      labels
    );
  }

  /**
   * Start a timer and return a function to stop it
   */
  public startTimer(labels?: MetricLabels): () => number {
    const start = performance.now();

    return () => {
      const duration = (performance.now() - start) / 1000; // Convert to seconds
      this.histogram.observe(duration, labels);
      return duration;
    };
  }

  /**
   * Time a function execution
   */
  public async time<T>(
    fn: () => Promise<T> | T,
    labels?: MetricLabels
  ): Promise<T> {
    const stopTimer = this.startTimer(labels);

    try {
      const result = await Promise.resolve(fn());
      stopTimer();
      return result;
    } catch (error) {
      stopTimer();
      throw error;
    }
  }

  /**
   * Observe a duration directly
   */
  public observe(duration: number, labels?: MetricLabels): void {
    this.histogram.observe(duration, labels);
  }

  /**
   * Get timer snapshot
   */
  public getSnapshot(): MetricSnapshot {
    return this.histogram.getSnapshot();
  }

  /**
   * Get all recorded values
   */
  public getValues(): MetricValue[] {
    return this.histogram.getValues();
  }

  /**
   * Reset timer
   */
  public reset(): void {
    this.histogram.reset();
  }

  /**
   * Export in Prometheus format
   */
  public toPrometheusFormat(): string {
    return this.histogram.toPrometheusFormat();
  }

  /**
   * Get metadata
   */
  public get metadata() {
    return this.histogram.metadata;
  }

  /**
   * Get value (returns snapshot)
   */
  public getValue(): MetricSnapshot {
    return this.getSnapshot();
  }
}

/**
 * Decorator for timing class methods
 */
export function Timed(timer: Timer, labels?: MetricLabels) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: any[]) {
      return timer.time(() => originalMethod.apply(this, args), labels);
    };

    return descriptor;
  };
}

/**
 * Measure execution time of a block
 */
export class TimerContext {
  private startTime: number;
  private timer: Timer;
  private labels?: MetricLabels;

  constructor(timer: Timer, labels?: MetricLabels) {
    this.timer = timer;
    this.labels = labels;
    this.startTime = performance.now();
  }

  /**
   * Stop the timer and record the duration
   */
  public stop(): number {
    const duration = (performance.now() - this.startTime) / 1000;
    this.timer.observe(duration, this.labels);
    return duration;
  }
}
