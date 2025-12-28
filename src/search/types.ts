/**
 * TypeScript type definitions for AccuScene Search
 */

export interface SearchQuery {
  text: string;
  fields?: string[];
  fuzzy?: boolean;
  boost?: number;
  operator?: 'AND' | 'OR';
}

export interface SearchFilters {
  categories?: string[];
  statuses?: string[];
  severities?: string[];
  tags?: string[];
  createdBy?: string[];
  dateRange?: DateRange;
  custom?: Record<string, any>;
}

export interface DateRange {
  start: Date;
  end: Date;
}

export interface SearchOptions {
  page?: number;
  perPage?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  highlight?: boolean;
  facets?: string[];
}

export interface SearchRequest {
  query: SearchQuery;
  filters?: SearchFilters;
  options?: SearchOptions;
}

export interface SearchResponse {
  total: number;
  hits: SearchHit[];
  facets?: FacetResult[];
  tookMs: number;
}

export interface SearchHit {
  id: string;
  score: number;
  document: Record<string, any>;
  highlights?: Highlight[];
}

export interface Highlight {
  field: string;
  snippet: string;
}

export interface FacetResult {
  field: string;
  values: FacetValue[];
}

export interface FacetValue {
  value: string;
  count: number;
  selected?: boolean;
}

export interface Suggestion {
  text: string;
  score?: number;
  type?: 'autocomplete' | 'correction' | 'popular';
}

export interface SavedSearch {
  id: string;
  name: string;
  query: SearchQuery;
  filters?: SearchFilters;
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
}

export interface SearchState {
  query: SearchQuery;
  filters: SearchFilters;
  results: SearchResponse | null;
  loading: boolean;
  error: Error | null;
  suggestions: Suggestion[];
}

export interface SearchStats {
  totalDocuments: number;
  indexSizeBytes: number;
  lastUpdated: Date;
}

// Filter presets
export interface FilterPreset {
  id: string;
  name: string;
  filters: SearchFilters;
  icon?: string;
}

// Search history
export interface SearchHistoryItem {
  query: string;
  timestamp: Date;
  resultCount: number;
}

// Export configuration types
export interface SearchConfig {
  apiBaseUrl: string;
  debounceMs: number;
  maxSuggestions: number;
  enableFuzzy: boolean;
  highlightTags: {
    pre: string;
    post: string;
  };
  defaultPageSize: number;
}
