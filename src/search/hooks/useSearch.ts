/**
 * Search hook with debouncing and state management
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  SearchRequest,
  SearchResponse,
  Suggestion,
  SearchQuery,
  SearchFilters,
} from '../types';
import { SearchService } from '../services/SearchService';

export interface UseSearchOptions {
  debounceMs?: number;
  autoSearch?: boolean;
  enableSuggestions?: boolean;
  service?: SearchService;
}

export interface UseSearchResult {
  // State
  results: SearchResponse | null;
  loading: boolean;
  error: Error | null;
  suggestions: Suggestion[];

  // Actions
  search: (request: SearchRequest) => Promise<void>;
  searchText: (text: string) => Promise<void>;
  reset: () => void;
  cancelSearch: () => void;

  // Query state
  query: SearchQuery;
  setQuery: (query: SearchQuery) => void;
  filters: SearchFilters;
  setFilters: (filters: SearchFilters) => void;
}

export function useSearch(options: UseSearchOptions = {}): UseSearchResult {
  const {
    debounceMs = 300,
    autoSearch = false,
    enableSuggestions = true,
    service = new SearchService(),
  } = options;

  // State
  const [results, setResults] = useState<SearchResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [query, setQuery] = useState<SearchQuery>({ text: '' });
  const [filters, setFilters] = useState<SearchFilters>({});

  // Refs
  const debounceTimer = useRef<NodeJS.Timeout | null>(null);
  const lastSearchRequest = useRef<SearchRequest | null>(null);

  /**
   * Execute search
   */
  const search = useCallback(
    async (request: SearchRequest) => {
      try {
        setLoading(true);
        setError(null);
        lastSearchRequest.current = request;

        const response = await service.search(request);
        setResults(response);
      } catch (err) {
        if (err instanceof Error && err.name !== 'AbortError') {
          setError(err);
          setResults(null);
        }
      } finally {
        setLoading(false);
      }
    },
    [service]
  );

  /**
   * Search with just text
   */
  const searchText = useCallback(
    async (text: string) => {
      const request: SearchRequest = {
        query: { ...query, text },
        filters: Object.keys(filters).length > 0 ? filters : undefined,
      };

      await search(request);
    },
    [query, filters, search]
  );

  /**
   * Get suggestions
   */
  const getSuggestions = useCallback(
    async (text: string) => {
      if (!enableSuggestions || text.length < 2) {
        setSuggestions([]);
        return;
      }

      try {
        const suggestions = await service.getSuggestions(text, 10);
        setSuggestions(suggestions);
      } catch (err) {
        console.error('Failed to get suggestions:', err);
        setSuggestions([]);
      }
    },
    [service, enableSuggestions]
  );

  /**
   * Reset search state
   */
  const reset = useCallback(() => {
    setResults(null);
    setError(null);
    setSuggestions([]);
    setQuery({ text: '' });
    setFilters({});
    service.cancelPending();
  }, [service]);

  /**
   * Cancel pending search
   */
  const cancelSearch = useCallback(() => {
    service.cancelPending();
    setLoading(false);
  }, [service]);

  /**
   * Debounced search effect
   */
  useEffect(() => {
    if (!autoSearch || !query.text) {
      return;
    }

    // Clear existing timer
    if (debounceTimer.current) {
      clearTimeout(debounceTimer.current);
    }

    // Set new timer
    debounceTimer.current = setTimeout(() => {
      searchText(query.text);
    }, debounceMs);

    // Cleanup
    return () => {
      if (debounceTimer.current) {
        clearTimeout(debounceTimer.current);
      }
    };
  }, [query.text, filters, autoSearch, debounceMs, searchText]);

  /**
   * Get suggestions when query changes
   */
  useEffect(() => {
    if (enableSuggestions && query.text) {
      getSuggestions(query.text);
    } else {
      setSuggestions([]);
    }
  }, [query.text, enableSuggestions, getSuggestions]);

  /**
   * Cleanup on unmount
   */
  useEffect(() => {
    return () => {
      service.cancelPending();
      if (debounceTimer.current) {
        clearTimeout(debounceTimer.current);
      }
    };
  }, [service]);

  return {
    // State
    results,
    loading,
    error,
    suggestions,

    // Actions
    search,
    searchText,
    reset,
    cancelSearch,

    // Query state
    query,
    setQuery,
    filters,
    setFilters,
  };
}

/**
 * Hook for search history
 */
export function useSearchHistory(maxItems: number = 10) {
  const [history, setHistory] = useState<string[]>([]);

  const addToHistory = useCallback(
    (query: string) => {
      if (!query.trim()) return;

      setHistory((prev) => {
        const filtered = prev.filter((q) => q !== query);
        return [query, ...filtered].slice(0, maxItems);
      });
    },
    [maxItems]
  );

  const clearHistory = useCallback(() => {
    setHistory([]);
  }, []);

  const removeFromHistory = useCallback((query: string) => {
    setHistory((prev) => prev.filter((q) => q !== query));
  }, []);

  // Load from localStorage on mount
  useEffect(() => {
    const stored = localStorage.getItem('search-history');
    if (stored) {
      try {
        setHistory(JSON.parse(stored));
      } catch (err) {
        console.error('Failed to load search history:', err);
      }
    }
  }, []);

  // Save to localStorage when changed
  useEffect(() => {
    localStorage.setItem('search-history', JSON.stringify(history));
  }, [history]);

  return {
    history,
    addToHistory,
    clearHistory,
    removeFromHistory,
  };
}
