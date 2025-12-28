/**
 * Accidents Routes
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Router } from 'express';
import * as accidentsController from '../controllers/accidents.controller';
import { validate, validateMultiple } from '../middleware/validator';
import { authenticate, authorize } from '../middleware/auth';
import { UserRole } from '../../auth/types';
import {
  createAccidentSchema,
  updateAccidentSchema,
  updateDiagramSchema,
  accidentSearchSchema,
  accidentIdParamSchema,
  exportDiagramSchema,
} from '../validators/accident.schemas';

const router = Router();

// All routes require authentication
router.use(authenticate);

/**
 * @route   GET /api/accidents
 * @desc    Get all accidents (paginated, filterable)
 * @access  Private (All authenticated users)
 */
router.get(
  '/',
  validate(accidentSearchSchema, 'query'),
  accidentsController.getAllAccidents
);

/**
 * @route   GET /api/accidents/:accidentId
 * @desc    Get accident by ID
 * @access  Private (All authenticated users)
 */
router.get(
  '/:accidentId',
  validate(accidentIdParamSchema, 'params'),
  accidentsController.getAccidentById
);

/**
 * @route   POST /api/accidents
 * @desc    Create new accident
 * @access  Private (Admin/Investigator)
 */
router.post(
  '/',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validate(createAccidentSchema),
  accidentsController.createAccident
);

/**
 * @route   PUT /api/accidents/:accidentId
 * @desc    Update accident
 * @access  Private (Admin/Investigator)
 */
router.put(
  '/:accidentId',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validateMultiple({
    params: accidentIdParamSchema,
    body: updateAccidentSchema,
  }),
  accidentsController.updateAccident
);

/**
 * @route   DELETE /api/accidents/:accidentId
 * @desc    Delete accident
 * @access  Private (Admin/Investigator)
 */
router.delete(
  '/:accidentId',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validate(accidentIdParamSchema, 'params'),
  accidentsController.deleteAccident
);

/**
 * @route   PUT /api/accidents/:accidentId/diagram
 * @desc    Update accident diagram data
 * @access  Private (Admin/Investigator)
 */
router.put(
  '/:accidentId/diagram',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validateMultiple({
    params: accidentIdParamSchema,
    body: updateDiagramSchema,
  }),
  accidentsController.updateDiagram
);

/**
 * @route   POST /api/accidents/:accidentId/export
 * @desc    Export accident diagram
 * @access  Private (All authenticated users)
 */
router.post(
  '/:accidentId/export',
  validateMultiple({
    params: accidentIdParamSchema,
    body: exportDiagramSchema,
  }),
  accidentsController.exportDiagram
);

export default router;
