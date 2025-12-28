/**
 * Case Service - Main service for case management
 * Handles CRUD operations, assignments, status changes, and business logic
 */

import { CaseStatus, isValidStatusTransition, InvalidStatusTransitionError, canDeleteCase } from './CaseStatus';
import { CaseData, CaseValidator, ValidationException } from './CaseValidator';
import { auditLog } from './CaseAuditLog';

export interface Case extends CaseData {
  id: string;
  caseNumber: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface CreateCaseOptions {
  skipValidation?: boolean;
  auditMetadata?: Record<string, any>;
}

export interface UpdateCaseOptions {
  skipValidation?: boolean;
  auditMetadata?: Record<string, any>;
}

/**
 * Main Case Service
 */
export class CaseService {
  private cases: Map<string, Case>;
  private casesByUser: Map<string, Set<string>>;
  private caseCounter: number;

  constructor() {
    this.cases = new Map();
    this.casesByUser = new Map();
    this.caseCounter = 0;
  }

  /**
   * Creates a new case
   */
  async createCase(
    userId: string,
    caseData: Partial<CaseData>,
    options: CreateCaseOptions = {}
  ): Promise<Case> {
    // Validate input
    if (!options.skipValidation) {
      const validation = CaseValidator.validateForCreation({
        ...caseData,
        createdBy: userId
      });

      if (!validation.isValid) {
        throw new ValidationException(validation);
      }
    }

    // Generate unique ID and case number
    const caseId = this.generateId();
    const caseNumber = this.generateCaseNumber();

    // Create case object
    const now = new Date();
    const newCase: Case = {
      id: caseId,
      caseNumber,
      title: caseData.title!,
      description: caseData.description || '',
      incidentDate: new Date(caseData.incidentDate!),
      incidentLocation: caseData.incidentLocation!,
      status: caseData.status || CaseStatus.DRAFT,
      assignedTo: caseData.assignedTo,
      createdBy: userId,
      createdAt: now,
      updatedAt: now,
      metadata: caseData.metadata || {},
      tags: caseData.tags || [],
      priority: caseData.priority || 'MEDIUM',
      estimatedCompletionDate: caseData.estimatedCompletionDate
        ? new Date(caseData.estimatedCompletionDate)
        : undefined
    };

    // Store case
    this.cases.set(caseId, newCase);

    // Index by user
    this.indexCaseByUser(userId, caseId);
    if (newCase.assignedTo) {
      this.indexCaseByUser(newCase.assignedTo, caseId);
    }

    // Audit log
    auditLog.logCreation(caseId, userId, newCase, options.auditMetadata);

    return newCase;
  }

  /**
   * Retrieves a case by ID
   */
  async getCase(caseId: string, userId?: string): Promise<Case | null> {
    const caseData = this.cases.get(caseId);

    if (caseData && userId) {
      // Log the view for audit purposes
      auditLog.logView(caseId, userId);
    }

    return caseData || null;
  }

  /**
   * Gets all cases created by or assigned to a user
   */
  async getCasesByUser(userId: string): Promise<Case[]> {
    const caseIds = this.casesByUser.get(userId);

    if (!caseIds) {
      return [];
    }

    const cases: Case[] = [];
    for (const caseId of caseIds) {
      const caseData = this.cases.get(caseId);
      if (caseData) {
        cases.push(caseData);
      }
    }

    // Sort by most recently updated
    return cases.sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime());
  }

  /**
   * Updates a case
   */
  async updateCase(
    caseId: string,
    updates: Partial<CaseData>,
    userId: string,
    options: UpdateCaseOptions = {}
  ): Promise<Case> {
    const existingCase = await this.getCase(caseId);

    if (!existingCase) {
      throw new Error(`Case not found: ${caseId}`);
    }

    // Check if case can be edited
    if (!canDeleteCase(existingCase.status) && !options.skipValidation) {
      const statusMeta = require('./CaseStatus').getStatusMetadata(existingCase.status);
      if (!statusMeta.canEdit) {
        throw new Error(
          `Case cannot be edited in ${existingCase.status} status`
        );
      }
    }

    // Validate updates
    if (!options.skipValidation) {
      const validation = CaseValidator.validateForUpdate(updates);
      if (!validation.isValid) {
        throw new ValidationException(validation);
      }
    }

    // Create updated case
    const oldData = { ...existingCase };
    const updatedCase: Case = {
      ...existingCase,
      ...updates,
      id: existingCase.id, // Ensure ID doesn't change
      caseNumber: existingCase.caseNumber, // Ensure case number doesn't change
      createdBy: existingCase.createdBy, // Ensure creator doesn't change
      createdAt: existingCase.createdAt, // Ensure creation date doesn't change
      updatedAt: new Date()
    };

    // Update indices if assigned user changed
    if (updates.assignedTo !== undefined && updates.assignedTo !== existingCase.assignedTo) {
      if (existingCase.assignedTo) {
        this.removeFromUserIndex(existingCase.assignedTo, caseId);
      }
      if (updates.assignedTo) {
        this.indexCaseByUser(updates.assignedTo, caseId);
      }
    }

    // Store updated case
    this.cases.set(caseId, updatedCase);

    // Audit log
    auditLog.logUpdate(caseId, userId, oldData, updatedCase, options.auditMetadata);

    return updatedCase;
  }

  /**
   * Deletes a case
   */
  async deleteCase(caseId: string, userId: string, reason?: string): Promise<boolean> {
    const existingCase = await this.getCase(caseId);

    if (!existingCase) {
      throw new Error(`Case not found: ${caseId}`);
    }

    // Check if case can be deleted
    if (!canDeleteCase(existingCase.status)) {
      throw new Error(
        `Case cannot be deleted in ${existingCase.status} status. Only DRAFT cases can be deleted.`
      );
    }

    // Check permissions
    if (existingCase.createdBy !== userId) {
      throw new Error('Only the case creator can delete the case');
    }

    // Remove from indices
    this.removeFromUserIndex(existingCase.createdBy, caseId);
    if (existingCase.assignedTo) {
      this.removeFromUserIndex(existingCase.assignedTo, caseId);
    }

    // Audit log before deletion
    auditLog.logDeletion(caseId, userId, reason);

    // Delete case
    this.cases.delete(caseId);

    return true;
  }

  /**
   * Assigns a case to a user
   */
  async assignCase(
    caseId: string,
    assignToUserId: string,
    assignedByUserId: string
  ): Promise<Case> {
    const existingCase = await this.getCase(caseId);

    if (!existingCase) {
      throw new Error(`Case not found: ${caseId}`);
    }

    const previousAssignee = existingCase.assignedTo;

    // Update assignment
    const updatedCase = await this.updateCase(
      caseId,
      { assignedTo: assignToUserId },
      assignedByUserId,
      { skipValidation: true }
    );

    // Log assignment
    auditLog.logAssignment(caseId, assignedByUserId, assignToUserId, previousAssignee);

    return updatedCase;
  }

  /**
   * Unassigns a case
   */
  async unassignCase(caseId: string, unassignedByUserId: string): Promise<Case> {
    const existingCase = await this.getCase(caseId);

    if (!existingCase) {
      throw new Error(`Case not found: ${caseId}`);
    }

    if (!existingCase.assignedTo) {
      throw new Error('Case is not assigned to anyone');
    }

    const previousAssignee = existingCase.assignedTo;

    // Update assignment
    const updatedCase = await this.updateCase(
      caseId,
      { assignedTo: undefined },
      unassignedByUserId,
      { skipValidation: true }
    );

    // Log unassignment
    auditLog.logUnassignment(caseId, unassignedByUserId, previousAssignee);

    return updatedCase;
  }

  /**
   * Changes the status of a case
   */
  async changeStatus(
    caseId: string,
    newStatus: CaseStatus,
    userId: string,
    reason?: string
  ): Promise<Case> {
    const existingCase = await this.getCase(caseId);

    if (!existingCase) {
      throw new Error(`Case not found: ${caseId}`);
    }

    const currentStatus = existingCase.status;

    // Validate status transition
    if (!isValidStatusTransition(currentStatus, newStatus)) {
      throw new InvalidStatusTransitionError(currentStatus, newStatus);
    }

    // Validate completeness for the target status
    const completenessValidation = CaseValidator.validateCompleteness(
      existingCase,
      newStatus
    );

    if (!completenessValidation.isValid) {
      throw new ValidationException(completenessValidation);
    }

    // Update status
    const updatedCase = await this.updateCase(
      caseId,
      { status: newStatus },
      userId,
      { skipValidation: true }
    );

    // Log status change
    auditLog.logStatusChange(caseId, userId, currentStatus, newStatus, reason);

    return updatedCase;
  }

  /**
   * Gets all cases (admin function)
   */
  async getAllCases(): Promise<Case[]> {
    return Array.from(this.cases.values())
      .sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime());
  }

  /**
   * Gets case count
   */
  async getCaseCount(): Promise<number> {
    return this.cases.size;
  }

  /**
   * Gets case count by status
   */
  async getCaseCountByStatus(status: CaseStatus): Promise<number> {
    let count = 0;
    for (const caseData of this.cases.values()) {
      if (caseData.status === status) {
        count++;
      }
    }
    return count;
  }

  /**
   * Gets cases by status
   */
  async getCasesByStatus(status: CaseStatus): Promise<Case[]> {
    const cases: Case[] = [];
    for (const caseData of this.cases.values()) {
      if (caseData.status === status) {
        cases.push(caseData);
      }
    }
    return cases.sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime());
  }

  /**
   * Bulk update cases
   */
  async bulkUpdateCases(
    caseIds: string[],
    updates: Partial<CaseData>,
    userId: string
  ): Promise<Case[]> {
    const updatedCases: Case[] = [];

    for (const caseId of caseIds) {
      try {
        const updated = await this.updateCase(caseId, updates, userId);
        updatedCases.push(updated);
      } catch (error) {
        // Continue with other cases even if one fails
        console.error(`Failed to update case ${caseId}:`, error);
      }
    }

    return updatedCases;
  }

  /**
   * Gets case statistics
   */
  async getStatistics(): Promise<{
    total: number;
    byStatus: Record<CaseStatus, number>;
    byPriority: Record<string, number>;
    recentlyUpdated: number;
    unassigned: number;
  }> {
    const stats = {
      total: this.cases.size,
      byStatus: {} as Record<CaseStatus, number>,
      byPriority: {} as Record<string, number>,
      recentlyUpdated: 0,
      unassigned: 0
    };

    // Initialize status counts
    for (const status of Object.values(CaseStatus)) {
      stats.byStatus[status] = 0;
    }

    const oneDayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000);

    for (const caseData of this.cases.values()) {
      stats.byStatus[caseData.status]++;

      const priority = caseData.priority || 'MEDIUM';
      stats.byPriority[priority] = (stats.byPriority[priority] || 0) + 1;

      if (caseData.updatedAt > oneDayAgo) {
        stats.recentlyUpdated++;
      }

      if (!caseData.assignedTo) {
        stats.unassigned++;
      }
    }

    return stats;
  }

  /**
   * Private helper to generate unique case ID
   */
  private generateId(): string {
    return `case_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Private helper to generate case number
   */
  private generateCaseNumber(): string {
    this.caseCounter++;
    const year = new Date().getFullYear();
    const paddedNumber = String(this.caseCounter).padStart(6, '0');
    return `CASE-${year}-${paddedNumber}`;
  }

  /**
   * Private helper to index case by user
   */
  private indexCaseByUser(userId: string, caseId: string): void {
    if (!this.casesByUser.has(userId)) {
      this.casesByUser.set(userId, new Set());
    }
    this.casesByUser.get(userId)!.add(caseId);
  }

  /**
   * Private helper to remove case from user index
   */
  private removeFromUserIndex(userId: string, caseId: string): void {
    const userCases = this.casesByUser.get(userId);
    if (userCases) {
      userCases.delete(caseId);
      if (userCases.size === 0) {
        this.casesByUser.delete(userId);
      }
    }
  }

  /**
   * Clears all cases (use with caution!)
   */
  clearAllCases(): void {
    this.cases.clear();
    this.casesByUser.clear();
    this.caseCounter = 0;
  }
}

// Singleton instance
export const caseService = new CaseService();
