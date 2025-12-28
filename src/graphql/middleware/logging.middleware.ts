/**
 * Logging Middleware
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { GraphQLRequestContext } from '@apollo/server';
import { GraphQLContext } from '../types';
import winston from 'winston';

// Configure Winston logger
const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  defaultMeta: { service: 'graphql-api' },
  transports: [
    new winston.transports.File({ filename: 'logs/graphql-error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/graphql-combined.log' }),
  ],
});

// Add console transport in development
if (process.env.NODE_ENV !== 'production') {
  logger.add(
    new winston.transports.Console({
      format: winston.format.combine(winston.format.colorize(), winston.format.simple()),
    })
  );
}

/**
 * Apollo Server plugin for request logging
 */
export const loggingPlugin = {
  async requestDidStart(requestContext: GraphQLRequestContext<GraphQLContext>) {
    const { request, contextValue } = requestContext;
    const startTime = Date.now();

    logger.info('GraphQL Request Started', {
      requestId: contextValue.requestId,
      operationName: request.operationName,
      userId: contextValue.user?.id,
      query: request.query?.substring(0, 200), // Log first 200 chars of query
    });

    return {
      async willSendResponse(requestContext: any) {
        const duration = Date.now() - startTime;

        logger.info('GraphQL Request Completed', {
          requestId: contextValue.requestId,
          operationName: request.operationName,
          userId: contextValue.user?.id,
          duration: `${duration}ms`,
          errors: requestContext.errors?.length || 0,
        });
      },

      async didEncounterErrors(requestContext: any) {
        const { errors } = requestContext;

        errors.forEach((error: any) => {
          logger.error('GraphQL Error', {
            requestId: contextValue.requestId,
            operationName: request.operationName,
            userId: contextValue.user?.id,
            error: {
              message: error.message,
              code: error.extensions?.code,
              path: error.path,
              stack: error.stack,
            },
          });
        });
      },

      async executionDidStart() {
        return {
          willResolveField({ info }: any) {
            const start = Date.now();

            return () => {
              const duration = Date.now() - start;

              // Log slow field resolvers (> 100ms)
              if (duration > 100) {
                logger.warn('Slow Field Resolver', {
                  requestId: contextValue.requestId,
                  field: `${info.parentType.name}.${info.fieldName}`,
                  duration: `${duration}ms`,
                });
              }
            };
          },
        };
      },
    };
  },
};

/**
 * Log GraphQL operation
 */
export function logOperation(
  operationName: string,
  userId?: string,
  metadata?: Record<string, unknown>
): void {
  logger.info('GraphQL Operation', {
    operationName,
    userId,
    ...metadata,
  });
}

/**
 * Log GraphQL error
 */
export function logError(
  error: Error,
  operationName?: string,
  userId?: string,
  metadata?: Record<string, unknown>
): void {
  logger.error('GraphQL Error', {
    operationName,
    userId,
    error: {
      message: error.message,
      stack: error.stack,
    },
    ...metadata,
  });
}

/**
 * Export logger for use in other modules
 */
export { logger };
