/**
 * Case Validation Service
 * Validates case data completeness, required fields, and relationships
 */

import { CaseStatus } from './CaseStatus';

export interface CaseData {
  id?: string;
  caseNumber?: string;
  title: string;
  description?: string;
  incidentDate: Date | string;
  incidentLocation: string;
  status: CaseStatus;
  assignedTo?: string;
  createdBy: string;
  createdAt?: Date | string;
  updatedAt?: Date | string;
  metadata?: Record<string, any>;
  tags?: string[];
  priority?: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  estimatedCompletionDate?: Date | string;
}

export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  field: string;
  message: string;
  code: string;
}

export interface ValidationWarning {
  field: string;
  message: string;
  code: string;
}

/**
 * Case Validator class for comprehensive validation
 */
export class CaseValidator {
  /**
   * Validates case data for creation
   */
  static validateForCreation(caseData: Partial<CaseData>): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Required fields validation
    if (!caseData.title || caseData.title.trim() === '') {
      errors.push({
        field: 'title',
        message: 'Case title is required',
        code: 'REQUIRED_FIELD'
      });
    } else if (caseData.title.length < 3) {
      errors.push({
        field: 'title',
        message: 'Case title must be at least 3 characters long',
        code: 'MIN_LENGTH'
      });
    } else if (caseData.title.length > 200) {
      errors.push({
        field: 'title',
        message: 'Case title must not exceed 200 characters',
        code: 'MAX_LENGTH'
      });
    }

    if (!caseData.incidentDate) {
      errors.push({
        field: 'incidentDate',
        message: 'Incident date is required',
        code: 'REQUIRED_FIELD'
      });
    } else {
      const incidentDate = new Date(caseData.incidentDate);
      if (isNaN(incidentDate.getTime())) {
        errors.push({
          field: 'incidentDate',
          message: 'Invalid incident date format',
          code: 'INVALID_FORMAT'
        });
      } else if (incidentDate > new Date()) {
        errors.push({
          field: 'incidentDate',
          message: 'Incident date cannot be in the future',
          code: 'INVALID_DATE'
        });
      }
    }

    if (!caseData.incidentLocation || caseData.incidentLocation.trim() === '') {
      errors.push({
        field: 'incidentLocation',
        message: 'Incident location is required',
        code: 'REQUIRED_FIELD'
      });
    } else if (caseData.incidentLocation.length < 3) {
      errors.push({
        field: 'incidentLocation',
        message: 'Incident location must be at least 3 characters long',
        code: 'MIN_LENGTH'
      });
    }

    if (!caseData.createdBy || caseData.createdBy.trim() === '') {
      errors.push({
        field: 'createdBy',
        message: 'Creator user ID is required',
        code: 'REQUIRED_FIELD'
      });
    }

    // Status validation
    if (!caseData.status) {
      // Default to DRAFT if not provided
      warnings.push({
        field: 'status',
        message: 'Status not provided, defaulting to DRAFT',
        code: 'DEFAULT_VALUE'
      });
    } else if (!Object.values(CaseStatus).includes(caseData.status)) {
      errors.push({
        field: 'status',
        message: `Invalid status: ${caseData.status}`,
        code: 'INVALID_VALUE'
      });
    }

    // Priority validation
    if (caseData.priority) {
      const validPriorities = ['LOW', 'MEDIUM', 'HIGH', 'CRITICAL'];
      if (!validPriorities.includes(caseData.priority)) {
        errors.push({
          field: 'priority',
          message: `Invalid priority: ${caseData.priority}. Must be one of: ${validPriorities.join(', ')}`,
          code: 'INVALID_VALUE'
        });
      }
    }

    // Estimated completion date validation
    if (caseData.estimatedCompletionDate) {
      const estDate = new Date(caseData.estimatedCompletionDate);
      if (isNaN(estDate.getTime())) {
        errors.push({
          field: 'estimatedCompletionDate',
          message: 'Invalid estimated completion date format',
          code: 'INVALID_FORMAT'
        });
      } else if (estDate < new Date()) {
        warnings.push({
          field: 'estimatedCompletionDate',
          message: 'Estimated completion date is in the past',
          code: 'PAST_DATE'
        });
      }
    }

    // Tags validation
    if (caseData.tags) {
      if (!Array.isArray(caseData.tags)) {
        errors.push({
          field: 'tags',
          message: 'Tags must be an array',
          code: 'INVALID_TYPE'
        });
      } else {
        caseData.tags.forEach((tag, index) => {
          if (typeof tag !== 'string') {
            errors.push({
              field: `tags[${index}]`,
              message: 'Each tag must be a string',
              code: 'INVALID_TYPE'
            });
          } else if (tag.trim() === '') {
            errors.push({
              field: `tags[${index}]`,
              message: 'Tag cannot be empty',
              code: 'EMPTY_VALUE'
            });
          }
        });
      }
    }

    // Description length validation
    if (caseData.description && caseData.description.length > 5000) {
      errors.push({
        field: 'description',
        message: 'Description must not exceed 5000 characters',
        code: 'MAX_LENGTH'
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings
    };
  }

  /**
   * Validates case data for update
   */
  static validateForUpdate(
    caseData: Partial<CaseData>,
    allowPartial: boolean = true
  ): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // For updates, we can have partial data unless specified otherwise
    if (!allowPartial) {
      return this.validateForCreation(caseData);
    }

    // Title validation (if provided)
    if (caseData.title !== undefined) {
      if (caseData.title.trim() === '') {
        errors.push({
          field: 'title',
          message: 'Case title cannot be empty',
          code: 'REQUIRED_FIELD'
        });
      } else if (caseData.title.length < 3) {
        errors.push({
          field: 'title',
          message: 'Case title must be at least 3 characters long',
          code: 'MIN_LENGTH'
        });
      } else if (caseData.title.length > 200) {
        errors.push({
          field: 'title',
          message: 'Case title must not exceed 200 characters',
          code: 'MAX_LENGTH'
        });
      }
    }

    // Incident date validation (if provided)
    if (caseData.incidentDate !== undefined) {
      const incidentDate = new Date(caseData.incidentDate);
      if (isNaN(incidentDate.getTime())) {
        errors.push({
          field: 'incidentDate',
          message: 'Invalid incident date format',
          code: 'INVALID_FORMAT'
        });
      } else if (incidentDate > new Date()) {
        errors.push({
          field: 'incidentDate',
          message: 'Incident date cannot be in the future',
          code: 'INVALID_DATE'
        });
      }
    }

    // Incident location validation (if provided)
    if (caseData.incidentLocation !== undefined) {
      if (caseData.incidentLocation.trim() === '') {
        errors.push({
          field: 'incidentLocation',
          message: 'Incident location cannot be empty',
          code: 'REQUIRED_FIELD'
        });
      } else if (caseData.incidentLocation.length < 3) {
        errors.push({
          field: 'incidentLocation',
          message: 'Incident location must be at least 3 characters long',
          code: 'MIN_LENGTH'
        });
      }
    }

    // Status validation (if provided)
    if (caseData.status !== undefined) {
      if (!Object.values(CaseStatus).includes(caseData.status)) {
        errors.push({
          field: 'status',
          message: `Invalid status: ${caseData.status}`,
          code: 'INVALID_VALUE'
        });
      }
    }

    // Priority validation (if provided)
    if (caseData.priority !== undefined) {
      const validPriorities = ['LOW', 'MEDIUM', 'HIGH', 'CRITICAL'];
      if (!validPriorities.includes(caseData.priority)) {
        errors.push({
          field: 'priority',
          message: `Invalid priority: ${caseData.priority}. Must be one of: ${validPriorities.join(', ')}`,
          code: 'INVALID_VALUE'
        });
      }
    }

    // Description validation (if provided)
    if (caseData.description !== undefined && caseData.description.length > 5000) {
      errors.push({
        field: 'description',
        message: 'Description must not exceed 5000 characters',
        code: 'MAX_LENGTH'
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings
    };
  }

  /**
   * Validates case completeness for status transitions
   */
  static validateCompleteness(
    caseData: CaseData,
    targetStatus: CaseStatus
  ): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Different statuses may require different levels of completeness
    switch (targetStatus) {
      case CaseStatus.PENDING_REVIEW:
      case CaseStatus.APPROVED:
        // Require description for review
        if (!caseData.description || caseData.description.trim() === '') {
          errors.push({
            field: 'description',
            message: 'Description is required before submitting for review',
            code: 'REQUIRED_FOR_STATUS'
          });
        }

        // Warn if no assigned user
        if (!caseData.assignedTo) {
          warnings.push({
            field: 'assignedTo',
            message: 'Case has no assigned investigator',
            code: 'MISSING_ASSIGNMENT'
          });
        }
        break;

      case CaseStatus.CLOSED:
        // Require description and assignment for closing
        if (!caseData.description || caseData.description.trim() === '') {
          errors.push({
            field: 'description',
            message: 'Description is required before closing case',
            code: 'REQUIRED_FOR_STATUS'
          });
        }

        if (!caseData.assignedTo) {
          errors.push({
            field: 'assignedTo',
            message: 'Case must be assigned before closing',
            code: 'REQUIRED_FOR_STATUS'
          });
        }
        break;
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings
    };
  }

  /**
   * Validates user permissions for case operations
   */
  static validateUserPermissions(
    userId: string,
    caseData: CaseData,
    operation: 'read' | 'update' | 'delete' | 'assign'
  ): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Basic permission checks
    const isCreator = caseData.createdBy === userId;
    const isAssigned = caseData.assignedTo === userId;

    switch (operation) {
      case 'delete':
        if (!isCreator && caseData.status !== CaseStatus.DRAFT) {
          errors.push({
            field: 'userId',
            message: 'Only the creator can delete non-draft cases',
            code: 'PERMISSION_DENIED'
          });
        }
        break;

      case 'assign':
        if (!isCreator && !isAssigned) {
          errors.push({
            field: 'userId',
            message: 'Only the creator or assigned user can reassign cases',
            code: 'PERMISSION_DENIED'
          });
        }
        break;

      case 'update':
        if (!isCreator && !isAssigned) {
          warnings.push({
            field: 'userId',
            message: 'User is neither creator nor assigned to this case',
            code: 'PERMISSION_WARNING'
          });
        }
        break;
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings
    };
  }
}

/**
 * Exception thrown when validation fails
 */
export class ValidationException extends Error {
  constructor(
    public validationResult: ValidationResult
  ) {
    const errorMessages = validationResult.errors
      .map(e => `${e.field}: ${e.message}`)
      .join('; ');

    super(`Validation failed: ${errorMessages}`);
    this.name = 'ValidationException';
  }
}
