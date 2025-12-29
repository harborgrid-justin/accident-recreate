/**
 * Dashboard Context Provider
 *
 * Provides centralized state management for the dashboard system
 */

import React, { createContext, useContext, useReducer, useCallback, useEffect, ReactNode } from 'react';
import {
  DashboardState,
  WidgetState,
  Breakpoint,
  WidgetLayout,
  WidgetConfig,
  WidgetData,
  UserPreferences,
} from '../types';

/**
 * Dashboard action types
 */
type DashboardAction =
  | { type: 'SET_STATE'; payload: DashboardState }
  | { type: 'SET_BREAKPOINT'; payload: Breakpoint }
  | { type: 'ADD_WIDGET'; payload: { config: WidgetConfig; layout: WidgetLayout } }
  | { type: 'REMOVE_WIDGET'; payload: string }
  | { type: 'UPDATE_WIDGET_DATA'; payload: { widgetId: string; data: WidgetData } }
  | { type: 'SET_WIDGET_LOADING'; payload: { widgetId: string; loading: boolean } }
  | { type: 'SET_WIDGET_ERROR'; payload: { widgetId: string; error: string } }
  | { type: 'UPDATE_LAYOUT'; payload: { breakpoint: Breakpoint; widgets: WidgetLayout[] } }
  | { type: 'UPDATE_PREFERENCES'; payload: Partial<UserPreferences> }
  | { type: 'TOGGLE_WIDGET_COLLAPSE'; payload: string };

/**
 * Dashboard context value
 */
interface DashboardContextValue {
  state: DashboardState;
  dispatch: React.Dispatch<DashboardAction>;
  addWidget: (config: WidgetConfig, layout: WidgetLayout) => void;
  removeWidget: (widgetId: string) => void;
  updateWidgetData: (widgetId: string, data: WidgetData) => void;
  setWidgetLoading: (widgetId: string, loading: boolean) => void;
  setWidgetError: (widgetId: string, error: string) => void;
  setBreakpoint: (breakpoint: Breakpoint) => void;
  updateLayout: (breakpoint: Breakpoint, widgets: WidgetLayout[]) => void;
  updatePreferences: (preferences: Partial<UserPreferences>) => void;
  toggleWidgetCollapse: (widgetId: string) => void;
  getWidget: (widgetId: string) => WidgetState | undefined;
  isWidgetCollapsed: (widgetId: string) => boolean;
}

/**
 * Dashboard context
 */
const DashboardContext = createContext<DashboardContextValue | undefined>(undefined);

/**
 * Dashboard reducer
 */
function dashboardReducer(state: DashboardState, action: DashboardAction): DashboardState {
  switch (action.type) {
    case 'SET_STATE':
      return action.payload;

    case 'SET_BREAKPOINT':
      return {
        ...state,
        layout: {
          ...state.layout,
          currentBreakpoint: action.payload,
        },
      };

    case 'ADD_WIDGET': {
      const { config, layout } = action.payload;
      const widgetState: WidgetState = {
        config,
        data: undefined,
        isLoading: false,
        error: undefined,
        lastRefresh: undefined,
        uiState: {},
      };

      return {
        ...state,
        widgets: {
          ...state.widgets,
          [config.metadata.id]: widgetState,
        },
        layout: {
          ...state.layout,
          breakpointLayouts: Object.fromEntries(
            Object.entries(state.layout.breakpointLayouts).map(([bp, bpLayout]) => [
              bp,
              {
                ...bpLayout,
                widgets: [...bpLayout.widgets, layout],
              },
            ])
          ) as Record<Breakpoint, any>,
        },
        version: state.version + 1,
        lastUpdated: new Date().toISOString(),
      };
    }

    case 'REMOVE_WIDGET': {
      const widgetId = action.payload;
      const { [widgetId]: removed, ...remainingWidgets } = state.widgets;

      return {
        ...state,
        widgets: remainingWidgets,
        layout: {
          ...state.layout,
          breakpointLayouts: Object.fromEntries(
            Object.entries(state.layout.breakpointLayouts).map(([bp, bpLayout]) => [
              bp,
              {
                ...bpLayout,
                widgets: bpLayout.widgets.filter((w) => w.widgetId !== widgetId),
              },
            ])
          ) as Record<Breakpoint, any>,
        },
        version: state.version + 1,
        lastUpdated: new Date().toISOString(),
      };
    }

    case 'UPDATE_WIDGET_DATA': {
      const { widgetId, data } = action.payload;
      const widget = state.widgets[widgetId];

      if (!widget) return state;

      return {
        ...state,
        widgets: {
          ...state.widgets,
          [widgetId]: {
            ...widget,
            data,
            isLoading: false,
            error: undefined,
            lastRefresh: new Date().toISOString(),
          },
        },
      };
    }

    case 'SET_WIDGET_LOADING': {
      const { widgetId, loading } = action.payload;
      const widget = state.widgets[widgetId];

      if (!widget) return state;

      return {
        ...state,
        widgets: {
          ...state.widgets,
          [widgetId]: {
            ...widget,
            isLoading: loading,
          },
        },
      };
    }

    case 'SET_WIDGET_ERROR': {
      const { widgetId, error } = action.payload;
      const widget = state.widgets[widgetId];

      if (!widget) return state;

      return {
        ...state,
        widgets: {
          ...state.widgets,
          [widgetId]: {
            ...widget,
            isLoading: false,
            error,
          },
        },
      };
    }

    case 'UPDATE_LAYOUT': {
      const { breakpoint, widgets } = action.payload;

      return {
        ...state,
        layout: {
          ...state.layout,
          breakpointLayouts: {
            ...state.layout.breakpointLayouts,
            [breakpoint]: {
              ...state.layout.breakpointLayouts[breakpoint],
              widgets,
            },
          },
        },
        version: state.version + 1,
        lastUpdated: new Date().toISOString(),
      };
    }

    case 'UPDATE_PREFERENCES':
      return {
        ...state,
        preferences: {
          ...state.preferences,
          ...action.payload,
        },
        lastUpdated: new Date().toISOString(),
      };

    case 'TOGGLE_WIDGET_COLLAPSE': {
      const widgetId = action.payload;
      const collapsed = state.preferences.collapsedWidgets;
      const isCollapsed = collapsed.includes(widgetId);

      return {
        ...state,
        preferences: {
          ...state.preferences,
          collapsedWidgets: isCollapsed
            ? collapsed.filter((id) => id !== widgetId)
            : [...collapsed, widgetId],
        },
      };
    }

    default:
      return state;
  }
}

/**
 * Dashboard provider props
 */
interface DashboardProviderProps {
  children: ReactNode;
  initialState: DashboardState;
  onStateChange?: (state: DashboardState) => void;
}

/**
 * Dashboard context provider
 */
export const DashboardProvider: React.FC<DashboardProviderProps> = ({
  children,
  initialState,
  onStateChange,
}) => {
  const [state, dispatch] = useReducer(dashboardReducer, initialState);

  // Notify parent of state changes
  useEffect(() => {
    if (onStateChange) {
      onStateChange(state);
    }
  }, [state, onStateChange]);

  // Action creators
  const addWidget = useCallback((config: WidgetConfig, layout: WidgetLayout) => {
    dispatch({ type: 'ADD_WIDGET', payload: { config, layout } });
  }, []);

  const removeWidget = useCallback((widgetId: string) => {
    dispatch({ type: 'REMOVE_WIDGET', payload: widgetId });
  }, []);

  const updateWidgetData = useCallback((widgetId: string, data: WidgetData) => {
    dispatch({ type: 'UPDATE_WIDGET_DATA', payload: { widgetId, data } });
  }, []);

  const setWidgetLoading = useCallback((widgetId: string, loading: boolean) => {
    dispatch({ type: 'SET_WIDGET_LOADING', payload: { widgetId, loading } });
  }, []);

  const setWidgetError = useCallback((widgetId: string, error: string) => {
    dispatch({ type: 'SET_WIDGET_ERROR', payload: { widgetId, error } });
  }, []);

  const setBreakpoint = useCallback((breakpoint: Breakpoint) => {
    dispatch({ type: 'SET_BREAKPOINT', payload: breakpoint });
  }, []);

  const updateLayout = useCallback((breakpoint: Breakpoint, widgets: WidgetLayout[]) => {
    dispatch({ type: 'UPDATE_LAYOUT', payload: { breakpoint, widgets } });
  }, []);

  const updatePreferences = useCallback((preferences: Partial<UserPreferences>) => {
    dispatch({ type: 'UPDATE_PREFERENCES', payload: preferences });
  }, []);

  const toggleWidgetCollapse = useCallback((widgetId: string) => {
    dispatch({ type: 'TOGGLE_WIDGET_COLLAPSE', payload: widgetId });
  }, []);

  const getWidget = useCallback(
    (widgetId: string) => state.widgets[widgetId],
    [state.widgets]
  );

  const isWidgetCollapsed = useCallback(
    (widgetId: string) => state.preferences.collapsedWidgets.includes(widgetId),
    [state.preferences.collapsedWidgets]
  );

  const value: DashboardContextValue = {
    state,
    dispatch,
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
  };

  return <DashboardContext.Provider value={value}>{children}</DashboardContext.Provider>;
};

/**
 * Hook to use dashboard context
 */
export const useDashboardContext = (): DashboardContextValue => {
  const context = useContext(DashboardContext);

  if (!context) {
    throw new Error('useDashboardContext must be used within DashboardProvider');
  }

  return context;
};

export default DashboardContext;
