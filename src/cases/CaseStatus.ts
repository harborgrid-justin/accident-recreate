/**
 * Case Status Management
 * Defines all possible case statuses and valid transitions between them
 */

export enum CaseStatus {
  DRAFT = 'DRAFT',
  UNDER_INVESTIGATION = 'UNDER_INVESTIGATION',
  PENDING_REVIEW = 'PENDING_REVIEW',
  APPROVED = 'APPROVED',
  CLOSED = 'CLOSED',
  ARCHIVED = 'ARCHIVED'
}

/**
 * Valid status transitions mapping
 * Key: Current status, Value: Array of allowed next statuses
 */
export const VALID_STATUS_TRANSITIONS: Record<CaseStatus, CaseStatus[]> = {
  [CaseStatus.DRAFT]: [
    CaseStatus.UNDER_INVESTIGATION,
    CaseStatus.ARCHIVED
  ],
  [CaseStatus.UNDER_INVESTIGATION]: [
    CaseStatus.DRAFT,
    CaseStatus.PENDING_REVIEW,
    CaseStatus.ARCHIVED
  ],
  [CaseStatus.PENDING_REVIEW]: [
    CaseStatus.UNDER_INVESTIGATION,
    CaseStatus.APPROVED,
    CaseStatus.ARCHIVED
  ],
  [CaseStatus.APPROVED]: [
    CaseStatus.PENDING_REVIEW,
    CaseStatus.CLOSED,
    CaseStatus.ARCHIVED
  ],
  [CaseStatus.CLOSED]: [
    CaseStatus.ARCHIVED
  ],
  [CaseStatus.ARCHIVED]: []
};

/**
 * Status metadata for display and business logic
 */
export interface CaseStatusMetadata {
  label: string;
  description: string;
  color: string;
  canEdit: boolean;
  canDelete: boolean;
  requiresApproval: boolean;
}

export const STATUS_METADATA: Record<CaseStatus, CaseStatusMetadata> = {
  [CaseStatus.DRAFT]: {
    label: 'Draft',
    description: 'Case is being created and has not been submitted',
    color: '#6B7280',
    canEdit: true,
    canDelete: true,
    requiresApproval: false
  },
  [CaseStatus.UNDER_INVESTIGATION]: {
    label: 'Under Investigation',
    description: 'Case is actively being investigated',
    color: '#3B82F6',
    canEdit: true,
    canDelete: false,
    requiresApproval: false
  },
  [CaseStatus.PENDING_REVIEW]: {
    label: 'Pending Review',
    description: 'Case investigation complete, awaiting review',
    color: '#F59E0B',
    canEdit: false,
    canDelete: false,
    requiresApproval: true
  },
  [CaseStatus.APPROVED]: {
    label: 'Approved',
    description: 'Case has been reviewed and approved',
    color: '#10B981',
    canEdit: false,
    canDelete: false,
    requiresApproval: false
  },
  [CaseStatus.CLOSED]: {
    label: 'Closed',
    description: 'Case is closed and finalized',
    color: '#8B5CF6',
    canEdit: false,
    canDelete: false,
    requiresApproval: false
  },
  [CaseStatus.ARCHIVED]: {
    label: 'Archived',
    description: 'Case has been archived for long-term storage',
    color: '#6B7280',
    canEdit: false,
    canDelete: false,
    requiresApproval: false
  }
};

/**
 * Validates if a status transition is allowed
 */
export function isValidStatusTransition(
  currentStatus: CaseStatus,
  newStatus: CaseStatus
): boolean {
  if (currentStatus === newStatus) {
    return true; // Same status is always valid
  }

  const allowedTransitions = VALID_STATUS_TRANSITIONS[currentStatus];
  return allowedTransitions.includes(newStatus);
}

/**
 * Gets metadata for a given status
 */
export function getStatusMetadata(status: CaseStatus): CaseStatusMetadata {
  return STATUS_METADATA[status];
}

/**
 * Gets all allowed next statuses for a current status
 */
export function getAllowedTransitions(currentStatus: CaseStatus): CaseStatus[] {
  return VALID_STATUS_TRANSITIONS[currentStatus];
}

/**
 * Checks if a case can be edited in its current status
 */
export function canEditCase(status: CaseStatus): boolean {
  return STATUS_METADATA[status].canEdit;
}

/**
 * Checks if a case can be deleted in its current status
 */
export function canDeleteCase(status: CaseStatus): boolean {
  return STATUS_METADATA[status].canDelete;
}

/**
 * Exception thrown when an invalid status transition is attempted
 */
export class InvalidStatusTransitionError extends Error {
  constructor(
    public currentStatus: CaseStatus,
    public attemptedStatus: CaseStatus
  ) {
    super(
      `Invalid status transition from ${currentStatus} to ${attemptedStatus}. ` +
      `Allowed transitions: ${VALID_STATUS_TRANSITIONS[currentStatus].join(', ')}`
    );
    this.name = 'InvalidStatusTransitionError';
  }
}
