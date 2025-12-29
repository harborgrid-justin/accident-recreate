/**
 * AccuScene Enterprise v0.3.0 - CAD Workspace Component
 * Main container integrating all CAD UI components
 */

import React, { useState, useCallback, useRef, useEffect } from 'react';
import { CADToolbar } from './CADToolbar';
import { CADPropertyPanel } from './CADPropertyPanel';
import { CADLayerPanel } from './CADLayerPanel';
import { CADCommandPalette } from './CADCommandPalette';
import { CADMeasurementTools } from './CADMeasurementTools';
import { CADSnapSystem } from './CADSnapSystem';
import { CADRulers } from './CADRulers';
import { CADGrid } from './CADGrid';
import { CADContextMenu } from './CADContextMenu';
import { CADStatusBar } from './CADStatusBar';
import { useCADTool } from './hooks/useCADTool';
import { useLayerManagement } from './hooks/useLayerManagement';
import { useSnapPoint } from './hooks/useSnapPoint';
import {
  Point2D,
  ContextMenuItem,
  Command,
  PropertySection,
  PropertyValue,
  CADWorkspaceConfig,
  ViewportState,
  CADToolType,
} from './types';

interface CADWorkspaceProps {
  config?: Partial<CADWorkspaceConfig>;
  onConfigChange?: (config: CADWorkspaceConfig) => void;
  className?: string;
}

const defaultConfig: CADWorkspaceConfig = {
  theme: 'light',
  showGrid: true,
  showRulers: true,
  showStatusBar: true,
  showToolbar: true,
  showLayerPanel: true,
  showPropertyPanel: true,
  snapSettings: {
    enabled: true,
    gridSnap: true,
    objectSnap: true,
    snapTypes: [],
    snapDistance: 10,
    gridSize: 20,
  },
  gridSettings: {
    enabled: true,
    type: 'line',
    majorSpacing: 100,
    minorSpacing: 20,
    majorColor: '#cbd5e1',
    minorColor: '#e2e8f0',
    opacity: 0.5,
  },
  rulerSettings: {
    enabled: true,
    size: 30,
    backgroundColor: '#f3f4f6',
    textColor: '#374151',
    lineColor: '#9ca3af',
    unit: 'px',
  },
};

export const CADWorkspace: React.FC<CADWorkspaceProps> = ({
  config: userConfig,
  onConfigChange,
  className = '',
}) => {
  const [config, setConfig] = useState<CADWorkspaceConfig>({
    ...defaultConfig,
    ...userConfig,
  });

  const [viewport, setViewport] = useState<ViewportState>({
    zoom: 1,
    panX: 0,
    panY: 0,
    rotation: 0,
    width: 1920,
    height: 1080,
  });

  const [cursorPosition, setCursorPosition] = useState<Point2D | null>(null);
  const [contextMenuPosition, setContextMenuPosition] = useState<Point2D | null>(null);
  const [selectedObjects, setSelectedObjects] = useState<string[]>([]);
  const [propertySections, setPropertySections] = useState<PropertySection[]>([]);

  const canvasRef = useRef<HTMLDivElement>(null);

  const { activeTool, setActiveTool } = useCADTool();
  const { layers } = useLayerManagement();
  const { snapSettings, toggleSnap } = useSnapPoint();

  // Update viewport size on mount and resize
  useEffect(() => {
    const updateViewportSize = () => {
      if (canvasRef.current) {
        setViewport((prev) => ({
          ...prev,
          width: canvasRef.current!.clientWidth,
          height: canvasRef.current!.clientHeight,
        }));
      }
    };

    updateViewportSize();
    window.addEventListener('resize', updateViewportSize);
    return () => window.removeEventListener('resize', updateViewportSize);
  }, []);

  // Update config when changed
  useEffect(() => {
    if (onConfigChange) {
      onConfigChange(config);
    }
  }, [config, onConfigChange]);

  const handleCanvasMouseMove = useCallback((e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (rect) {
      const x = (e.clientX - rect.left) / viewport.zoom + viewport.panX;
      const y = (e.clientY - rect.top) / viewport.zoom + viewport.panY;
      setCursorPosition({ x, y });
    }
  }, [viewport]);

  const handleCanvasMouseLeave = useCallback(() => {
    setCursorPosition(null);
  }, []);

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setContextMenuPosition({ x: e.clientX, y: e.clientY });
  }, []);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();

    if (e.ctrlKey || e.metaKey) {
      // Zoom
      const zoomDelta = e.deltaY > 0 ? 0.9 : 1.1;
      setViewport((prev) => ({
        ...prev,
        zoom: Math.max(0.1, Math.min(10, prev.zoom * zoomDelta)),
      }));
    } else {
      // Pan
      setViewport((prev) => ({
        ...prev,
        panX: prev.panX + e.deltaX / prev.zoom,
        panY: prev.panY + e.deltaY / prev.zoom,
      }));
    }
  }, []);

  const handlePropertyChange = useCallback((id: string, value: PropertyValue) => {
    console.log('Property changed:', id, value);
    // Update property in the state
    setPropertySections((prev) =>
      prev.map((section) => ({
        ...section,
        properties: section.properties.map((prop) =>
          prop.id === id ? { ...prop, value } : prop
        ),
      }))
    );
  }, []);

  const toggleGrid = useCallback(() => {
    setConfig((prev) => ({
      ...prev,
      gridSettings: {
        ...prev.gridSettings,
        enabled: !prev.gridSettings.enabled,
      },
    }));
  }, []);

  // Sample commands for command palette
  const commands: Command[] = [
    {
      id: 'cmd-new',
      name: 'New Drawing',
      description: 'Create a new drawing',
      icon: '📄',
      shortcut: 'Ctrl+N',
      category: 'File',
      tags: ['new', 'create', 'file'],
      action: () => console.log('New drawing'),
    },
    {
      id: 'cmd-save',
      name: 'Save',
      description: 'Save current drawing',
      icon: '💾',
      shortcut: 'Ctrl+S',
      category: 'File',
      tags: ['save', 'file'],
      action: () => console.log('Save'),
    },
    {
      id: 'cmd-undo',
      name: 'Undo',
      description: 'Undo last action',
      icon: '↶',
      shortcut: 'Ctrl+Z',
      category: 'Edit',
      tags: ['undo', 'edit'],
      action: () => console.log('Undo'),
    },
    {
      id: 'cmd-redo',
      name: 'Redo',
      description: 'Redo last undone action',
      icon: '↷',
      shortcut: 'Ctrl+Y',
      category: 'Edit',
      tags: ['redo', 'edit'],
      action: () => console.log('Redo'),
    },
  ];

  // Sample context menu items
  const contextMenuItems: ContextMenuItem[] = [
    {
      id: 'cut',
      label: 'Cut',
      icon: '✂',
      shortcut: 'Ctrl+X',
      action: () => console.log('Cut'),
    },
    {
      id: 'copy',
      label: 'Copy',
      icon: '📋',
      shortcut: 'Ctrl+C',
      action: () => console.log('Copy'),
    },
    {
      id: 'paste',
      label: 'Paste',
      icon: '📄',
      shortcut: 'Ctrl+V',
      action: () => console.log('Paste'),
    },
    { id: 'sep1', separator: true, label: '' },
    {
      id: 'delete',
      label: 'Delete',
      icon: '🗑',
      shortcut: 'Del',
      action: () => console.log('Delete'),
    },
    { id: 'sep2', separator: true, label: '' },
    {
      id: 'properties',
      label: 'Properties',
      icon: '⚙',
      action: () => console.log('Properties'),
    },
  ];

  return (
    <div
      className={`
        relative w-full h-screen
        bg-gray-50 dark:bg-gray-950
        overflow-hidden
        ${className}
      `}
    >
      {/* Toolbar */}
      {config.showToolbar && (
        <CADToolbar position="left" />
      )}

      {/* Layer Panel */}
      {config.showLayerPanel && (
        <CADLayerPanel position="right" />
      )}

      {/* Property Panel */}
      {config.showPropertyPanel && propertySections.length > 0 && (
        <CADPropertyPanel
          sections={propertySections}
          onPropertyChange={handlePropertyChange}
          position="right"
        />
      )}

      {/* Main Canvas Area */}
      <div
        ref={canvasRef}
        className="
          absolute inset-0
          overflow-hidden
        "
        style={{
          left: config.showToolbar ? '256px' : '0',
          right: config.showLayerPanel ? '256px' : '0',
          top: config.showRulers ? '30px' : '0',
          bottom: config.showStatusBar ? '32px' : '0',
        }}
        onMouseMove={handleCanvasMouseMove}
        onMouseLeave={handleCanvasMouseLeave}
        onContextMenu={handleContextMenu}
        onWheel={handleWheel}
      >
        {/* Grid */}
        {config.showGrid && config.gridSettings.enabled && (
          <CADGrid viewport={viewport} settings={config.gridSettings} />
        )}

        {/* Rulers */}
        {config.showRulers && config.rulerSettings.enabled && (
          <CADRulers viewport={viewport} settings={config.rulerSettings} />
        )}

        {/* Snap System */}
        <CADSnapSystem
          currentPoint={cursorPosition}
          scale={viewport.zoom}
        />

        {/* Measurement Tools */}
        <CADMeasurementTools
          activeTool={activeTool}
          scale={viewport.zoom}
        />

        {/* Canvas Content Area */}
        <div
          className="
            absolute inset-0
            bg-white dark:bg-gray-900
          "
        >
          {/* User canvas content goes here */}
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-gray-400 dark:text-gray-600">
              <p className="text-lg mb-2">CAD Canvas Area</p>
              <p className="text-sm">
                Tool: {activeTool} | Zoom: {(viewport.zoom * 100).toFixed(0)}%
              </p>
              {cursorPosition && (
                <p className="text-xs mt-2">
                  Cursor: ({cursorPosition.x.toFixed(0)}, {cursorPosition.y.toFixed(0)})
                </p>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Command Palette */}
      <CADCommandPalette commands={commands} />

      {/* Context Menu */}
      <CADContextMenu
        items={contextMenuItems}
        position={contextMenuPosition}
        onClose={() => setContextMenuPosition(null)}
      />

      {/* Status Bar */}
      {config.showStatusBar && (
        <CADStatusBar
          activeTool={activeTool}
          cursorPosition={cursorPosition}
          viewport={viewport}
          snapSettings={snapSettings}
          selectedObjectCount={selectedObjects.length}
          onToggleSnap={toggleSnap}
          onToggleGrid={toggleGrid}
        />
      )}
    </div>
  );
};
