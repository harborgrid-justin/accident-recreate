/**
 * AccuScene Enterprise v0.2.0
 * Liveness Probe
 *
 * Kubernetes-style liveness probe for detecting if application is alive
 */

import { HealthCheck, HealthCheckResult, HealthStatus } from '../types';

/**
 * Liveness probe checks if the application is running
 * If liveness fails, the application should be restarted
 */
export class LivenessProbe implements HealthCheck {
  public readonly name = 'liveness';
  private lastHeartbeat: number = Date.now();
  private heartbeatInterval: number = 5000; // 5 seconds
  private heartbeatTimeout: number = 30000; // 30 seconds
  private intervalId?: NodeJS.Timeout;

  constructor(heartbeatInterval?: number, heartbeatTimeout?: number) {
    if (heartbeatInterval) {
      this.heartbeatInterval = heartbeatInterval;
    }

    if (heartbeatTimeout) {
      this.heartbeatTimeout = heartbeatTimeout;
    }

    this.start();
  }

  /**
   * Start heartbeat monitoring
   */
  public start(): void {
    if (this.intervalId) {
      return;
    }

    this.lastHeartbeat = Date.now();

    this.intervalId = setInterval(() => {
      this.heartbeat();
    }, this.heartbeatInterval);
  }

  /**
   * Stop heartbeat monitoring
   */
  public stop(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = undefined;
    }
  }

  /**
   * Record heartbeat
   */
  private heartbeat(): void {
    this.lastHeartbeat = Date.now();
  }

  /**
   * Check liveness
   */
  public async check(): Promise<HealthCheckResult> {
    const now = Date.now();
    const timeSinceHeartbeat = now - this.lastHeartbeat;

    const isAlive = timeSinceHeartbeat < this.heartbeatTimeout;

    return {
      status: isAlive ? HealthStatus.HEALTHY : HealthStatus.UNHEALTHY,
      message: isAlive
        ? 'Application is alive'
        : 'Application appears to be frozen',
      timestamp: now,
      details: {
        lastHeartbeat: this.lastHeartbeat,
        timeSinceHeartbeat,
        heartbeatTimeout: this.heartbeatTimeout
      }
    };
  }

  /**
   * Force heartbeat
   */
  public forceHeartbeat(): void {
    this.heartbeat();
  }
}

/**
 * Event loop liveness check
 * Detects if the event loop is blocked
 */
export class EventLoopLivenessCheck implements HealthCheck {
  public readonly name = 'event_loop';
  private maxBlockTime: number = 100; // milliseconds
  private lastCheckTime: number = Date.now();
  private isBlocked: boolean = false;
  private blockDuration: number = 0;

  constructor(maxBlockTime?: number) {
    if (maxBlockTime) {
      this.maxBlockTime = maxBlockTime;
    }

    this.startMonitoring();
  }

  /**
   * Start monitoring event loop
   */
  private startMonitoring(): void {
    const checkLoop = () => {
      const now = Date.now();
      const delta = now - this.lastCheckTime;

      if (delta > this.maxBlockTime) {
        this.isBlocked = true;
        this.blockDuration = delta;
      } else {
        this.isBlocked = false;
        this.blockDuration = 0;
      }

      this.lastCheckTime = now;
      setImmediate(checkLoop);
    };

    setImmediate(checkLoop);
  }

  /**
   * Check event loop health
   */
  public async check(): Promise<HealthCheckResult> {
    const status = this.isBlocked ? HealthStatus.DEGRADED : HealthStatus.HEALTHY;

    return {
      status,
      message: this.isBlocked
        ? `Event loop blocked for ${this.blockDuration}ms`
        : 'Event loop is responsive',
      timestamp: Date.now(),
      details: {
        isBlocked: this.isBlocked,
        blockDuration: this.blockDuration,
        maxBlockTime: this.maxBlockTime
      }
    };
  }
}

/**
 * Process liveness check
 * Checks if critical processes are running
 */
export class ProcessLivenessCheck implements HealthCheck {
  public readonly name = 'process';

  public async check(): Promise<HealthCheckResult> {
    if (typeof process === 'undefined') {
      return {
        status: HealthStatus.HEALTHY,
        message: 'Process monitoring not available',
        timestamp: Date.now()
      };
    }

    const uptime = process.uptime();
    const memoryUsage = process.memoryUsage();

    return {
      status: HealthStatus.HEALTHY,
      message: 'Process is running',
      timestamp: Date.now(),
      details: {
        uptime,
        pid: process.pid,
        platform: process.platform,
        nodeVersion: process.version,
        memoryUsage: {
          heapUsed: memoryUsage.heapUsed,
          heapTotal: memoryUsage.heapTotal,
          rss: memoryUsage.rss
        }
      }
    };
  }
}
