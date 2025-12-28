/**
 * Case Audit Log Service
 * Tracks all changes to cases with who, what, when logging
 */

import { CaseStatus } from './CaseStatus';

export interface AuditLogEntry {
  id: string;
  caseId: string;
  userId: string;
  userName?: string;
  action: AuditAction;
  timestamp: Date;
  changes?: AuditChange[];
  metadata?: Record<string, any>;
  ipAddress?: string;
  userAgent?: string;
}

export enum AuditAction {
  CREATED = 'CREATED',
  UPDATED = 'UPDATED',
  DELETED = 'DELETED',
  STATUS_CHANGED = 'STATUS_CHANGED',
  ASSIGNED = 'ASSIGNED',
  UNASSIGNED = 'UNASSIGNED',
  EXPORTED = 'EXPORTED',
  VIEWED = 'VIEWED',
  COMMENTED = 'COMMENTED',
  ATTACHMENT_ADDED = 'ATTACHMENT_ADDED',
  ATTACHMENT_REMOVED = 'ATTACHMENT_REMOVED'
}

export interface AuditChange {
  field: string;
  oldValue: any;
  newValue: any;
  displayName?: string;
}

export interface AuditLogQuery {
  caseId?: string;
  userId?: string;
  action?: AuditAction;
  startDate?: Date;
  endDate?: Date;
  limit?: number;
  offset?: number;
}

/**
 * Case Audit Log Service
 */
export class CaseAuditLog {
  private logs: Map<string, AuditLogEntry[]>;
  private logCounter: number;

  constructor() {
    this.logs = new Map();
    this.logCounter = 0;
  }

  /**
   * Logs a case creation event
   */
  logCreation(
    caseId: string,
    userId: string,
    caseData: any,
    metadata?: Record<string, any>
  ): AuditLogEntry {
    const entry: AuditLogEntry = {
      id: this.generateId(),
      caseId,
      userId,
      action: AuditAction.CREATED,
      timestamp: new Date(),
      metadata: {
        ...metadata,
        initialData: caseData
      }
    };

    this.addLogEntry(caseId, entry);
    return entry;
  }

  /**
   * Logs a case update event with field-level changes
   */
  logUpdate(
    caseId: string,
    userId: string,
    oldData: any,
    newData: any,
    metadata?: Record<string, any>
  ): AuditLogEntry {
    const changes = this.detectChanges(oldData, newData);

    const entry: AuditLogEntry = {
      id: this.generateId(),
      caseId,
      userId,
      action: AuditAction.UPDATED,
      timestamp: new Date(),
      changes,
      metadata
    };

    this.addLogEntry(caseId, entry);
    return entry;
  }

  /**
   * Logs a status change event
   */
  logStatusChange(
    caseId: string,
    userId: string,
    oldStatus: CaseStatus,
    newStatus: CaseStatus,
    reason?: string
  ): AuditLogEntry {
    const entry: AuditLogEntry = {
      id: this.generateId(),
      caseId,
      userId,
      action: AuditAction.STATUS_CHANGED,
      timestamp: new Date(),
      changes: [
        {
          field: 'status',
          oldValue: oldStatus,
          newValue: newStatus,
          displayName: 'Status'
        }
      ],
      metadata: reason ? { reason } : undefined
    };

    this.addLogEntry(caseId, entry);
    return entry;
  }

  /**
   * Logs a case assignment event
   */
  logAssignment(
    caseId: string,
    userId: string,
    assignedToUserId: string,
    previousAssignee?: string
  ): AuditLogEntry {
    const entry: AuditLogEntry = {
      id: this.generateId(),
      caseId,
      userId,
      action: previousAssignee ? AuditAction.ASSIGNED : AuditAction.ASSIGNED,
      timestamp: new Date(),
      changes: [
        {
          field: 'assignedTo',
          oldValue: previousAssignee,
          newValue: assignedToUserId,
          displayName: 'Assigned To'
        }
      ]
    };

    this.addLogEntry(caseId, entry);
    return entry;
  }

  /**
   * Logs a case unassignment event
   */
  logUnassignment(
    caseId: string,
    userId: string,
    previousAssignee: string
  ): AuditLogEntry {
    const entry: AuditLogEntry = {
      id: this.generateId(),
      caseId,
      userId,
      action: AuditAction.UNASSIGNED,
      timestamp: new Date(),
      changes: [
        {
          field: 'assignedTo',
          oldValue: previousAssignee,
          newValue: null,
          displayName: 'Assigned To'
        }
      ]
    };

    this.addLogEntry(caseId, entry);
    return entry;
  }

  /**
   * Logs a case deletion event
   */
  logDeletion(
    caseId: string,
    userId: string,
    reason?: string
  ): AuditLogEntry {
    const entry: AuditLogEntry = {
      id: this.generateId(),
      caseId,
      userId,
      action: AuditAction.DELETED,
      timestamp: new Date(),
      metadata: reason ? { reason } : undefined
    };

    this.addLogEntry(caseId, entry);
    return entry;
  }

  /**
   * Logs a case export event
   */
  logExport(
    caseId: string,
    userId: string,
    format: string,
    metadata?: Record<string, any>
  ): AuditLogEntry {
    const entry: AuditLogEntry = {
      id: this.generateId(),
      caseId,
      userId,
      action: AuditAction.EXPORTED,
      timestamp: new Date(),
      metadata: {
        ...metadata,
        format
      }
    };

    this.addLogEntry(caseId, entry);
    return entry;
  }

  /**
   * Logs a case view event (for compliance/tracking)
   */
  logView(
    caseId: string,
    userId: string,
    metadata?: Record<string, any>
  ): AuditLogEntry {
    const entry: AuditLogEntry = {
      id: this.generateId(),
      caseId,
      userId,
      action: AuditAction.VIEWED,
      timestamp: new Date(),
      metadata
    };

    this.addLogEntry(caseId, entry);
    return entry;
  }

  /**
   * Logs a custom action
   */
  logCustomAction(
    caseId: string,
    userId: string,
    action: AuditAction,
    changes?: AuditChange[],
    metadata?: Record<string, any>
  ): AuditLogEntry {
    const entry: AuditLogEntry = {
      id: this.generateId(),
      caseId,
      userId,
      action,
      timestamp: new Date(),
      changes,
      metadata
    };

    this.addLogEntry(caseId, entry);
    return entry;
  }

  /**
   * Retrieves audit logs for a specific case
   */
  getLogsByCase(caseId: string, limit?: number, offset: number = 0): AuditLogEntry[] {
    const caseLogs = this.logs.get(caseId) || [];

    // Sort by timestamp descending (most recent first)
    const sorted = [...caseLogs].sort((a, b) =>
      b.timestamp.getTime() - a.timestamp.getTime()
    );

    if (limit !== undefined) {
      return sorted.slice(offset, offset + limit);
    }

    return sorted.slice(offset);
  }

  /**
   * Retrieves audit logs by user
   */
  getLogsByUser(userId: string, limit?: number, offset: number = 0): AuditLogEntry[] {
    const allLogs: AuditLogEntry[] = [];

    for (const caseLogs of this.logs.values()) {
      const userLogs = caseLogs.filter(log => log.userId === userId);
      allLogs.push(...userLogs);
    }

    // Sort by timestamp descending
    const sorted = allLogs.sort((a, b) =>
      b.timestamp.getTime() - a.timestamp.getTime()
    );

    if (limit !== undefined) {
      return sorted.slice(offset, offset + limit);
    }

    return sorted.slice(offset);
  }

  /**
   * Queries audit logs with multiple criteria
   */
  queryLogs(query: AuditLogQuery): AuditLogEntry[] {
    let results: AuditLogEntry[] = [];

    if (query.caseId) {
      // Query specific case
      results = this.logs.get(query.caseId) || [];
    } else {
      // Query all cases
      for (const caseLogs of this.logs.values()) {
        results.push(...caseLogs);
      }
    }

    // Apply filters
    if (query.userId) {
      results = results.filter(log => log.userId === query.userId);
    }

    if (query.action) {
      results = results.filter(log => log.action === query.action);
    }

    if (query.startDate) {
      results = results.filter(log => log.timestamp >= query.startDate!);
    }

    if (query.endDate) {
      results = results.filter(log => log.timestamp <= query.endDate!);
    }

    // Sort by timestamp descending
    results.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());

    // Apply pagination
    const offset = query.offset || 0;
    const limit = query.limit;

    if (limit !== undefined) {
      return results.slice(offset, offset + limit);
    }

    return results.slice(offset);
  }

  /**
   * Gets a summary of actions for a case
   */
  getActionSummary(caseId: string): Record<AuditAction, number> {
    const caseLogs = this.logs.get(caseId) || [];
    const summary: Record<string, number> = {};

    for (const log of caseLogs) {
      summary[log.action] = (summary[log.action] || 0) + 1;
    }

    return summary as Record<AuditAction, number>;
  }

  /**
   * Gets the complete audit trail for a case in chronological order
   */
  getAuditTrail(caseId: string): AuditLogEntry[] {
    const caseLogs = this.logs.get(caseId) || [];

    // Sort by timestamp ascending (chronological order)
    return [...caseLogs].sort((a, b) =>
      a.timestamp.getTime() - b.timestamp.getTime()
    );
  }

  /**
   * Clears all logs (use with caution!)
   */
  clearAllLogs(): void {
    this.logs.clear();
    this.logCounter = 0;
  }

  /**
   * Clears logs for a specific case
   */
  clearCaseLogs(caseId: string): void {
    this.logs.delete(caseId);
  }

  /**
   * Private helper to add a log entry
   */
  private addLogEntry(caseId: string, entry: AuditLogEntry): void {
    if (!this.logs.has(caseId)) {
      this.logs.set(caseId, []);
    }
    this.logs.get(caseId)!.push(entry);
  }

  /**
   * Generates a unique ID for log entries
   */
  private generateId(): string {
    this.logCounter++;
    return `audit_${Date.now()}_${this.logCounter}`;
  }

  /**
   * Detects changes between old and new data
   */
  private detectChanges(oldData: any, newData: any): AuditChange[] {
    const changes: AuditChange[] = [];

    // Get all unique keys from both objects
    const allKeys = new Set([
      ...Object.keys(oldData || {}),
      ...Object.keys(newData || {})
    ]);

    for (const key of allKeys) {
      // Skip system fields and functions
      if (key === 'updatedAt' || typeof oldData[key] === 'function') {
        continue;
      }

      const oldValue = oldData[key];
      const newValue = newData[key];

      // Check if values are different
      if (JSON.stringify(oldValue) !== JSON.stringify(newValue)) {
        changes.push({
          field: key,
          oldValue,
          newValue,
          displayName: this.getDisplayName(key)
        });
      }
    }

    return changes;
  }

  /**
   * Converts field names to display names
   */
  private getDisplayName(field: string): string {
    const displayNames: Record<string, string> = {
      caseNumber: 'Case Number',
      title: 'Title',
      description: 'Description',
      incidentDate: 'Incident Date',
      incidentLocation: 'Incident Location',
      status: 'Status',
      assignedTo: 'Assigned To',
      priority: 'Priority',
      estimatedCompletionDate: 'Estimated Completion Date',
      tags: 'Tags'
    };

    return displayNames[field] || field;
  }

  /**
   * Exports audit logs to JSON
   */
  exportLogs(caseId?: string): string {
    const logs = caseId ? this.getLogsByCase(caseId) : this.getAllLogs();
    return JSON.stringify(logs, null, 2);
  }

  /**
   * Gets all logs across all cases
   */
  private getAllLogs(): AuditLogEntry[] {
    const allLogs: AuditLogEntry[] = [];

    for (const caseLogs of this.logs.values()) {
      allLogs.push(...caseLogs);
    }

    return allLogs.sort((a, b) =>
      b.timestamp.getTime() - a.timestamp.getTime()
    );
  }
}

// Singleton instance
export const auditLog = new CaseAuditLog();
