/**
 * AccuScene Enterprise v0.2.0
 * Memory Profiling
 *
 * Memory usage profiling and leak detection
 */

import { MemoryProfile, AllocationProfile } from '../types';

export class MemoryProfiler {
  private baselineSnapshot?: MemoryProfile;
  private allocationTracking: boolean = false;
  private allocations: Map<string, AllocationProfile> = new Map();

  /**
   * Take a memory snapshot
   */
  public takeSnapshot(): MemoryProfile {
    const memoryUsage = this.getMemoryUsage();

    return {
      timestamp: Date.now(),
      heapUsed: memoryUsage.heapUsed,
      heapTotal: memoryUsage.heapTotal,
      external: memoryUsage.external,
      arrayBuffers: memoryUsage.arrayBuffers,
      rss: memoryUsage.rss,
      allocations: this.allocationTracking
        ? Array.from(this.allocations.values())
        : undefined
    };
  }

  /**
   * Set baseline snapshot for comparison
   */
  public setBaseline(): void {
    this.baselineSnapshot = this.takeSnapshot();
  }

  /**
   * Get memory growth since baseline
   */
  public getGrowth(): MemoryProfile | null {
    if (!this.baselineSnapshot) {
      return null;
    }

    const current = this.takeSnapshot();

    return {
      timestamp: current.timestamp,
      heapUsed: current.heapUsed - this.baselineSnapshot.heapUsed,
      heapTotal: current.heapTotal - this.baselineSnapshot.heapTotal,
      external: current.external - this.baselineSnapshot.external,
      arrayBuffers: current.arrayBuffers - this.baselineSnapshot.arrayBuffers,
      rss: current.rss ? (current.rss - (this.baselineSnapshot.rss || 0)) : undefined
    };
  }

  /**
   * Start tracking allocations
   */
  public startAllocationTracking(): void {
    this.allocationTracking = true;
    this.allocations.clear();

    // Hook into object creation (simplified version)
    // In production, this would use v8 profiling APIs
  }

  /**
   * Stop tracking allocations
   */
  public stopAllocationTracking(): AllocationProfile[] {
    this.allocationTracking = false;
    return Array.from(this.allocations.values());
  }

  /**
   * Record an allocation
   */
  private recordAllocation(type: string, size: number, stack?: string[]): void {
    if (!this.allocationTracking) {
      return;
    }

    const key = type;
    const existing = this.allocations.get(key);

    if (existing) {
      existing.count++;
      existing.size += size;
    } else {
      this.allocations.set(key, {
        type,
        size,
        count: 1,
        stack
      });
    }
  }

  /**
   * Get memory usage
   */
  private getMemoryUsage(): {
    heapUsed: number;
    heapTotal: number;
    external: number;
    arrayBuffers: number;
    rss?: number;
  } {
    if (typeof process !== 'undefined' && process.memoryUsage) {
      const usage = process.memoryUsage();

      return {
        heapUsed: usage.heapUsed,
        heapTotal: usage.heapTotal,
        external: usage.external,
        arrayBuffers: usage.arrayBuffers || 0,
        rss: usage.rss
      };
    }

    // Browser fallback
    if (typeof performance !== 'undefined' && (performance as any).memory) {
      const memory = (performance as any).memory;

      return {
        heapUsed: memory.usedJSHeapSize || 0,
        heapTotal: memory.totalJSHeapSize || 0,
        external: 0,
        arrayBuffers: 0
      };
    }

    return {
      heapUsed: 0,
      heapTotal: 0,
      external: 0,
      arrayBuffers: 0
    };
  }

  /**
   * Detect potential memory leaks
   */
  public static detectLeaks(profiles: MemoryProfile[], threshold: number = 1.5): boolean {
    if (profiles.length < 3) {
      return false;
    }

    // Check if memory is consistently growing
    const growthRates: number[] = [];

    for (let i = 1; i < profiles.length; i++) {
      const rate = profiles[i].heapUsed / profiles[i - 1].heapUsed;
      growthRates.push(rate);
    }

    // If average growth rate exceeds threshold
    const avgGrowth = growthRates.reduce((a, b) => a + b, 0) / growthRates.length;

    return avgGrowth > threshold;
  }

  /**
   * Get top memory consumers
   */
  public getTopConsumers(limit: number = 10): AllocationProfile[] {
    return Array.from(this.allocations.values())
      .sort((a, b) => b.size - a.size)
      .slice(0, limit);
  }

  /**
   * Format memory size
   */
  public static formatSize(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(2)} ${units[unitIndex]}`;
  }

  /**
   * Get memory statistics
   */
  public getStatistics(profiles: MemoryProfile[]): {
    min: number;
    max: number;
    avg: number;
    current: number;
  } {
    if (profiles.length === 0) {
      return { min: 0, max: 0, avg: 0, current: 0 };
    }

    const heapUsages = profiles.map(p => p.heapUsed);

    return {
      min: Math.min(...heapUsages),
      max: Math.max(...heapUsages),
      avg: heapUsages.reduce((a, b) => a + b, 0) / heapUsages.length,
      current: heapUsages[heapUsages.length - 1]
    };
  }

  /**
   * Force garbage collection (if available)
   */
  public static async forceGC(): Promise<boolean> {
    if (typeof global !== 'undefined' && (global as any).gc) {
      (global as any).gc();
      return true;
    }

    return false;
  }
}
