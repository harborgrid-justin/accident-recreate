/**
 * Request Validation Middleware
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Request, Response, NextFunction } from 'express';
import { z, ZodSchema, ZodError } from 'zod';
import { ValidationError } from '../../auth/types';

export type ValidationType = 'body' | 'query' | 'params';

/**
 * Validate request data using Zod schema
 */
export function validate(schema: ZodSchema, type: ValidationType = 'body') {
  return (req: Request, res: Response, next: NextFunction): void => {
    try {
      const data = type === 'body' ? req.body : type === 'query' ? req.query : req.params;

      // Parse and validate
      const validated = schema.parse(data);

      // Replace request data with validated data
      if (type === 'body') {
        req.body = validated;
      } else if (type === 'query') {
        req.query = validated as any;
      } else {
        req.params = validated as any;
      }

      next();
    } catch (err) {
      if (err instanceof ZodError) {
        const fields: Record<string, string> = {};

        err.errors.forEach(error => {
          const path = error.path.join('.');
          fields[path] = error.message;
        });

        next(new ValidationError('Validation failed', fields));
      } else {
        next(err);
      }
    }
  };
}

/**
 * Validate multiple parts of request
 */
export function validateMultiple(schemas: Partial<Record<ValidationType, ZodSchema>>) {
  return (req: Request, res: Response, next: NextFunction): void => {
    try {
      const errors: Record<string, string> = {};

      // Validate each part
      for (const [type, schema] of Object.entries(schemas) as [ValidationType, ZodSchema][]) {
        try {
          const data = type === 'body' ? req.body : type === 'query' ? req.query : req.params;
          const validated = schema.parse(data);

          // Replace request data with validated data
          if (type === 'body') {
            req.body = validated;
          } else if (type === 'query') {
            req.query = validated as any;
          } else {
            req.params = validated as any;
          }
        } catch (err) {
          if (err instanceof ZodError) {
            err.errors.forEach(error => {
              const path = `${type}.${error.path.join('.')}`;
              errors[path] = error.message;
            });
          }
        }
      }

      // If any errors, throw validation error
      if (Object.keys(errors).length > 0) {
        throw new ValidationError('Validation failed', errors);
      }

      next();
    } catch (err) {
      next(err);
    }
  };
}

/**
 * Common validation schemas
 */
export const commonSchemas = {
  // ID parameter
  id: z.object({
    id: z.string().uuid('Invalid ID format'),
  }),

  // Pagination query
  pagination: z.object({
    page: z.coerce.number().int().min(1).default(1),
    limit: z.coerce.number().int().min(1).max(100).default(20),
  }),

  // Search query
  search: z.object({
    q: z.string().min(1).optional(),
    page: z.coerce.number().int().min(1).default(1),
    limit: z.coerce.number().int().min(1).max(100).default(20),
  }),

  // Date range query
  dateRange: z.object({
    startDate: z.coerce.date().optional(),
    endDate: z.coerce.date().optional(),
  }),

  // Sort query
  sort: z.object({
    sortBy: z.string().optional(),
    sortOrder: z.enum(['asc', 'desc']).default('desc'),
  }),
};

/**
 * Sanitize string input
 */
export function sanitizeString(str: string): string {
  return str
    .trim()
    .replace(/[<>]/g, '') // Remove potential HTML tags
    .replace(/[\x00-\x1F\x7F]/g, ''); // Remove control characters
}

/**
 * Sanitize object recursively
 */
export function sanitizeObject(obj: any): any {
  if (typeof obj === 'string') {
    return sanitizeString(obj);
  }

  if (Array.isArray(obj)) {
    return obj.map(item => sanitizeObject(item));
  }

  if (obj && typeof obj === 'object') {
    const sanitized: any = {};
    for (const key in obj) {
      sanitized[key] = sanitizeObject(obj[key]);
    }
    return sanitized;
  }

  return obj;
}

/**
 * Input sanitization middleware
 */
export function sanitizeInput(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  if (req.body) {
    req.body = sanitizeObject(req.body);
  }

  if (req.query) {
    req.query = sanitizeObject(req.query);
  }

  if (req.params) {
    req.params = sanitizeObject(req.params);
  }

  next();
}

/**
 * Email validation
 */
export const emailSchema = z.string().email('Invalid email address').toLowerCase();

/**
 * Password validation
 */
export const passwordSchema = z
  .string()
  .min(8, 'Password must be at least 8 characters')
  .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
  .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
  .regex(/[0-9]/, 'Password must contain at least one number')
  .regex(/[^a-zA-Z0-9]/, 'Password must contain at least one special character');

/**
 * UUID validation
 */
export const uuidSchema = z.string().uuid('Invalid UUID format');
