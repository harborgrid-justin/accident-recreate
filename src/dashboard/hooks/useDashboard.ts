/**
 * useDashboard Hook
 *
 * Main hook for interacting with dashboard state and operations
 */

import { useCallback, useEffect, useRef } from 'react';
import { useDashboardContext } from '../context/DashboardContext';
import {
  WidgetConfig,
  WidgetLayout,
  WidgetData,
  Breakpoint,
  WidgetState,
  UserPreferences,
  DashboardState,
} from '../types';

/**
 * Hook return type
 */
interface UseDashboardReturn {
  state: DashboardState;
  currentBreakpoint: Breakpoint;
  widgets: Record<string, WidgetState>;
  preferences: UserPreferences;
  addWidget: (config: WidgetConfig, layout: WidgetLayout) => void;
  removeWidget: (widgetId: string) => void;
  getWidget: (widgetId: string) => WidgetState | undefined;
  updateWidgetData: (widgetId: string, data: WidgetData) => Promise<void>;
  refreshWidget: (widgetId: string) => Promise<void>;
  refreshAllWidgets: () => Promise<void>;
  setWidgetLoading: (widgetId: string, loading: boolean) => void;
  setWidgetError: (widgetId: string, error: string) => void;
  setBreakpoint: (breakpoint: Breakpoint) => void;
  updateLayout: (breakpoint: Breakpoint, widgets: WidgetLayout[]) => void;
  updatePreferences: (preferences: Partial<UserPreferences>) => void;
  toggleWidgetCollapse: (widgetId: string) => void;
  isWidgetCollapsed: (widgetId: string) => boolean;
  getStaleWidgets: (ttlSeconds: number) => string[];
  exportDashboard: () => string;
  importDashboard: (data: string) => void;
}

/**
 * Custom hook for dashboard operations
 */
export function useDashboard(): UseDashboardReturn {
  const {
    state,
    addWidget,
    removeWidget,
    updateWidgetData,
    setWidgetLoading,
    setWidgetError,
    setBreakpoint,
    updateLayout,
    updatePreferences,
    toggleWidgetCollapse,
    getWidget,
    isWidgetCollapsed,
  } = useDashboardContext();

  const refreshTimersRef = useRef<Map<string, NodeJS.Timeout>>(new Map());

  /**
   * Refresh a single widget
   */
  const refreshWidget = useCallback(
    async (widgetId: string) => {
      const widget = getWidget(widgetId);
      if (!widget) {
        throw new Error(`Widget not found: ${widgetId}`);
      }

      setWidgetLoading(widgetId, true);

      try {
        // Simulate data fetching (in production, this would call the actual API)
        await new Promise((resolve) => setTimeout(resolve, 500));

        const mockData: WidgetData = {
          data: { mock: true },
          timestamp: new Date().toISOString(),
          isStale: false,
          metadata: {},
        };

        updateWidgetData(widgetId, mockData);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error';
        setWidgetError(widgetId, errorMessage);
      }
    },
    [getWidget, setWidgetLoading, updateWidgetData, setWidgetError]
  );

  /**
   * Refresh all widgets
   */
  const refreshAllWidgets = useCallback(async () => {
    const widgetIds = Object.keys(state.widgets);
    await Promise.all(widgetIds.map((id) => refreshWidget(id)));
  }, [state.widgets, refreshWidget]);

  /**
   * Get stale widgets based on TTL
   */
  const getStaleWidgets = useCallback(
    (ttlSeconds: number) => {
      const now = new Date();
      const staleWidgetIds: string[] = [];

      Object.entries(state.widgets).forEach(([widgetId, widget]) => {
        if (!widget.lastRefresh) {
          staleWidgetIds.push(widgetId);
          return;
        }

        const lastRefresh = new Date(widget.lastRefresh);
        const ageSeconds = (now.getTime() - lastRefresh.getTime()) / 1000;

        if (ageSeconds > ttlSeconds) {
          staleWidgetIds.push(widgetId);
        }
      });

      return staleWidgetIds;
    },
    [state.widgets]
  );

  /**
   * Export dashboard state as JSON
   */
  const exportDashboard = useCallback(() => {
    return JSON.stringify(state, null, 2);
  }, [state]);

  /**
   * Import dashboard state from JSON
   */
  const importDashboard = useCallback(
    (data: string) => {
      try {
        const importedState: DashboardState = JSON.parse(data);
        // Validate the imported state structure
        if (!importedState.config || !importedState.layout || !importedState.widgets) {
          throw new Error('Invalid dashboard data structure');
        }
        // In a real implementation, this would dispatch a SET_STATE action
        console.log('Imported dashboard state:', importedState);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Import failed';
        throw new Error(`Failed to import dashboard: ${errorMessage}`);
      }
    },
    []
  );

  /**
   * Setup auto-refresh for widgets
   */
  useEffect(() => {
    if (!state.config.refresh.enabled) return;

    const intervalMs = state.config.refresh.intervalSeconds * 1000;

    Object.keys(state.widgets).forEach((widgetId) => {
      const widget = getWidget(widgetId);
      if (!widget || !widget.config.dataSource) return;

      const { refresh } = widget.config.dataSource;

      if (refresh.type === 'interval') {
        const timer = setInterval(() => {
          refreshWidget(widgetId);
        }, refresh.seconds * 1000);

        refreshTimersRef.current.set(widgetId, timer);
      }
    });

    return () => {
      refreshTimersRef.current.forEach((timer) => clearInterval(timer));
      refreshTimersRef.current.clear();
    };
  }, [state.config.refresh, state.widgets, getWidget, refreshWidget]);

  /**
   * Refresh on window focus
   */
  useEffect(() => {
    if (!state.config.refresh.refreshOnFocus) return;

    const handleFocus = () => {
      const staleWidgets = getStaleWidgets(state.config.refresh.staleThresholdSeconds);
      staleWidgets.forEach((widgetId) => refreshWidget(widgetId));
    };

    window.addEventListener('focus', handleFocus);
    return () => window.removeEventListener('focus', handleFocus);
  }, [state.config.refresh, getStaleWidgets, refreshWidget]);

  return {
    state,
    currentBreakpoint: state.layout.currentBreakpoint,
    widgets: state.widgets,
    preferences: state.preferences,
    addWidget,
    removeWidget,
    getWidget,
    updateWidgetData: async (widgetId: string, data: WidgetData) => {
      updateWidgetData(widgetId, data);
    },
    refreshWidget,
    refreshAllWidgets,
    setWidgetLoading,
    setWidgetError,
    setBreakpoint,
    updateLayout,
    updatePreferences,
    toggleWidgetCollapse,
    isWidgetCollapsed,
    getStaleWidgets,
    exportDashboard,
    importDashboard,
  };
}

/**
 * Hook for a specific widget
 */
export function useWidget(widgetId: string) {
  const {
    getWidget,
    updateWidgetData,
    setWidgetLoading,
    setWidgetError,
    toggleWidgetCollapse,
    isWidgetCollapsed,
  } = useDashboard();

  const widget = getWidget(widgetId);

  const updateData = useCallback(
    async (data: WidgetData) => {
      await updateWidgetData(widgetId, data);
    },
    [widgetId, updateWidgetData]
  );

  const setLoading = useCallback(
    (loading: boolean) => {
      setWidgetLoading(widgetId, loading);
    },
    [widgetId, setWidgetLoading]
  );

  const setError = useCallback(
    (error: string) => {
      setWidgetError(widgetId, error);
    },
    [widgetId, setWidgetError]
  );

  const toggleCollapse = useCallback(() => {
    toggleWidgetCollapse(widgetId);
  }, [widgetId, toggleWidgetCollapse]);

  const collapsed = isWidgetCollapsed(widgetId);

  return {
    widget,
    updateData,
    setLoading,
    setError,
    toggleCollapse,
    collapsed,
    isLoading: widget?.isLoading ?? false,
    error: widget?.error,
    data: widget?.data,
  };
}

export default useDashboard;
