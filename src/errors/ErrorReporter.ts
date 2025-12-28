/**
 * Error reporting service for logging and reporting errors
 */

import { AccuSceneError, ErrorCode, ErrorSeverity } from './types';

export interface ErrorReport {
  id: string;
  code: string;
  severity: string;
  message: string;
  details?: string;
  context: Array<string>;
  timestamp: string;
  location?: string;
  metadata: Record<string, string>;
  suggestedActions: string[];
  recoverable: boolean;
}

/**
 * Error reporter for formatting and reporting errors
 */
export class ErrorReporter {
  private static instance: ErrorReporter;
  private errorLog: AccuSceneError[] = [];
  private maxLogSize = 100;

  private constructor() {
    // Set up global error handlers
    if (typeof window !== 'undefined') {
      window.addEventListener('error', this.handleGlobalError);
      window.addEventListener('unhandledrejection', this.handleUnhandledRejection);
    }
  }

  static getInstance(): ErrorReporter {
    if (!ErrorReporter.instance) {
      ErrorReporter.instance = new ErrorReporter();
    }
    return ErrorReporter.instance;
  }

  private handleGlobalError = (event: ErrorEvent): void => {
    const error = new AccuSceneError({
      code: ErrorCode.INTERNAL,
      message: event.message,
      details: `${event.filename}:${event.lineno}:${event.colno}`,
      location: `${event.filename}:${event.lineno}`,
    });

    this.reportError(error);
  };

  private handleUnhandledRejection = (event: PromiseRejectionEvent): void => {
    const error = new AccuSceneError({
      code: ErrorCode.INTERNAL,
      message: String(event.reason),
    }).withContext('Unhandled promise rejection');

    this.reportError(error);
  };

  /**
   * Reports an error to the logging system
   */
  static reportError(error: AccuSceneError, context?: Record<string, unknown>): void {
    const reporter = ErrorReporter.getInstance();

    // Add to error log
    reporter.errorLog.push(error);
    if (reporter.errorLog.length > reporter.maxLogSize) {
      reporter.errorLog.shift();
    }

    // Log to console
    if (error.severity === ErrorSeverity.CRITICAL || error.severity === ErrorSeverity.HIGH) {
      console.error('[AccuScene Error]', error.toString(), { error, context });
    } else if (error.severity === ErrorSeverity.MEDIUM) {
      console.warn('[AccuScene Error]', error.toString(), { error, context });
    } else {
      console.log('[AccuScene Error]', error.toString(), { error, context });
    }

    // Send to external logging service (implement based on your setup)
    reporter.sendToExternalService(error, context);
  }

  private sendToExternalService(error: AccuSceneError, context?: Record<string, unknown>): void {
    // TODO: Implement integration with logging service (e.g., Sentry, LogRocket, etc.)
    // This is a placeholder for external error reporting

    // Example implementation:
    /*
    if (window.Sentry) {
      Sentry.captureException(error, {
        contexts: {
          accuscene: {
            errorId: error.id,
            errorCode: error.code,
            severity: error.severity,
            ...context,
          },
        },
      });
    }
    */
  }

  /**
   * Creates an error report
   */
  static createReport(error: AccuSceneError): ErrorReport {
    const contextChain: string[] = [];
    let currentContext = error.context;
    while (currentContext) {
      contextChain.push(currentContext.message);
      currentContext = currentContext.parent;
    }

    return {
      id: error.id,
      code: error.code,
      severity: error.severity,
      message: error.message,
      details: error.details,
      context: contextChain,
      timestamp: error.timestamp.toISOString(),
      location: error.location,
      metadata: Object.fromEntries(error.metadata),
      suggestedActions: ErrorReporter.suggestActions(error),
      recoverable: error.recoverable,
    };
  }

  /**
   * Suggests actions for resolving the error
   */
  static suggestActions(error: AccuSceneError): string[] {
    const actions: string[] = [];

    switch (error.code) {
      case ErrorCode.VALIDATION:
        actions.push('Check input parameters and ensure they meet validation requirements');
        actions.push('Review API documentation for correct request format');
        break;

      case ErrorCode.AUTHENTICATION:
        actions.push('Verify authentication credentials are correct');
        actions.push('Check if authentication token has expired');
        actions.push('Ensure proper authentication headers are included');
        break;

      case ErrorCode.AUTHORIZATION:
        actions.push('Verify you have the necessary permissions for this operation');
        actions.push('Contact your administrator to request access');
        break;

      case ErrorCode.NOT_FOUND:
        actions.push('Verify the resource ID or path is correct');
        actions.push('Check if the resource has been deleted');
        break;

      case ErrorCode.NETWORK:
        actions.push('Check your network connection');
        actions.push('Verify the service endpoint is accessible');
        actions.push('Retry the operation');
        break;

      case ErrorCode.DATABASE:
        actions.push('Check database connection settings');
        actions.push('Verify database is running and accessible');
        actions.push('Review database logs for more details');
        break;

      case ErrorCode.TIMEOUT:
        actions.push('Retry the operation');
        actions.push('Increase timeout duration if possible');
        actions.push('Check system load and performance');
        break;

      case ErrorCode.RATE_LIMIT:
        actions.push('Wait before retrying the operation');
        actions.push('Implement exponential backoff');
        actions.push('Review rate limit quotas');
        break;

      case ErrorCode.INTERNAL:
        actions.push('Report this error to the development team');
        actions.push('Include the error ID in your report');
        actions.push('Check application logs for more details');
        break;

      default:
        actions.push('Review error details and context');
        actions.push('Check application logs for more information');
        break;
    }

    return actions;
  }

  /**
   * Formats error for display to users
   */
  static formatForUser(error: AccuSceneError): string {
    const report = ErrorReporter.createReport(error);

    let output = '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n';
    output += `Error [${report.id}] - ${report.code} - ${report.severity}\n`;
    output += '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n';
    output += `Message: ${report.message}\n`;

    if (report.details) {
      output += `\nDetails:\n  ${report.details}\n`;
    }

    if (report.context.length > 0) {
      output += '\nContext:\n';
      report.context.forEach((ctx, i) => {
        output += `  ${i + 1}. ${ctx}\n`;
      });
    }

    if (report.location) {
      output += `\nLocation: ${report.location}\n`;
    }

    if (Object.keys(report.metadata).length > 0) {
      output += '\nMetadata:\n';
      Object.entries(report.metadata).forEach(([key, value]) => {
        output += `  ${key}: ${value}\n`;
      });
    }

    output += `\nTimestamp: ${report.timestamp}\n`;

    if (report.suggestedActions.length > 0) {
      output += '\nSuggested Actions:\n';
      report.suggestedActions.forEach((action, i) => {
        output += `  ${i + 1}. ${action}\n`;
      });
    }

    output += `\nRecoverable: ${report.recoverable ? 'Yes' : 'No'}\n`;
    output += '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n';

    return output;
  }

  /**
   * Formats error for API response
   */
  static formatForAPI(error: AccuSceneError): Record<string, unknown> {
    return {
      error: {
        id: error.id,
        code: error.code,
        message: error.message,
        details: error.details,
        severity: error.severity,
        timestamp: error.timestamp.toISOString(),
        recoverable: error.recoverable,
      },
    };
  }

  /**
   * Reports error to support (placeholder for actual implementation)
   */
  static reportToSupport(error: AccuSceneError): void {
    const report = ErrorReporter.createReport(error);

    // TODO: Implement actual support reporting (e.g., send to support API)
    console.log('Reporting to support:', report);

    // Example implementation:
    /*
    fetch('/api/support/report-error', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(report),
    });
    */
  }

  /**
   * Gets recent errors from the log
   */
  static getRecentErrors(count = 10): AccuSceneError[] {
    const reporter = ErrorReporter.getInstance();
    return reporter.errorLog.slice(-count);
  }

  /**
   * Clears the error log
   */
  static clearErrorLog(): void {
    const reporter = ErrorReporter.getInstance();
    reporter.errorLog = [];
  }

  /**
   * Gets error statistics
   */
  static getErrorStats(): {
    total: number;
    bySeverity: Record<ErrorSeverity, number>;
    byCode: Record<string, number>;
  } {
    const reporter = ErrorReporter.getInstance();
    const stats = {
      total: reporter.errorLog.length,
      bySeverity: {} as Record<ErrorSeverity, number>,
      byCode: {} as Record<string, number>,
    };

    reporter.errorLog.forEach((error) => {
      // Count by severity
      stats.bySeverity[error.severity] = (stats.bySeverity[error.severity] || 0) + 1;

      // Count by code
      stats.byCode[error.code] = (stats.byCode[error.code] || 0) + 1;
    });

    return stats;
  }
}

// Initialize singleton
ErrorReporter.getInstance();
