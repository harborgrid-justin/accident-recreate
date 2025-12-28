/**
 * Accident Validation Schemas
 * AccuScene Enterprise Accident Recreation Platform
 */

import { z } from 'zod';
import { uuidSchema } from '../middleware/validator';

/**
 * Create accident schema
 */
export const createAccidentSchema = z.object({
  caseId: uuidSchema,
  name: z.string().min(1).max(100),
  description: z.string().optional(),
  diagram: z.object({
    width: z.number().positive(),
    height: z.number().positive(),
    scale: z.number().positive().default(1),
    gridEnabled: z.boolean().default(true),
    gridSize: z.number().positive().default(10),
    backgroundImage: z.string().optional(),
  }).optional(),
});

/**
 * Update accident schema
 */
export const updateAccidentSchema = z.object({
  name: z.string().min(1).max(100).optional(),
  description: z.string().optional(),
  diagram: z.object({
    width: z.number().positive(),
    height: z.number().positive(),
    scale: z.number().positive(),
    gridEnabled: z.boolean(),
    gridSize: z.number().positive(),
    backgroundImage: z.string().optional(),
  }).optional(),
});

/**
 * Update diagram schema
 */
export const updateDiagramSchema = z.object({
  data: z.any(), // JSON diagram data
  thumbnail: z.string().optional(), // Base64 thumbnail
});

/**
 * Accident search schema
 */
export const accidentSearchSchema = z.object({
  caseId: uuidSchema.optional(),
  q: z.string().optional(),
  page: z.coerce.number().int().min(1).default(1),
  limit: z.coerce.number().int().min(1).max(100).default(20),
  sortBy: z.enum(['name', 'createdAt', 'updatedAt']).default('createdAt'),
  sortOrder: z.enum(['asc', 'desc']).default('desc'),
});

/**
 * Accident ID param schema
 */
export const accidentIdParamSchema = z.object({
  accidentId: uuidSchema,
});

/**
 * Export diagram schema
 */
export const exportDiagramSchema = z.object({
  format: z.enum(['png', 'jpg', 'svg', 'pdf']).default('png'),
  quality: z.coerce.number().min(0.1).max(1).default(1),
  width: z.coerce.number().positive().optional(),
  height: z.coerce.number().positive().optional(),
});
