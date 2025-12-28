/**
 * AccuScene Enterprise v0.2.0
 * Metrics Collector
 *
 * Centralized metrics collection and management
 */

import { Metric, MetricType, CollectorConfig, CollectedMetrics } from '../types';
import { Counter, Gauge, Histogram, Summary, Timer } from '../metrics';
import { Span } from '../tracing/span';
import { CPUProfiler } from '../profiling/cpu';
import { MemoryProfiler } from '../profiling/memory';
import { RenderProfiler } from '../profiling/render';
import { NetworkProfiler } from '../profiling/network';

export class MetricsCollector {
  private metrics: Map<string, Metric> = new Map();
  private traces: Span[] = [];
  private config: CollectorConfig;
  private cpuProfiler: CPUProfiler;
  private memoryProfiler: MemoryProfiler;
  private renderProfiler: RenderProfiler;
  private networkProfiler: NetworkProfiler;
  private collectionInterval?: NodeJS.Timeout;

  constructor(config?: Partial<CollectorConfig>) {
    this.config = {
      interval: config?.interval || 60000, // 1 minute
      batchSize: config?.batchSize || 1000,
      maxBufferSize: config?.maxBufferSize || 10000,
      flushInterval: config?.flushInterval || 5000
    };

    this.cpuProfiler = new CPUProfiler();
    this.memoryProfiler = new MemoryProfiler();
    this.renderProfiler = new RenderProfiler();
    this.networkProfiler = new NetworkProfiler();
  }

  /**
   * Register a metric
   */
  public registerMetric(metric: Metric): void {
    const name = metric.metadata.name;

    if (this.metrics.has(name)) {
      console.warn(`Metric ${name} is already registered`);
      return;
    }

    this.metrics.set(name, metric);
  }

  /**
   * Get a registered metric
   */
  public getMetric<T extends Metric>(name: string): T | undefined {
    return this.metrics.get(name) as T;
  }

  /**
   * Get or create a counter
   */
  public getOrCreateCounter(name: string, help: string, unit?: string): Counter {
    let metric = this.getMetric<Counter>(name);

    if (!metric) {
      metric = new Counter(name, help, unit);
      this.registerMetric(metric);
    }

    return metric;
  }

  /**
   * Get or create a gauge
   */
  public getOrCreateGauge(name: string, help: string, unit?: string): Gauge {
    let metric = this.getMetric<Gauge>(name);

    if (!metric) {
      metric = new Gauge(name, help, unit);
      this.registerMetric(metric);
    }

    return metric;
  }

  /**
   * Get or create a histogram
   */
  public getOrCreateHistogram(
    name: string,
    help: string,
    buckets?: number[],
    unit?: string
  ): Histogram {
    let metric = this.getMetric<Histogram>(name);

    if (!metric) {
      metric = new Histogram(name, help, buckets, unit);
      this.registerMetric(metric);
    }

    return metric;
  }

  /**
   * Get or create a summary
   */
  public getOrCreateSummary(
    name: string,
    help: string,
    quantiles?: number[],
    maxAge?: number,
    unit?: string
  ): Summary {
    let metric = this.getMetric<Summary>(name);

    if (!metric) {
      metric = new Summary(name, help, quantiles, maxAge, unit);
      this.registerMetric(metric);
    }

    return metric;
  }

  /**
   * Get or create a timer
   */
  public getOrCreateTimer(name: string, help: string, buckets?: number[]): Timer {
    const histogramName = `${name}_duration_seconds`;
    let histogram = this.getMetric<Histogram>(histogramName);

    if (!histogram) {
      const timer = new Timer(name, help, buckets);
      this.registerMetric(timer as any);
      return timer;
    }

    return new Timer(name, help, buckets);
  }

  /**
   * Add a trace span
   */
  public addTrace(span: Span): void {
    this.traces.push(span);

    // Limit buffer size
    if (this.traces.length > this.config.maxBufferSize) {
      this.traces.shift();
    }
  }

  /**
   * Collect all metrics
   */
  public collect(): CollectedMetrics {
    const timestamp = Date.now();
    const metricsData = new Map();

    // Collect all registered metrics
    for (const [name, metric] of this.metrics.entries()) {
      const values = metric.getValues();
      metricsData.set(name, values);
    }

    // Collect profiling data
    const cpuUsage = CPUProfiler.getCPUUsage();
    const memorySnapshot = this.memoryProfiler.takeSnapshot();
    const renderProfile = this.renderProfiler.getProfile();
    const networkProfile = this.networkProfiler.getProfile();

    // Update system metrics
    this.updateSystemMetrics(cpuUsage, memorySnapshot);

    return {
      timestamp,
      metrics: metricsData,
      traces: [...this.traces],
      profiles: {
        memory: memorySnapshot,
        render: renderProfile,
        network: networkProfile
      }
    };
  }

  /**
   * Update system metrics automatically
   */
  private async updateSystemMetrics(cpuUsage: Promise<number>, memorySnapshot: any): Promise<void> {
    // CPU usage
    const cpu = await cpuUsage;
    const cpuGauge = this.getOrCreateGauge(
      'system_cpu_usage_percent',
      'Current CPU usage percentage',
      'percent'
    );
    cpuGauge.set(cpu);

    // Memory usage
    const memoryGauge = this.getOrCreateGauge(
      'system_memory_heap_used_bytes',
      'Current heap memory usage',
      'bytes'
    );
    memoryGauge.set(memorySnapshot.heapUsed);

    const memoryTotalGauge = this.getOrCreateGauge(
      'system_memory_heap_total_bytes',
      'Total heap memory',
      'bytes'
    );
    memoryTotalGauge.set(memorySnapshot.heapTotal);
  }

  /**
   * Start automatic collection
   */
  public startCollection(): void {
    if (this.collectionInterval) {
      return;
    }

    // Start profilers
    this.renderProfiler.start();
    this.networkProfiler.start();
    this.memoryProfiler.setBaseline();

    // Collect metrics at regular intervals
    this.collectionInterval = setInterval(() => {
      this.collect();
    }, this.config.interval);
  }

  /**
   * Stop automatic collection
   */
  public stopCollection(): void {
    if (this.collectionInterval) {
      clearInterval(this.collectionInterval);
      this.collectionInterval = undefined;
    }

    // Stop profilers
    this.renderProfiler.stop();
    this.networkProfiler.stop();
  }

  /**
   * Reset all metrics
   */
  public reset(): void {
    for (const metric of this.metrics.values()) {
      metric.reset();
    }

    this.traces = [];
  }

  /**
   * Get all registered metric names
   */
  public getMetricNames(): string[] {
    return Array.from(this.metrics.keys());
  }

  /**
   * Get metrics count
   */
  public getMetricsCount(): number {
    return this.metrics.size;
  }

  /**
   * Clear traces
   */
  public clearTraces(): void {
    this.traces = [];
  }

  /**
   * Get CPU profiler
   */
  public getCPUProfiler(): CPUProfiler {
    return this.cpuProfiler;
  }

  /**
   * Get memory profiler
   */
  public getMemoryProfiler(): MemoryProfiler {
    return this.memoryProfiler;
  }

  /**
   * Get render profiler
   */
  public getRenderProfiler(): RenderProfiler {
    return this.renderProfiler;
  }

  /**
   * Get network profiler
   */
  public getNetworkProfiler(): NetworkProfiler {
    return this.networkProfiler;
  }
}

// Global collector instance
export const globalCollector = new MetricsCollector();
