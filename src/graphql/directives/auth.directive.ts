/**
 * Authentication Directive
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { GraphQLError } from 'graphql';
import { getDirective, MapperKind, mapSchema } from '@graphql-tools/utils';
import { GraphQLSchema } from 'graphql';
import { GraphQLContext, Permission } from '../types';

export function authDirective(directiveName: string = 'auth') {
  return {
    authDirectiveTypeDefs: `directive @${directiveName}(requires: [Permission!]!) on FIELD_DEFINITION | OBJECT`,

    authDirectiveTransformer: (schema: GraphQLSchema) =>
      mapSchema(schema, {
        [MapperKind.OBJECT_FIELD]: (fieldConfig) => {
          const authDirective = getDirective(schema, fieldConfig, directiveName)?.[0];

          if (authDirective) {
            const { requires } = authDirective;
            const { resolve = defaultFieldResolver } = fieldConfig;

            fieldConfig.resolve = async function (source, args, context: GraphQLContext, info) {
              // Check if user is authenticated
              if (!context.user) {
                throw new GraphQLError('Authentication required', {
                  extensions: {
                    code: 'UNAUTHENTICATED',
                    http: { status: 401 },
                  },
                });
              }

              // Check if user has required permissions
              if (requires && requires.length > 0) {
                const hasPermission = requires.every((permission: Permission) =>
                  context.user!.permissions.includes(permission)
                );

                if (!hasPermission) {
                  throw new GraphQLError('Insufficient permissions', {
                    extensions: {
                      code: 'UNAUTHORIZED',
                      http: { status: 403 },
                      required: requires,
                      actual: context.user.permissions,
                    },
                  });
                }
              }

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
