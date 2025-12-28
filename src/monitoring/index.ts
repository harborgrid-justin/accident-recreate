/**
 * AccuScene Enterprise v0.2.0
 * Performance Monitoring System - Main Exports
 *
 * Comprehensive APM and observability platform
 */

// Type definitions
export * from './types';

// Core monitoring
export * from './core';

// Metrics
export * from './metrics';

// Distributed tracing
export * from './tracing';

// Profiling
export * from './profiling';

// Health checks
export * from './health';

// Alerting
export * from './alerting';

// Integrations
export * from './integrations';

// Dashboard components
export { MonitoringDashboard } from './dashboard';
export { Overview } from './dashboard/Overview';
export { Metrics as MetricsPanel } from './dashboard/Metrics';
export { Traces as TracesPanel } from './dashboard/Traces';
export { Alerts as AlertsPanel } from './dashboard/Alerts';
export { Performance as PerformancePanel } from './dashboard/Performance';

// Convenience exports
import { globalCollector } from './core/collector';
import { globalAggregator } from './core/aggregator';
import { globalHealthRegistry } from './health/checks';

/**
 * Initialize monitoring system with default configuration
 */
export function initializeMonitoring(config?: {
  serviceName?: string;
  environment?: string;
  collectInterval?: number;
  enableProfiling?: boolean;
}): void {
  const serviceName = config?.serviceName || 'accuscene-enterprise';
  const environment = config?.environment || 'production';
  const enableProfiling = config?.enableProfiling !== false;

  console.log(`[Monitoring] Initializing for ${serviceName} (${environment})`);

  // Start collection
  globalCollector.startCollection();

  // Start profiling if enabled
  if (enableProfiling) {
    const renderProfiler = globalCollector.getRenderProfiler();
    const networkProfiler = globalCollector.getNetworkProfiler();

    renderProfiler.start();
    networkProfiler.start();
  }

  // Register default health checks
  // (Additional checks can be registered by the application)

  console.log('[Monitoring] Initialization complete');
}

/**
 * Shutdown monitoring system
 */
export function shutdownMonitoring(): void {
  console.log('[Monitoring] Shutting down...');

  globalCollector.stopCollection();

  const renderProfiler = globalCollector.getRenderProfiler();
  const networkProfiler = globalCollector.getNetworkProfiler();

  renderProfiler.stop();
  networkProfiler.stop();

  console.log('[Monitoring] Shutdown complete');
}

/**
 * Get monitoring instance for manual instrumentation
 */
export const monitoring = {
  collector: globalCollector,
  aggregator: globalAggregator,
  health: globalHealthRegistry
};
