/**
 * AccuScene Enterprise v0.3.0 - Dashboard Grid
 * Drag-and-drop widget grid with responsive layout
 */

import React, { useState, useCallback, useRef, useEffect } from 'react';
import WidgetContainer from './WidgetContainer';
import {
  DashboardLayout,
  WidgetConfig,
  AnalyticsData,
  DashboardTheme,
} from './types';

interface DashboardGridProps {
  layout: DashboardLayout;
  data: AnalyticsData | null;
  isEditing?: boolean;
  onWidgetUpdate?: (config: WidgetConfig) => void;
  onWidgetRemove?: (id: string) => void;
  theme: DashboardTheme;
}

interface DragState {
  widgetId: string;
  startX: number;
  startY: number;
  startGridX: number;
  startGridY: number;
}

interface ResizeState {
  widgetId: string;
  startX: number;
  startY: number;
  startW: number;
  startH: number;
}

const DashboardGrid: React.FC<DashboardGridProps> = ({
  layout,
  data,
  isEditing = false,
  onWidgetUpdate,
  onWidgetRemove,
  theme,
}) => {
  const gridRef = useRef<HTMLDivElement>(null);
  const [dragState, setDragState] = useState<DragState | null>(null);
  const [resizeState, setResizeState] = useState<ResizeState | null>(null);
  const [gridWidth, setGridWidth] = useState(0);

  // Update grid width on resize
  useEffect(() => {
    const updateWidth = () => {
      if (gridRef.current) {
        setGridWidth(gridRef.current.offsetWidth);
      }
    };

    updateWidth();
    window.addEventListener('resize', updateWidth);
    return () => window.removeEventListener('resize', updateWidth);
  }, []);

  // Calculate cell dimensions
  const cellWidth =
    (gridWidth -
      layout.gridSettings.containerPadding[0] * 2 -
      layout.gridSettings.margin[0] * (layout.gridSettings.cols - 1)) /
    layout.gridSettings.cols;
  const cellHeight = layout.gridSettings.rowHeight;

  // Convert grid position to pixels
  const gridToPixels = useCallback(
    (x: number, y: number, w: number, h: number) => {
      return {
        left: x * (cellWidth + layout.gridSettings.margin[0]),
        top: y * (cellHeight + layout.gridSettings.margin[1]),
        width: w * cellWidth + (w - 1) * layout.gridSettings.margin[0],
        height: h * cellHeight + (h - 1) * layout.gridSettings.margin[1],
      };
    },
    [cellWidth, cellHeight, layout.gridSettings.margin]
  );

  // Convert pixels to grid position
  const pixelsToGrid = useCallback(
    (x: number, y: number) => {
      const gridX = Math.round(x / (cellWidth + layout.gridSettings.margin[0]));
      const gridY = Math.round(y / (cellHeight + layout.gridSettings.margin[1]));
      return {
        x: Math.max(0, Math.min(gridX, layout.gridSettings.cols - 1)),
        y: Math.max(0, gridY),
      };
    },
    [cellWidth, cellHeight, layout.gridSettings]
  );

  // Handle drag start
  const handleDragStart = useCallback(
    (widgetId: string, event: React.MouseEvent) => {
      if (!isEditing || !layout.gridSettings.isDraggable) return;

      const widget = layout.widgets.find((w) => w.id === widgetId);
      if (!widget) return;

      setDragState({
        widgetId,
        startX: event.clientX,
        startY: event.clientY,
        startGridX: widget.position.x,
        startGridY: widget.position.y,
      });

      event.preventDefault();
    },
    [isEditing, layout.widgets, layout.gridSettings.isDraggable]
  );

  // Handle drag move
  useEffect(() => {
    if (!dragState) return;

    const handleMouseMove = (event: MouseEvent) => {
      const deltaX = event.clientX - dragState.startX;
      const deltaY = event.clientY - dragState.startY;

      const deltaGrid = pixelsToGrid(deltaX, deltaY);
      const newX = Math.max(
        0,
        Math.min(
          dragState.startGridX + deltaGrid.x,
          layout.gridSettings.cols - 1
        )
      );
      const newY = Math.max(0, dragState.startGridY + deltaGrid.y);

      const widget = layout.widgets.find((w) => w.id === dragState.widgetId);
      if (widget && (newX !== widget.position.x || newY !== widget.position.y)) {
        onWidgetUpdate?.({
          ...widget,
          position: { ...widget.position, x: newX, y: newY },
        });
      }
    };

    const handleMouseUp = () => {
      setDragState(null);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [dragState, layout.widgets, layout.gridSettings.cols, pixelsToGrid, onWidgetUpdate]);

  // Handle resize start
  const handleResizeStart = useCallback(
    (widgetId: string, event: React.MouseEvent) => {
      if (!isEditing || !layout.gridSettings.isResizable) return;

      const widget = layout.widgets.find((w) => w.id === widgetId);
      if (!widget) return;

      setResizeState({
        widgetId,
        startX: event.clientX,
        startY: event.clientY,
        startW: widget.position.w,
        startH: widget.position.h,
      });

      event.preventDefault();
      event.stopPropagation();
    },
    [isEditing, layout.widgets, layout.gridSettings.isResizable]
  );

  // Handle resize move
  useEffect(() => {
    if (!resizeState) return;

    const handleMouseMove = (event: MouseEvent) => {
      const deltaX = event.clientX - resizeState.startX;
      const deltaY = event.clientY - resizeState.startY;

      const deltaGrid = pixelsToGrid(deltaX, deltaY);
      const newW = Math.max(1, resizeState.startW + deltaGrid.x);
      const newH = Math.max(1, resizeState.startH + deltaGrid.y);

      const widget = layout.widgets.find((w) => w.id === resizeState.widgetId);
      if (widget && (newW !== widget.position.w || newH !== widget.position.h)) {
        onWidgetUpdate?.({
          ...widget,
          position: { ...widget.position, w: newW, h: newH },
        });
      }
    };

    const handleMouseUp = () => {
      setResizeState(null);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [resizeState, layout.widgets, pixelsToGrid, onWidgetUpdate]);

  // Calculate grid height
  const maxY = layout.widgets.reduce(
    (max, widget) => Math.max(max, widget.position.y + widget.position.h),
    0
  );
  const gridHeight = maxY * (cellHeight + layout.gridSettings.margin[1]);

  return (
    <div
      ref={gridRef}
      className="dashboard-grid"
      style={{
        position: 'relative',
        width: '100%',
        minHeight: '100%',
        height: gridHeight > 0 ? `${gridHeight}px` : '100%',
      }}
    >
      {layout.widgets
        .filter((w) => w.isVisible)
        .map((widget) => {
          const pos = gridToPixels(
            widget.position.x,
            widget.position.y,
            widget.position.w,
            widget.position.h
          );

          return (
            <WidgetContainer
              key={widget.id}
              config={widget}
              data={data}
              isEditing={isEditing}
              onUpdate={onWidgetUpdate}
              onRemove={onWidgetRemove}
              onDragStart={(e) => handleDragStart(widget.id, e)}
              onResizeStart={(e) => handleResizeStart(widget.id, e)}
              style={{
                position: 'absolute',
                left: `${pos.left}px`,
                top: `${pos.top}px`,
                width: `${pos.width}px`,
                height: `${pos.height}px`,
                transition:
                  dragState?.widgetId === widget.id ||
                  resizeState?.widgetId === widget.id
                    ? 'none'
                    : 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
              }}
              theme={theme}
            />
          );
        })}
    </div>
  );
};

export default DashboardGrid;
