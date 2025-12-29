/**
 * Search API Service
 * Handles all search-related API calls
 */

import type {
  SearchRequest,
  SearchResponse,
  SearchFilters,
  SearchQuery,
  Suggestion,
  SavedSearch,
  SearchStats,
  FacetResult,
} from '../types';

export class SearchService {
  private baseUrl: string;
  private abortController: AbortController | null = null;

  constructor(baseUrl: string = '/api/search') {
    this.baseUrl = baseUrl;
  }

  /**
   * Execute a search query
   */
  async search(request: SearchRequest): Promise<SearchResponse> {
    // Cancel any pending requests
    this.cancelPending();

    this.abortController = new AbortController();

    const response = await fetch(`${this.baseUrl}/query`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
      signal: this.abortController.signal,
    });

    if (!response.ok) {
      throw new Error(`Search failed: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Get auto-complete suggestions
   */
  async getSuggestions(
    prefix: string,
    limit: number = 10
  ): Promise<Suggestion[]> {
    const response = await fetch(
      `${this.baseUrl}/suggest?prefix=${encodeURIComponent(prefix)}&limit=${limit}`
    );

    if (!response.ok) {
      throw new Error(`Suggestions failed: ${response.statusText}`);
    }

    const suggestions: string[] = await response.json();
    return suggestions.map((text) => ({
      text,
      type: 'autocomplete' as const,
    }));
  }

  /**
   * Get fuzzy suggestions (for typo correction)
   */
  async getFuzzySuggestions(
    query: string,
    limit: number = 5
  ): Promise<Suggestion[]> {
    const response = await fetch(
      `${this.baseUrl}/suggest/fuzzy?query=${encodeURIComponent(query)}&limit=${limit}`
    );

    if (!response.ok) {
      throw new Error(`Fuzzy suggestions failed: ${response.statusText}`);
    }

    const suggestions: Array<[string, number]> = await response.json();
    return suggestions.map(([text, score]) => ({
      text,
      score,
      type: 'correction' as const,
    }));
  }

  /**
   * Get popular/trending queries
   */
  async getPopularQueries(limit: number = 10): Promise<Suggestion[]> {
    const response = await fetch(
      `${this.baseUrl}/popular?limit=${limit}`
    );

    if (!response.ok) {
      throw new Error(`Popular queries failed: ${response.statusText}`);
    }

    const queries: Array<[string, number]> = await response.json();
    return queries.map(([text, count]) => ({
      text,
      score: count,
      type: 'popular' as const,
    }));
  }

  /**
   * Get facets for a field
   */
  async getFacets(
    field: string,
    filters?: SearchFilters
  ): Promise<FacetResult> {
    const response = await fetch(`${this.baseUrl}/facets/${field}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ filters }),
    });

    if (!response.ok) {
      throw new Error(`Facets failed: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Save a search
   */
  async saveSearch(
    name: string,
    query: SearchQuery,
    filters?: SearchFilters
  ): Promise<SavedSearch> {
    const response = await fetch(`${this.baseUrl}/saved`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ name, query, filters }),
    });

    if (!response.ok) {
      throw new Error(`Save search failed: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Get saved searches
   */
  async getSavedSearches(): Promise<SavedSearch[]> {
    const response = await fetch(`${this.baseUrl}/saved`);

    if (!response.ok) {
      throw new Error(`Get saved searches failed: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Delete a saved search
   */
  async deleteSavedSearch(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/saved/${id}`, {
      method: 'DELETE',
    });

    if (!response.ok) {
      throw new Error(`Delete saved search failed: ${response.statusText}`);
    }
  }

  /**
   * Get search statistics
   */
  async getStats(): Promise<SearchStats> {
    const response = await fetch(`${this.baseUrl}/stats`);

    if (!response.ok) {
      throw new Error(`Get stats failed: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Index a document
   */
  async indexDocument(id: string, document: Record<string, any>): Promise<void> {
    const response = await fetch(`${this.baseUrl}/index`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ id, document }),
    });

    if (!response.ok) {
      throw new Error(`Index document failed: ${response.statusText}`);
    }
  }

  /**
   * Batch index documents
   */
  async indexBatch(
    documents: Array<{ id: string; document: Record<string, any> }>
  ): Promise<void> {
    const response = await fetch(`${this.baseUrl}/index/batch`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ documents }),
    });

    if (!response.ok) {
      throw new Error(`Batch index failed: ${response.statusText}`);
    }
  }

  /**
   * Delete a document
   */
  async deleteDocument(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/index/${id}`, {
      method: 'DELETE',
    });

    if (!response.ok) {
      throw new Error(`Delete document failed: ${response.statusText}`);
    }
  }

  /**
   * Commit pending changes
   */
  async commit(): Promise<void> {
    const response = await fetch(`${this.baseUrl}/index/commit`, {
      method: 'POST',
    });

    if (!response.ok) {
      throw new Error(`Commit failed: ${response.statusText}`);
    }
  }

  /**
   * Cancel any pending search requests
   */
  cancelPending(): void {
    if (this.abortController) {
      this.abortController.abort();
      this.abortController = null;
    }
  }
}

// Default singleton instance
export const searchService = new SearchService();
