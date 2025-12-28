/**
 * Validation Schemas Module Exports
 * AccuScene Enterprise Accident Recreation Platform
 */

// Auth schemas
export {
  loginSchema,
  registerSchema,
  refreshTokenSchema,
  changePasswordSchema,
  resetPasswordRequestSchema,
  resetPasswordSchema,
  verifyEmailSchema,
} from './auth.schemas';

// User schemas
export {
  createUserSchema,
  updateUserSchema,
  updateProfileSchema,
  userSearchSchema,
  userIdParamSchema,
} from './user.schemas';

// Case schemas
export {
  createCaseSchema,
  updateCaseSchema,
  updateCaseStatusSchema,
  caseSearchSchema,
  caseIdParamSchema,
  assignCaseSchema,
} from './case.schemas';

// Accident schemas
export {
  createAccidentSchema,
  updateAccidentSchema,
  updateDiagramSchema,
  accidentSearchSchema,
  accidentIdParamSchema,
  exportDiagramSchema,
} from './accident.schemas';

// Vehicle schemas
export {
  createVehicleSchema,
  updateVehicleSchema,
  vehiclePhysicsSchema,
  vehicleSearchSchema,
  vehicleIdParamSchema,
} from './vehicle.schemas';

// Evidence schemas
export {
  createEvidenceSchema,
  updateEvidenceSchema,
  evidenceSearchSchema,
  evidenceIdParamSchema,
  batchUploadSchema,
  EvidenceType,
} from './evidence.schemas';

// Report schemas
export {
  generateReportSchema,
  updateReportSchema,
  reportSearchSchema,
  reportIdParamSchema,
  regenerateReportSchema,
  ReportType,
  ReportFormat,
} from './report.schemas';
