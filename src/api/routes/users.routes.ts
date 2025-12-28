/**
 * Users Routes
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Router } from 'express';
import * as usersController from '../controllers/users.controller';
import { validate, validateMultiple } from '../middleware/validator';
import { authenticate, requireAdmin, requireOwnershipOrAdmin } from '../middleware/auth';
import {
  createUserSchema,
  updateUserSchema,
  updateProfileSchema,
  userSearchSchema,
  userIdParamSchema,
} from '../validators/user.schemas';

const router = Router();

// All routes require authentication
router.use(authenticate);

/**
 * @route   GET /api/users/profile
 * @desc    Get current user profile
 * @access  Private
 */
router.get('/profile', usersController.getProfile);

/**
 * @route   PUT /api/users/profile
 * @desc    Update current user profile
 * @access  Private
 */
router.put(
  '/profile',
  validate(updateProfileSchema),
  usersController.updateProfile
);

/**
 * @route   GET /api/users
 * @desc    Get all users (paginated, filterable)
 * @access  Private (Admin/Investigator)
 */
router.get(
  '/',
  validate(userSearchSchema, 'query'),
  usersController.getAllUsers
);

/**
 * @route   GET /api/users/:userId
 * @desc    Get user by ID
 * @access  Private (Admin or own profile)
 */
router.get(
  '/:userId',
  validate(userIdParamSchema, 'params'),
  requireOwnershipOrAdmin('userId'),
  usersController.getUserById
);

/**
 * @route   POST /api/users
 * @desc    Create new user
 * @access  Private (Admin only)
 */
router.post(
  '/',
  requireAdmin,
  validate(createUserSchema),
  usersController.createUser
);

/**
 * @route   PUT /api/users/:userId
 * @desc    Update user
 * @access  Private (Admin only)
 */
router.put(
  '/:userId',
  requireAdmin,
  validateMultiple({
    params: userIdParamSchema,
    body: updateUserSchema,
  }),
  usersController.updateUser
);

/**
 * @route   DELETE /api/users/:userId
 * @desc    Delete user
 * @access  Private (Admin only)
 */
router.delete(
  '/:userId',
  requireAdmin,
  validate(userIdParamSchema, 'params'),
  usersController.deleteUser
);

export default router;
