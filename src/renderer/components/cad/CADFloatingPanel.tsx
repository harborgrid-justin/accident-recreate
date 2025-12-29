/**
 * AccuScene Enterprise v0.3.0 - CAD Floating Panel Component
 */

import React, { useState, useRef, useEffect } from 'react';
import { FloatingPanelConfig, Point2D } from './types';

interface CADFloatingPanelProps {
  config: FloatingPanelConfig;
  children: React.ReactNode;
  onClose?: () => void;
  onPositionChange?: (position: Point2D) => void;
  onSizeChange?: (size: { width: number; height: number }) => void;
  className?: string;
}

export const CADFloatingPanel: React.FC<CADFloatingPanelProps> = ({
  config,
  children,
  onClose,
  onPositionChange,
  onSizeChange,
  className = '',
}) => {
  const [position, setPosition] = useState<Point2D>(config.defaultPosition);
  const [size, setSize] = useState(config.defaultSize);
  const [collapsed, setCollapsed] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const [dragOffset, setDragOffset] = useState<Point2D>({ x: 0, y: 0 });

  const panelRef = useRef<HTMLDivElement>(null);

  const handleMouseDown = (e: React.MouseEvent, action: 'drag' | 'resize') => {
    e.preventDefault();

    if (action === 'drag') {
      setIsDragging(true);
      setDragOffset({
        x: e.clientX - position.x,
        y: e.clientY - position.y,
      });
    } else if (action === 'resize' && config.resizable) {
      setIsResizing(true);
    }
  };

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (isDragging) {
        const newPosition = {
          x: e.clientX - dragOffset.x,
          y: e.clientY - dragOffset.y,
        };
        setPosition(newPosition);
        onPositionChange?.(newPosition);
      } else if (isResizing) {
        const newSize = {
          width: Math.max(
            config.minSize?.width || 200,
            e.clientX - position.x
          ),
          height: Math.max(
            config.minSize?.height || 150,
            e.clientY - position.y
          ),
        };
        setSize(newSize);
        onSizeChange?.(newSize);
      }
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      setIsResizing(false);
    };

    if (isDragging || isResizing) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, isResizing, dragOffset, position, config, onPositionChange, onSizeChange]);

  return (
    <div
      ref={panelRef}
      className={`
        fixed
        bg-white dark:bg-gray-900
        border border-gray-200 dark:border-gray-700
        rounded-lg shadow-2xl
        flex flex-col
        ${isDragging || isResizing ? 'select-none' : ''}
        ${className}
      `}
      style={{
        left: `${position.x}px`,
        top: `${position.y}px`,
        width: collapsed ? 'auto' : `${size.width}px`,
        height: collapsed ? 'auto' : `${size.height}px`,
        zIndex: 1000,
      }}
    >
      {/* Header */}
      <div
        className={`
          flex items-center justify-between
          px-3 py-2
          bg-gray-100 dark:bg-gray-800
          border-b border-gray-200 dark:border-gray-700
          rounded-t-lg
          ${isDragging ? 'cursor-grabbing' : 'cursor-grab'}
        `}
        onMouseDown={(e) => handleMouseDown(e, 'drag')}
      >
        <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
          {config.title}
        </h3>

        <div className="flex items-center gap-1">
          {/* Collapse button */}
          {config.collapsible && (
            <button
              onClick={() => setCollapsed(!collapsed)}
              className="
                p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700
                transition-colors
              "
              aria-label={collapsed ? 'Expand' : 'Collapse'}
            >
              <span className="text-gray-600 dark:text-gray-400 text-xs">
                {collapsed ? '▢' : '─'}
              </span>
            </button>
          )}

          {/* Close button */}
          {config.closable && onClose && (
            <button
              onClick={onClose}
              className="
                p-1 rounded hover:bg-red-100 dark:hover:bg-red-900
                transition-colors
              "
              aria-label="Close"
            >
              <span className="text-red-500 text-xs">✕</span>
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      {!collapsed && (
        <div className="flex-1 overflow-auto p-3">
          {children}
        </div>
      )}

      {/* Resize Handle */}
      {config.resizable && !collapsed && (
        <div
          className="
            absolute bottom-0 right-0
            w-4 h-4
            cursor-nwse-resize
            hover:bg-blue-500 hover:bg-opacity-30
            transition-colors
          "
          onMouseDown={(e) => handleMouseDown(e, 'resize')}
          style={{
            background: `linear-gradient(135deg, transparent 0%, transparent 50%, currentColor 50%, currentColor 100%)`,
            color: '#9ca3af',
          }}
        />
      )}
    </div>
  );
};
