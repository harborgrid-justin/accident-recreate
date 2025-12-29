/**
 * AccuScene Enterprise v0.3.0 - Widget Hook
 * Custom hook for widget lifecycle management
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { WidgetConfig, WidgetState } from '../types';

/**
 * Hook for managing widget state and data
 */
export function useWidget(
  initialConfig: WidgetConfig,
  dataSource?: any
): WidgetState {
  const [config, setConfig] = useState<WidgetConfig>(initialConfig);
  const [data, setData] = useState<any>(dataSource);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<Error | null>(null);
  const refreshTimerRef = useRef<NodeJS.Timeout | null>(null);

  /**
   * Update widget configuration
   */
  const update = useCallback((updates: Partial<WidgetConfig>) => {
    setConfig((prev) => ({
      ...prev,
      ...updates,
    }));
  }, []);

  /**
   * Fetch widget data
   */
  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      // In production, this would fetch data based on widget config
      // For now, use the provided dataSource
      await new Promise((resolve) => setTimeout(resolve, 200)); // Simulate network delay

      if (dataSource) {
        setData(dataSource);
      }
    } catch (err) {
      if (err instanceof Error) {
        setError(err);
      }
    } finally {
      setLoading(false);
    }
  }, [dataSource]);

  /**
   * Refresh widget data
   */
  const refresh = useCallback(async () => {
    await fetchData();
  }, [fetchData]);

  // Update data when dataSource changes
  useEffect(() => {
    setData(dataSource);
  }, [dataSource]);

  // Set up auto-refresh if configured
  useEffect(() => {
    if (!config.refreshRate || config.refreshRate <= 0) return;

    // Clear existing timer
    if (refreshTimerRef.current) {
      clearInterval(refreshTimerRef.current);
    }

    // Set up new timer
    refreshTimerRef.current = setInterval(() => {
      fetchData();
    }, config.refreshRate);

    return () => {
      if (refreshTimerRef.current) {
        clearInterval(refreshTimerRef.current);
      }
    };
  }, [config.refreshRate, fetchData]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (refreshTimerRef.current) {
        clearInterval(refreshTimerRef.current);
      }
    };
  }, []);

  return {
    config,
    data,
    loading,
    error,
    update,
    refresh,
  };
}

/**
 * Hook for widget visibility management
 */
export function useWidgetVisibility(initialVisible: boolean = true) {
  const [isVisible, setIsVisible] = useState(initialVisible);
  const elementRef = useRef<HTMLElement | null>(null);

  const show = useCallback(() => setIsVisible(true), []);
  const hide = useCallback(() => setIsVisible(false), []);
  const toggle = useCallback(() => setIsVisible((prev) => !prev), []);

  // Intersection Observer for lazy loading
  useEffect(() => {
    if (!elementRef.current) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        // Widget is visible in viewport
        if (entry.isIntersecting) {
          // Could trigger data loading here
        }
      },
      {
        threshold: 0.1,
        rootMargin: '50px',
      }
    );

    observer.observe(elementRef.current);

    return () => {
      observer.disconnect();
    };
  }, []);

  return {
    isVisible,
    show,
    hide,
    toggle,
    elementRef,
  };
}

/**
 * Hook for widget resize handling
 */
export function useWidgetResize(
  onResize?: (width: number, height: number) => void
) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [size, setSize] = useState({ width: 0, height: 0 });

  useEffect(() => {
    if (!containerRef.current) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        setSize({ width, height });
        onResize?.(width, height);
      }
    });

    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
    };
  }, [onResize]);

  return {
    containerRef,
    width: size.width,
    height: size.height,
  };
}

/**
 * Hook for widget settings management
 */
export function useWidgetSettings<T extends Record<string, any>>(
  initialSettings: T
) {
  const [settings, setSettings] = useState<T>(initialSettings);
  const [isDirty, setIsDirty] = useState(false);

  const updateSetting = useCallback(<K extends keyof T>(
    key: K,
    value: T[K]
  ) => {
    setSettings((prev) => ({
      ...prev,
      [key]: value,
    }));
    setIsDirty(true);
  }, []);

  const updateSettings = useCallback((updates: Partial<T>) => {
    setSettings((prev) => ({
      ...prev,
      ...updates,
    }));
    setIsDirty(true);
  }, []);

  const reset = useCallback(() => {
    setSettings(initialSettings);
    setIsDirty(false);
  }, [initialSettings]);

  const save = useCallback(() => {
    // In production, persist settings to backend
    setIsDirty(false);
    return settings;
  }, [settings]);

  return {
    settings,
    isDirty,
    updateSetting,
    updateSettings,
    reset,
    save,
  };
}

/**
 * Hook for widget data filtering
 */
export function useWidgetFilter<T>(
  data: T[],
  initialFilter?: (item: T) => boolean
) {
  const [filterFn, setFilterFn] = useState<((item: T) => boolean) | undefined>(
    () => initialFilter
  );
  const [filteredData, setFilteredData] = useState<T[]>(data);

  useEffect(() => {
    if (!filterFn) {
      setFilteredData(data);
    } else {
      setFilteredData(data.filter(filterFn));
    }
  }, [data, filterFn]);

  const setFilter = useCallback((fn: ((item: T) => boolean) | undefined) => {
    setFilterFn(() => fn);
  }, []);

  const clearFilter = useCallback(() => {
    setFilterFn(undefined);
  }, []);

  return {
    filteredData,
    setFilter,
    clearFilter,
    hasFilter: !!filterFn,
  };
}

/**
 * Hook for widget data sorting
 */
export function useWidgetSort<T>(data: T[], initialSortKey?: keyof T) {
  const [sortKey, setSortKey] = useState<keyof T | undefined>(initialSortKey);
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
  const [sortedData, setSortedData] = useState<T[]>(data);

  useEffect(() => {
    if (!sortKey) {
      setSortedData(data);
      return;
    }

    const sorted = [...data].sort((a, b) => {
      const aVal = a[sortKey];
      const bVal = b[sortKey];

      if (aVal < bVal) return sortOrder === 'asc' ? -1 : 1;
      if (aVal > bVal) return sortOrder === 'asc' ? 1 : -1;
      return 0;
    });

    setSortedData(sorted);
  }, [data, sortKey, sortOrder]);

  const setSort = useCallback(
    (key: keyof T, order?: 'asc' | 'desc') => {
      if (key === sortKey) {
        // Toggle order if same key
        setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
      } else {
        setSortKey(key);
        setSortOrder(order || 'asc');
      }
    },
    [sortKey, sortOrder]
  );

  const clearSort = useCallback(() => {
    setSortKey(undefined);
    setSortOrder('asc');
  }, []);

  return {
    sortedData,
    sortKey,
    sortOrder,
    setSort,
    clearSort,
  };
}
