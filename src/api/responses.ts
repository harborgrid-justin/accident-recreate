/**
 * Standardized API Response Utilities
 * AccuScene Enterprise Accident Recreation Platform
 */

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
  meta?: {
    timestamp: string;
    requestId?: string;
  };
}

export interface PaginatedResponse<T = any> extends ApiResponse<T> {
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}

/**
 * Success response with data
 */
export function success<T>(
  data: T,
  message?: string,
  requestId?: string
): ApiResponse<T> {
  return {
    success: true,
    data,
    message,
    meta: {
      timestamp: new Date().toISOString(),
      requestId,
    },
  };
}

/**
 * Error response
 */
export function error(
  code: string,
  message: string,
  details?: any,
  requestId?: string
): ApiResponse {
  return {
    success: false,
    error: {
      code,
      message,
      details,
    },
    meta: {
      timestamp: new Date().toISOString(),
      requestId,
    },
  };
}

/**
 * Paginated response with data
 */
export function paginated<T>(
  data: T[],
  page: number,
  limit: number,
  total: number,
  requestId?: string
): PaginatedResponse<T[]> {
  const totalPages = Math.ceil(total / limit);

  return {
    success: true,
    data,
    pagination: {
      page,
      limit,
      total,
      totalPages,
      hasNext: page < totalPages,
      hasPrev: page > 1,
    },
    meta: {
      timestamp: new Date().toISOString(),
      requestId,
    },
  };
}

/**
 * No content success response
 */
export function noContent(message?: string, requestId?: string): ApiResponse {
  return {
    success: true,
    message: message || 'Operation completed successfully',
    meta: {
      timestamp: new Date().toISOString(),
      requestId,
    },
  };
}

/**
 * Created response (201)
 */
export function created<T>(
  data: T,
  message?: string,
  requestId?: string
): ApiResponse<T> {
  return {
    success: true,
    data,
    message: message || 'Resource created successfully',
    meta: {
      timestamp: new Date().toISOString(),
      requestId,
    },
  };
}

/**
 * Common error codes
 */
export enum ErrorCode {
  // Authentication & Authorization
  UNAUTHORIZED = 'UNAUTHORIZED',
  FORBIDDEN = 'FORBIDDEN',
  INVALID_CREDENTIALS = 'INVALID_CREDENTIALS',
  TOKEN_EXPIRED = 'TOKEN_EXPIRED',
  TOKEN_INVALID = 'TOKEN_INVALID',

  // Validation
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  INVALID_INPUT = 'INVALID_INPUT',
  MISSING_REQUIRED_FIELD = 'MISSING_REQUIRED_FIELD',

  // Resource
  NOT_FOUND = 'NOT_FOUND',
  ALREADY_EXISTS = 'ALREADY_EXISTS',
  CONFLICT = 'CONFLICT',

  // Business Logic
  INVALID_STATUS_TRANSITION = 'INVALID_STATUS_TRANSITION',
  OPERATION_NOT_ALLOWED = 'OPERATION_NOT_ALLOWED',
  INSUFFICIENT_PERMISSIONS = 'INSUFFICIENT_PERMISSIONS',

  // System
  INTERNAL_ERROR = 'INTERNAL_ERROR',
  SERVICE_UNAVAILABLE = 'SERVICE_UNAVAILABLE',
  DATABASE_ERROR = 'DATABASE_ERROR',

  // Rate Limiting
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
  TOO_MANY_REQUESTS = 'TOO_MANY_REQUESTS',
}
