/**
 * Validation Directive
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { GraphQLError } from 'graphql';
import { getDirective, MapperKind, mapSchema } from '@graphql-tools/utils';
import { GraphQLSchema } from 'graphql';
import { z } from 'zod';

// Validation schemas registry
const validationSchemas = new Map<string, z.ZodSchema>();

// Register common validation schemas
validationSchemas.set('email', z.string().email());
validationSchemas.set('uuid', z.string().uuid());
validationSchemas.set('url', z.string().url());
validationSchemas.set('phone', z.string().regex(/^\+?[1-9]\d{1,14}$/));
validationSchemas.set('zipCode', z.string().regex(/^\d{5}(-\d{4})?$/));
validationSchemas.set('vin', z.string().length(17).regex(/^[A-HJ-NPR-Z0-9]{17}$/));
validationSchemas.set('positiveInt', z.number().int().positive());
validationSchemas.set('positiveFloat', z.number().positive());
validationSchemas.set('nonEmpty', z.string().min(1));

export function validateDirective(directiveName: string = 'validate') {
  return {
    validateDirectiveTypeDefs: `directive @${directiveName}(schema: String!) on INPUT_FIELD_DEFINITION | ARGUMENT_DEFINITION`,

    validateDirectiveTransformer: (schema: GraphQLSchema) =>
      mapSchema(schema, {
        [MapperKind.OBJECT_FIELD]: (fieldConfig) => {
          const validateDirective = getDirective(schema, fieldConfig, directiveName)?.[0];

          if (validateDirective) {
            const { schema: schemaName } = validateDirective;
            const { resolve = defaultFieldResolver } = fieldConfig;

            fieldConfig.resolve = async function (source, args, context, info) {
              // Get the validation schema
              const validationSchema = validationSchemas.get(schemaName);

              if (!validationSchema) {
                console.warn(`Validation schema "${schemaName}" not found`);
                return resolve(source, args, context, info);
              }

              // Validate arguments
              for (const [argName, argValue] of Object.entries(args)) {
                try {
                  validationSchema.parse(argValue);
                } catch (error) {
                  if (error instanceof z.ZodError) {
                    throw new GraphQLError(`Validation failed for ${argName}`, {
                      extensions: {
                        code: 'VALIDATION_ERROR',
                        validationErrors: error.errors,
                        field: argName,
                      },
                    });
                  }
                  throw error;
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

// Helper to register custom validation schemas
export function registerValidationSchema(name: string, schema: z.ZodSchema) {
  validationSchemas.set(name, schema);
}
