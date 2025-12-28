/**
 * Evidence Validation Schemas
 * AccuScene Enterprise Accident Recreation Platform
 */

import { z } from 'zod';
import { uuidSchema } from '../middleware/validator';

/**
 * Evidence type enum
 */
export enum EvidenceType {
  PHOTO = 'PHOTO',
  VIDEO = 'VIDEO',
  DOCUMENT = 'DOCUMENT',
  AUDIO = 'AUDIO',
  OTHER = 'OTHER',
}

/**
 * Create evidence metadata schema
 */
export const createEvidenceSchema = z.object({
  caseId: uuidSchema,
  type: z.nativeEnum(EvidenceType),
  title: z.string().min(1).max(200),
  description: z.string().optional(),
  tags: z.array(z.string()).default([]),
  capturedAt: z.coerce.date().optional(),
  location: z.object({
    latitude: z.number().min(-90).max(90),
    longitude: z.number().min(-180).max(180),
  }).optional(),
  metadata: z.record(z.any()).optional(),
});

/**
 * Update evidence schema
 */
export const updateEvidenceSchema = z.object({
  title: z.string().min(1).max(200).optional(),
  description: z.string().optional(),
  tags: z.array(z.string()).optional(),
  capturedAt: z.coerce.date().optional(),
  location: z.object({
    latitude: z.number().min(-90).max(90),
    longitude: z.number().min(-180).max(180),
  }).optional(),
  metadata: z.record(z.any()).optional(),
});

/**
 * Evidence search schema
 */
export const evidenceSearchSchema = z.object({
  caseId: uuidSchema.optional(),
  type: z.nativeEnum(EvidenceType).optional(),
  q: z.string().optional(),
  tags: z.string().optional(), // Comma-separated
  fromDate: z.coerce.date().optional(),
  toDate: z.coerce.date().optional(),
  page: z.coerce.number().int().min(1).default(1),
  limit: z.coerce.number().int().min(1).max(100).default(20),
  sortBy: z.enum(['title', 'capturedAt', 'createdAt', 'type']).default('createdAt'),
  sortOrder: z.enum(['asc', 'desc']).default('desc'),
});

/**
 * Evidence ID param schema
 */
export const evidenceIdParamSchema = z.object({
  evidenceId: uuidSchema,
});

/**
 * Batch upload schema
 */
export const batchUploadSchema = z.object({
  caseId: uuidSchema,
  files: z.array(z.object({
    filename: z.string(),
    type: z.nativeEnum(EvidenceType),
    title: z.string().min(1).max(200),
    description: z.string().optional(),
  })),
});
