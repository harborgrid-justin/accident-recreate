/**
 * AccuScene Enterprise Dashboard v0.2.5
 *
 * Main export file for the dashboard system
 */

// Core components
export { default as DashboardContainer } from './DashboardContainer';
export { default as DashboardGrid } from './DashboardGrid';

// Context
export { DashboardProvider, useDashboardContext } from './context/DashboardContext';

// Hooks
export { useDashboard, useWidget } from './hooks/useDashboard';
export {
  useResponsive,
  useMediaQuery,
  useOrientation,
  useTouch,
  getBreakpointFromWidth,
  getDefaultGridConfig,
} from './hooks/useResponsive';

// Widgets
export { default as MetricsWidget } from './widgets/MetricsWidget';
export { default as ChartWidget } from './widgets/ChartWidget';
export { default as TableWidget } from './widgets/TableWidget';

// Types
export * from './types';

// Version
export const DASHBOARD_VERSION = '0.2.5';

/**
 * Default theme configuration
 */
export const DEFAULT_THEME = {
  primaryColor: '#1976d2',
  secondaryColor: '#dc004e',
  backgroundColor: '#f5f5f5',
  surfaceColor: '#ffffff',
  textColor: '#212121',
  darkMode: false,
  borderRadius: 8,
  spacingUnit: 8,
};

/**
 * Default grid configurations for all breakpoints
 */
export const DEFAULT_GRID_CONFIGS = {
  mobile: {
    columns: 4,
    rowHeight: 60,
    horizontalGap: 8,
    verticalGap: 8,
    containerPadding: 12,
  },
  'mobile-landscape': {
    columns: 6,
    rowHeight: 70,
    horizontalGap: 12,
    verticalGap: 12,
    containerPadding: 16,
  },
  tablet: {
    columns: 8,
    rowHeight: 80,
    horizontalGap: 16,
    verticalGap: 16,
    containerPadding: 20,
  },
  desktop: {
    columns: 12,
    rowHeight: 80,
    horizontalGap: 16,
    verticalGap: 16,
    containerPadding: 24,
  },
  'desktop-large': {
    columns: 12,
    rowHeight: 80,
    horizontalGap: 16,
    verticalGap: 16,
    containerPadding: 24,
  },
  'desktop-xl': {
    columns: 16,
    rowHeight: 90,
    horizontalGap: 20,
    verticalGap: 20,
    containerPadding: 32,
  },
};

/**
 * Create a default dashboard configuration
 */
export function createDefaultDashboardConfig(name: string, options?: {
  description?: string;
  darkMode?: boolean;
  refreshInterval?: number;
}): import('./types').DashboardConfig {
  const id = `dashboard-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

  return {
    id,
    name,
    description: options?.description,
    theme: {
      ...DEFAULT_THEME,
      darkMode: options?.darkMode ?? false,
    },
    gridConfigs: DEFAULT_GRID_CONFIGS as any,
    refresh: {
      enabled: true,
      intervalSeconds: options?.refreshInterval ?? 30,
      refreshOnFocus: true,
      staleThresholdSeconds: 300,
    },
    animationsEnabled: true,
    dragDropEnabled: true,
    resizeEnabled: true,
    maxWidgets: 50,
    persistenceEnabled: true,
    version: 0,
  };
}

/**
 * Create a widget configuration helper
 */
export function createWidgetConfig(
  widgetType: import('./types').WidgetType,
  title: string,
  options?: {
    description?: string;
    dataSource?: import('./types').DataSource;
    config?: Record<string, any>;
    showTitle?: boolean;
    showBorder?: boolean;
  }
): import('./types').WidgetConfig {
  const id = `widget-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

  return {
    metadata: {
      id,
      widgetType,
      title,
      description: options?.description,
      tags: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      version: 0,
    },
    dataSource: options?.dataSource,
    config: options?.config || {},
    display: {
      showTitle: options?.showTitle ?? true,
      showBorder: options?.showBorder ?? true,
      backgroundColor: undefined,
      textColor: undefined,
      padding: 16,
      cssClasses: [],
    },
    interaction: {
      clickable: false,
      hoverable: true,
      drilldownEnabled: false,
      exportEnabled: true,
      fullscreenEnabled: true,
    },
  };
}

/**
 * Create a widget layout helper
 */
export function createWidgetLayout(
  widgetId: string,
  position: { x: number; y: number },
  size: { width: number; height: number },
  options?: {
    minWidth?: number;
    maxWidth?: number;
    minHeight?: number;
    maxHeight?: number;
    isStatic?: boolean;
    isResizable?: boolean;
    zIndex?: number;
  }
): import('./types').WidgetLayout {
  return {
    widgetId,
    x: position.x,
    y: position.y,
    width: size.width,
    height: size.height,
    minWidth: options?.minWidth,
    maxWidth: options?.maxWidth,
    minHeight: options?.minHeight,
    maxHeight: options?.maxHeight,
    isStatic: options?.isStatic ?? false,
    isResizable: options?.isResizable ?? true,
    zIndex: options?.zIndex ?? 0,
  };
}

/**
 * Dashboard builder class for fluent API
 */
export class DashboardBuilder {
  private config: import('./types').DashboardConfig;

  constructor(name: string) {
    this.config = createDefaultDashboardConfig(name);
  }

  description(desc: string): this {
    this.config.description = desc;
    return this;
  }

  darkMode(enabled: boolean = true): this {
    this.config.theme.darkMode = enabled;
    return this;
  }

  refreshInterval(seconds: number): this {
    this.config.refresh.intervalSeconds = seconds;
    return this;
  }

  animations(enabled: boolean): this {
    this.config.animationsEnabled = enabled;
    return this;
  }

  dragDrop(enabled: boolean): this {
    this.config.dragDropEnabled = enabled;
    return this;
  }

  resize(enabled: boolean): this {
    this.config.resizeEnabled = enabled;
    return this;
  }

  maxWidgets(max: number): this {
    this.config.maxWidgets = max;
    return this;
  }

  build(): import('./types').DashboardConfig {
    return this.config;
  }
}

/**
 * Utility: Format metric value
 */
export function formatMetricValue(value: number, format: import('./types').MetricFormat): string {
  switch (format) {
    case 'number':
      if (Math.abs(value) >= 1_000_000) return `${(value / 1_000_000).toFixed(2)}M`;
      if (Math.abs(value) >= 1_000) return `${(value / 1_000).toFixed(2)}K`;
      return value.toFixed(2);
    case 'percentage':
      return `${value.toFixed(1)}%`;
    case 'currency':
      return `$${value.toFixed(2)}`;
    case 'duration':
      if (value >= 3600) return `${(value / 3600).toFixed(1)}h`;
      if (value >= 60) return `${(value / 60).toFixed(1)}m`;
      return `${value.toFixed(0)}s`;
    case 'bytes':
      if (value >= 1_073_741_824) return `${(value / 1_073_741_824).toFixed(2)} GB`;
      if (value >= 1_048_576) return `${(value / 1_048_576).toFixed(2)} MB`;
      if (value >= 1024) return `${(value / 1024).toFixed(2)} KB`;
      return `${value.toFixed(0)} B`;
    default:
      return value.toString();
  }
}

/**
 * Utility: Validate dashboard configuration
 */
export function validateDashboardConfig(config: import('./types').DashboardConfig): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (!config.id) errors.push('Dashboard ID is required');
  if (!config.name || config.name.length === 0) errors.push('Dashboard name is required');
  if (config.maxWidgets < 1 || config.maxWidgets > 100) {
    errors.push('Max widgets must be between 1 and 100');
  }
  if (config.refresh.intervalSeconds < 1 || config.refresh.intervalSeconds > 3600) {
    errors.push('Refresh interval must be between 1 and 3600 seconds');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}
