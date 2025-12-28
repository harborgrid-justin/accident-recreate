/**
 * User Validation Schemas
 * AccuScene Enterprise Accident Recreation Platform
 */

import { z } from 'zod';
import { UserRole } from '../../auth/types';
import { emailSchema, uuidSchema } from '../middleware/validator';

/**
 * Create user schema
 */
export const createUserSchema = z.object({
  email: emailSchema,
  password: z.string().min(8, 'Password must be at least 8 characters'),
  role: z.nativeEnum(UserRole),
  firstName: z.string().min(1).max(50).optional(),
  lastName: z.string().min(1).max(50).optional(),
  isActive: z.boolean().default(true),
});

/**
 * Update user schema
 */
export const updateUserSchema = z.object({
  email: emailSchema.optional(),
  role: z.nativeEnum(UserRole).optional(),
  firstName: z.string().min(1).max(50).optional(),
  lastName: z.string().min(1).max(50).optional(),
  isActive: z.boolean().optional(),
});

/**
 * Update profile schema
 */
export const updateProfileSchema = z.object({
  firstName: z.string().min(1).max(50).optional(),
  lastName: z.string().min(1).max(50).optional(),
});

/**
 * User filter/search query schema
 */
export const userSearchSchema = z.object({
  q: z.string().optional(),
  role: z.nativeEnum(UserRole).optional(),
  isActive: z.coerce.boolean().optional(),
  page: z.coerce.number().int().min(1).default(1),
  limit: z.coerce.number().int().min(1).max(100).default(20),
  sortBy: z.enum(['email', 'createdAt', 'lastLogin']).default('createdAt'),
  sortOrder: z.enum(['asc', 'desc']).default('desc'),
});

/**
 * User ID param schema
 */
export const userIdParamSchema = z.object({
  userId: uuidSchema,
});
