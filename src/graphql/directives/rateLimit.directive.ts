/**
 * Rate Limit Directive
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { GraphQLError } from 'graphql';
import { getDirective, MapperKind, mapSchema } from '@graphql-tools/utils';
import { GraphQLSchema } from 'graphql';
import { GraphQLContext } from '../types';

// Simple in-memory rate limit store (use Redis in production)
const rateLimitStore = new Map<string, { count: number; resetAt: number }>();

export function rateLimitDirective(directiveName: string = 'rateLimit') {
  return {
    rateLimitDirectiveTypeDefs: `directive @${directiveName}(max: Int!, window: Int!) on FIELD_DEFINITION`,

    rateLimitDirectiveTransformer: (schema: GraphQLSchema) =>
      mapSchema(schema, {
        [MapperKind.OBJECT_FIELD]: (fieldConfig) => {
          const rateLimitDirective = getDirective(schema, fieldConfig, directiveName)?.[0];

          if (rateLimitDirective) {
            const { max, window } = rateLimitDirective;
            const { resolve = defaultFieldResolver } = fieldConfig;

            fieldConfig.resolve = async function (source, args, context: GraphQLContext, info) {
              // Create a unique key for this user/field combination
              const userId = context.user?.id || context.req.ip || 'anonymous';
              const key = `${userId}:${info.parentType.name}.${info.fieldName}`;

              const now = Date.now();
              const windowMs = window * 1000; // Convert to milliseconds

              let limitInfo = rateLimitStore.get(key);

              // Reset if window has passed
              if (!limitInfo || now > limitInfo.resetAt) {
                limitInfo = {
                  count: 0,
                  resetAt: now + windowMs,
                };
              }

              // Check if limit exceeded
              if (limitInfo.count >= max) {
                const resetIn = Math.ceil((limitInfo.resetAt - now) / 1000);
                throw new GraphQLError('Rate limit exceeded', {
                  extensions: {
                    code: 'RATE_LIMIT_EXCEEDED',
                    http: { status: 429 },
                    max,
                    window,
                    resetIn,
                  },
                });
              }

              // Increment counter
              limitInfo.count += 1;
              rateLimitStore.set(key, limitInfo);

              // Call the original resolver
              return resolve(source, args, context, info);
            };
          }

          return fieldConfig;
        },
      }),
  };
}

// Default field resolver
function defaultFieldResolver(source: any, args: any, context: any, info: any) {
  if (typeof source === 'object' && source !== null && info.fieldName in source) {
    return source[info.fieldName];
  }
  return null;
}

// Cleanup old entries periodically
setInterval(() => {
  const now = Date.now();
  for (const [key, value] of rateLimitStore.entries()) {
    if (now > value.resetAt) {
      rateLimitStore.delete(key);
    }
  }
}, 60000); // Clean up every minute
