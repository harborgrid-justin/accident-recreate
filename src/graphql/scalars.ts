/**
 * Custom GraphQL Scalar Types
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { GraphQLScalarType, Kind, ValueNode } from 'graphql';
import { GraphQLError } from 'graphql';

// ============================================================================
// DateTime Scalar
// ============================================================================

export const DateTimeScalar = new GraphQLScalarType({
  name: 'DateTime',
  description: 'DateTime custom scalar type (ISO 8601)',

  serialize(value: unknown): string {
    if (value instanceof Date) {
      return value.toISOString();
    }
    if (typeof value === 'string') {
      return new Date(value).toISOString();
    }
    if (typeof value === 'number') {
      return new Date(value).toISOString();
    }
    throw new GraphQLError('DateTime cannot represent non-date value');
  },

  parseValue(value: unknown): Date {
    if (typeof value === 'string' || typeof value === 'number') {
      const date = new Date(value);
      if (isNaN(date.getTime())) {
        throw new GraphQLError('DateTime cannot represent invalid date string');
      }
      return date;
    }
    throw new GraphQLError('DateTime cannot represent non-string/number value');
  },

  parseLiteral(ast: ValueNode): Date {
    if (ast.kind === Kind.STRING || ast.kind === Kind.INT) {
      const date = new Date(ast.value);
      if (isNaN(date.getTime())) {
        throw new GraphQLError('DateTime cannot represent invalid date literal');
      }
      return date;
    }
    throw new GraphQLError('DateTime cannot represent non-string/number literal');
  },
});

// ============================================================================
// JSON Scalar
// ============================================================================

export const JSONScalar = new GraphQLScalarType({
  name: 'JSON',
  description: 'The JSON scalar type represents JSON values as specified by ECMA-404',

  serialize(value: unknown): unknown {
    return value;
  },

  parseValue(value: unknown): unknown {
    return value;
  },

  parseLiteral(ast: ValueNode): unknown {
    switch (ast.kind) {
      case Kind.STRING:
      case Kind.BOOLEAN:
        return ast.value;
      case Kind.INT:
      case Kind.FLOAT:
        return parseFloat(ast.value);
      case Kind.OBJECT:
        return parseObject(ast);
      case Kind.LIST:
        return ast.values.map((n) => JSONScalar.parseLiteral(n));
      case Kind.NULL:
        return null;
      default:
        throw new GraphQLError(`JSON cannot represent value: ${ast.kind}`);
    }
  },
});

function parseObject(ast: any): Record<string, unknown> {
  const value: Record<string, unknown> = {};
  ast.fields.forEach((field: any) => {
    value[field.name.value] = JSONScalar.parseLiteral(field.value);
  });
  return value;
}

// ============================================================================
// JSONObject Scalar
// ============================================================================

export const JSONObjectScalar = new GraphQLScalarType({
  name: 'JSONObject',
  description: 'The JSONObject scalar type represents JSON objects',

  serialize(value: unknown): Record<string, unknown> {
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
      throw new GraphQLError('JSONObject cannot represent non-object value');
    }
    return value as Record<string, unknown>;
  },

  parseValue(value: unknown): Record<string, unknown> {
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
      throw new GraphQLError('JSONObject cannot represent non-object value');
    }
    return value as Record<string, unknown>;
  },

  parseLiteral(ast: ValueNode): Record<string, unknown> {
    if (ast.kind !== Kind.OBJECT) {
      throw new GraphQLError('JSONObject cannot represent non-object literal');
    }
    return parseObject(ast);
  },
});

// ============================================================================
// URL Scalar
// ============================================================================

export const URLScalar = new GraphQLScalarType({
  name: 'URL',
  description: 'A valid URL string',

  serialize(value: unknown): string {
    if (typeof value !== 'string') {
      throw new GraphQLError('URL cannot represent non-string value');
    }

    try {
      new URL(value);
      return value;
    } catch {
      throw new GraphQLError('URL cannot represent invalid URL string');
    }
  },

  parseValue(value: unknown): string {
    if (typeof value !== 'string') {
      throw new GraphQLError('URL cannot represent non-string value');
    }

    try {
      new URL(value);
      return value;
    } catch {
      throw new GraphQLError('URL cannot represent invalid URL string');
    }
  },

  parseLiteral(ast: ValueNode): string {
    if (ast.kind !== Kind.STRING) {
      throw new GraphQLError('URL cannot represent non-string literal');
    }

    try {
      new URL(ast.value);
      return ast.value;
    } catch {
      throw new GraphQLError('URL cannot represent invalid URL literal');
    }
  },
});

// ============================================================================
// Email Scalar
// ============================================================================

const EMAIL_REGEX = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;

export const EmailScalar = new GraphQLScalarType({
  name: 'Email',
  description: 'A valid email address',

  serialize(value: unknown): string {
    if (typeof value !== 'string') {
      throw new GraphQLError('Email cannot represent non-string value');
    }

    if (!EMAIL_REGEX.test(value)) {
      throw new GraphQLError('Email cannot represent invalid email address');
    }

    return value.toLowerCase();
  },

  parseValue(value: unknown): string {
    if (typeof value !== 'string') {
      throw new GraphQLError('Email cannot represent non-string value');
    }

    if (!EMAIL_REGEX.test(value)) {
      throw new GraphQLError('Email cannot represent invalid email address');
    }

    return value.toLowerCase();
  },

  parseLiteral(ast: ValueNode): string {
    if (ast.kind !== Kind.STRING) {
      throw new GraphQLError('Email cannot represent non-string literal');
    }

    if (!EMAIL_REGEX.test(ast.value)) {
      throw new GraphQLError('Email cannot represent invalid email literal');
    }

    return ast.value.toLowerCase();
  },
});

// ============================================================================
// PositiveInt Scalar
// ============================================================================

export const PositiveIntScalar = new GraphQLScalarType({
  name: 'PositiveInt',
  description: 'Integers that are greater than 0',

  serialize(value: unknown): number {
    if (typeof value !== 'number' || !Number.isInteger(value) || value <= 0) {
      throw new GraphQLError('PositiveInt cannot represent non-positive-integer value');
    }
    return value;
  },

  parseValue(value: unknown): number {
    if (typeof value !== 'number' || !Number.isInteger(value) || value <= 0) {
      throw new GraphQLError('PositiveInt cannot represent non-positive-integer value');
    }
    return value;
  },

  parseLiteral(ast: ValueNode): number {
    if (ast.kind !== Kind.INT) {
      throw new GraphQLError('PositiveInt cannot represent non-integer literal');
    }

    const value = parseInt(ast.value, 10);
    if (value <= 0) {
      throw new GraphQLError('PositiveInt cannot represent non-positive literal');
    }

    return value;
  },
});

// ============================================================================
// PositiveFloat Scalar
// ============================================================================

export const PositiveFloatScalar = new GraphQLScalarType({
  name: 'PositiveFloat',
  description: 'Floats that are greater than 0',

  serialize(value: unknown): number {
    if (typeof value !== 'number' || value <= 0) {
      throw new GraphQLError('PositiveFloat cannot represent non-positive-float value');
    }
    return value;
  },

  parseValue(value: unknown): number {
    if (typeof value !== 'number' || value <= 0) {
      throw new GraphQLError('PositiveFloat cannot represent non-positive-float value');
    }
    return value;
  },

  parseLiteral(ast: ValueNode): number {
    if (ast.kind !== Kind.FLOAT && ast.kind !== Kind.INT) {
      throw new GraphQLError('PositiveFloat cannot represent non-float literal');
    }

    const value = parseFloat(ast.value);
    if (value <= 0) {
      throw new GraphQLError('PositiveFloat cannot represent non-positive literal');
    }

    return value;
  },
});

// ============================================================================
// Upload Scalar (for file uploads)
// ============================================================================

export const UploadScalar = new GraphQLScalarType({
  name: 'Upload',
  description: 'The Upload scalar type represents a file upload',

  serialize(value: unknown): never {
    throw new GraphQLError('Upload serialization is not supported');
  },

  parseValue(value: unknown): unknown {
    return value;
  },

  parseLiteral(ast: ValueNode): never {
    throw new GraphQLError('Upload literal is not supported');
  },
});

// ============================================================================
// Export all scalars
// ============================================================================

export const customScalars = {
  DateTime: DateTimeScalar,
  JSON: JSONScalar,
  JSONObject: JSONObjectScalar,
  URL: URLScalar,
  Email: EmailScalar,
  PositiveInt: PositiveIntScalar,
  PositiveFloat: PositiveFloatScalar,
  Upload: UploadScalar,
};
