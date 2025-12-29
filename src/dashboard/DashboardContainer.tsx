/**
 * Dashboard Container Component
 *
 * Main container component that manages the dashboard layout and
 * provides responsive behavior
 */

import React, { useEffect, useMemo } from 'react';
import { DashboardProvider } from './context/DashboardContext';
import DashboardGrid from './DashboardGrid';
import { useResponsive } from './hooks/useResponsive';
import { DashboardState, DashboardConfig } from './types';

/**
 * Dashboard container props
 */
interface DashboardContainerProps {
  config: DashboardConfig;
  initialState?: DashboardState;
  onStateChange?: (state: DashboardState) => void;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * Create default dashboard state
 */
function createDefaultState(config: DashboardConfig): DashboardState {
  return {
    config,
    layout: {
      dashboardId: config.id,
      breakpointLayouts: Object.fromEntries(
        Object.keys(config.gridConfigs).map((bp) => [
          bp,
          {
            breakpoint: bp,
            widgets: [],
            columns: config.gridConfigs[bp as keyof typeof config.gridConfigs].columns,
            autoCompact: true,
          },
        ])
      ) as any,
      currentBreakpoint: 'desktop' as any,
    },
    widgets: {},
    preferences: {
      theme: config.theme.darkMode ? 'dark' : 'light',
      autoRefresh: config.refresh.enabled,
      defaultPageSize: 25,
      collapsedWidgets: [],
      custom: {},
    },
    lastUpdated: new Date().toISOString(),
    version: 0,
  };
}

/**
 * Dashboard Container Component
 */
const DashboardContainer: React.FC<DashboardContainerProps> = ({
  config,
  initialState,
  onStateChange,
  className = '',
  style = {},
}) => {
  const { breakpoint, width } = useResponsive();

  const state = useMemo(
    () => initialState || createDefaultState(config),
    [initialState, config]
  );

  const containerStyle = useMemo<React.CSSProperties>(
    () => ({
      width: '100%',
      minHeight: '100vh',
      backgroundColor: config.theme.backgroundColor,
      color: config.theme.textColor,
      padding: config.gridConfigs[breakpoint]?.containerPadding || 24,
      boxSizing: 'border-box',
      ...style,
    }),
    [config, breakpoint, style]
  );

  return (
    <DashboardProvider initialState={state} onStateChange={onStateChange}>
      <DashboardContainerInner
        config={config}
        className={className}
        style={containerStyle}
      />
    </DashboardProvider>
  );
};

/**
 * Inner container component (within provider)
 */
interface DashboardContainerInnerProps {
  config: DashboardConfig;
  className: string;
  style: React.CSSProperties;
}

const DashboardContainerInner: React.FC<DashboardContainerInnerProps> = ({
  config,
  className,
  style,
}) => {
  const { breakpoint } = useResponsive();
  const { setBreakpoint } = useDashboard();

  // Update breakpoint in state when it changes
  useEffect(() => {
    setBreakpoint(breakpoint);
  }, [breakpoint, setBreakpoint]);

  const gridConfig = config.gridConfigs[breakpoint];

  return (
    <div className={`accuscene-dashboard ${className}`} style={style}>
      <div className="dashboard-header" style={headerStyle}>
        <h1 style={titleStyle}>{config.name}</h1>
        {config.description && (
          <p style={descriptionStyle}>{config.description}</p>
        )}
      </div>

      <div className="dashboard-body" style={bodyStyle}>
        <DashboardGrid
          gridConfig={gridConfig}
          breakpoint={breakpoint}
          animationsEnabled={config.animationsEnabled}
          dragDropEnabled={config.dragDropEnabled}
          resizeEnabled={config.resizeEnabled}
        />
      </div>
    </div>
  );
};

// Lazy import for circular dependency
import { useDashboard } from './hooks/useDashboard';

/**
 * Styles
 */
const headerStyle: React.CSSProperties = {
  marginBottom: 24,
};

const titleStyle: React.CSSProperties = {
  margin: 0,
  fontSize: 28,
  fontWeight: 600,
  lineHeight: 1.2,
};

const descriptionStyle: React.CSSProperties = {
  margin: '8px 0 0',
  fontSize: 14,
  opacity: 0.7,
};

const bodyStyle: React.CSSProperties = {
  position: 'relative',
};

export default DashboardContainer;
