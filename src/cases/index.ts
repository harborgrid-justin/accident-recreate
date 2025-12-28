/**
 * Case Management Module - Main Entry Point
 * AccuScene Enterprise Accident Recreation Platform
 *
 * This module provides comprehensive case management functionality including:
 * - CRUD operations for accident investigation cases
 * - Status management with workflow transitions
 * - Full-text search and advanced filtering
 * - Case assignment and user management
 * - Audit logging for compliance and tracking
 * - Export capabilities (JSON, CSV, PDF-ready)
 * - Validation and business rule enforcement
 */

// Import for use in CaseManagementAPI
import { caseService } from './CaseService';
import { caseSearchService } from './CaseSearchService';
import { caseExporter } from './CaseExporter';
import { auditLog } from './CaseAuditLog';
import { CaseValidator } from './CaseValidator';
import type { CaseData } from './CaseValidator';
import type { SearchCriteria, SearchOptions } from './CaseSearchService';
import type { ExportFormat, ExportOptions } from './CaseExporter';

// Core Services
export { CaseService, caseService } from './CaseService';
export type { Case, CreateCaseOptions, UpdateCaseOptions } from './CaseService';

// Search Service
export { CaseSearchService, caseSearchService } from './CaseSearchService';
export type {
  SearchCriteria,
  SearchOptions,
  SearchResult,
  SortField
} from './CaseSearchService';

// Export Service
export { CaseExporter, caseExporter } from './CaseExporter';
export type {
  ExportFormat,
  ExportOptions,
  ExportResult,
  PDFExportData,
  PDFSection
} from './CaseExporter';

// Validation
export { CaseValidator, ValidationException } from './CaseValidator';
export type {
  CaseData,
  ValidationResult,
  ValidationError,
  ValidationWarning
} from './CaseValidator';

// Status Management
export {
  CaseStatus,
  VALID_STATUS_TRANSITIONS,
  STATUS_METADATA,
  isValidStatusTransition,
  getStatusMetadata,
  getAllowedTransitions,
  canEditCase,
  canDeleteCase,
  InvalidStatusTransitionError
} from './CaseStatus';
export type { CaseStatusMetadata } from './CaseStatus';

// Audit Logging
export { CaseAuditLog, auditLog, AuditAction } from './CaseAuditLog';
export type {
  AuditLogEntry,
  AuditChange,
  AuditLogQuery
} from './CaseAuditLog';

/**
 * Main Case Management API
 *
 * This provides a unified interface to all case management functionality.
 * For most use cases, you can import and use the singleton instances:
 *
 * @example
 * ```typescript
 * import { caseService, CaseStatus } from './cases';
 *
 * // Create a new case
 * const newCase = await caseService.createCase('user-123', {
 *   title: 'Highway accident investigation',
 *   incidentDate: new Date('2024-01-15'),
 *   incidentLocation: 'I-95 Mile Marker 45',
 *   description: 'Multi-vehicle collision investigation',
 *   priority: 'HIGH'
 * });
 *
 * // Search cases
 * import { caseSearchService } from './cases';
 * const results = await caseSearchService.searchCases(
 *   await caseService.getAllCases(),
 *   { query: 'highway', status: CaseStatus.UNDER_INVESTIGATION },
 *   { page: 1, pageSize: 20, sortBy: 'updatedAt' }
 * );
 *
 * // Export a case
 * import { caseExporter } from './cases';
 * const exportResult = await caseExporter.exportCase(
 *   newCase,
 *   'json',
 *   'user-123',
 *   { includeAuditLog: true }
 * );
 * ```
 */
export const CaseManagementAPI = {
  // Service instances
  service: caseService,
  search: caseSearchService,
  exporter: caseExporter,
  auditLog: auditLog,

  // Utilities
  validator: CaseValidator,

  /**
   * Quick access to common operations
   */
  async createCase(userId: string, caseData: Partial<CaseData>) {
    return caseService.createCase(userId, caseData);
  },

  async getCase(caseId: string, userId?: string) {
    return caseService.getCase(caseId, userId);
  },

  async updateCase(caseId: string, updates: Partial<CaseData>, userId: string) {
    return caseService.updateCase(caseId, updates, userId);
  },

  async deleteCase(caseId: string, userId: string, reason?: string) {
    return caseService.deleteCase(caseId, userId, reason);
  },

  async searchCases(criteria: SearchCriteria, options?: SearchOptions) {
    const allCases = await caseService.getAllCases();
    return caseSearchService.searchCases(allCases, criteria, options);
  },

  async exportCase(caseId: string, format: ExportFormat, userId: string, options?: ExportOptions) {
    const caseData = await caseService.getCase(caseId);
    if (!caseData) {
      throw new Error(`Case not found: ${caseId}`);
    }
    return caseExporter.exportCase(caseData, format, userId, options);
  },

  async getCaseStatistics() {
    return caseService.getStatistics();
  }
};
