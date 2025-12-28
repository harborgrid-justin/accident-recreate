/**
 * Global Error Handling Middleware
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Request, Response, NextFunction } from 'express';
import { error, ErrorCode } from '../responses';

// Re-export ErrorCode for convenience
export { ErrorCode };
import { AuthError, ValidationError } from '../../auth/types';
import { InvalidStatusTransitionError } from '../../cases/CaseStatus';

export class AppError extends Error {
  constructor(
    message: string,
    public statusCode: number = 500,
    public code: string = ErrorCode.INTERNAL_ERROR,
    public details?: any
  ) {
    super(message);
    this.name = 'AppError';
    Object.setPrototypeOf(this, AppError.prototype);
  }
}

export class NotFoundError extends AppError {
  constructor(resource: string = 'Resource', id?: string) {
    const message = id
      ? `${resource} with ID '${id}' not found`
      : `${resource} not found`;
    super(message, 404, ErrorCode.NOT_FOUND);
    this.name = 'NotFoundError';
  }
}

export class ConflictError extends AppError {
  constructor(message: string, details?: any) {
    super(message, 409, ErrorCode.CONFLICT, details);
    this.name = 'ConflictError';
  }
}

export class ForbiddenError extends AppError {
  constructor(message: string = 'Access forbidden') {
    super(message, 403, ErrorCode.FORBIDDEN);
    this.name = 'ForbiddenError';
  }
}

/**
 * Main error handling middleware
 */
export function errorHandler(
  err: Error,
  req: Request,
  res: Response,
  next: NextFunction
): void {
  // Generate request ID if not present
  const requestId = (req as any).id || generateRequestId();

  // Log error for debugging
  console.error('[Error Handler]', {
    requestId,
    error: err.message,
    stack: err.stack,
    path: req.path,
    method: req.method,
  });

  // Handle different error types
  if (err instanceof AppError) {
    res.status(err.statusCode).json(
      error(err.code, err.message, err.details, requestId)
    );
    return;
  }

  if (err instanceof AuthError) {
    res.status(err.statusCode).json(
      error(err.code, err.message, undefined, requestId)
    );
    return;
  }

  if (err instanceof ValidationError) {
    res.status(400).json(
      error(ErrorCode.VALIDATION_ERROR, err.message, err.fields, requestId)
    );
    return;
  }

  if (err instanceof InvalidStatusTransitionError) {
    res.status(400).json(
      error(
        ErrorCode.INVALID_STATUS_TRANSITION,
        err.message,
        {
          currentStatus: err.currentStatus,
          attemptedStatus: err.attemptedStatus,
        },
        requestId
      )
    );
    return;
  }

  // Handle Multer file upload errors
  if (err.name === 'MulterError') {
    const message = getMulterErrorMessage(err);
    res.status(400).json(
      error(ErrorCode.VALIDATION_ERROR, message, undefined, requestId)
    );
    return;
  }

  // Handle JWT errors
  if (err.name === 'JsonWebTokenError') {
    res.status(401).json(
      error(ErrorCode.TOKEN_INVALID, 'Invalid authentication token', undefined, requestId)
    );
    return;
  }

  if (err.name === 'TokenExpiredError') {
    res.status(401).json(
      error(ErrorCode.TOKEN_EXPIRED, 'Authentication token has expired', undefined, requestId)
    );
    return;
  }

  // Handle syntax errors (malformed JSON)
  if (err instanceof SyntaxError && 'body' in err) {
    res.status(400).json(
      error(ErrorCode.INVALID_INPUT, 'Invalid JSON in request body', undefined, requestId)
    );
    return;
  }

  // Default to 500 Internal Server Error
  res.status(500).json(
    error(
      ErrorCode.INTERNAL_ERROR,
      process.env.NODE_ENV === 'production'
        ? 'An unexpected error occurred'
        : err.message,
      process.env.NODE_ENV === 'production' ? undefined : err.stack,
      requestId
    )
  );
}

/**
 * 404 Not Found handler
 */
export function notFoundHandler(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const requestId = (req as any).id || generateRequestId();
  res.status(404).json(
    error(
      ErrorCode.NOT_FOUND,
      `Cannot ${req.method} ${req.path}`,
      undefined,
      requestId
    )
  );
}

/**
 * Async handler wrapper to catch promise rejections
 */
export function asyncHandler(
  fn: (req: Request, res: Response, next: NextFunction) => Promise<any>
) {
  return (req: Request, res: Response, next: NextFunction) => {
    Promise.resolve(fn(req, res, next)).catch(next);
  };
}

/**
 * Get user-friendly Multer error messages
 */
function getMulterErrorMessage(err: any): string {
  switch (err.code) {
    case 'LIMIT_FILE_SIZE':
      return 'File size exceeds the maximum allowed limit';
    case 'LIMIT_FILE_COUNT':
      return 'Too many files uploaded';
    case 'LIMIT_UNEXPECTED_FILE':
      return 'Unexpected file field';
    case 'LIMIT_PART_COUNT':
      return 'Too many parts in multipart request';
    default:
      return 'File upload error';
  }
}

/**
 * Generate a unique request ID
 */
function generateRequestId(): string {
  return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}
