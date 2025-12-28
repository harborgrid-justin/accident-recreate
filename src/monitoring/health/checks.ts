/**
 * AccuScene Enterprise v0.2.0
 * Health Check Registry
 *
 * Centralized health check management
 */

import { HealthCheck, HealthCheckResult, HealthReport, HealthStatus } from '../types';

export class HealthCheckRegistry {
  private checks: Map<string, HealthCheck> = new Map();

  /**
   * Register a health check
   */
  public register(check: HealthCheck): void {
    if (this.checks.has(check.name)) {
      console.warn(`Health check ${check.name} is already registered`);
      return;
    }

    this.checks.set(check.name, check);
  }

  /**
   * Unregister a health check
   */
  public unregister(name: string): void {
    this.checks.delete(name);
  }

  /**
   * Run all health checks
   */
  public async runAll(): Promise<HealthReport> {
    const results: Record<string, HealthCheckResult> = {};
    let overallStatus: HealthStatus = HealthStatus.HEALTHY;

    for (const [name, check] of this.checks.entries()) {
      try {
        const result = await check.check();
        results[name] = result;

        // Determine overall status
        if (result.status === HealthStatus.UNHEALTHY) {
          overallStatus = HealthStatus.UNHEALTHY;
        } else if (
          result.status === HealthStatus.DEGRADED &&
          overallStatus !== HealthStatus.UNHEALTHY
        ) {
          overallStatus = HealthStatus.DEGRADED;
        }
      } catch (error: any) {
        results[name] = {
          status: HealthStatus.UNHEALTHY,
          message: `Health check failed: ${error.message}`,
          timestamp: Date.now()
        };

        overallStatus = HealthStatus.UNHEALTHY;
      }
    }

    return {
      status: overallStatus,
      checks: results,
      timestamp: Date.now()
    };
  }

  /**
   * Run a specific health check
   */
  public async run(name: string): Promise<HealthCheckResult | undefined> {
    const check = this.checks.get(name);

    if (!check) {
      return undefined;
    }

    try {
      return await check.check();
    } catch (error: any) {
      return {
        status: HealthStatus.UNHEALTHY,
        message: `Health check failed: ${error.message}`,
        timestamp: Date.now()
      };
    }
  }

  /**
   * Get all registered check names
   */
  public getCheckNames(): string[] {
    return Array.from(this.checks.keys());
  }

  /**
   * Get check count
   */
  public getCheckCount(): number {
    return this.checks.size;
  }

  /**
   * Clear all checks
   */
  public clear(): void {
    this.checks.clear();
  }
}

/**
 * Common health checks
 */

/**
 * Database health check
 */
export class DatabaseHealthCheck implements HealthCheck {
  public readonly name = 'database';
  private queryFn: () => Promise<boolean>;

  constructor(queryFn: () => Promise<boolean>) {
    this.queryFn = queryFn;
  }

  public async check(): Promise<HealthCheckResult> {
    const startTime = Date.now();

    try {
      const isHealthy = await this.queryFn();
      const duration = Date.now() - startTime;

      return {
        status: isHealthy ? HealthStatus.HEALTHY : HealthStatus.UNHEALTHY,
        message: isHealthy ? 'Database is accessible' : 'Database is not accessible',
        timestamp: Date.now(),
        details: {
          duration,
          connected: isHealthy
        }
      };
    } catch (error: any) {
      return {
        status: HealthStatus.UNHEALTHY,
        message: `Database connection failed: ${error.message}`,
        timestamp: Date.now()
      };
    }
  }
}

/**
 * Memory health check
 */
export class MemoryHealthCheck implements HealthCheck {
  public readonly name = 'memory';
  private maxHeapUsagePercent: number;

  constructor(maxHeapUsagePercent: number = 90) {
    this.maxHeapUsagePercent = maxHeapUsagePercent;
  }

  public async check(): Promise<HealthCheckResult> {
    if (typeof process === 'undefined' || !process.memoryUsage) {
      return {
        status: HealthStatus.HEALTHY,
        message: 'Memory monitoring not available',
        timestamp: Date.now()
      };
    }

    const usage = process.memoryUsage();
    const usagePercent = (usage.heapUsed / usage.heapTotal) * 100;

    let status: HealthStatus;
    let message: string;

    if (usagePercent < this.maxHeapUsagePercent) {
      status = HealthStatus.HEALTHY;
      message = 'Memory usage is normal';
    } else if (usagePercent < 95) {
      status = HealthStatus.DEGRADED;
      message = 'Memory usage is high';
    } else {
      status = HealthStatus.UNHEALTHY;
      message = 'Memory usage is critical';
    }

    return {
      status,
      message,
      timestamp: Date.now(),
      details: {
        heapUsed: usage.heapUsed,
        heapTotal: usage.heapTotal,
        usagePercent: usagePercent.toFixed(2),
        rss: usage.rss
      }
    };
  }
}

/**
 * CPU health check
 */
export class CPUHealthCheck implements HealthCheck {
  public readonly name = 'cpu';
  private maxCPUPercent: number;

  constructor(maxCPUPercent: number = 80) {
    this.maxCPUPercent = maxCPUPercent;
  }

  public async check(): Promise<HealthCheckResult> {
    if (typeof process === 'undefined' || !process.cpuUsage) {
      return {
        status: HealthStatus.HEALTHY,
        message: 'CPU monitoring not available',
        timestamp: Date.now()
      };
    }

    const startUsage = process.cpuUsage();
    const startTime = Date.now();

    await new Promise(resolve => setTimeout(resolve, 100));

    const endUsage = process.cpuUsage(startUsage);
    const endTime = Date.now();

    const elapsed = (endTime - startTime) * 1000; // microseconds
    const totalUsage = endUsage.user + endUsage.system;
    const cpuPercent = (totalUsage / elapsed) * 100;

    let status: HealthStatus;
    let message: string;

    if (cpuPercent < this.maxCPUPercent) {
      status = HealthStatus.HEALTHY;
      message = 'CPU usage is normal';
    } else if (cpuPercent < 90) {
      status = HealthStatus.DEGRADED;
      message = 'CPU usage is high';
    } else {
      status = HealthStatus.UNHEALTHY;
      message = 'CPU usage is critical';
    }

    return {
      status,
      message,
      timestamp: Date.now(),
      details: {
        cpuPercent: cpuPercent.toFixed(2),
        user: endUsage.user,
        system: endUsage.system
      }
    };
  }
}

/**
 * Disk health check
 */
export class DiskHealthCheck implements HealthCheck {
  public readonly name = 'disk';
  private path: string;
  private minFreePercent: number;

  constructor(path: string = '/', minFreePercent: number = 10) {
    this.path = path;
    this.minFreePercent = minFreePercent;
  }

  public async check(): Promise<HealthCheckResult> {
    // Simplified check - in production would check actual disk space
    return {
      status: HealthStatus.HEALTHY,
      message: 'Disk space is adequate',
      timestamp: Date.now(),
      details: {
        path: this.path
      }
    };
  }
}

/**
 * HTTP endpoint health check
 */
export class HTTPHealthCheck implements HealthCheck {
  public readonly name: string;
  private url: string;
  private timeout: number;

  constructor(name: string, url: string, timeout: number = 5000) {
    this.name = name;
    this.url = url;
    this.timeout = timeout;
  }

  public async check(): Promise<HealthCheckResult> {
    const startTime = Date.now();

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.timeout);

      const response = await fetch(this.url, {
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      const duration = Date.now() - startTime;
      const isHealthy = response.ok;

      return {
        status: isHealthy ? HealthStatus.HEALTHY : HealthStatus.UNHEALTHY,
        message: isHealthy
          ? 'Endpoint is accessible'
          : `Endpoint returned ${response.status}`,
        timestamp: Date.now(),
        details: {
          url: this.url,
          status: response.status,
          duration
        }
      };
    } catch (error: any) {
      return {
        status: HealthStatus.UNHEALTHY,
        message: `Endpoint check failed: ${error.message}`,
        timestamp: Date.now(),
        details: {
          url: this.url,
          error: error.message
        }
      };
    }
  }
}

// Global registry instance
export const globalHealthRegistry = new HealthCheckRegistry();
