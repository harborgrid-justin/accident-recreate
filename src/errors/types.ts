/**
 * Error type definitions for AccuScene Enterprise
 * Comprehensive error handling with categorization, severity, and recovery strategies
 */

/**
 * Error codes matching the Rust backend
 */
export enum ErrorCode {
  // Client Errors
  VALIDATION = 'VALIDATION',
  AUTHENTICATION = 'AUTHENTICATION',
  AUTHORIZATION = 'AUTHORIZATION',
  NOT_FOUND = 'NOT_FOUND',
  CONFLICT = 'CONFLICT',
  PAYLOAD_TOO_LARGE = 'PAYLOAD_TOO_LARGE',
  RATE_LIMIT = 'RATE_LIMIT',
  INVALID_STATE = 'INVALID_STATE',

  // Server Errors
  INTERNAL = 'INTERNAL',
  DATABASE = 'DATABASE',
  NETWORK = 'NETWORK',
  EXTERNAL_SERVICE = 'EXTERNAL_SERVICE',
  TIMEOUT = 'TIMEOUT',
  UNAVAILABLE = 'UNAVAILABLE',

  // Domain-Specific Errors
  PHYSICS = 'PHYSICS',
  RENDERING = 'RENDERING',
  FILE_SYSTEM = 'FILE_SYSTEM',
  CACHE = 'CACHE',
  JOB = 'JOB',
  STREAMING = 'STREAMING',
  COMPRESSION = 'COMPRESSION',
  CRYPTO = 'CRYPTO',
  CLUSTER = 'CLUSTER',
  ANALYTICS = 'ANALYTICS',
  MACHINE_LEARNING = 'MACHINE_LEARNING',
  SEARCH = 'SEARCH',
  NOTIFICATION = 'NOTIFICATION',
  SSO = 'SSO',
  GESTURE = 'GESTURE',
  OFFLINE = 'OFFLINE',
  TRANSFER = 'TRANSFER',
  PREFERENCES = 'PREFERENCES',
  ACCESSIBILITY = 'ACCESSIBILITY',
  DASHBOARD = 'DASHBOARD',

  // System Errors
  CONFIGURATION = 'CONFIGURATION',
  RESOURCE_EXHAUSTED = 'RESOURCE_EXHAUSTED',
  UNIMPLEMENTED = 'UNIMPLEMENTED',
  DEPRECATED = 'DEPRECATED',
}

/**
 * Error severity levels
 */
export enum ErrorSeverity {
  INFO = 'INFO',
  WARNING = 'WARNING',
  LOW = 'LOW',
  MEDIUM = 'MEDIUM',
  HIGH = 'HIGH',
  CRITICAL = 'CRITICAL',
}

/**
 * Recovery actions for errors
 */
export enum RecoveryAction {
  RETRY = 'RETRY',
  FALLBACK = 'FALLBACK',
  SKIP = 'SKIP',
  ABORT = 'ABORT',
  USER_INTERVENTION = 'USER_INTERVENTION',
  USE_CACHE = 'USE_CACHE',
  DEGRADE = 'DEGRADE',
}

/**
 * Error context for chaining
 */
export interface ErrorContext {
  message: string;
  parent?: ErrorContext;
  data?: Record<string, unknown>;
  timestamp: string;
}

/**
 * Main error class for AccuScene Enterprise
 */
export class AccuSceneError extends Error {
  public readonly id: string;
  public readonly code: ErrorCode;
  public readonly severity: ErrorSeverity;
  public readonly details?: string;
  public readonly context?: ErrorContext;
  public readonly timestamp: Date;
  public readonly location?: string;
  public readonly metadata: Map<string, string>;
  public readonly recoverable: boolean;
  public readonly httpStatus: number;

  constructor(options: {
    code: ErrorCode;
    message: string;
    details?: string;
    context?: ErrorContext;
    location?: string;
    metadata?: Record<string, string>;
    severity?: ErrorSeverity;
    recoverable?: boolean;
  }) {
    super(options.message);

    this.name = 'AccuSceneError';
    this.id = this.generateId();
    this.code = options.code;
    this.severity = options.severity ?? this.getDefaultSeverity(options.code);
    this.details = options.details;
    this.context = options.context;
    this.timestamp = new Date();
    this.location = options.location;
    this.metadata = new Map(Object.entries(options.metadata ?? {}));
    this.recoverable = options.recoverable ?? this.isRecoverable(options.code);
    this.httpStatus = this.getHttpStatus(options.code);

    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, AccuSceneError);
    }
  }

  // Factory methods for common error types
  static validation(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.VALIDATION, message, details });
  }

  static authentication(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.AUTHENTICATION, message, details });
  }

  static authorization(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.AUTHORIZATION, message, details });
  }

  static notFound(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.NOT_FOUND, message, details });
  }

  static conflict(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.CONFLICT, message, details });
  }

  static internal(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.INTERNAL, message, details });
  }

  static database(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.DATABASE, message, details });
  }

  static network(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.NETWORK, message, details });
  }

  static timeout(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.TIMEOUT, message, details });
  }

  static rateLimit(message: string, details?: string): AccuSceneError {
    return new AccuSceneError({ code: ErrorCode.RATE_LIMIT, message, details });
  }

  // Chainable methods
  withDetails(details: string): AccuSceneError {
    return new AccuSceneError({
      code: this.code,
      message: this.message,
      details,
      context: this.context,
      location: this.location,
      metadata: Object.fromEntries(this.metadata),
      severity: this.severity,
      recoverable: this.recoverable,
    });
  }

  withContext(message: string, data?: Record<string, unknown>): AccuSceneError {
    const newContext: ErrorContext = {
      message,
      parent: this.context,
      data,
      timestamp: new Date().toISOString(),
    };

    return new AccuSceneError({
      code: this.code,
      message: this.message,
      details: this.details,
      context: newContext,
      location: this.location,
      metadata: Object.fromEntries(this.metadata),
      severity: this.severity,
      recoverable: this.recoverable,
    });
  }

  withMetadata(key: string, value: string): AccuSceneError {
    const metadata = Object.fromEntries(this.metadata);
    metadata[key] = value;

    return new AccuSceneError({
      code: this.code,
      message: this.message,
      details: this.details,
      context: this.context,
      location: this.location,
      metadata,
      severity: this.severity,
      recoverable: this.recoverable,
    });
  }

  // Utility methods
  private generateId(): string {
    return `err_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private getDefaultSeverity(code: ErrorCode): ErrorSeverity {
    const severityMap: Record<ErrorCode, ErrorSeverity> = {
      [ErrorCode.INTERNAL]: ErrorSeverity.CRITICAL,
      [ErrorCode.DATABASE]: ErrorSeverity.CRITICAL,
      [ErrorCode.RESOURCE_EXHAUSTED]: ErrorSeverity.CRITICAL,

      [ErrorCode.AUTHENTICATION]: ErrorSeverity.HIGH,
      [ErrorCode.AUTHORIZATION]: ErrorSeverity.HIGH,
      [ErrorCode.EXTERNAL_SERVICE]: ErrorSeverity.HIGH,
      [ErrorCode.CLUSTER]: ErrorSeverity.HIGH,
      [ErrorCode.CRYPTO]: ErrorSeverity.HIGH,

      [ErrorCode.NETWORK]: ErrorSeverity.MEDIUM,
      [ErrorCode.TIMEOUT]: ErrorSeverity.MEDIUM,
      [ErrorCode.UNAVAILABLE]: ErrorSeverity.MEDIUM,
      [ErrorCode.FILE_SYSTEM]: ErrorSeverity.MEDIUM,
      [ErrorCode.CONFIGURATION]: ErrorSeverity.MEDIUM,

      [ErrorCode.VALIDATION]: ErrorSeverity.LOW,
      [ErrorCode.NOT_FOUND]: ErrorSeverity.LOW,
      [ErrorCode.CONFLICT]: ErrorSeverity.LOW,
      [ErrorCode.INVALID_STATE]: ErrorSeverity.LOW,
      [ErrorCode.RATE_LIMIT]: ErrorSeverity.LOW,

      [ErrorCode.DEPRECATED]: ErrorSeverity.WARNING,
      [ErrorCode.UNIMPLEMENTED]: ErrorSeverity.WARNING,

      // Domain-specific defaults
      [ErrorCode.PHYSICS]: ErrorSeverity.MEDIUM,
      [ErrorCode.RENDERING]: ErrorSeverity.MEDIUM,
      [ErrorCode.CACHE]: ErrorSeverity.MEDIUM,
      [ErrorCode.JOB]: ErrorSeverity.MEDIUM,
      [ErrorCode.STREAMING]: ErrorSeverity.MEDIUM,
      [ErrorCode.COMPRESSION]: ErrorSeverity.MEDIUM,
      [ErrorCode.ANALYTICS]: ErrorSeverity.MEDIUM,
      [ErrorCode.MACHINE_LEARNING]: ErrorSeverity.MEDIUM,
      [ErrorCode.SEARCH]: ErrorSeverity.MEDIUM,
      [ErrorCode.NOTIFICATION]: ErrorSeverity.MEDIUM,
      [ErrorCode.SSO]: ErrorSeverity.MEDIUM,
      [ErrorCode.GESTURE]: ErrorSeverity.MEDIUM,
      [ErrorCode.OFFLINE]: ErrorSeverity.MEDIUM,
      [ErrorCode.TRANSFER]: ErrorSeverity.MEDIUM,
      [ErrorCode.PREFERENCES]: ErrorSeverity.MEDIUM,
      [ErrorCode.ACCESSIBILITY]: ErrorSeverity.MEDIUM,
      [ErrorCode.DASHBOARD]: ErrorSeverity.MEDIUM,
      [ErrorCode.PAYLOAD_TOO_LARGE]: ErrorSeverity.MEDIUM,
    };

    return severityMap[code] ?? ErrorSeverity.MEDIUM;
  }

  private isRecoverable(code: ErrorCode): boolean {
    const nonRecoverable = [
      ErrorCode.INTERNAL,
      ErrorCode.UNIMPLEMENTED,
      ErrorCode.CONFIGURATION,
      ErrorCode.RESOURCE_EXHAUSTED,
    ];

    return !nonRecoverable.includes(code);
  }

  private getHttpStatus(code: ErrorCode): number {
    const statusMap: Partial<Record<ErrorCode, number>> = {
      [ErrorCode.VALIDATION]: 400,
      [ErrorCode.INVALID_STATE]: 400,
      [ErrorCode.AUTHENTICATION]: 401,
      [ErrorCode.AUTHORIZATION]: 403,
      [ErrorCode.NOT_FOUND]: 404,
      [ErrorCode.CONFLICT]: 409,
      [ErrorCode.PAYLOAD_TOO_LARGE]: 413,
      [ErrorCode.RATE_LIMIT]: 429,
      [ErrorCode.UNIMPLEMENTED]: 501,
      [ErrorCode.UNAVAILABLE]: 503,
      [ErrorCode.TIMEOUT]: 504,
    };

    return statusMap[code] ?? 500;
  }

  /**
   * Converts error to a plain object for serialization
   */
  toJSON() {
    return {
      id: this.id,
      code: this.code,
      severity: this.severity,
      message: this.message,
      details: this.details,
      context: this.context,
      timestamp: this.timestamp.toISOString(),
      location: this.location,
      metadata: Object.fromEntries(this.metadata),
      recoverable: this.recoverable,
      httpStatus: this.httpStatus,
    };
  }

  /**
   * Returns a formatted error report
   */
  toString(): string {
    return `[${this.code}] ${this.severity}: ${this.message}${
      this.details ? `\nDetails: ${this.details}` : ''
    }${this.context ? `\nContext: ${this.context.message}` : ''}`;
  }
}

/**
 * Type guard to check if an error is an AccuSceneError
 */
export function isAccuSceneError(error: unknown): error is AccuSceneError {
  return error instanceof AccuSceneError;
}

/**
 * Converts unknown errors to AccuSceneError
 */
export function toAccuSceneError(error: unknown): AccuSceneError {
  if (isAccuSceneError(error)) {
    return error;
  }

  if (error instanceof Error) {
    return new AccuSceneError({
      code: ErrorCode.INTERNAL,
      message: error.message,
      details: error.stack,
    });
  }

  return new AccuSceneError({
    code: ErrorCode.INTERNAL,
    message: String(error),
  });
}
