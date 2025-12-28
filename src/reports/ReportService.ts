/**
 * Report Service - Main service for report management
 * Coordinates template selection, report building, rendering, and storage
 */

import { ReportTemplate, TemplateRegistry } from './ReportTemplates';
import { ReportBuilder, ReportContent } from './ReportBuilder';
import { ReportRenderer, RenderOptions } from './ReportRenderer';
import { ReportStorage, StoredReport, ReportMetadata, StorageOptions } from './ReportStorage';
import { BrandingConfig } from './ReportStyles';

export interface GenerateReportOptions {
  templateId: string;
  caseId: string;
  content: ReportContent;
  format?: 'html' | 'pdf';
  branding?: BrandingConfig;
  renderOptions?: Partial<RenderOptions>;
  createdBy: string;
  autoSave?: boolean;
}

export interface ExportReportOptions {
  reportId: string;
  format: 'html' | 'pdf';
  outputPath: string;
  watermark?: string;
}

/**
 * Main Report Service
 */
export class ReportService {
  private storage: ReportStorage;
  private defaultBranding?: BrandingConfig;

  constructor(storageOptions: StorageOptions, defaultBranding?: BrandingConfig) {
    this.storage = new ReportStorage(storageOptions);
    this.defaultBranding = defaultBranding;
  }

  /**
   * Initialize the service
   */
  async initialize(): Promise<void> {
    await this.storage.initialize();
  }

  /**
   * Generate a new report
   */
  async generateReport(options: GenerateReportOptions): Promise<{
    reportId: string;
    rendered: any;
    stored?: StoredReport;
  }> {
    const {
      templateId,
      caseId,
      content,
      format = 'pdf',
      branding = this.defaultBranding,
      renderOptions = {},
      createdBy,
      autoSave = true
    } = options;

    // Get template
    const template = TemplateRegistry.getTemplate(templateId);
    if (!template) {
      throw new Error(`Template not found: ${templateId}`);
    }

    // Build report
    const builder = new ReportBuilder(template, content);

    // Render report
    const renderer = new ReportRenderer(template, builder, branding);
    const rendered = await renderer.render({
      format,
      ...renderOptions
    });

    // Generate report ID
    const reportId = this.generateReportId(caseId, templateId);

    // Save to storage if auto-save is enabled
    let stored: StoredReport | undefined;

    if (autoSave) {
      const metadata: ReportMetadata = {
        title: content.caseData.title,
        description: template.description,
        pageCount: rendered.metadata.pageCount,
        generatedAt: rendered.metadata.generatedAt,
        templateVersion: template.metadata.version
      };

      stored = await this.storage.saveReport(
        reportId,
        caseId,
        templateId,
        rendered,
        metadata,
        createdBy
      );
    }

    return {
      reportId,
      rendered,
      stored
    };
  }

  /**
   * Get reports by case ID
   */
  async getReportsByCase(caseId: string): Promise<StoredReport[]> {
    return await this.storage.getReportsByCase(caseId);
  }

  /**
   * Get report by ID
   */
  async getReport(reportId: string): Promise<StoredReport | null> {
    return await this.storage.getReport(reportId);
  }

  /**
   * Get report content
   */
  async getReportContent(reportId: string): Promise<string | Buffer | null> {
    return await this.storage.getReportContent(reportId);
  }

  /**
   * Get all available templates
   */
  getReportTemplates(): ReportTemplate[] {
    return TemplateRegistry.getAllTemplates();
  }

  /**
   * Get templates by category
   */
  getTemplatesByCategory(category: ReportTemplate['category']): ReportTemplate[] {
    return TemplateRegistry.getTemplatesByCategory(category);
  }

  /**
   * Get specific template
   */
  getTemplate(templateId: string): ReportTemplate | null {
    return TemplateRegistry.getTemplate(templateId);
  }

  /**
   * Register a custom template
   */
  registerCustomTemplate(template: ReportTemplate): void {
    TemplateRegistry.registerCustomTemplate(template);
  }

  /**
   * Update custom template
   */
  updateCustomTemplate(templateId: string, updates: Partial<ReportTemplate>): void {
    TemplateRegistry.updateCustomTemplate(templateId, updates);
  }

  /**
   * Clone template
   */
  cloneTemplate(templateId: string, newId: string, newName: string): ReportTemplate {
    return TemplateRegistry.cloneTemplate(templateId, newId, newName);
  }

  /**
   * Delete a report
   */
  async deleteReport(reportId: string, permanent: boolean = false): Promise<boolean> {
    return await this.storage.deleteReport(reportId, permanent);
  }

  /**
   * Archive a report
   */
  async archiveReport(reportId: string): Promise<boolean> {
    return await this.storage.archiveReport(reportId);
  }

  /**
   * Restore archived report
   */
  async restoreReport(reportId: string): Promise<boolean> {
    return await this.storage.restoreReport(reportId);
  }

  /**
   * Export report to file
   */
  async exportReport(reportId: string, format: 'html' | 'pdf', outputPath: string): Promise<boolean> {
    const fs = require('fs').promises;

    // Get the report
    const report = await this.storage.getReport(reportId);
    if (!report) {
      throw new Error(`Report not found: ${reportId}`);
    }

    // If format matches stored format, just copy the file
    if (report.format === format) {
      const content = await this.storage.getReportContent(reportId);
      if (!content) {
        return false;
      }

      try {
        if (typeof content === 'string') {
          await fs.writeFile(outputPath, content, 'utf-8');
        } else {
          await fs.writeFile(outputPath, content);
        }
        return true;
      } catch (error) {
        console.error('Failed to export report:', error);
        return false;
      }
    }

    // Need to re-render in different format
    // For now, throw an error as we'd need the original content
    throw new Error('Format conversion requires re-generating the report with original content');
  }

  /**
   * Update report status
   */
  async updateReportStatus(reportId: string, status: 'draft' | 'final' | 'archived'): Promise<boolean> {
    return await this.storage.updateReportStatus(reportId, status);
  }

  /**
   * Update report metadata
   */
  async updateReportMetadata(reportId: string, metadata: Partial<ReportMetadata>): Promise<boolean> {
    return await this.storage.updateReportMetadata(reportId, metadata);
  }

  /**
   * Get report versions
   */
  async getReportVersions(reportId: string): Promise<any[]> {
    return await this.storage.getReportVersions(reportId);
  }

  /**
   * Get specific version content
   */
  async getVersionContent(reportId: string, version: number): Promise<string | Buffer | null> {
    return await this.storage.getVersionContent(reportId, version);
  }

  /**
   * Add tags to report
   */
  async addTags(reportId: string, tags: string[]): Promise<boolean> {
    return await this.storage.addTags(reportId, tags);
  }

  /**
   * Search reports
   */
  async searchReports(query: {
    caseId?: string;
    status?: 'draft' | 'final' | 'archived';
    tags?: string[];
    fromDate?: Date;
    toDate?: Date;
  }): Promise<StoredReport[]> {
    return await this.storage.searchReports(query);
  }

  /**
   * Get storage statistics
   */
  async getStorageStats(): Promise<{
    totalReports: number;
    totalSize: number;
    reportsByStatus: Record<string, number>;
    reportsByFormat: Record<string, number>;
  }> {
    return await this.storage.getStorageStats();
  }

  /**
   * Clean up old archived reports
   */
  async cleanupArchive(olderThanDays: number): Promise<number> {
    return await this.storage.cleanupArchive(olderThanDays);
  }

  /**
   * Regenerate report with new template or options
   */
  async regenerateReport(
    reportId: string,
    content: ReportContent,
    options?: {
      templateId?: string;
      format?: 'html' | 'pdf';
      branding?: BrandingConfig;
      createdBy: string;
    }
  ): Promise<StoredReport> {
    const existingReport = await this.storage.getReport(reportId);
    if (!existingReport) {
      throw new Error(`Report not found: ${reportId}`);
    }

    const templateId = options?.templateId || existingReport.templateId;
    const format = options?.format || existingReport.format;
    const createdBy = options?.createdBy || existingReport.createdBy;

    const result = await this.generateReport({
      templateId,
      caseId: existingReport.caseId,
      content,
      format,
      branding: options?.branding || this.defaultBranding,
      createdBy,
      autoSave: false
    });

    // Update existing report
    const metadata: ReportMetadata = {
      title: content.caseData.title,
      description: existingReport.metadata.description,
      pageCount: result.rendered.metadata.pageCount,
      generatedAt: result.rendered.metadata.generatedAt,
      templateVersion: existingReport.metadata.templateVersion
    };

    const stored = await this.storage.saveReport(
      reportId,
      existingReport.caseId,
      templateId,
      result.rendered,
      metadata,
      createdBy
    );

    return stored;
  }

  /**
   * Batch generate reports
   */
  async batchGenerateReports(
    reportConfigs: GenerateReportOptions[]
  ): Promise<Array<{
    reportId: string;
    success: boolean;
    error?: string;
    stored?: StoredReport;
  }>> {
    const results: Array<{
      reportId: string;
      success: boolean;
      error?: string;
      stored?: StoredReport;
    }> = [];

    for (const config of reportConfigs) {
      try {
        const result = await this.generateReport(config);
        results.push({
          reportId: result.reportId,
          success: true,
          stored: result.stored
        });
      } catch (error) {
        results.push({
          reportId: this.generateReportId(config.caseId, config.templateId),
          success: false,
          error: error instanceof Error ? error.message : 'Unknown error'
        });
      }
    }

    return results;
  }

  /**
   * Generate preview HTML for report
   */
  async generatePreview(templateId: string, content: ReportContent, branding?: BrandingConfig): Promise<string> {
    const template = TemplateRegistry.getTemplate(templateId);
    if (!template) {
      throw new Error(`Template not found: ${templateId}`);
    }

    const builder = new ReportBuilder(template, content);
    const renderer = new ReportRenderer(template, builder, branding || this.defaultBranding);

    return renderer.generatePreview();
  }

  /**
   * Compare two report versions
   */
  async compareVersions(
    reportId: string,
    version1: number,
    version2: number
  ): Promise<{
    version1Content: string | Buffer | null;
    version2Content: string | Buffer | null;
    differences?: string[];
  }> {
    const content1 = await this.storage.getVersionContent(reportId, version1);
    const content2 = await this.storage.getVersionContent(reportId, version2);

    // Basic comparison - in a real implementation, this would do detailed diff
    const differences: string[] = [];

    if (typeof content1 === 'string' && typeof content2 === 'string') {
      if (content1.length !== content2.length) {
        differences.push(`Content length differs: ${content1.length} vs ${content2.length}`);
      }
    }

    return {
      version1Content: content1,
      version2Content: content2,
      differences
    };
  }

  /**
   * Generate report ID
   */
  private generateReportId(caseId: string, templateId: string): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 8);
    return `${caseId}_${templateId}_${timestamp}_${random}`;
  }

  /**
   * Validate report content
   */
  validateReportContent(content: ReportContent): {
    valid: boolean;
    errors: string[];
    warnings: string[];
  } {
    const errors: string[] = [];
    const warnings: string[] = [];

    // Required fields
    if (!content.caseData) {
      errors.push('Case data is required');
    } else {
      if (!content.caseData.caseId) errors.push('Case ID is required');
      if (!content.caseData.caseNumber) errors.push('Case number is required');
      if (!content.caseData.title) errors.push('Case title is required');
      if (!content.caseData.dateOfIncident) errors.push('Date of incident is required');
      if (!content.caseData.location) errors.push('Location is required');
      if (!content.caseData.investigator) errors.push('Investigator information is required');
    }

    if (!content.vehicles || content.vehicles.length === 0) {
      errors.push('At least one vehicle is required');
    }

    if (!content.environment) {
      errors.push('Environmental data is required');
    }

    // Warnings
    if (!content.physics) {
      warnings.push('Physics analysis is not included');
    }

    if (!content.witnesses || content.witnesses.length === 0) {
      warnings.push('No witness statements included');
    }

    if (!content.diagrams || content.diagrams.length === 0) {
      warnings.push('No diagrams included');
    }

    if (!content.conclusions) {
      warnings.push('No conclusions section included');
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings
    };
  }

  /**
   * Get report statistics for a case
   */
  async getCaseReportStats(caseId: string): Promise<{
    totalReports: number;
    byTemplate: Record<string, number>;
    byStatus: Record<string, number>;
    latestReport?: StoredReport;
    oldestReport?: StoredReport;
  }> {
    const reports = await this.getReportsByCase(caseId);

    const stats = {
      totalReports: reports.length,
      byTemplate: {} as Record<string, number>,
      byStatus: {} as Record<string, number>,
      latestReport: reports[0],
      oldestReport: reports[reports.length - 1]
    };

    for (const report of reports) {
      stats.byTemplate[report.templateId] = (stats.byTemplate[report.templateId] || 0) + 1;
      stats.byStatus[report.status] = (stats.byStatus[report.status] || 0) + 1;
    }

    return stats;
  }
}

/**
 * Factory function to create ReportService instance
 */
export function createReportService(
  basePath: string,
  branding?: BrandingConfig,
  options?: Partial<StorageOptions>
): ReportService {
  const storageOptions: StorageOptions = {
    basePath,
    enableVersioning: true,
    maxVersions: 10,
    compressionEnabled: false,
    encryptionEnabled: false,
    ...options
  };

  return new ReportService(storageOptions, branding);
}

/**
 * Default export
 */
export default ReportService;
