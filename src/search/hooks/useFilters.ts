/**
 * Filter state management hook
 */

import { useState, useCallback, useMemo } from 'react';
import type { SearchFilters, DateRange, FilterPreset } from '../types';

export interface UseFiltersResult {
  filters: SearchFilters;

  // Category filters
  toggleCategory: (category: string) => void;
  setCategories: (categories: string[]) => void;
  clearCategories: () => void;

  // Status filters
  toggleStatus: (status: string) => void;
  setStatuses: (statuses: string[]) => void;
  clearStatuses: () => void;

  // Severity filters
  toggleSeverity: (severity: string) => void;
  setSeverities: (severities: string[]) => void;
  clearSeverities: () => void;

  // Tag filters
  toggleTag: (tag: string) => void;
  setTags: (tags: string[]) => void;
  clearTags: () => void;

  // Created by filters
  toggleCreatedBy: (user: string) => void;
  setCreatedBy: (users: string[]) => void;
  clearCreatedBy: () => void;

  // Date range
  setDateRange: (range: DateRange | null) => void;
  setLastNDays: (days: number) => void;
  clearDateRange: () => void;

  // General
  clearAll: () => void;
  hasActiveFilters: boolean;
  activeFilterCount: number;

  // Presets
  applyPreset: (preset: FilterPreset) => void;
  saveAsPreset: (name: string) => FilterPreset;
}

export function useFilters(initialFilters: SearchFilters = {}): UseFiltersResult {
  const [filters, setFilters] = useState<SearchFilters>(initialFilters);

  // Category filters
  const toggleCategory = useCallback((category: string) => {
    setFilters((prev) => {
      const categories = prev.categories || [];
      const newCategories = categories.includes(category)
        ? categories.filter((c) => c !== category)
        : [...categories, category];

      return {
        ...prev,
        categories: newCategories.length > 0 ? newCategories : undefined,
      };
    });
  }, []);

  const setCategories = useCallback((categories: string[]) => {
    setFilters((prev) => ({
      ...prev,
      categories: categories.length > 0 ? categories : undefined,
    }));
  }, []);

  const clearCategories = useCallback(() => {
    setFilters((prev) => ({ ...prev, categories: undefined }));
  }, []);

  // Status filters
  const toggleStatus = useCallback((status: string) => {
    setFilters((prev) => {
      const statuses = prev.statuses || [];
      const newStatuses = statuses.includes(status)
        ? statuses.filter((s) => s !== status)
        : [...statuses, status];

      return {
        ...prev,
        statuses: newStatuses.length > 0 ? newStatuses : undefined,
      };
    });
  }, []);

  const setStatuses = useCallback((statuses: string[]) => {
    setFilters((prev) => ({
      ...prev,
      statuses: statuses.length > 0 ? statuses : undefined,
    }));
  }, []);

  const clearStatuses = useCallback(() => {
    setFilters((prev) => ({ ...prev, statuses: undefined }));
  }, []);

  // Severity filters
  const toggleSeverity = useCallback((severity: string) => {
    setFilters((prev) => {
      const severities = prev.severities || [];
      const newSeverities = severities.includes(severity)
        ? severities.filter((s) => s !== severity)
        : [...severities, severity];

      return {
        ...prev,
        severities: newSeverities.length > 0 ? newSeverities : undefined,
      };
    });
  }, []);

  const setSeverities = useCallback((severities: string[]) => {
    setFilters((prev) => ({
      ...prev,
      severities: severities.length > 0 ? severities : undefined,
    }));
  }, []);

  const clearSeverities = useCallback(() => {
    setFilters((prev) => ({ ...prev, severities: undefined }));
  }, []);

  // Tag filters
  const toggleTag = useCallback((tag: string) => {
    setFilters((prev) => {
      const tags = prev.tags || [];
      const newTags = tags.includes(tag)
        ? tags.filter((t) => t !== tag)
        : [...tags, tag];

      return {
        ...prev,
        tags: newTags.length > 0 ? newTags : undefined,
      };
    });
  }, []);

  const setTags = useCallback((tags: string[]) => {
    setFilters((prev) => ({
      ...prev,
      tags: tags.length > 0 ? tags : undefined,
    }));
  }, []);

  const clearTags = useCallback(() => {
    setFilters((prev) => ({ ...prev, tags: undefined }));
  }, []);

  // Created by filters
  const toggleCreatedBy = useCallback((user: string) => {
    setFilters((prev) => {
      const users = prev.createdBy || [];
      const newUsers = users.includes(user)
        ? users.filter((u) => u !== user)
        : [...users, user];

      return {
        ...prev,
        createdBy: newUsers.length > 0 ? newUsers : undefined,
      };
    });
  }, []);

  const setCreatedBy = useCallback((users: string[]) => {
    setFilters((prev) => ({
      ...prev,
      createdBy: users.length > 0 ? users : undefined,
    }));
  }, []);

  const clearCreatedBy = useCallback(() => {
    setFilters((prev) => ({ ...prev, createdBy: undefined }));
  }, []);

  // Date range
  const setDateRange = useCallback((range: DateRange | null) => {
    setFilters((prev) => ({
      ...prev,
      dateRange: range || undefined,
    }));
  }, []);

  const setLastNDays = useCallback((days: number) => {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - days);

    setFilters((prev) => ({
      ...prev,
      dateRange: { start, end },
    }));
  }, []);

  const clearDateRange = useCallback(() => {
    setFilters((prev) => ({ ...prev, dateRange: undefined }));
  }, []);

  // General
  const clearAll = useCallback(() => {
    setFilters({});
  }, []);

  const hasActiveFilters = useMemo(() => {
    return !!(
      filters.categories?.length ||
      filters.statuses?.length ||
      filters.severities?.length ||
      filters.tags?.length ||
      filters.createdBy?.length ||
      filters.dateRange ||
      filters.custom
    );
  }, [filters]);

  const activeFilterCount = useMemo(() => {
    let count = 0;

    if (filters.categories?.length) count += filters.categories.length;
    if (filters.statuses?.length) count += filters.statuses.length;
    if (filters.severities?.length) count += filters.severities.length;
    if (filters.tags?.length) count += filters.tags.length;
    if (filters.createdBy?.length) count += filters.createdBy.length;
    if (filters.dateRange) count += 1;
    if (filters.custom) count += Object.keys(filters.custom).length;

    return count;
  }, [filters]);

  // Presets
  const applyPreset = useCallback((preset: FilterPreset) => {
    setFilters(preset.filters);
  }, []);

  const saveAsPreset = useCallback(
    (name: string): FilterPreset => {
      return {
        id: `preset-${Date.now()}`,
        name,
        filters: { ...filters },
      };
    },
    [filters]
  );

  return {
    filters,

    // Category
    toggleCategory,
    setCategories,
    clearCategories,

    // Status
    toggleStatus,
    setStatuses,
    clearStatuses,

    // Severity
    toggleSeverity,
    setSeverities,
    clearSeverities,

    // Tags
    toggleTag,
    setTags,
    clearTags,

    // Created by
    toggleCreatedBy,
    setCreatedBy,
    clearCreatedBy,

    // Date range
    setDateRange,
    setLastNDays,
    clearDateRange,

    // General
    clearAll,
    hasActiveFilters,
    activeFilterCount,

    // Presets
    applyPreset,
    saveAsPreset,
  };
}
