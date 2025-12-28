/**
 * Report Validation Schemas
 * AccuScene Enterprise Accident Recreation Platform
 */

import { z } from 'zod';
import { uuidSchema } from '../middleware/validator';

/**
 * Report type enum
 */
export enum ReportType {
  ACCIDENT_SUMMARY = 'ACCIDENT_SUMMARY',
  DETAILED_ANALYSIS = 'DETAILED_ANALYSIS',
  VEHICLE_DAMAGE = 'VEHICLE_DAMAGE',
  EVIDENCE_CATALOG = 'EVIDENCE_CATALOG',
  PHYSICS_SIMULATION = 'PHYSICS_SIMULATION',
  TIMELINE = 'TIMELINE',
  CUSTOM = 'CUSTOM',
}

/**
 * Report format enum
 */
export enum ReportFormat {
  PDF = 'PDF',
  DOCX = 'DOCX',
  HTML = 'HTML',
  JSON = 'JSON',
}

/**
 * Generate report schema
 */
export const generateReportSchema = z.object({
  caseId: uuidSchema,
  type: z.nativeEnum(ReportType),
  format: z.nativeEnum(ReportFormat).default(ReportFormat.PDF),
  title: z.string().min(1).max(200),
  description: z.string().optional(),
  includeEvidence: z.boolean().default(true),
  includeDiagrams: z.boolean().default(true),
  includeVehicleDetails: z.boolean().default(true),
  includePhysicsAnalysis: z.boolean().default(true),
  includePhotos: z.boolean().default(true),
  sections: z.array(z.string()).optional(),
  customFields: z.record(z.any()).optional(),
  template: z.string().optional(),
  watermark: z.string().optional(),
  confidential: z.boolean().default(true),
});

/**
 * Update report schema
 */
export const updateReportSchema = z.object({
  title: z.string().min(1).max(200).optional(),
  description: z.string().optional(),
  status: z.enum(['GENERATING', 'COMPLETED', 'FAILED']).optional(),
  notes: z.string().optional(),
});

/**
 * Report search schema
 */
export const reportSearchSchema = z.object({
  caseId: uuidSchema.optional(),
  type: z.nativeEnum(ReportType).optional(),
  format: z.nativeEnum(ReportFormat).optional(),
  status: z.enum(['GENERATING', 'COMPLETED', 'FAILED']).optional(),
  q: z.string().optional(),
  fromDate: z.coerce.date().optional(),
  toDate: z.coerce.date().optional(),
  page: z.coerce.number().int().min(1).default(1),
  limit: z.coerce.number().int().min(1).max(100).default(20),
  sortBy: z.enum(['title', 'createdAt', 'type']).default('createdAt'),
  sortOrder: z.enum(['asc', 'desc']).default('desc'),
});

/**
 * Report ID param schema
 */
export const reportIdParamSchema = z.object({
  reportId: uuidSchema,
});

/**
 * Regenerate report schema
 */
export const regenerateReportSchema = z.object({
  format: z.nativeEnum(ReportFormat).optional(),
  includeEvidence: z.boolean().optional(),
  includeDiagrams: z.boolean().optional(),
  includeVehicleDetails: z.boolean().optional(),
  includePhysicsAnalysis: z.boolean().optional(),
  includePhotos: z.boolean().optional(),
});
