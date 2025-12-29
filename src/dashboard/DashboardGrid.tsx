/**
 * Dashboard Grid Component
 *
 * Manages the grid layout system with drag-and-drop, resize, and
 * responsive capabilities
 */

import React, { useMemo, useCallback, useState } from 'react';
import { useDashboard } from './hooks/useDashboard';
import MetricsWidget from './widgets/MetricsWidget';
import ChartWidget from './widgets/ChartWidget';
import TableWidget from './widgets/TableWidget';
import { Breakpoint, GridConfig, WidgetLayout, WidgetType } from './types';

/**
 * Dashboard grid props
 */
interface DashboardGridProps {
  gridConfig: GridConfig;
  breakpoint: Breakpoint;
  animationsEnabled: boolean;
  dragDropEnabled: boolean;
  resizeEnabled: boolean;
}

/**
 * Widget wrapper component
 */
interface WidgetWrapperProps {
  layout: WidgetLayout;
  gridConfig: GridConfig;
  animationsEnabled: boolean;
  dragDropEnabled: boolean;
  resizeEnabled: boolean;
}

const WidgetWrapper: React.FC<WidgetWrapperProps> = ({
  layout,
  gridConfig,
  animationsEnabled,
  dragDropEnabled,
  resizeEnabled,
}) => {
  const { getWidget, isWidgetCollapsed } = useDashboard();
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);

  const widget = getWidget(layout.widgetId);
  const collapsed = isWidgetCollapsed(layout.widgetId);

  const style = useMemo<React.CSSProperties>(() => {
    const x = layout.x * (100 / gridConfig.columns);
    const width = layout.width * (100 / gridConfig.columns);
    const y = layout.y * gridConfig.rowHeight;
    const height = layout.height * gridConfig.rowHeight;

    return {
      position: 'absolute',
      left: `${x}%`,
      top: y + layout.y * gridConfig.verticalGap,
      width: `calc(${width}% - ${gridConfig.horizontalGap}px)`,
      height: collapsed ? 'auto' : height + (layout.height - 1) * gridConfig.verticalGap,
      transition: animationsEnabled && !isDragging && !isResizing
        ? 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)'
        : 'none',
      zIndex: layout.zIndex,
      cursor: dragDropEnabled && !layout.isStatic ? 'move' : 'default',
      opacity: isDragging ? 0.5 : 1,
    };
  }, [
    layout,
    gridConfig,
    animationsEnabled,
    isDragging,
    isResizing,
    dragDropEnabled,
    collapsed,
  ]);

  const handleDragStart = useCallback(
    (e: React.DragEvent) => {
      if (!dragDropEnabled || layout.isStatic) {
        e.preventDefault();
        return;
      }
      setIsDragging(true);
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('widgetId', layout.widgetId);
    },
    [dragDropEnabled, layout.isStatic, layout.widgetId]
  );

  const handleDragEnd = useCallback(() => {
    setIsDragging(false);
  }, []);

  if (!widget) return null;

  return (
    <div
      className="dashboard-widget-wrapper"
      style={style}
      draggable={dragDropEnabled && !layout.isStatic}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      <WidgetRenderer widgetId={layout.widgetId} />
    </div>
  );
};

/**
 * Widget renderer component
 */
interface WidgetRendererProps {
  widgetId: string;
}

const WidgetRenderer: React.FC<WidgetRendererProps> = ({ widgetId }) => {
  const { getWidget } = useDashboard();
  const widget = getWidget(widgetId);

  if (!widget) {
    return (
      <div style={errorContainerStyle}>
        <p>Widget not found: {widgetId}</p>
      </div>
    );
  }

  const { metadata } = widget.config;

  switch (metadata.widgetType) {
    case WidgetType.Metrics:
      return <MetricsWidget widgetId={widgetId} />;
    case WidgetType.Chart:
      return <ChartWidget widgetId={widgetId} />;
    case WidgetType.Table:
      return <TableWidget widgetId={widgetId} />;
    case WidgetType.Text:
    case WidgetType.Image:
    case WidgetType.Custom:
    default:
      return (
        <div style={placeholderStyle}>
          <h3>{metadata.title}</h3>
          <p>Widget type: {metadata.widgetType}</p>
          <p style={{ fontSize: 12, opacity: 0.6 }}>
            (Widget component not implemented)
          </p>
        </div>
      );
  }
};

/**
 * Dashboard Grid Component
 */
const DashboardGrid: React.FC<DashboardGridProps> = ({
  gridConfig,
  breakpoint,
  animationsEnabled,
  dragDropEnabled,
  resizeEnabled,
}) => {
  const { state, updateLayout } = useDashboard();

  const currentLayout = state.layout.breakpointLayouts[breakpoint];

  const containerHeight = useMemo(() => {
    if (!currentLayout || currentLayout.widgets.length === 0) return 400;

    const maxY = Math.max(
      ...currentLayout.widgets.map((w) => w.y + w.height)
    );

    return maxY * gridConfig.rowHeight + (maxY - 1) * gridConfig.verticalGap;
  }, [currentLayout, gridConfig]);

  const containerStyle = useMemo<React.CSSProperties>(
    () => ({
      position: 'relative',
      width: '100%',
      minHeight: containerHeight,
      backgroundColor: 'transparent',
    }),
    [containerHeight]
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const widgetId = e.dataTransfer.getData('widgetId');

      if (!widgetId) return;

      // Calculate new position based on drop location
      const rect = e.currentTarget.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      const columnWidth = rect.width / gridConfig.columns;
      const newX = Math.floor(x / columnWidth);
      const newY = Math.floor(y / (gridConfig.rowHeight + gridConfig.verticalGap));

      // Update widget position
      const updatedWidgets = currentLayout.widgets.map((w) =>
        w.widgetId === widgetId
          ? { ...w, x: Math.max(0, Math.min(newX, gridConfig.columns - w.width)), y: Math.max(0, newY) }
          : w
      );

      updateLayout(breakpoint, updatedWidgets);
    },
    [breakpoint, currentLayout, gridConfig, updateLayout]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  }, []);

  if (!currentLayout) {
    return (
      <div style={emptyStateStyle}>
        <p>No widgets added yet</p>
      </div>
    );
  }

  return (
    <div
      className="dashboard-grid"
      style={containerStyle}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
    >
      {currentLayout.widgets.map((layout) => (
        <WidgetWrapper
          key={layout.widgetId}
          layout={layout}
          gridConfig={gridConfig}
          animationsEnabled={animationsEnabled}
          dragDropEnabled={dragDropEnabled}
          resizeEnabled={resizeEnabled}
        />
      ))}
    </div>
  );
};

/**
 * Styles
 */
const errorContainerStyle: React.CSSProperties = {
  width: '100%',
  height: '100%',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  backgroundColor: '#ffebee',
  color: '#c62828',
  borderRadius: 8,
  padding: 16,
};

const placeholderStyle: React.CSSProperties = {
  width: '100%',
  height: '100%',
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'center',
  backgroundColor: '#f5f5f5',
  border: '2px dashed #ccc',
  borderRadius: 8,
  padding: 16,
  textAlign: 'center',
};

const emptyStateStyle: React.CSSProperties = {
  width: '100%',
  height: 400,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  fontSize: 16,
  color: '#999',
};

export default DashboardGrid;
