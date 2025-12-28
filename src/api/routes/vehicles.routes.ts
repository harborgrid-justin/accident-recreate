/**
 * Vehicles Routes
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Router } from 'express';
import * as vehiclesController from '../controllers/vehicles.controller';
import { validate, validateMultiple } from '../middleware/validator';
import { authenticate, authorize } from '../middleware/auth';
import { UserRole } from '../../auth/types';
import {
  createVehicleSchema,
  updateVehicleSchema,
  vehiclePhysicsSchema,
  vehicleSearchSchema,
  vehicleIdParamSchema,
} from '../validators/vehicle.schemas';

const router = Router();

// All routes require authentication
router.use(authenticate);

/**
 * @route   GET /api/vehicles
 * @desc    Get all vehicles (paginated, filterable)
 * @access  Private (All authenticated users)
 */
router.get(
  '/',
  validate(vehicleSearchSchema, 'query'),
  vehiclesController.getAllVehicles
);

/**
 * @route   GET /api/vehicles/:vehicleId
 * @desc    Get vehicle by ID
 * @access  Private (All authenticated users)
 */
router.get(
  '/:vehicleId',
  validate(vehicleIdParamSchema, 'params'),
  vehiclesController.getVehicleById
);

/**
 * @route   POST /api/vehicles
 * @desc    Create new vehicle
 * @access  Private (Admin/Investigator)
 */
router.post(
  '/',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validate(createVehicleSchema),
  vehiclesController.createVehicle
);

/**
 * @route   PUT /api/vehicles/:vehicleId
 * @desc    Update vehicle
 * @access  Private (Admin/Investigator)
 */
router.put(
  '/:vehicleId',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validateMultiple({
    params: vehicleIdParamSchema,
    body: updateVehicleSchema,
  }),
  vehiclesController.updateVehicle
);

/**
 * @route   DELETE /api/vehicles/:vehicleId
 * @desc    Delete vehicle
 * @access  Private (Admin/Investigator)
 */
router.delete(
  '/:vehicleId',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validate(vehicleIdParamSchema, 'params'),
  vehiclesController.deleteVehicle
);

/**
 * @route   POST /api/vehicles/:vehicleId/simulate
 * @desc    Run physics simulation for vehicle
 * @access  Private (Admin/Investigator)
 */
router.post(
  '/:vehicleId/simulate',
  authorize(UserRole.ADMIN, UserRole.INVESTIGATOR),
  validateMultiple({
    params: vehicleIdParamSchema,
    body: vehiclePhysicsSchema,
  }),
  vehiclesController.simulatePhysics
);

export default router;
