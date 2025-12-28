/**
 * Case Search Service
 * Provides full-text search, filtering, pagination, and sorting capabilities
 */

import { Case } from './CaseService';
import { CaseStatus } from './CaseStatus';

export interface SearchCriteria {
  query?: string;
  status?: CaseStatus | CaseStatus[];
  assignedTo?: string;
  createdBy?: string;
  priority?: string | string[];
  tags?: string[];
  startDate?: Date | string;
  endDate?: Date | string;
  incidentStartDate?: Date | string;
  incidentEndDate?: Date | string;
  caseNumber?: string;
}

export interface SearchOptions {
  page?: number;
  pageSize?: number;
  sortBy?: SortField;
  sortOrder?: 'asc' | 'desc';
  includeArchived?: boolean;
}

export type SortField =
  | 'caseNumber'
  | 'title'
  | 'createdAt'
  | 'updatedAt'
  | 'incidentDate'
  | 'status'
  | 'priority';

export interface SearchResult {
  cases: Case[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasMore: boolean;
}

/**
 * Case Search Service
 */
export class CaseSearchService {
  /**
   * Searches cases with full-text search and filters
   */
  async searchCases(
    allCases: Case[],
    criteria: SearchCriteria = {},
    options: SearchOptions = {}
  ): Promise<SearchResult> {
    let results = [...allCases];

    // Apply filters
    results = this.applyFilters(results, criteria);

    // Apply sorting
    results = this.applySorting(results, options.sortBy, options.sortOrder);

    // Calculate pagination
    const total = results.length;
    const page = options.page || 1;
    const pageSize = options.pageSize || 20;
    const totalPages = Math.ceil(total / pageSize);
    const startIndex = (page - 1) * pageSize;
    const endIndex = startIndex + pageSize;

    // Apply pagination
    const paginatedResults = results.slice(startIndex, endIndex);

    return {
      cases: paginatedResults,
      total,
      page,
      pageSize,
      totalPages,
      hasMore: page < totalPages
    };
  }

  /**
   * Performs full-text search across case fields
   */
  private fullTextSearch(cases: Case[], query: string): Case[] {
    const searchTerms = query.toLowerCase().trim().split(/\s+/);

    return cases.filter(caseData => {
      const searchableText = this.getSearchableText(caseData).toLowerCase();

      // All search terms must be found
      return searchTerms.every(term => searchableText.includes(term));
    });
  }

  /**
   * Applies all filters to cases
   */
  private applyFilters(cases: Case[], criteria: SearchCriteria): Case[] {
    let results = cases;

    // Full-text search
    if (criteria.query && criteria.query.trim() !== '') {
      results = this.fullTextSearch(results, criteria.query);
    }

    // Status filter
    if (criteria.status) {
      const statuses = Array.isArray(criteria.status)
        ? criteria.status
        : [criteria.status];
      results = results.filter(c => statuses.includes(c.status));
    }

    // Assigned to filter
    if (criteria.assignedTo) {
      results = results.filter(c => c.assignedTo === criteria.assignedTo);
    }

    // Created by filter
    if (criteria.createdBy) {
      results = results.filter(c => c.createdBy === criteria.createdBy);
    }

    // Priority filter
    if (criteria.priority) {
      const priorities = Array.isArray(criteria.priority)
        ? criteria.priority
        : [criteria.priority];
      results = results.filter(c => c.priority && priorities.includes(c.priority));
    }

    // Tags filter (case must have all specified tags)
    if (criteria.tags && criteria.tags.length > 0) {
      results = results.filter(c => {
        if (!c.tags || c.tags.length === 0) return false;
        return criteria.tags!.every(tag => c.tags!.includes(tag));
      });
    }

    // Case number filter
    if (criteria.caseNumber) {
      const searchNumber = criteria.caseNumber.toLowerCase();
      results = results.filter(c =>
        c.caseNumber.toLowerCase().includes(searchNumber)
      );
    }

    // Created date range filter
    if (criteria.startDate) {
      const startDate = new Date(criteria.startDate);
      results = results.filter(c => new Date(c.createdAt) >= startDate);
    }

    if (criteria.endDate) {
      const endDate = new Date(criteria.endDate);
      results = results.filter(c => new Date(c.createdAt) <= endDate);
    }

    // Incident date range filter
    if (criteria.incidentStartDate) {
      const startDate = new Date(criteria.incidentStartDate);
      results = results.filter(c => new Date(c.incidentDate) >= startDate);
    }

    if (criteria.incidentEndDate) {
      const endDate = new Date(criteria.incidentEndDate);
      results = results.filter(c => new Date(c.incidentDate) <= endDate);
    }

    return results;
  }

  /**
   * Applies sorting to results
   */
  private applySorting(
    cases: Case[],
    sortBy: SortField = 'updatedAt',
    sortOrder: 'asc' | 'desc' = 'desc'
  ): Case[] {
    const sorted = [...cases].sort((a, b) => {
      let comparison = 0;

      switch (sortBy) {
        case 'caseNumber':
          comparison = a.caseNumber.localeCompare(b.caseNumber);
          break;

        case 'title':
          comparison = a.title.localeCompare(b.title);
          break;

        case 'createdAt':
          comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
          break;

        case 'updatedAt':
          comparison = new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime();
          break;

        case 'incidentDate':
          comparison = new Date(a.incidentDate).getTime() - new Date(b.incidentDate).getTime();
          break;

        case 'status':
          comparison = a.status.localeCompare(b.status);
          break;

        case 'priority':
          comparison = this.comparePriority(a.priority, b.priority);
          break;

        default:
          comparison = 0;
      }

      return sortOrder === 'asc' ? comparison : -comparison;
    });

    return sorted;
  }

  /**
   * Compares priority levels for sorting
   */
  private comparePriority(a?: string, b?: string): number {
    const priorityOrder = { CRITICAL: 4, HIGH: 3, MEDIUM: 2, LOW: 1 };
    const aPriority = priorityOrder[a as keyof typeof priorityOrder] || 0;
    const bPriority = priorityOrder[b as keyof typeof priorityOrder] || 0;
    return aPriority - bPriority;
  }

  /**
   * Extracts searchable text from a case
   */
  private getSearchableText(caseData: Case): string {
    const parts = [
      caseData.caseNumber,
      caseData.title,
      caseData.description || '',
      caseData.incidentLocation,
      caseData.status,
      caseData.priority || '',
      ...(caseData.tags || [])
    ];

    return parts.join(' ');
  }

  /**
   * Searches cases by tag
   */
  async searchByTag(allCases: Case[], tag: string): Promise<Case[]> {
    return allCases.filter(c =>
      c.tags && c.tags.some(t => t.toLowerCase() === tag.toLowerCase())
    );
  }

  /**
   * Searches cases by status and assigned user
   */
  async searchByStatusAndUser(
    allCases: Case[],
    status: CaseStatus,
    userId: string
  ): Promise<Case[]> {
    return allCases.filter(c =>
      c.status === status && c.assignedTo === userId
    );
  }

  /**
   * Searches cases by date range
   */
  async searchByDateRange(
    allCases: Case[],
    startDate: Date,
    endDate: Date,
    dateField: 'createdAt' | 'updatedAt' | 'incidentDate' = 'createdAt'
  ): Promise<Case[]> {
    return allCases.filter(c => {
      const date = new Date(c[dateField]);
      return date >= startDate && date <= endDate;
    });
  }

  /**
   * Gets cases that are overdue (past estimated completion date)
   */
  async getOverdueCases(allCases: Case[]): Promise<Case[]> {
    const now = new Date();
    return allCases.filter(c =>
      c.estimatedCompletionDate &&
      new Date(c.estimatedCompletionDate) < now &&
      c.status !== CaseStatus.CLOSED &&
      c.status !== CaseStatus.ARCHIVED
    );
  }

  /**
   * Gets unassigned cases
   */
  async getUnassignedCases(allCases: Case[]): Promise<Case[]> {
    return allCases.filter(c => !c.assignedTo);
  }

  /**
   * Gets cases by priority
   */
  async getCasesByPriority(
    allCases: Case[],
    priority: string
  ): Promise<Case[]> {
    return allCases.filter(c => c.priority === priority);
  }

  /**
   * Gets high-priority cases (HIGH and CRITICAL)
   */
  async getHighPriorityCases(allCases: Case[]): Promise<Case[]> {
    return allCases.filter(c =>
      c.priority === 'HIGH' || c.priority === 'CRITICAL'
    );
  }

  /**
   * Advanced search with complex criteria
   */
  async advancedSearch(
    allCases: Case[],
    criteria: {
      textSearch?: string;
      filters?: SearchCriteria;
      options?: SearchOptions;
    }
  ): Promise<SearchResult> {
    const combinedCriteria: SearchCriteria = {
      query: criteria.textSearch,
      ...criteria.filters
    };

    return this.searchCases(allCases, combinedCriteria, criteria.options);
  }

  /**
   * Suggests search terms based on existing case data
   */
  async getSuggestedTags(allCases: Case[]): Promise<string[]> {
    const tagCounts = new Map<string, number>();

    for (const caseData of allCases) {
      if (caseData.tags) {
        for (const tag of caseData.tags) {
          tagCounts.set(tag, (tagCounts.get(tag) || 0) + 1);
        }
      }
    }

    // Sort by frequency
    return Array.from(tagCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .map(([tag]) => tag);
  }

  /**
   * Gets similar cases based on various criteria
   */
  async getSimilarCases(
    allCases: Case[],
    referenceCase: Case,
    limit: number = 5
  ): Promise<Case[]> {
    const similarities = allCases
      .filter(c => c.id !== referenceCase.id)
      .map(c => ({
        case: c,
        score: this.calculateSimilarity(referenceCase, c)
      }))
      .sort((a, b) => b.score - a.score)
      .slice(0, limit);

    return similarities.map(s => s.case);
  }

  /**
   * Calculates similarity score between two cases
   */
  private calculateSimilarity(case1: Case, case2: Case): number {
    let score = 0;

    // Same location
    if (case1.incidentLocation === case2.incidentLocation) {
      score += 3;
    }

    // Same priority
    if (case1.priority === case2.priority) {
      score += 1;
    }

    // Common tags
    if (case1.tags && case2.tags) {
      const commonTags = case1.tags.filter(tag => case2.tags!.includes(tag));
      score += commonTags.length * 2;
    }

    // Similar incident dates (within 30 days)
    const date1 = new Date(case1.incidentDate).getTime();
    const date2 = new Date(case2.incidentDate).getTime();
    const daysDiff = Math.abs(date1 - date2) / (1000 * 60 * 60 * 24);
    if (daysDiff <= 30) {
      score += 2;
    }

    return score;
  }

  /**
   * Exports search results to a structured format
   */
  async exportSearchResults(
    searchResult: SearchResult,
    format: 'json' | 'csv' = 'json'
  ): Promise<string> {
    if (format === 'json') {
      return JSON.stringify(searchResult, null, 2);
    } else {
      return this.convertToCSV(searchResult.cases);
    }
  }

  /**
   * Converts cases to CSV format
   */
  private convertToCSV(cases: Case[]): string {
    if (cases.length === 0) return '';

    const headers = [
      'Case Number',
      'Title',
      'Status',
      'Priority',
      'Incident Date',
      'Location',
      'Assigned To',
      'Created By',
      'Created At',
      'Updated At'
    ];

    const rows = cases.map(c => [
      c.caseNumber,
      c.title,
      c.status,
      c.priority || '',
      (typeof c.incidentDate === 'string' ? c.incidentDate : c.incidentDate.toISOString()).split('T')[0],
      c.incidentLocation,
      c.assignedTo || '',
      c.createdBy,
      typeof c.createdAt === 'string' ? c.createdAt : c.createdAt.toISOString(),
      typeof c.updatedAt === 'string' ? c.updatedAt : c.updatedAt.toISOString()
    ]);

    const csvContent = [
      headers.join(','),
      ...rows.map(row => row.map(cell => `"${cell}"`).join(','))
    ].join('\n');

    return csvContent;
  }
}

// Singleton instance
export const caseSearchService = new CaseSearchService();
