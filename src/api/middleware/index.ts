/**
 * Middleware Module Exports
 * AccuScene Enterprise Accident Recreation Platform
 */

// Authentication & Authorization
export {
  authenticate,
  optionalAuthenticate,
  authorize,
  requireAdmin,
  requireOwnershipOrAdmin,
  rateLimit,
} from './auth';

// Error Handling
export {
  errorHandler,
  notFoundHandler,
  asyncHandler,
  AppError,
  NotFoundError,
  ConflictError,
  ForbiddenError,
} from './errorHandler';

// Logging & Security
export {
  requestLogger,
  securityHeaders,
  corsOptions,
  requestSizeLimiter,
  sanitizeForLog,
} from './logger';

// Validation
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
} from './validator';

// File Upload
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
  isValidFileExtension,
  formatFileSize,
} from './upload';

export type { FileInfo } from './upload';
export type { LogEntry } from './logger';
export type { ValidationType } from './validator';
