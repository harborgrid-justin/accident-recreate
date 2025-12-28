/**
 * AccuScene Enterprise Accident Recreation Platform
 * Standardized Error Classes
 *
 * This file provides standardized error handling across the application.
 * All modules should use these error classes for consistent error reporting.
 */

// ============================================================================
// BASE ERROR CLASS
// ============================================================================

export class ApplicationError extends Error {
  public code: string;
  public statusCode: number;
  public readonly isOperational: boolean;
  public readonly context?: Record<string, any>;
  public readonly timestamp: Date;

  constructor(
    message: string,
    code: string,
    statusCode: number = 500,
    isOperational: boolean = true,
    context?: Record<string, any>
  ) {
    super(message);

    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }

    this.name = this.constructor.name;
    this.code = code;
    this.statusCode = statusCode;
    this.isOperational = isOperational;
    this.context = context;
    this.timestamp = new Date();

    // Set the prototype explicitly to ensure instanceof works correctly
    Object.setPrototypeOf(this, ApplicationError.prototype);
  }

  toJSON() {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      statusCode: this.statusCode,
      context: this.context,
      timestamp: this.timestamp,
      stack: this.stack,
    };
  }
}

// ============================================================================
// AUTHENTICATION & AUTHORIZATION ERRORS
// ============================================================================

export class AuthenticationError extends ApplicationError {
  constructor(message: string = 'Authentication failed', context?: Record<string, any>) {
    super(message, 'AUTH_FAILED', 401, true, context);
    Object.setPrototypeOf(this, AuthenticationError.prototype);
  }
}

export class InvalidCredentialsError extends AuthenticationError {
  constructor(message: string = 'Invalid username or password', context?: Record<string, any>) {
    super(message, context);
    this.code = 'INVALID_CREDENTIALS';
    Object.setPrototypeOf(this, InvalidCredentialsError.prototype);
  }
}

export class TokenExpiredError extends AuthenticationError {
  constructor(message: string = 'Authentication token has expired', context?: Record<string, any>) {
    super(message, context);
    this.code = 'TOKEN_EXPIRED';
    Object.setPrototypeOf(this, TokenExpiredError.prototype);
  }
}

export class InvalidTokenError extends AuthenticationError {
  constructor(message: string = 'Invalid authentication token', context?: Record<string, any>) {
    super(message, context);
    this.code = 'INVALID_TOKEN';
    Object.setPrototypeOf(this, InvalidTokenError.prototype);
  }
}

export class AuthorizationError extends ApplicationError {
  constructor(message: string = 'Insufficient permissions', context?: Record<string, any>) {
    super(message, 'UNAUTHORIZED', 403, true, context);
    Object.setPrototypeOf(this, AuthorizationError.prototype);
  }
}

export class SessionExpiredError extends AuthenticationError {
  constructor(message: string = 'Session has expired', context?: Record<string, any>) {
    super(message, context);
    this.code = 'SESSION_EXPIRED';
    Object.setPrototypeOf(this, SessionExpiredError.prototype);
  }
}

// ============================================================================
// VALIDATION ERRORS
// ============================================================================

export class ValidationError extends ApplicationError {
  public readonly fields?: ValidationErrorField[];

  constructor(
    message: string = 'Validation failed',
    fields?: ValidationErrorField[],
    context?: Record<string, any>
  ) {
    super(message, 'VALIDATION_ERROR', 400, true, context);
    this.fields = fields;
    Object.setPrototypeOf(this, ValidationError.prototype);
  }

  toJSON() {
    return {
      ...super.toJSON(),
      fields: this.fields,
    };
  }
}

export interface ValidationErrorField {
  field: string;
  message: string;
  value?: any;
  constraint?: string;
}

export class RequiredFieldError extends ValidationError {
  constructor(fieldName: string, context?: Record<string, any>) {
    super(
      `Required field '${fieldName}' is missing`,
      [{ field: fieldName, message: 'This field is required' }],
      context
    );
    this.code = 'REQUIRED_FIELD';
    Object.setPrototypeOf(this, RequiredFieldError.prototype);
  }
}

export class InvalidFormatError extends ValidationError {
  constructor(fieldName: string, expectedFormat: string, context?: Record<string, any>) {
    super(
      `Invalid format for field '${fieldName}'. Expected: ${expectedFormat}`,
      [{ field: fieldName, message: `Expected format: ${expectedFormat}` }],
      context
    );
    this.code = 'INVALID_FORMAT';
    Object.setPrototypeOf(this, InvalidFormatError.prototype);
  }
}

export class InvalidRangeError extends ValidationError {
  constructor(
    fieldName: string,
    min?: number,
    max?: number,
    value?: any,
    context?: Record<string, any>
  ) {
    const rangeMsg = min !== undefined && max !== undefined
      ? `between ${min} and ${max}`
      : min !== undefined
      ? `at least ${min}`
      : `at most ${max}`;

    super(
      `Field '${fieldName}' must be ${rangeMsg}`,
      [{ field: fieldName, message: `Value must be ${rangeMsg}`, value }],
      context
    );
    this.code = 'INVALID_RANGE';
    Object.setPrototypeOf(this, InvalidRangeError.prototype);
  }
}

// ============================================================================
// RESOURCE ERRORS
// ============================================================================

export class ResourceNotFoundError extends ApplicationError {
  constructor(
    resourceType: string,
    resourceId: string | number,
    context?: Record<string, any>
  ) {
    super(
      `${resourceType} with ID '${resourceId}' not found`,
      'RESOURCE_NOT_FOUND',
      404,
      true,
      { resourceType, resourceId, ...context }
    );
    Object.setPrototypeOf(this, ResourceNotFoundError.prototype);
  }
}

export class ResourceAlreadyExistsError extends ApplicationError {
  constructor(resourceType: string, identifier: string, context?: Record<string, any>) {
    super(
      `${resourceType} '${identifier}' already exists`,
      'RESOURCE_ALREADY_EXISTS',
      409,
      true,
      { resourceType, identifier, ...context }
    );
    Object.setPrototypeOf(this, ResourceAlreadyExistsError.prototype);
  }
}

export class ResourceLockedError extends ApplicationError {
  constructor(resourceType: string, resourceId: string, lockedBy?: string, context?: Record<string, any>) {
    const lockMsg = lockedBy ? ` by ${lockedBy}` : '';
    super(
      `${resourceType} '${resourceId}' is locked${lockMsg}`,
      'RESOURCE_LOCKED',
      423,
      true,
      { resourceType, resourceId, lockedBy, ...context }
    );
    Object.setPrototypeOf(this, ResourceLockedError.prototype);
  }
}

export class ResourceConflictError extends ApplicationError {
  constructor(message: string, context?: Record<string, any>) {
    super(message, 'RESOURCE_CONFLICT', 409, true, context);
    Object.setPrototypeOf(this, ResourceConflictError.prototype);
  }
}

// ============================================================================
// DATABASE ERRORS
// ============================================================================

export class DatabaseError extends ApplicationError {
  constructor(message: string, operation?: string, context?: Record<string, any>) {
    super(
      message,
      'DATABASE_ERROR',
      500,
      true,
      { operation, ...context }
    );
    Object.setPrototypeOf(this, DatabaseError.prototype);
  }
}

export class DatabaseConnectionError extends DatabaseError {
  constructor(message: string = 'Failed to connect to database', context?: Record<string, any>) {
    super(message, 'connect', context);
    this.code = 'DB_CONNECTION_ERROR';
    Object.setPrototypeOf(this, DatabaseConnectionError.prototype);
  }
}

export class DatabaseQueryError extends DatabaseError {
  constructor(query?: string, originalError?: Error, context?: Record<string, any>) {
    super(
      `Database query failed${originalError ? ': ' + originalError.message : ''}`,
      'query',
      { query, originalError: originalError?.message, ...context }
    );
    this.code = 'DB_QUERY_ERROR';
    Object.setPrototypeOf(this, DatabaseQueryError.prototype);
  }
}

export class DatabaseTransactionError extends DatabaseError {
  constructor(message: string = 'Database transaction failed', context?: Record<string, any>) {
    super(message, 'transaction', context);
    this.code = 'DB_TRANSACTION_ERROR';
    Object.setPrototypeOf(this, DatabaseTransactionError.prototype);
  }
}

export class DatabaseConstraintError extends DatabaseError {
  constructor(constraint: string, context?: Record<string, any>) {
    super(
      `Database constraint violation: ${constraint}`,
      'constraint',
      { constraint, ...context }
    );
    this.code = 'DB_CONSTRAINT_ERROR';
    Object.setPrototypeOf(this, DatabaseConstraintError.prototype);
  }
}

// ============================================================================
// FILE SYSTEM ERRORS
// ============================================================================

export class FileSystemError extends ApplicationError {
  constructor(message: string, filePath?: string, context?: Record<string, any>) {
    super(message, 'FILE_SYSTEM_ERROR', 500, true, { filePath, ...context });
    Object.setPrototypeOf(this, FileSystemError.prototype);
  }
}

export class FileNotFoundError extends FileSystemError {
  constructor(filePath: string, context?: Record<string, any>) {
    super(`File not found: ${filePath}`, filePath, context);
    this.code = 'FILE_NOT_FOUND';
    this.statusCode = 404;
    Object.setPrototypeOf(this, FileNotFoundError.prototype);
  }
}

export class FileUploadError extends FileSystemError {
  constructor(message: string, fileName?: string, context?: Record<string, any>) {
    super(message, fileName, { fileName, ...context });
    this.code = 'FILE_UPLOAD_ERROR';
    Object.setPrototypeOf(this, FileUploadError.prototype);
  }
}

export class FileTypeNotSupportedError extends FileSystemError {
  constructor(fileType: string, supportedTypes?: string[], context?: Record<string, any>) {
    const supportedMsg = supportedTypes ? `. Supported types: ${supportedTypes.join(', ')}` : '';
    super(
      `File type '${fileType}' is not supported${supportedMsg}`,
      undefined,
      { fileType, supportedTypes, ...context }
    );
    this.code = 'FILE_TYPE_NOT_SUPPORTED';
    this.statusCode = 415;
    Object.setPrototypeOf(this, FileTypeNotSupportedError.prototype);
  }
}

export class FileSizeExceededError extends FileSystemError {
  constructor(fileSize: number, maxSize: number, context?: Record<string, any>) {
    super(
      `File size ${fileSize} bytes exceeds maximum allowed size of ${maxSize} bytes`,
      undefined,
      { fileSize, maxSize, ...context }
    );
    this.code = 'FILE_SIZE_EXCEEDED';
    this.statusCode = 413;
    Object.setPrototypeOf(this, FileSizeExceededError.prototype);
  }
}

// ============================================================================
// BUSINESS LOGIC ERRORS
// ============================================================================

export class BusinessLogicError extends ApplicationError {
  constructor(message: string, code: string = 'BUSINESS_LOGIC_ERROR', context?: Record<string, any>) {
    super(message, code, 422, true, context);
    Object.setPrototypeOf(this, BusinessLogicError.prototype);
  }
}

export class InvalidStateTransitionError extends BusinessLogicError {
  constructor(
    entityType: string,
    currentState: string,
    targetState: string,
    context?: Record<string, any>
  ) {
    super(
      `Cannot transition ${entityType} from '${currentState}' to '${targetState}'`,
      'INVALID_STATE_TRANSITION',
      { entityType, currentState, targetState, ...context }
    );
    Object.setPrototypeOf(this, InvalidStateTransitionError.prototype);
  }
}

export class InvalidOperationError extends BusinessLogicError {
  constructor(operation: string, reason: string, context?: Record<string, any>) {
    super(
      `Cannot perform operation '${operation}': ${reason}`,
      'INVALID_OPERATION',
      { operation, reason, ...context }
    );
    Object.setPrototypeOf(this, InvalidOperationError.prototype);
  }
}

export class DependencyError extends BusinessLogicError {
  constructor(
    entityType: string,
    entityId: string,
    dependentEntities: string[],
    context?: Record<string, any>
  ) {
    super(
      `Cannot perform operation on ${entityType} '${entityId}' due to dependencies: ${dependentEntities.join(', ')}`,
      'DEPENDENCY_ERROR',
      { entityType, entityId, dependentEntities, ...context }
    );
    Object.setPrototypeOf(this, DependencyError.prototype);
  }
}

// ============================================================================
// PHYSICS & SIMULATION ERRORS
// ============================================================================

export class SimulationError extends ApplicationError {
  constructor(message: string, context?: Record<string, any>) {
    super(message, 'SIMULATION_ERROR', 500, true, context);
    Object.setPrototypeOf(this, SimulationError.prototype);
  }
}

export class ConvergenceError extends SimulationError {
  constructor(iterations: number, threshold: number, context?: Record<string, any>) {
    super(
      `Simulation failed to converge after ${iterations} iterations (threshold: ${threshold})`,
      { iterations, threshold, ...context }
    );
    this.code = 'CONVERGENCE_ERROR';
    Object.setPrototypeOf(this, ConvergenceError.prototype);
  }
}

export class InvalidPhysicsParametersError extends SimulationError {
  constructor(parameter: string, value: any, reason: string, context?: Record<string, any>) {
    super(
      `Invalid physics parameter '${parameter}' = ${value}: ${reason}`,
      { parameter, value, reason, ...context }
    );
    this.code = 'INVALID_PHYSICS_PARAMETERS';
    Object.setPrototypeOf(this, InvalidPhysicsParametersError.prototype);
  }
}

export class InsufficientDataError extends SimulationError {
  constructor(requiredData: string[], missingData: string[], context?: Record<string, any>) {
    super(
      `Insufficient data for simulation. Missing: ${missingData.join(', ')}`,
      { requiredData, missingData, ...context }
    );
    this.code = 'INSUFFICIENT_DATA';
    Object.setPrototypeOf(this, InsufficientDataError.prototype);
  }
}

// ============================================================================
// EXTERNAL SERVICE ERRORS
// ============================================================================

export class ExternalServiceError extends ApplicationError {
  constructor(
    serviceName: string,
    message: string,
    statusCode: number = 502,
    context?: Record<string, any>
  ) {
    super(
      `External service error (${serviceName}): ${message}`,
      'EXTERNAL_SERVICE_ERROR',
      statusCode,
      true,
      { serviceName, ...context }
    );
    Object.setPrototypeOf(this, ExternalServiceError.prototype);
  }
}

export class ServiceTimeoutError extends ExternalServiceError {
  constructor(serviceName: string, timeout: number, context?: Record<string, any>) {
    super(
      serviceName,
      `Request timed out after ${timeout}ms`,
      504,
      { timeout, ...context }
    );
    this.code = 'SERVICE_TIMEOUT';
    Object.setPrototypeOf(this, ServiceTimeoutError.prototype);
  }
}

export class ServiceUnavailableError extends ExternalServiceError {
  constructor(serviceName: string, context?: Record<string, any>) {
    super(serviceName, 'Service is currently unavailable', 503, context);
    this.code = 'SERVICE_UNAVAILABLE';
    Object.setPrototypeOf(this, ServiceUnavailableError.prototype);
  }
}

// ============================================================================
// RATE LIMITING & QUOTA ERRORS
// ============================================================================

export class RateLimitError extends ApplicationError {
  constructor(
    limit: number,
    windowMs: number,
    retryAfter?: number,
    context?: Record<string, any>
  ) {
    super(
      `Rate limit exceeded. Limit: ${limit} requests per ${windowMs}ms`,
      'RATE_LIMIT_EXCEEDED',
      429,
      true,
      { limit, windowMs, retryAfter, ...context }
    );
    Object.setPrototypeOf(this, RateLimitError.prototype);
  }
}

export class QuotaExceededError extends ApplicationError {
  constructor(quotaType: string, limit: number, current: number, context?: Record<string, any>) {
    super(
      `${quotaType} quota exceeded. Limit: ${limit}, Current: ${current}`,
      'QUOTA_EXCEEDED',
      429,
      true,
      { quotaType, limit, current, ...context }
    );
    Object.setPrototypeOf(this, QuotaExceededError.prototype);
  }
}

// ============================================================================
// CONFIGURATION ERRORS
// ============================================================================

export class ConfigurationError extends ApplicationError {
  constructor(message: string, context?: Record<string, any>) {
    super(message, 'CONFIGURATION_ERROR', 500, false, context);
    Object.setPrototypeOf(this, ConfigurationError.prototype);
  }
}

export class MissingConfigurationError extends ConfigurationError {
  constructor(configKey: string, context?: Record<string, any>) {
    super(
      `Missing required configuration: ${configKey}`,
      { configKey, ...context }
    );
    this.code = 'MISSING_CONFIGURATION';
    Object.setPrototypeOf(this, MissingConfigurationError.prototype);
  }
}

export class InvalidConfigurationError extends ConfigurationError {
  constructor(configKey: string, reason: string, context?: Record<string, any>) {
    super(
      `Invalid configuration for '${configKey}': ${reason}`,
      { configKey, reason, ...context }
    );
    this.code = 'INVALID_CONFIGURATION';
    Object.setPrototypeOf(this, InvalidConfigurationError.prototype);
  }
}

// ============================================================================
// ERROR UTILITIES
// ============================================================================

export function isOperationalError(error: Error): boolean {
  if (error instanceof ApplicationError) {
    return error.isOperational;
  }
  return false;
}

export function getErrorCode(error: Error): string {
  if (error instanceof ApplicationError) {
    return error.code;
  }
  return 'UNKNOWN_ERROR';
}

export function getErrorStatusCode(error: Error): number {
  if (error instanceof ApplicationError) {
    return error.statusCode;
  }
  return 500;
}

export function formatErrorForClient(error: Error, includeStack: boolean = false): object {
  if (error instanceof ApplicationError) {
    const response: any = {
      error: {
        name: error.name,
        message: error.message,
        code: error.code,
        timestamp: error.timestamp,
      },
    };

    if (error.context) {
      response.error.context = error.context;
    }

    if (error instanceof ValidationError && error.fields) {
      response.error.fields = error.fields;
    }

    if (includeStack && error.stack) {
      response.error.stack = error.stack;
    }

    return response;
  }

  return {
    error: {
      name: 'Error',
      message: error.message,
      code: 'UNKNOWN_ERROR',
      timestamp: new Date(),
    },
  };
}

export function wrapError(error: unknown, defaultMessage: string = 'An error occurred'): ApplicationError {
  if (error instanceof ApplicationError) {
    return error;
  }

  if (error instanceof Error) {
    return new ApplicationError(
      error.message || defaultMessage,
      'WRAPPED_ERROR',
      500,
      false,
      { originalError: error.name, stack: error.stack }
    );
  }

  return new ApplicationError(
    defaultMessage,
    'UNKNOWN_ERROR',
    500,
    false,
    { originalError: String(error) }
  );
}
