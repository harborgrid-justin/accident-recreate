/**
 * Reports Routes
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Router } from 'express';
import * as reportsController from '../controllers/reports.controller';
import { validate, validateMultiple } from '../middleware/validator';
import { authenticate, authorize } from '../middleware/auth';
import { UserRole } from '../../auth/types';
import {
  generateReportSchema,
  reportSearchSchema,
  reportIdParamSchema,
  regenerateReportSchema,
} from '../validators/report.schemas';
import { uuidSchema } from '../middleware/validator';
import { z } from 'zod';

const router = Router();

// All routes require authentication
router.use(authenticate);

/**
 * @route   GET /api/reports
 * @desc    Get all reports (paginated, filterable)
 * @access  Private (All authenticated users)
 */
router.get(
  '/',
  validate(reportSearchSchema, 'query'),
  reportsController.getAllReports
);

/**
 * @route   GET /api/reports/case/:caseId
 * @desc    Get reports by case ID
 * @access  Private (All authenticated users)
 */
router.get(
  '/case/:caseId',
  validate(z.object({ caseId: uuidSchema }), 'params'),
  reportsController.getReportsByCase
);

/**
 * @route   GET /api/reports/:reportId
 * @desc    Get report by ID
 * @access  Private (All authenticated users)
 */
router.get(
  '/:reportId',
  validate(reportIdParamSchema, 'params'),
  reportsController.getReportById
);

/**
 * @route   GET /api/reports/:reportId/status
 * @desc    Get report generation status
 * @access  Private (All authenticated users)
 */
router.get(
  '/:reportId/status',
  validate(reportIdParamSchema, 'params'),
  reportsController.getReportStatus
);

/**
 * @route   POST /api/reports/generate
 * @desc    Generate new report
 * @access  Private (Admin/Investigator)
 */
router.post(
  '/generate',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validate(generateReportSchema),
  reportsController.generateReport
);

/**
 * @route   POST /api/reports/:reportId/regenerate
 * @desc    Regenerate existing report
 * @access  Private (Admin/Investigator)
 */
router.post(
  '/:reportId/regenerate',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validateMultiple({
    params: reportIdParamSchema,
    body: regenerateReportSchema,
  }),
  reportsController.regenerateReport
);

/**
 * @route   GET /api/reports/:reportId/download
 * @desc    Download report file
 * @access  Private (All authenticated users)
 */
router.get(
  '/:reportId/download',
  validate(reportIdParamSchema, 'params'),
  reportsController.downloadReport
);

/**
 * @route   DELETE /api/reports/:reportId
 * @desc    Delete report
 * @access  Private (Admin only)
 */
router.delete(
  '/:reportId',
  authorize(UserRole.ADMIN),
  validate(reportIdParamSchema, 'params'),
  reportsController.deleteReport
);

export default router;
