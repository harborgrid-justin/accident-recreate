/**
 * AccuScene Enterprise v0.2.0
 * Alert Thresholds
 *
 * Pre-defined alert thresholds for common metrics
 */

import { AlertRule, AlertSeverity } from '../types';
import { AlertRuleBuilder } from './rules';

/**
 * System performance thresholds
 */
export const PERFORMANCE_THRESHOLDS = {
  CPU_HIGH: 80,
  CPU_CRITICAL: 95,
  MEMORY_HIGH: 80,
  MEMORY_CRITICAL: 90,
  DISK_HIGH: 80,
  DISK_CRITICAL: 90,
  RESPONSE_TIME_WARNING: 1000,
  RESPONSE_TIME_CRITICAL: 3000,
  ERROR_RATE_WARNING: 0.01,
  ERROR_RATE_CRITICAL: 0.05
};

/**
 * Create default alert rules
 */
export function createDefaultAlertRules(): AlertRule[] {
  return [
    // CPU alerts
    new AlertRuleBuilder()
      .id('cpu-usage-high')
      .name('High CPU Usage')
      .description('CPU usage is above 80%')
      .severity(AlertSeverity.WARNING)
      .metric('system_cpu_usage_percent')
      .operator('gt')
      .threshold(PERFORMANCE_THRESHOLDS.CPU_HIGH)
      .duration(60000) // 1 minute
      .annotations({
        summary: 'CPU usage is high',
        description: 'CPU usage has been above 80% for more than 1 minute'
      })
      .build(),

    new AlertRuleBuilder()
      .id('cpu-usage-critical')
      .name('Critical CPU Usage')
      .description('CPU usage is above 95%')
      .severity(AlertSeverity.CRITICAL)
      .metric('system_cpu_usage_percent')
      .operator('gt')
      .threshold(PERFORMANCE_THRESHOLDS.CPU_CRITICAL)
      .duration(30000) // 30 seconds
      .annotations({
        summary: 'CPU usage is critical',
        description: 'CPU usage has been above 95% for more than 30 seconds'
      })
      .build(),

    // Memory alerts
    new AlertRuleBuilder()
      .id('memory-usage-high')
      .name('High Memory Usage')
      .description('Memory usage is above 80%')
      .severity(AlertSeverity.WARNING)
      .metric('system_memory_usage_percent')
      .operator('gt')
      .threshold(PERFORMANCE_THRESHOLDS.MEMORY_HIGH)
      .duration(60000)
      .annotations({
        summary: 'Memory usage is high',
        description: 'Memory usage has been above 80% for more than 1 minute'
      })
      .build(),

    new AlertRuleBuilder()
      .id('memory-usage-critical')
      .name('Critical Memory Usage')
      .description('Memory usage is above 90%')
      .severity(AlertSeverity.CRITICAL)
      .metric('system_memory_usage_percent')
      .operator('gt')
      .threshold(PERFORMANCE_THRESHOLDS.MEMORY_CRITICAL)
      .duration(30000)
      .annotations({
        summary: 'Memory usage is critical',
        description: 'Memory usage has been above 90% for more than 30 seconds'
      })
      .build(),

    // Response time alerts
    new AlertRuleBuilder()
      .id('response-time-high')
      .name('High Response Time')
      .description('Average response time is above 1 second')
      .severity(AlertSeverity.WARNING)
      .metric('http_request_duration_seconds_p95')
      .operator('gt')
      .threshold(PERFORMANCE_THRESHOLDS.RESPONSE_TIME_WARNING / 1000)
      .duration(120000) // 2 minutes
      .annotations({
        summary: 'Response time is high',
        description: 'P95 response time has been above 1s for more than 2 minutes'
      })
      .build(),

    new AlertRuleBuilder()
      .id('response-time-critical')
      .name('Critical Response Time')
      .description('Average response time is above 3 seconds')
      .severity(AlertSeverity.CRITICAL)
      .metric('http_request_duration_seconds_p95')
      .operator('gt')
      .threshold(PERFORMANCE_THRESHOLDS.RESPONSE_TIME_CRITICAL / 1000)
      .duration(60000) // 1 minute
      .annotations({
        summary: 'Response time is critical',
        description: 'P95 response time has been above 3s for more than 1 minute'
      })
      .build(),

    // Error rate alerts
    new AlertRuleBuilder()
      .id('error-rate-high')
      .name('High Error Rate')
      .description('Error rate is above 1%')
      .severity(AlertSeverity.WARNING)
      .metric('http_requests_error_rate')
      .operator('gt')
      .threshold(PERFORMANCE_THRESHOLDS.ERROR_RATE_WARNING)
      .duration(60000)
      .annotations({
        summary: 'Error rate is high',
        description: 'Error rate has been above 1% for more than 1 minute'
      })
      .build(),

    new AlertRuleBuilder()
      .id('error-rate-critical')
      .name('Critical Error Rate')
      .description('Error rate is above 5%')
      .severity(AlertSeverity.CRITICAL)
      .metric('http_requests_error_rate')
      .operator('gt')
      .threshold(PERFORMANCE_THRESHOLDS.ERROR_RATE_CRITICAL)
      .duration(30000)
      .annotations({
        summary: 'Error rate is critical',
        description: 'Error rate has been above 5% for more than 30 seconds'
      })
      .build(),

    // Service health alerts
    new AlertRuleBuilder()
      .id('service-down')
      .name('Service Down')
      .description('Service is not responding')
      .severity(AlertSeverity.CRITICAL)
      .metric('service_up')
      .operator('eq')
      .threshold(0)
      .duration(0) // Immediate
      .annotations({
        summary: 'Service is down',
        description: 'Service health check is failing'
      })
      .build(),

    // Database alerts
    new AlertRuleBuilder()
      .id('database-connections-high')
      .name('High Database Connections')
      .description('Database connection pool is nearly exhausted')
      .severity(AlertSeverity.WARNING)
      .metric('database_connections_active')
      .operator('gt')
      .threshold(80) // 80% of pool
      .duration(60000)
      .annotations({
        summary: 'Database connection pool is high',
        description: 'Active database connections exceed 80% of pool size'
      })
      .build()
  ];
}

/**
 * Threshold validator
 */
export class ThresholdValidator {
  /**
   * Validate threshold value is within acceptable range
   */
  public static validate(
    metric: string,
    threshold: number,
    operator: string
  ): { valid: boolean; message?: string } {
    // Percentage metrics should be 0-100
    if (metric.includes('_percent')) {
      if (threshold < 0 || threshold > 100) {
        return {
          valid: false,
          message: 'Percentage thresholds must be between 0 and 100'
        };
      }
    }

    // Rate metrics should be 0-1
    if (metric.includes('_rate')) {
      if (threshold < 0 || threshold > 1) {
        return {
          valid: false,
          message: 'Rate thresholds must be between 0 and 1'
        };
      }
    }

    // Duration metrics should be positive
    if (metric.includes('_duration') || metric.includes('_seconds')) {
      if (threshold < 0) {
        return {
          valid: false,
          message: 'Duration thresholds must be positive'
        };
      }
    }

    return { valid: true };
  }

  /**
   * Suggest threshold based on metric type
   */
  public static suggest(metric: string): number | undefined {
    if (metric.includes('cpu_usage_percent')) {
      return PERFORMANCE_THRESHOLDS.CPU_HIGH;
    }

    if (metric.includes('memory_usage_percent')) {
      return PERFORMANCE_THRESHOLDS.MEMORY_HIGH;
    }

    if (metric.includes('disk_usage_percent')) {
      return PERFORMANCE_THRESHOLDS.DISK_HIGH;
    }

    if (metric.includes('response_time') || metric.includes('duration_seconds')) {
      return PERFORMANCE_THRESHOLDS.RESPONSE_TIME_WARNING / 1000;
    }

    if (metric.includes('error_rate')) {
      return PERFORMANCE_THRESHOLDS.ERROR_RATE_WARNING;
    }

    return undefined;
  }
}

/**
 * Dynamic threshold calculator
 */
export class DynamicThresholdCalculator {
  /**
   * Calculate threshold based on historical data
   */
  public static calculate(
    values: number[],
    method: 'stddev' | 'percentile' = 'stddev',
    factor: number = 3
  ): number {
    if (values.length === 0) {
      return 0;
    }

    if (method === 'stddev') {
      return this.calculateStdDevThreshold(values, factor);
    } else {
      return this.calculatePercentileThreshold(values, factor);
    }
  }

  /**
   * Calculate threshold using standard deviation
   */
  private static calculateStdDevThreshold(values: number[], factor: number): number {
    const mean = values.reduce((a, b) => a + b, 0) / values.length;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
    const stddev = Math.sqrt(variance);

    return mean + factor * stddev;
  }

  /**
   * Calculate threshold using percentile
   */
  private static calculatePercentileThreshold(values: number[], percentile: number): number {
    const sorted = [...values].sort((a, b) => a - b);
    const index = Math.ceil(sorted.length * (percentile / 100)) - 1;

    return sorted[Math.max(0, index)];
  }
}
