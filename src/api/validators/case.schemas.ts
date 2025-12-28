/**
 * Case Validation Schemas
 * AccuScene Enterprise Accident Recreation Platform
 */

import { z } from 'zod';
import { CaseStatus } from '../../cases/CaseStatus';
import { uuidSchema } from '../middleware/validator';

/**
 * Create case schema
 */
export const createCaseSchema = z.object({
  caseNumber: z.string().min(1).max(50),
  title: z.string().min(1).max(200),
  description: z.string().optional(),
  incidentDate: z.coerce.date(),
  location: z.object({
    address: z.string().min(1),
    city: z.string().min(1),
    state: z.string().min(2).max(2),
    zipCode: z.string().regex(/^\d{5}(-\d{4})?$/),
    coordinates: z.object({
      latitude: z.number().min(-90).max(90),
      longitude: z.number().min(-180).max(180),
    }).optional(),
  }),
  assignedToUserId: uuidSchema.optional(),
  weatherConditions: z.string().optional(),
  roadConditions: z.string().optional(),
  lightingConditions: z.string().optional(),
  priority: z.enum(['LOW', 'MEDIUM', 'HIGH', 'CRITICAL']).default('MEDIUM'),
  tags: z.array(z.string()).default([]),
});

/**
 * Update case schema
 */
export const updateCaseSchema = z.object({
  title: z.string().min(1).max(200).optional(),
  description: z.string().optional(),
  incidentDate: z.coerce.date().optional(),
  location: z.object({
    address: z.string().min(1),
    city: z.string().min(1),
    state: z.string().min(2).max(2),
    zipCode: z.string().regex(/^\d{5}(-\d{4})?$/),
    coordinates: z.object({
      latitude: z.number().min(-90).max(90),
      longitude: z.number().min(-180).max(180),
    }).optional(),
  }).optional(),
  assignedToUserId: uuidSchema.optional().nullable(),
  weatherConditions: z.string().optional(),
  roadConditions: z.string().optional(),
  lightingConditions: z.string().optional(),
  priority: z.enum(['LOW', 'MEDIUM', 'HIGH', 'CRITICAL']).optional(),
  tags: z.array(z.string()).optional(),
});

/**
 * Update case status schema
 */
export const updateCaseStatusSchema = z.object({
  status: z.nativeEnum(CaseStatus),
  notes: z.string().optional(),
});

/**
 * Case search/filter schema
 */
export const caseSearchSchema = z.object({
  q: z.string().optional(),
  status: z.nativeEnum(CaseStatus).optional(),
  assignedToUserId: uuidSchema.optional(),
  priority: z.enum(['LOW', 'MEDIUM', 'HIGH', 'CRITICAL']).optional(),
  fromDate: z.coerce.date().optional(),
  toDate: z.coerce.date().optional(),
  tags: z.string().optional(), // Comma-separated
  page: z.coerce.number().int().min(1).default(1),
  limit: z.coerce.number().int().min(1).max(100).default(20),
  sortBy: z.enum(['caseNumber', 'title', 'incidentDate', 'createdAt', 'priority']).default('createdAt'),
  sortOrder: z.enum(['asc', 'desc']).default('desc'),
});

/**
 * Case ID param schema
 */
export const caseIdParamSchema = z.object({
  caseId: uuidSchema,
});

/**
 * Assign case schema
 */
export const assignCaseSchema = z.object({
  assignedToUserId: uuidSchema,
});
