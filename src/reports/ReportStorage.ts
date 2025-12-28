/**
 * Report Storage System
 * Manages report persistence, versioning, and archival
 */

import { RenderedReport } from './ReportRenderer';

export interface StoredReport {
  reportId: string;
  caseId: string;
  templateId: string;
  version: number;
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
  format: 'html' | 'pdf';
  filePath: string;
  fileSize: number;
  metadata: ReportMetadata;
  status: 'draft' | 'final' | 'archived';
  tags?: string[];
}

export interface ReportMetadata {
  title: string;
  description?: string;
  pageCount?: number;
  generatedAt: Date;
  templateVersion: string;
  customFields?: Record<string, any>;
}

export interface ReportVersion {
  versionId: string;
  reportId: string;
  version: number;
  createdAt: Date;
  createdBy: string;
  changes: string;
  filePath: string;
  fileSize: number;
}

export interface StorageOptions {
  basePath: string;
  enableVersioning?: boolean;
  maxVersions?: number;
  compressionEnabled?: boolean;
  encryptionEnabled?: boolean;
}

/**
 * Report Storage Class
 */
export class ReportStorage {
  private basePath: string;
  private enableVersioning: boolean;
  private maxVersions: number;
  private compressionEnabled: boolean;
  private encryptionEnabled: boolean;
  private reports: Map<string, StoredReport> = new Map();
  private versions: Map<string, ReportVersion[]> = new Map();

  constructor(options: StorageOptions) {
    this.basePath = options.basePath;
    this.enableVersioning = options.enableVersioning !== false;
    this.maxVersions = options.maxVersions || 10;
    this.compressionEnabled = options.compressionEnabled || false;
    this.encryptionEnabled = options.encryptionEnabled || false;
  }

  /**
   * Initialize storage system
   */
  async initialize(): Promise<void> {
    const fs = require('fs').promises;

    try {
      // Create base directory structure
      await fs.mkdir(this.basePath, { recursive: true });
      await fs.mkdir(this.getReportsPath(), { recursive: true });
      await fs.mkdir(this.getVersionsPath(), { recursive: true });
      await fs.mkdir(this.getArchivePath(), { recursive: true });

      // Load existing reports index
      await this.loadReportsIndex();
    } catch (error) {
      console.error('Failed to initialize report storage:', error);
      throw error;
    }
  }

  /**
   * Save a new report
   */
  async saveReport(
    reportId: string,
    caseId: string,
    templateId: string,
    rendered: RenderedReport,
    metadata: ReportMetadata,
    createdBy: string
  ): Promise<StoredReport> {
    const fs = require('fs').promises;
    const path = require('path');

    // Check if report already exists
    const existingReport = this.reports.get(reportId);
    const version = existingReport ? existingReport.version + 1 : 1;

    // Save previous version if versioning is enabled
    if (existingReport && this.enableVersioning) {
      await this.createVersion(existingReport, createdBy, 'Updated report');
    }

    // Generate file path
    const extension = rendered.format === 'html' ? 'html' : 'pdf';
    const fileName = `${reportId}_v${version}.${extension}`;
    const filePath = path.join(this.getReportsPath(), fileName);

    // Save file
    if (rendered.format === 'html') {
      await fs.writeFile(filePath, rendered.content, 'utf-8');
    } else {
      await fs.writeFile(filePath, rendered.content);
    }

    // Create stored report record
    const storedReport: StoredReport = {
      reportId,
      caseId,
      templateId,
      version,
      createdAt: existingReport?.createdAt || new Date(),
      updatedAt: new Date(),
      createdBy,
      format: rendered.format,
      filePath,
      fileSize: rendered.metadata.fileSize || 0,
      metadata,
      status: 'draft',
      tags: []
    };

    // Store in memory
    this.reports.set(reportId, storedReport);

    // Persist index
    await this.saveReportsIndex();

    return storedReport;
  }

  /**
   * Get report by ID
   */
  async getReport(reportId: string): Promise<StoredReport | null> {
    return this.reports.get(reportId) || null;
  }

  /**
   * Get reports by case ID
   */
  async getReportsByCase(caseId: string): Promise<StoredReport[]> {
    const reports: StoredReport[] = [];

    for (const report of this.reports.values()) {
      if (report.caseId === caseId) {
        reports.push(report);
      }
    }

    return reports.sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime());
  }

  /**
   * Get report content
   */
  async getReportContent(reportId: string): Promise<string | Buffer | null> {
    const fs = require('fs').promises;
    const report = this.reports.get(reportId);

    if (!report) {
      return null;
    }

    try {
      if (report.format === 'html') {
        return await fs.readFile(report.filePath, 'utf-8');
      } else {
        return await fs.readFile(report.filePath);
      }
    } catch (error) {
      console.error(`Failed to read report ${reportId}:`, error);
      return null;
    }
  }

  /**
   * Update report status
   */
  async updateReportStatus(reportId: string, status: StoredReport['status']): Promise<boolean> {
    const report = this.reports.get(reportId);

    if (!report) {
      return false;
    }

    report.status = status;
    report.updatedAt = new Date();

    await this.saveReportsIndex();

    return true;
  }

  /**
   * Update report metadata
   */
  async updateReportMetadata(reportId: string, metadata: Partial<ReportMetadata>): Promise<boolean> {
    const report = this.reports.get(reportId);

    if (!report) {
      return false;
    }

    report.metadata = { ...report.metadata, ...metadata };
    report.updatedAt = new Date();

    await this.saveReportsIndex();

    return true;
  }

  /**
   * Delete report
   */
  async deleteReport(reportId: string, permanent: boolean = false): Promise<boolean> {
    const fs = require('fs').promises;
    const report = this.reports.get(reportId);

    if (!report) {
      return false;
    }

    if (permanent) {
      // Permanently delete file and all versions
      try {
        await fs.unlink(report.filePath);

        // Delete all versions
        const versions = this.versions.get(reportId);
        if (versions) {
          for (const version of versions) {
            try {
              await fs.unlink(version.filePath);
            } catch (error) {
              console.error(`Failed to delete version ${version.versionId}:`, error);
            }
          }
          this.versions.delete(reportId);
        }

        this.reports.delete(reportId);
        await this.saveReportsIndex();

        return true;
      } catch (error) {
        console.error(`Failed to delete report ${reportId}:`, error);
        return false;
      }
    } else {
      // Archive the report
      return await this.archiveReport(reportId);
    }
  }

  /**
   * Archive report
   */
  async archiveReport(reportId: string): Promise<boolean> {
    const fs = require('fs').promises;
    const path = require('path');
    const report = this.reports.get(reportId);

    if (!report) {
      return false;
    }

    try {
      // Move file to archive directory
      const archiveFileName = path.basename(report.filePath);
      const archivePath = path.join(this.getArchivePath(), archiveFileName);

      await fs.rename(report.filePath, archivePath);

      // Update report record
      report.filePath = archivePath;
      report.status = 'archived';
      report.updatedAt = new Date();

      await this.saveReportsIndex();

      return true;
    } catch (error) {
      console.error(`Failed to archive report ${reportId}:`, error);
      return false;
    }
  }

  /**
   * Restore archived report
   */
  async restoreReport(reportId: string): Promise<boolean> {
    const fs = require('fs').promises;
    const path = require('path');
    const report = this.reports.get(reportId);

    if (!report || report.status !== 'archived') {
      return false;
    }

    try {
      // Move file back to reports directory
      const fileName = path.basename(report.filePath);
      const restoredPath = path.join(this.getReportsPath(), fileName);

      await fs.rename(report.filePath, restoredPath);

      // Update report record
      report.filePath = restoredPath;
      report.status = 'draft';
      report.updatedAt = new Date();

      await this.saveReportsIndex();

      return true;
    } catch (error) {
      console.error(`Failed to restore report ${reportId}:`, error);
      return false;
    }
  }

  /**
   * Create a version of the report
   */
  private async createVersion(report: StoredReport, createdBy: string, changes: string): Promise<void> {
    const fs = require('fs').promises;
    const path = require('path');

    const versionId = `${report.reportId}_v${report.version}`;
    const versionFileName = path.basename(report.filePath);
    const versionPath = path.join(this.getVersionsPath(), versionFileName);

    // Copy current file to versions directory
    await fs.copyFile(report.filePath, versionPath);

    const version: ReportVersion = {
      versionId,
      reportId: report.reportId,
      version: report.version,
      createdAt: new Date(),
      createdBy,
      changes,
      filePath: versionPath,
      fileSize: report.fileSize
    };

    // Add to versions map
    const versions = this.versions.get(report.reportId) || [];
    versions.push(version);

    // Enforce max versions limit
    if (versions.length > this.maxVersions) {
      const oldestVersion = versions.shift();
      if (oldestVersion) {
        try {
          await fs.unlink(oldestVersion.filePath);
        } catch (error) {
          console.error(`Failed to delete old version ${oldestVersion.versionId}:`, error);
        }
      }
    }

    this.versions.set(report.reportId, versions);
  }

  /**
   * Get all versions of a report
   */
  async getReportVersions(reportId: string): Promise<ReportVersion[]> {
    return this.versions.get(reportId) || [];
  }

  /**
   * Get specific version content
   */
  async getVersionContent(reportId: string, version: number): Promise<string | Buffer | null> {
    const fs = require('fs').promises;
    const versions = this.versions.get(reportId);

    if (!versions) {
      return null;
    }

    const versionRecord = versions.find(v => v.version === version);

    if (!versionRecord) {
      return null;
    }

    try {
      // Determine format from file extension
      const isHTML = versionRecord.filePath.endsWith('.html');

      if (isHTML) {
        return await fs.readFile(versionRecord.filePath, 'utf-8');
      } else {
        return await fs.readFile(versionRecord.filePath);
      }
    } catch (error) {
      console.error(`Failed to read version ${versionRecord.versionId}:`, error);
      return null;
    }
  }

  /**
   * Add tags to report
   */
  async addTags(reportId: string, tags: string[]): Promise<boolean> {
    const report = this.reports.get(reportId);

    if (!report) {
      return false;
    }

    report.tags = [...new Set([...(report.tags || []), ...tags])];
    report.updatedAt = new Date();

    await this.saveReportsIndex();

    return true;
  }

  /**
   * Search reports
   */
  async searchReports(query: {
    caseId?: string;
    status?: StoredReport['status'];
    tags?: string[];
    fromDate?: Date;
    toDate?: Date;
  }): Promise<StoredReport[]> {
    let results = Array.from(this.reports.values());

    if (query.caseId) {
      results = results.filter(r => r.caseId === query.caseId);
    }

    if (query.status) {
      results = results.filter(r => r.status === query.status);
    }

    if (query.tags && query.tags.length > 0) {
      results = results.filter(r =>
        r.tags && query.tags!.some(tag => r.tags!.includes(tag))
      );
    }

    if (query.fromDate) {
      results = results.filter(r => r.createdAt >= query.fromDate!);
    }

    if (query.toDate) {
      results = results.filter(r => r.createdAt <= query.toDate!);
    }

    return results.sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime());
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
    const reports = Array.from(this.reports.values());

    const stats = {
      totalReports: reports.length,
      totalSize: reports.reduce((sum, r) => sum + r.fileSize, 0),
      reportsByStatus: {} as Record<string, number>,
      reportsByFormat: {} as Record<string, number>
    };

    for (const report of reports) {
      stats.reportsByStatus[report.status] = (stats.reportsByStatus[report.status] || 0) + 1;
      stats.reportsByFormat[report.format] = (stats.reportsByFormat[report.format] || 0) + 1;
    }

    return stats;
  }

  /**
   * Clean up old archived reports
   */
  async cleanupArchive(olderThanDays: number): Promise<number> {
    const fs = require('fs').promises;
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - olderThanDays);

    let deletedCount = 0;

    for (const [reportId, report] of this.reports.entries()) {
      if (report.status === 'archived' && report.updatedAt < cutoffDate) {
        try {
          await fs.unlink(report.filePath);
          this.reports.delete(reportId);
          deletedCount++;
        } catch (error) {
          console.error(`Failed to delete archived report ${reportId}:`, error);
        }
      }
    }

    if (deletedCount > 0) {
      await this.saveReportsIndex();
    }

    return deletedCount;
  }

  /**
   * Helper: Get reports directory path
   */
  private getReportsPath(): string {
    const path = require('path');
    return path.join(this.basePath, 'reports');
  }

  /**
   * Helper: Get versions directory path
   */
  private getVersionsPath(): string {
    const path = require('path');
    return path.join(this.basePath, 'versions');
  }

  /**
   * Helper: Get archive directory path
   */
  private getArchivePath(): string {
    const path = require('path');
    return path.join(this.basePath, 'archive');
  }

  /**
   * Load reports index from disk
   */
  private async loadReportsIndex(): Promise<void> {
    const fs = require('fs').promises;
    const path = require('path');
    const indexPath = path.join(this.basePath, 'reports-index.json');

    try {
      const data = await fs.readFile(indexPath, 'utf-8');
      const index = JSON.parse(data);

      // Restore reports
      for (const reportData of index.reports || []) {
        this.reports.set(reportData.reportId, {
          ...reportData,
          createdAt: new Date(reportData.createdAt),
          updatedAt: new Date(reportData.updatedAt),
          metadata: {
            ...reportData.metadata,
            generatedAt: new Date(reportData.metadata.generatedAt)
          }
        });
      }

      // Restore versions
      for (const [reportId, versionsData] of Object.entries(index.versions || {})) {
        this.versions.set(
          reportId,
          (versionsData as any[]).map(v => ({
            ...v,
            createdAt: new Date(v.createdAt)
          }))
        );
      }
    } catch (error) {
      // Index doesn't exist yet, that's okay
      console.log('No existing reports index found, starting fresh');
    }
  }

  /**
   * Save reports index to disk
   */
  private async saveReportsIndex(): Promise<void> {
    const fs = require('fs').promises;
    const path = require('path');
    const indexPath = path.join(this.basePath, 'reports-index.json');

    const index = {
      reports: Array.from(this.reports.values()),
      versions: Object.fromEntries(this.versions.entries()),
      updatedAt: new Date().toISOString()
    };

    await fs.writeFile(indexPath, JSON.stringify(index, null, 2), 'utf-8');
  }
}

/**
 * Report Export Manager
 */
export class ReportExportManager {
  private storage: ReportStorage;

  constructor(storage: ReportStorage) {
    this.storage = storage;
  }

  /**
   * Export report to external location
   */
  async exportReport(reportId: string, destinationPath: string): Promise<boolean> {
    const fs = require('fs').promises;
    const content = await this.storage.getReportContent(reportId);

    if (!content) {
      return false;
    }

    try {
      if (typeof content === 'string') {
        await fs.writeFile(destinationPath, content, 'utf-8');
      } else {
        await fs.writeFile(destinationPath, content);
      }
      return true;
    } catch (error) {
      console.error(`Failed to export report ${reportId}:`, error);
      return false;
    }
  }

  /**
   * Batch export reports
   */
  async batchExport(reportIds: string[], outputDir: string): Promise<number> {
    const fs = require('fs').promises;
    const path = require('path');

    await fs.mkdir(outputDir, { recursive: true });

    let successCount = 0;

    for (const reportId of reportIds) {
      const report = await this.storage.getReport(reportId);
      if (!report) continue;

      const fileName = path.basename(report.filePath);
      const destinationPath = path.join(outputDir, fileName);

      const success = await this.exportReport(reportId, destinationPath);
      if (success) successCount++;
    }

    return successCount;
  }
}
