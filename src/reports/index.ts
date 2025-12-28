/**
 * AccuScene Enterprise - Report Generation System
 * Central export module for all report-related functionality
 */

// Main Service
export { ReportService, createReportService } from './ReportService';
export type { GenerateReportOptions, ExportReportOptions } from './ReportService';

// Templates
export {
  TemplateRegistry,
  STANDARD_INVESTIGATION,
  INSURANCE_SUMMARY,
  POLICE_SUPPLEMENT,
  LITIGATION_SUPPORT
} from './ReportTemplates';
export type { ReportTemplate, ReportSection } from './ReportTemplates';

// Builder
export { ReportBuilder } from './ReportBuilder';
export type {
  ReportContent,
  CaseData,
  VehicleData,
  EnvironmentalData,
  PhysicsData,
  WitnessStatement
} from './ReportBuilder';

// Renderer
export { ReportRenderer, BatchReportRenderer } from './ReportRenderer';
export type {
  RenderOptions,
  PDFOptions,
  RenderedReport
} from './ReportRenderer';

// Styles
export {
  StyleGenerator,
  STYLE_THEMES,
  DEFAULT_COLOR_SCHEME,
  DEFAULT_TYPOGRAPHY,
  DEFAULT_LAYOUT
} from './ReportStyles';
export type {
  ColorScheme,
  Typography,
  BrandingConfig,
  LayoutConfig
} from './ReportStyles';

// Diagram Inserter
export {
  DiagramInserter,
  STANDARD_LEGEND_ITEMS
} from './DiagramInserter';
export type {
  DiagramImage,
  DiagramMetadata,
  DiagramLegendItem,
  DiagramInsertOptions
} from './DiagramInserter';

// Storage
export {
  ReportStorage,
  ReportExportManager
} from './ReportStorage';
export type {
  StoredReport,
  ReportMetadata,
  ReportVersion,
  StorageOptions
} from './ReportStorage';

/**
 * Quick Start Example:
 *
 * ```typescript
 * import { createReportService, STANDARD_INVESTIGATION } from './reports';
 *
 * // Create service
 * const reportService = createReportService('/path/to/storage', {
 *   companyName: 'AccuScene Investigations',
 *   primaryColor: '#1a365d'
 * });
 *
 * await reportService.initialize();
 *
 * // Generate report
 * const result = await reportService.generateReport({
 *   templateId: 'standard-investigation',
 *   caseId: 'CASE-2024-001',
 *   content: {
 *     caseData: { ... },
 *     vehicles: [ ... ],
 *     environment: { ... }
 *   },
 *   format: 'pdf',
 *   createdBy: 'investigator@example.com'
 * });
 *
 * // Export report
 * await reportService.exportReport(
 *   result.reportId,
 *   'pdf',
 *   '/path/to/output.pdf'
 * );
 * ```
 */

/**
 * Version information
 */
export const VERSION = '1.0.0';

/**
 * Module metadata
 */
export const MODULE_INFO = {
  name: 'AccuScene Report Generation System',
  version: VERSION,
  description: 'Comprehensive report generation system for accident reconstruction',
  features: [
    'Multiple professional templates',
    'HTML and PDF rendering',
    'Diagram insertion and management',
    'Version control',
    'Custom branding support',
    'Report archival and storage',
    'Batch processing',
    'Search and filtering'
  ]
};
