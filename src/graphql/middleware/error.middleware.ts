/**
 * Error Handling Middleware
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { GraphQLError, GraphQLFormattedError } from 'graphql';
import { unwrapResolverError } from '@apollo/server/errors';
import { logger } from './logging.middleware';

/**
 * Custom error codes
 */
export enum ErrorCode {
  UNAUTHENTICATED = 'UNAUTHENTICATED',
  UNAUTHORIZED = 'UNAUTHORIZED',
  BAD_REQUEST = 'BAD_REQUEST',
  NOT_FOUND = 'NOT_FOUND',
  CONFLICT = 'CONFLICT',
  INTERNAL_ERROR = 'INTERNAL_ERROR',
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
}

/**
 * Apollo Server plugin for error formatting
 */
export const errorHandlingPlugin = {
  async requestDidStart() {
    return {
      async didEncounterErrors(requestContext: any) {
        const { errors, contextValue } = requestContext;

        errors.forEach((error: GraphQLError) => {
          // Unwrap the error to get the original error
          const originalError = unwrapResolverError(error);

          // Log error details
          logger.error('GraphQL Error Encountered', {
            requestId: contextValue.requestId,
            userId: contextValue.user?.id,
            error: {
              message: error.message,
              code: error.extensions?.code,
              path: error.path,
              originalError: originalError.message,
              stack: originalError.stack,
            },
          });
        });
      },
    };
  },
};

/**
 * Format GraphQL errors for client response
 */
export function formatError(error: GraphQLError): GraphQLFormattedError {
  const formattedError: GraphQLFormattedError = {
    message: error.message,
    locations: error.locations,
    path: error.path,
    extensions: {
      code: error.extensions?.code || ErrorCode.INTERNAL_ERROR,
      ...error.extensions,
    },
  };

  // In production, hide internal error details
  if (process.env.NODE_ENV === 'production') {
    // Don't expose stack traces in production
    delete formattedError.extensions?.exception;

    // Sanitize internal errors
    if (formattedError.extensions?.code === ErrorCode.INTERNAL_ERROR) {
      formattedError.message = 'An internal error occurred';
    }
  } else {
    // In development, include more details
    if (error.originalError) {
      formattedError.extensions = {
        ...formattedError.extensions,
        exception: {
          message: error.originalError.message,
          stacktrace: error.originalError.stack?.split('\n'),
        },
      };
    }
  }

  return formattedError;
}

/**
 * Create a standardized GraphQL error
 */
export function createError(
  message: string,
  code: ErrorCode = ErrorCode.INTERNAL_ERROR,
  extensions?: Record<string, unknown>
): GraphQLError {
  return new GraphQLError(message, {
    extensions: {
      code,
      ...extensions,
    },
  });
}

/**
 * Create a validation error
 */
export function createValidationError(
  message: string,
  validationErrors?: unknown[]
): GraphQLError {
  return createError(message, ErrorCode.VALIDATION_ERROR, {
    validationErrors,
  });
}

/**
 * Create a not found error
 */
export function createNotFoundError(resource: string, id?: string): GraphQLError {
  const message = id
    ? `${resource} with ID "${id}" not found`
    : `${resource} not found`;

  return createError(message, ErrorCode.NOT_FOUND, {
    resource,
    id,
  });
}

/**
 * Create an unauthorized error
 */
export function createUnauthorizedError(
  message: string = 'You are not authorized to perform this action'
): GraphQLError {
  return createError(message, ErrorCode.UNAUTHORIZED, {
    http: { status: 403 },
  });
}

/**
 * Create an unauthenticated error
 */
export function createUnauthenticatedError(
  message: string = 'Authentication required'
): GraphQLError {
  return createError(message, ErrorCode.UNAUTHENTICATED, {
    http: { status: 401 },
  });
}

/**
 * Create a rate limit error
 */
export function createRateLimitError(
  max: number,
  window: number,
  resetIn: number
): GraphQLError {
  return createError('Rate limit exceeded', ErrorCode.RATE_LIMIT_EXCEEDED, {
    http: { status: 429 },
    max,
    window,
    resetIn,
  });
}

/**
 * Create a conflict error
 */
export function createConflictError(message: string, resource?: string): GraphQLError {
  return createError(message, ErrorCode.CONFLICT, {
    resource,
    http: { status: 409 },
  });
}

/**
 * Create a bad request error
 */
export function createBadRequestError(
  message: string,
  details?: Record<string, unknown>
): GraphQLError {
  return createError(message, ErrorCode.BAD_REQUEST, {
    http: { status: 400 },
    ...details,
  });
}
