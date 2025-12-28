/**
 * Authentication Routes
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Router } from 'express';
import * as authController from '../controllers/auth.controller';
import { validate } from '../middleware/validator';
import { authenticate, rateLimit } from '../middleware/auth';
import {
  loginSchema,
  registerSchema,
  refreshTokenSchema,
  changePasswordSchema,
  resetPasswordRequestSchema,
  resetPasswordSchema,
  verifyEmailSchema,
} from '../validators/auth.schemas';

const router = Router();

/**
 * @route   POST /api/auth/login
 * @desc    User login
 * @access  Public
 */
router.post(
  '/login',
  rateLimit(5, 15 * 60 * 1000), // 5 attempts per 15 minutes
  validate(loginSchema),
  authController.login
);

/**
 * @route   POST /api/auth/register
 * @desc    User registration
 * @access  Public
 */
router.post(
  '/register',
  rateLimit(3, 60 * 60 * 1000), // 3 attempts per hour
  validate(registerSchema),
  authController.register
);

/**
 * @route   POST /api/auth/logout
 * @desc    User logout
 * @access  Private
 */
router.post('/logout', authenticate, authController.logout);

/**
 * @route   POST /api/auth/refresh
 * @desc    Refresh access token
 * @access  Public
 */
router.post(
  '/refresh',
  validate(refreshTokenSchema),
  authController.refreshToken
);

/**
 * @route   GET /api/auth/me
 * @desc    Get current user
 * @access  Private
 */
router.get('/me', authenticate, authController.getCurrentUser);

/**
 * @route   POST /api/auth/change-password
 * @desc    Change password
 * @access  Private
 */
router.post(
  '/change-password',
  authenticate,
  rateLimit(5, 60 * 60 * 1000), // 5 attempts per hour
  validate(changePasswordSchema),
  authController.changePassword
);

/**
 * @route   POST /api/auth/reset-password-request
 * @desc    Request password reset
 * @access  Public
 */
router.post(
  '/reset-password-request',
  rateLimit(3, 60 * 60 * 1000), // 3 attempts per hour
  validate(resetPasswordRequestSchema),
  authController.requestPasswordReset
);

/**
 * @route   POST /api/auth/reset-password
 * @desc    Reset password with token
 * @access  Public
 */
router.post(
  '/reset-password',
  rateLimit(3, 60 * 60 * 1000), // 3 attempts per hour
  validate(resetPasswordSchema),
  authController.resetPassword
);

/**
 * @route   POST /api/auth/verify-email
 * @desc    Verify email address
 * @access  Public
 */
router.post(
  '/verify-email',
  validate(verifyEmailSchema),
  authController.verifyEmail
);

export default router;
