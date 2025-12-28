/**
 * Evidence Routes
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Router } from 'express';
import * as evidenceController from '../controllers/evidence.controller';
import { validate, validateMultiple } from '../middleware/validator';
import { authenticate, authorize } from '../middleware/auth';
import { uploadMultiple } from '../middleware/upload';
import { UserRole } from '../../auth/types';
import {
  createEvidenceSchema,
  updateEvidenceSchema,
  evidenceSearchSchema,
  evidenceIdParamSchema,
} from '../validators/evidence.schemas';
import { uuidSchema } from '../middleware/validator';
import { z } from 'zod';

const router = Router();

// All routes require authentication
router.use(authenticate);

/**
 * @route   GET /api/evidence
 * @desc    Get all evidence (paginated, filterable)
 * @access  Private (All authenticated users)
 */
router.get(
  '/',
  validate(evidenceSearchSchema, 'query'),
  evidenceController.getAllEvidence
);

/**
 * @route   GET /api/evidence/case/:caseId
 * @desc    Get evidence by case ID
 * @access  Private (All authenticated users)
 */
router.get(
  '/case/:caseId',
  validate(z.object({ caseId: uuidSchema }), 'params'),
  evidenceController.getEvidenceByCase
);

/**
 * @route   GET /api/evidence/:evidenceId
 * @desc    Get evidence by ID
 * @access  Private (All authenticated users)
 */
router.get(
  '/:evidenceId',
  validate(evidenceIdParamSchema, 'params'),
  evidenceController.getEvidenceById
);

/**
 * @route   POST /api/evidence/upload
 * @desc    Upload evidence files
 * @access  Private (Admin/Investigator)
 */
router.post(
  '/upload',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  uploadMultiple('files', 10),
  validate(createEvidenceSchema),
  evidenceController.uploadEvidence
);

/**
 * @route   PUT /api/evidence/:evidenceId
 * @desc    Update evidence metadata
 * @access  Private (Admin/Investigator)
 */
router.put(
  '/:evidenceId',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validateMultiple({
    params: evidenceIdParamSchema,
    body: updateEvidenceSchema,
  }),
  evidenceController.updateEvidence
);

/**
 * @route   DELETE /api/evidence/:evidenceId
 * @desc    Delete evidence
 * @access  Private (Admin only)
 */
router.delete(
  '/:evidenceId',
  authorize(UserRole.ADMIN),
  validate(evidenceIdParamSchema, 'params'),
  evidenceController.deleteEvidence
);

/**
 * @route   GET /api/evidence/:evidenceId/download
 * @desc    Download evidence file
 * @access  Private (All authenticated users)
 */
router.get(
  '/:evidenceId/download',
  validate(evidenceIdParamSchema, 'params'),
  evidenceController.downloadEvidence
);

export default router;
