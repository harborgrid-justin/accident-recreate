/**
 * API Module Entry Point
 * AccuScene Enterprise Accident Recreation Platform
 */

// Import for default export and re-export
import { createApp, startServer } from './server';
export { createApp, startServer };

// Export responses utilities
export {
  success,
  error,
  paginated,
  noContent,
  created,
  ErrorCode,
} from './responses';

// Export middleware
export {
  authenticate,
  optionalAuthenticate,
  authorize,
  requireAdmin,
  requireOwnershipOrAdmin,
  rateLimit,
} from './middleware/auth';

export {
  errorHandler,
  notFoundHandler,
  asyncHandler,
  AppError,
  NotFoundError,
  ConflictError,
  ForbiddenError,
} from './middleware/errorHandler';

export {
  requestLogger,
  securityHeaders,
  corsOptions,
  requestSizeLimiter,
} from './middleware/logger';

export {
  validate,
  validateMultiple,
  sanitizeInput,
  sanitizeString,
  sanitizeObject,
  commonSchemas,
  emailSchema,
  passwordSchema,
  uuidSchema,
} from './middleware/validator';

export {
  uploadEvidence,
  uploadImage,
  uploadDocument,
  uploadToMemory,
  uploadSingle,
  uploadMultiple,
  uploadFields,
  getFileInfo,
  getFilesInfo,
  deleteFile,
} from './middleware/upload';

// Export controllers
export * as authController from './controllers/auth.controller';
export * as usersController from './controllers/users.controller';
export * as casesController from './controllers/cases.controller';
export * as accidentsController from './controllers/accidents.controller';
export * as vehiclesController from './controllers/vehicles.controller';
export * as evidenceController from './controllers/evidence.controller';
export * as reportsController from './controllers/reports.controller';

// Export routes
export { default as authRoutes } from './routes/auth.routes';
export { default as usersRoutes } from './routes/users.routes';
export { default as casesRoutes } from './routes/cases.routes';
export { default as accidentsRoutes } from './routes/accidents.routes';
export { default as vehiclesRoutes } from './routes/vehicles.routes';
export { default as evidenceRoutes } from './routes/evidence.routes';
export { default as reportsRoutes } from './routes/reports.routes';

// Export validation schemas
export * from './validators/auth.schemas';
export * from './validators/user.schemas';
export * from './validators/case.schemas';
export * from './validators/accident.schemas';
export * from './validators/vehicle.schemas';
export * from './validators/evidence.schemas';
export * from './validators/report.schemas';

// Re-export types from other modules for convenience
export { UserRole } from '../auth/types';
export type {
  User,
  SafeUser,
  AuthTokens,
  AuthTokenPayload,
  LoginCredentials,
  RegisterData,
} from '../auth/types';

export { CaseStatus } from '../cases/CaseStatus';
export { VehicleType } from '../vehicles/VehicleTypes';
export type {
  VehicleDimensions,
  VehicleWeightSpec,
  VehicleTypeSpec,
} from '../vehicles/VehicleTypes';

/**
 * API version
 */
export const API_VERSION = '1.0.0';

/**
 * API configuration
 */
export const API_CONFIG = {
  version: API_VERSION,
  name: 'AccuScene Enterprise API',
  description: 'Accident Recreation Platform API',
  baseUrl: process.env.API_BASE_URL || 'http://localhost:3000',
  port: parseInt(process.env.PORT || '3000', 10),
  environment: process.env.NODE_ENV || 'development',
};

/**
 * Default export for convenience
 */
export default {
  createApp,
  startServer,
  API_VERSION,
  API_CONFIG,
};
