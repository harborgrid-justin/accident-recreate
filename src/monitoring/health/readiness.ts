/**
 * AccuScene Enterprise v0.2.0
 * Readiness Probe
 *
 * Kubernetes-style readiness probe for detecting if application is ready to serve traffic
 */

import { HealthCheck, HealthCheckResult, HealthStatus } from '../types';

/**
 * Readiness probe checks if the application is ready to receive traffic
 * If readiness fails, traffic should not be routed to the application
 */
export class ReadinessProbe implements HealthCheck {
  public readonly name = 'readiness';
  private dependencies: HealthCheck[] = [];
  private isReady: boolean = false;

  /**
   * Add a dependency check
   */
  public addDependency(check: HealthCheck): void {
    this.dependencies.push(check);
  }

  /**
   * Remove a dependency check
   */
  public removeDependency(name: string): void {
    this.dependencies = this.dependencies.filter(c => c.name !== name);
  }

  /**
   * Mark application as ready
   */
  public setReady(ready: boolean): void {
    this.isReady = ready;
  }

  /**
   * Check readiness
   */
  public async check(): Promise<HealthCheckResult> {
    if (!this.isReady) {
      return {
        status: HealthStatus.UNHEALTHY,
        message: 'Application is not ready',
        timestamp: Date.now(),
        details: {
          ready: false
        }
      };
    }

    // Check all dependencies
    const dependencyResults: Record<string, HealthCheckResult> = {};
    let allHealthy = true;

    for (const dependency of this.dependencies) {
      try {
        const result = await dependency.check();
        dependencyResults[dependency.name] = result;

        if (result.status !== HealthStatus.HEALTHY) {
          allHealthy = false;
        }
      } catch (error: any) {
        dependencyResults[dependency.name] = {
          status: HealthStatus.UNHEALTHY,
          message: `Dependency check failed: ${error.message}`,
          timestamp: Date.now()
        };

        allHealthy = false;
      }
    }

    return {
      status: allHealthy ? HealthStatus.HEALTHY : HealthStatus.UNHEALTHY,
      message: allHealthy
        ? 'Application is ready to serve traffic'
        : 'Application is not ready - dependencies unhealthy',
      timestamp: Date.now(),
      details: {
        ready: this.isReady,
        dependencies: dependencyResults
      }
    };
  }
}

/**
 * Database readiness check
 */
export class DatabaseReadinessCheck implements HealthCheck {
  public readonly name = 'database_ready';
  private testQueryFn: () => Promise<boolean>;
  private isInitialized: boolean = false;

  constructor(testQueryFn: () => Promise<boolean>) {
    this.testQueryFn = testQueryFn;
  }

  /**
   * Mark database as initialized
   */
  public setInitialized(initialized: boolean): void {
    this.isInitialized = initialized;
  }

  /**
   * Check database readiness
   */
  public async check(): Promise<HealthCheckResult> {
    if (!this.isInitialized) {
      return {
        status: HealthStatus.UNHEALTHY,
        message: 'Database not initialized',
        timestamp: Date.now(),
        details: {
          initialized: false
        }
      };
    }

    try {
      const canQuery = await this.testQueryFn();

      return {
        status: canQuery ? HealthStatus.HEALTHY : HealthStatus.UNHEALTHY,
        message: canQuery
          ? 'Database is ready'
          : 'Database is not accepting queries',
        timestamp: Date.now(),
        details: {
          initialized: true,
          canQuery
        }
      };
    } catch (error: any) {
      return {
        status: HealthStatus.UNHEALTHY,
        message: `Database readiness check failed: ${error.message}`,
        timestamp: Date.now(),
        details: {
          error: error.message
        }
      };
    }
  }
}

/**
 * Cache readiness check
 */
export class CacheReadinessCheck implements HealthCheck {
  public readonly name = 'cache_ready';
  private testCacheFn: () => Promise<boolean>;

  constructor(testCacheFn: () => Promise<boolean>) {
    this.testCacheFn = testCacheFn;
  }

  /**
   * Check cache readiness
   */
  public async check(): Promise<HealthCheckResult> {
    try {
      const isReady = await this.testCacheFn();

      return {
        status: isReady ? HealthStatus.HEALTHY : HealthStatus.DEGRADED,
        message: isReady
          ? 'Cache is ready'
          : 'Cache is not available',
        timestamp: Date.now(),
        details: {
          ready: isReady
        }
      };
    } catch (error: any) {
      return {
        status: HealthStatus.DEGRADED,
        message: `Cache check failed: ${error.message}`,
        timestamp: Date.now(),
        details: {
          error: error.message
        }
      };
    }
  }
}

/**
 * External service readiness check
 */
export class ExternalServiceReadinessCheck implements HealthCheck {
  public readonly name: string;
  private healthEndpoint: string;
  private timeout: number;
  private required: boolean;

  constructor(
    name: string,
    healthEndpoint: string,
    timeout: number = 5000,
    required: boolean = true
  ) {
    this.name = name;
    this.healthEndpoint = healthEndpoint;
    this.timeout = timeout;
    this.required = required;
  }

  /**
   * Check external service readiness
   */
  public async check(): Promise<HealthCheckResult> {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.timeout);

      const response = await fetch(this.healthEndpoint, {
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      const isHealthy = response.ok;

      return {
        status: isHealthy
          ? HealthStatus.HEALTHY
          : this.required
          ? HealthStatus.UNHEALTHY
          : HealthStatus.DEGRADED,
        message: isHealthy
          ? `External service ${this.name} is ready`
          : `External service ${this.name} is not ready`,
        timestamp: Date.now(),
        details: {
          endpoint: this.healthEndpoint,
          status: response.status,
          required: this.required
        }
      };
    } catch (error: any) {
      return {
        status: this.required ? HealthStatus.UNHEALTHY : HealthStatus.DEGRADED,
        message: `External service ${this.name} check failed: ${error.message}`,
        timestamp: Date.now(),
        details: {
          endpoint: this.healthEndpoint,
          error: error.message,
          required: this.required
        }
      };
    }
  }
}

/**
 * Startup readiness check
 * Ensures application startup tasks are complete
 */
export class StartupReadinessCheck implements HealthCheck {
  public readonly name = 'startup';
  private tasks: Map<string, boolean> = new Map();
  private requiredTasks: Set<string> = new Set();

  /**
   * Add required startup task
   */
  public addTask(name: string, required: boolean = true): void {
    this.tasks.set(name, false);

    if (required) {
      this.requiredTasks.add(name);
    }
  }

  /**
   * Mark task as complete
   */
  public completeTask(name: string): void {
    this.tasks.set(name, true);
  }

  /**
   * Check if all required tasks are complete
   */
  public async check(): Promise<HealthCheckResult> {
    const taskStatus: Record<string, boolean> = {};
    let allRequiredComplete = true;

    for (const [name, complete] of this.tasks.entries()) {
      taskStatus[name] = complete;

      if (this.requiredTasks.has(name) && !complete) {
        allRequiredComplete = false;
      }
    }

    const completedCount = Array.from(this.tasks.values()).filter(Boolean).length;
    const totalCount = this.tasks.size;

    return {
      status: allRequiredComplete ? HealthStatus.HEALTHY : HealthStatus.UNHEALTHY,
      message: allRequiredComplete
        ? 'All startup tasks complete'
        : 'Startup tasks incomplete',
      timestamp: Date.now(),
      details: {
        completed: completedCount,
        total: totalCount,
        tasks: taskStatus
      }
    };
  }
}
