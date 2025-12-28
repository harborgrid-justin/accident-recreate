/**
 * Case Exporter Service
 * Handles exporting cases to various formats (JSON, CSV, PDF-ready)
 */

import { Case } from './CaseService';
import { auditLog } from './CaseAuditLog';
import { getStatusMetadata } from './CaseStatus';

export type ExportFormat = 'json' | 'csv' | 'pdf-data';

export interface ExportOptions {
  includeMetadata?: boolean;
  includeAuditLog?: boolean;
  includeTags?: boolean;
  dateFormat?: 'iso' | 'locale' | 'short';
  fields?: string[];
}

export interface ExportResult {
  format: ExportFormat;
  data: string | PDFExportData;
  filename: string;
  mimeType: string;
  timestamp: Date;
}

export interface PDFExportData {
  case: Case;
  metadata: {
    exportedBy?: string;
    exportedAt: Date;
    generatedBy: string;
  };
  sections: PDFSection[];
  auditLog?: any[];
}

export interface PDFSection {
  title: string;
  content: Record<string, any>;
  type: 'table' | 'text' | 'list';
}

/**
 * Case Exporter Service
 */
export class CaseExporter {
  /**
   * Exports a case to the specified format
   */
  async exportCase(
    caseData: Case,
    format: ExportFormat,
    userId?: string,
    options: ExportOptions = {}
  ): Promise<ExportResult> {
    let result: ExportResult;

    switch (format) {
      case 'json':
        result = await this.exportToJSON(caseData, options);
        break;

      case 'csv':
        result = await this.exportToCSV(caseData, options);
        break;

      case 'pdf-data':
        result = await this.exportToPDFData(caseData, userId, options);
        break;

      default:
        throw new Error(`Unsupported export format: ${format}`);
    }

    // Log the export
    if (userId) {
      auditLog.logExport(caseData.id, userId, format, {
        filename: result.filename,
        options
      });
    }

    return result;
  }

  /**
   * Exports a case to JSON format
   */
  private async exportToJSON(
    caseData: Case,
    options: ExportOptions
  ): Promise<ExportResult> {
    const exportData: any = { ...caseData };

    // Apply field filtering
    if (options.fields && options.fields.length > 0) {
      const filtered: any = {};
      for (const field of options.fields) {
        if (field in exportData) {
          filtered[field] = exportData[field];
        }
      }
      Object.assign(exportData, filtered);
    }

    // Add metadata if requested
    if (options.includeMetadata) {
      exportData._metadata = {
        exportedAt: new Date(),
        exportFormat: 'json',
        version: '1.0'
      };
    }

    // Add audit log if requested
    if (options.includeAuditLog) {
      exportData._auditLog = auditLog.getLogsByCase(caseData.id);
    }

    const jsonString = JSON.stringify(exportData, null, 2);

    return {
      format: 'json',
      data: jsonString,
      filename: `case_${caseData.caseNumber}_${Date.now()}.json`,
      mimeType: 'application/json',
      timestamp: new Date()
    };
  }

  /**
   * Exports a case to CSV format
   */
  private async exportToCSV(
    caseData: Case,
    options: ExportOptions
  ): Promise<ExportResult> {
    const rows: string[][] = [];

    // Define fields to export
    const fields = options.fields || [
      'caseNumber',
      'title',
      'description',
      'status',
      'priority',
      'incidentDate',
      'incidentLocation',
      'assignedTo',
      'createdBy',
      'createdAt',
      'updatedAt'
    ];

    // Add header row
    rows.push(fields.map(field => this.getFieldDisplayName(field)));

    // Add data row
    const dataRow = fields.map(field => {
      const value = (caseData as any)[field];
      return this.formatValueForCSV(value, options.dateFormat);
    });
    rows.push(dataRow);

    // Add tags if requested
    if (options.includeTags && caseData.tags && caseData.tags.length > 0) {
      rows.push([]);
      rows.push(['Tags']);
      rows.push([caseData.tags.join(', ')]);
    }

    // Add metadata if requested
    if (options.includeMetadata && caseData.metadata) {
      rows.push([]);
      rows.push(['Metadata']);
      for (const [key, value] of Object.entries(caseData.metadata)) {
        rows.push([key, String(value)]);
      }
    }

    // Convert to CSV string
    const csvString = rows
      .map(row => row.map(cell => this.escapeCSVCell(cell)).join(','))
      .join('\n');

    return {
      format: 'csv',
      data: csvString,
      filename: `case_${caseData.caseNumber}_${Date.now()}.csv`,
      mimeType: 'text/csv',
      timestamp: new Date()
    };
  }

  /**
   * Exports a case to PDF-ready data structure
   */
  private async exportToPDFData(
    caseData: Case,
    userId?: string,
    options: ExportOptions = {}
  ): Promise<ExportResult> {
    const sections: PDFSection[] = [];

    // Case Overview Section
    sections.push({
      title: 'Case Overview',
      type: 'table',
      content: {
        'Case Number': caseData.caseNumber,
        'Title': caseData.title,
        'Status': getStatusMetadata(caseData.status).label,
        'Priority': caseData.priority || 'N/A',
        'Created By': caseData.createdBy,
        'Assigned To': caseData.assignedTo || 'Unassigned',
        'Created Date': this.formatDate(caseData.createdAt),
        'Last Updated': this.formatDate(caseData.updatedAt)
      }
    });

    // Incident Details Section
    sections.push({
      title: 'Incident Details',
      type: 'table',
      content: {
        'Incident Date': this.formatDate(caseData.incidentDate),
        'Location': caseData.incidentLocation,
        'Description': caseData.description || 'No description provided'
      }
    });

    // Timeline Section
    if (caseData.estimatedCompletionDate) {
      sections.push({
        title: 'Timeline',
        type: 'table',
        content: {
          'Estimated Completion': this.formatDate(caseData.estimatedCompletionDate)
        }
      });
    }

    // Tags Section
    if (options.includeTags && caseData.tags && caseData.tags.length > 0) {
      sections.push({
        title: 'Tags',
        type: 'list',
        content: {
          items: caseData.tags
        }
      });
    }

    // Metadata Section
    if (options.includeMetadata && caseData.metadata) {
      sections.push({
        title: 'Additional Metadata',
        type: 'table',
        content: caseData.metadata
      });
    }

    const pdfData: PDFExportData = {
      case: caseData,
      metadata: {
        exportedBy: userId,
        exportedAt: new Date(),
        generatedBy: 'AccuScene Case Management System v1.0'
      },
      sections
    };

    // Add audit log if requested
    if (options.includeAuditLog) {
      pdfData.auditLog = auditLog.getLogsByCase(caseData.id);
    }

    return {
      format: 'pdf-data',
      data: pdfData,
      filename: `case_${caseData.caseNumber}_${Date.now()}.pdf`,
      mimeType: 'application/pdf',
      timestamp: new Date()
    };
  }

  /**
   * Exports multiple cases to a single file
   */
  async exportMultipleCases(
    cases: Case[],
    format: ExportFormat,
    userId?: string,
    options: ExportOptions = {}
  ): Promise<ExportResult> {
    switch (format) {
      case 'json':
        return this.exportMultipleCasesToJSON(cases, options);

      case 'csv':
        return this.exportMultipleCasesToCSV(cases, options);

      default:
        throw new Error(`Batch export not supported for format: ${format}`);
    }
  }

  /**
   * Exports multiple cases to JSON
   */
  private async exportMultipleCasesToJSON(
    cases: Case[],
    options: ExportOptions
  ): Promise<ExportResult> {
    const exportData: any = {
      cases,
      summary: {
        total: cases.length,
        exportedAt: new Date()
      }
    };

    if (options.includeMetadata) {
      exportData.metadata = {
        exportFormat: 'json',
        version: '1.0',
        caseCount: cases.length
      };
    }

    const jsonString = JSON.stringify(exportData, null, 2);

    return {
      format: 'json',
      data: jsonString,
      filename: `cases_export_${Date.now()}.json`,
      mimeType: 'application/json',
      timestamp: new Date()
    };
  }

  /**
   * Exports multiple cases to CSV
   */
  private async exportMultipleCasesToCSV(
    cases: Case[],
    options: ExportOptions
  ): Promise<ExportResult> {
    if (cases.length === 0) {
      return {
        format: 'csv',
        data: '',
        filename: `cases_export_${Date.now()}.csv`,
        mimeType: 'text/csv',
        timestamp: new Date()
      };
    }

    const fields = options.fields || [
      'caseNumber',
      'title',
      'status',
      'priority',
      'incidentDate',
      'incidentLocation',
      'assignedTo',
      'createdBy',
      'createdAt',
      'updatedAt'
    ];

    // Header row
    const rows: string[][] = [
      fields.map(field => this.getFieldDisplayName(field))
    ];

    // Data rows
    for (const caseData of cases) {
      const dataRow = fields.map(field => {
        const value = (caseData as any)[field];
        return this.formatValueForCSV(value, options.dateFormat);
      });
      rows.push(dataRow);
    }

    // Convert to CSV string
    const csvString = rows
      .map(row => row.map(cell => this.escapeCSVCell(cell)).join(','))
      .join('\n');

    return {
      format: 'csv',
      data: csvString,
      filename: `cases_export_${Date.now()}.csv`,
      mimeType: 'text/csv',
      timestamp: new Date()
    };
  }

  /**
   * Exports case with full audit trail
   */
  async exportCaseWithAuditTrail(
    caseData: Case,
    userId?: string
  ): Promise<ExportResult> {
    const auditTrail = auditLog.getAuditTrail(caseData.id);

    const exportData = {
      case: caseData,
      auditTrail,
      summary: {
        totalChanges: auditTrail.length,
        exportedAt: new Date(),
        exportedBy: userId
      }
    };

    const jsonString = JSON.stringify(exportData, null, 2);

    return {
      format: 'json',
      data: jsonString,
      filename: `case_${caseData.caseNumber}_full_audit_${Date.now()}.json`,
      mimeType: 'application/json',
      timestamp: new Date()
    };
  }

  /**
   * Helper: Formats a value for CSV output
   */
  private formatValueForCSV(value: any, dateFormat?: string): string {
    if (value === null || value === undefined) {
      return '';
    }

    if (value instanceof Date) {
      return this.formatDate(value, dateFormat);
    }

    if (typeof value === 'object') {
      return JSON.stringify(value);
    }

    return String(value);
  }

  /**
   * Helper: Escapes a CSV cell
   */
  private escapeCSVCell(cell: string): string {
    if (cell.includes(',') || cell.includes('"') || cell.includes('\n')) {
      return `"${cell.replace(/"/g, '""')}"`;
    }
    return cell;
  }

  /**
   * Helper: Formats a date
   */
  private formatDate(date: Date | string, format: string = 'iso'): string {
    const d = new Date(date);

    switch (format) {
      case 'locale':
        return d.toLocaleString();

      case 'short':
        return d.toLocaleDateString();

      case 'iso':
      default:
        return d.toISOString();
    }
  }

  /**
   * Helper: Gets display name for a field
   */
  private getFieldDisplayName(field: string): string {
    const displayNames: Record<string, string> = {
      caseNumber: 'Case Number',
      title: 'Title',
      description: 'Description',
      status: 'Status',
      priority: 'Priority',
      incidentDate: 'Incident Date',
      incidentLocation: 'Incident Location',
      assignedTo: 'Assigned To',
      createdBy: 'Created By',
      createdAt: 'Created At',
      updatedAt: 'Updated At',
      estimatedCompletionDate: 'Est. Completion Date'
    };

    return displayNames[field] || field;
  }

  /**
   * Generates a summary report for a case
   */
  async generateCaseSummary(caseData: Case): Promise<string> {
    const statusMeta = getStatusMetadata(caseData.status);
    const auditTrail = auditLog.getLogsByCase(caseData.id);

    const summary = `
CASE SUMMARY REPORT
===================

Case Number: ${caseData.caseNumber}
Title: ${caseData.title}
Status: ${statusMeta.label}

INCIDENT DETAILS
----------------
Date: ${this.formatDate(caseData.incidentDate)}
Location: ${caseData.incidentLocation}
Description: ${caseData.description || 'N/A'}

CASE INFORMATION
----------------
Priority: ${caseData.priority || 'N/A'}
Assigned To: ${caseData.assignedTo || 'Unassigned'}
Created By: ${caseData.createdBy}
Created: ${this.formatDate(caseData.createdAt)}
Last Updated: ${this.formatDate(caseData.updatedAt)}

ACTIVITY
--------
Total Actions: ${auditTrail.length}
Last Action: ${auditTrail.length > 0 && auditTrail[auditTrail.length - 1] ? auditTrail[auditTrail.length - 1]?.action : 'N/A'}

${caseData.tags && caseData.tags.length > 0 ? `\nTAGS\n----\n${caseData.tags.join(', ')}` : ''}

---
Generated by AccuScene Case Management System
Report Date: ${this.formatDate(new Date())}
`.trim();

    return summary;
  }
}

// Singleton instance
export const caseExporter = new CaseExporter();
