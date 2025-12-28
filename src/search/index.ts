/**
 * AccuScene Enterprise Search Module
 *
 * Advanced full-text search with faceted filtering, auto-complete,
 * and real-time suggestions.
 */

// Export types
export * from './types';

// Export components
export { default as SearchBar } from './SearchBar';
export { default as SearchResults } from './SearchResults';
export { default as FilterPanel } from './FilterPanel';
export { default as FacetFilter } from './FacetFilter';
export { default as DateRangeFilter } from './DateRangeFilter';
export { default as SavedSearches } from './SavedSearches';

// Export hooks
export { useSearch } from './hooks/useSearch';
export { useFilters } from './hooks/useFilters';

// Export services
export { SearchService } from './services/SearchService';

// Export utilities
export * from './utils/queryBuilder';
