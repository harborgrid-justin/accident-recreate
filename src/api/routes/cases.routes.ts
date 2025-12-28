/**
 * Cases Routes
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Router } from 'express';
import * as casesController from '../controllers/cases.controller';
import { validate, validateMultiple } from '../middleware/validator';
import { authenticate, authorize } from '../middleware/auth';
import { UserRole } from '../../auth/types';
import {
  createCaseSchema,
  updateCaseSchema,
  updateCaseStatusSchema,
  caseSearchSchema,
  caseIdParamSchema,
  assignCaseSchema,
} from '../validators/case.schemas';

const router = Router();

// All routes require authentication
router.use(authenticate);

/**
 * @route   GET /api/cases/search
 * @desc    Search cases
 * @access  Private (All authenticated users)
 */
router.get(
  '/search',
  validate(caseSearchSchema, 'query'),
  casesController.searchCases
);

/**
 * @route   GET /api/cases
 * @desc    Get all cases (paginated, filterable)
 * @access  Private (All authenticated users)
 */
router.get(
  '/',
  validate(caseSearchSchema, 'query'),
  casesController.getAllCases
);

/**
 * @route   GET /api/cases/:caseId
 * @desc    Get case by ID
 * @access  Private (All authenticated users)
 */
router.get(
  '/:caseId',
  validate(caseIdParamSchema, 'params'),
  casesController.getCaseById
);

/**
 * @route   POST /api/cases
 * @desc    Create new case
 * @access  Private (Admin/Investigator)
 */
router.post(
  '/',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validate(createCaseSchema),
  casesController.createCase
);

/**
 * @route   PUT /api/cases/:caseId
 * @desc    Update case
 * @access  Private (Admin/Investigator)
 */
router.put(
  '/:caseId',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validateMultiple({
    params: caseIdParamSchema,
    body: updateCaseSchema,
  }),
  casesController.updateCase
);

/**
 * @route   DELETE /api/cases/:caseId
 * @desc    Delete case
 * @access  Private (Admin only)
 */
router.delete(
  '/:caseId',
  authorize(UserRole.ADMIN),
  validate(caseIdParamSchema, 'params'),
  casesController.deleteCase
);

/**
 * @route   PATCH /api/cases/:caseId/status
 * @desc    Update case status
 * @access  Private (Admin/Investigator)
 */
router.patch(
  '/:caseId/status',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validateMultiple({
    params: caseIdParamSchema,
    body: updateCaseStatusSchema,
  }),
  casesController.updateCaseStatus
);

/**
 * @route   POST /api/cases/:caseId/assign
 * @desc    Assign case to user
 * @access  Private (Admin/Investigator)
 */
router.post(
  '/:caseId/assign',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validateMultiple({
    params: caseIdParamSchema,
    body: assignCaseSchema,
  }),
  casesController.assignCase
);

export default router;
