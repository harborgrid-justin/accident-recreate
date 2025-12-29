/**
 * AccuScene Enterprise v0.3.0 - CAD Layer Panel Component
 */

import React, { useState } from 'react';
import { LayerItem } from './LayerItem';
import { useLayerManagement } from './hooks/useLayerManagement';

interface CADLayerPanelProps {
  position?: 'left' | 'right' | 'bottom';
  collapsible?: boolean;
  className?: string;
}

export const CADLayerPanel: React.FC<CADLayerPanelProps> = ({
  position = 'right',
  collapsible = true,
  className = '',
}) => {
  const {
    layers,
    activeLayer,
    addLayer,
    removeLayer,
    updateLayer,
    setActiveLayer,
    toggleLayerVisibility,
    toggleLayerLock,
    reorderLayers,
    duplicateLayer,
  } = useLayerManagement();

  const [collapsed, setCollapsed] = useState(false);
  const [draggedLayerId, setDraggedLayerId] = useState<string | null>(null);
  const [newLayerName, setNewLayerName] = useState('');

  const handleAddLayer = () => {
    const name = newLayerName.trim() || `Layer ${layers.length}`;
    addLayer(name);
    setNewLayerName('');
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleAddLayer();
    }
  };

  const positionClasses = {
    left: 'left-0 top-0 h-full w-64 border-r flex-col',
    right: 'right-0 top-0 h-full w-64 border-l flex-col',
    bottom: 'bottom-0 left-0 w-full h-64 border-t flex-row',
  };

  return (
    <div
      className={`
        fixed ${positionClasses[position]}
        bg-white dark:bg-gray-900
        border-gray-200 dark:border-gray-700
        shadow-lg
        ${collapsed ? (position === 'bottom' ? 'h-12' : 'w-12') : ''}
        transition-all duration-300
        z-30
        flex
        ${className}
      `}
    >
      {/* Header */}
      <div
        className="
          flex items-center justify-between
          p-3 border-b border-gray-200 dark:border-gray-700
        "
      >
        {!collapsed && (
          <h2 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
            Layers
          </h2>
        )}
        {collapsible && (
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="
              p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-800
              transition-colors
            "
            aria-label={collapsed ? 'Expand panel' : 'Collapse panel'}
          >
            <span className="text-gray-600 dark:text-gray-400">
              {collapsed ? '▶' : '◀'}
            </span>
          </button>
        )}
      </div>

      {/* Content */}
      {!collapsed && (
        <>
          {/* Add Layer Input */}
          <div className="p-3 border-b border-gray-200 dark:border-gray-700">
            <div className="flex gap-2">
              <input
                type="text"
                value={newLayerName}
                onChange={(e) => setNewLayerName(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="New layer name..."
                className="
                  flex-1 px-2 py-1.5 text-sm
                  bg-white dark:bg-gray-800
                  border border-gray-300 dark:border-gray-600
                  rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent
                "
              />
              <button
                onClick={handleAddLayer}
                className="
                  px-3 py-1.5 text-sm font-medium
                  bg-blue-600 hover:bg-blue-700
                  text-white rounded
                  transition-colors
                "
                title="Add new layer"
              >
                +
              </button>
            </div>
          </div>

          {/* Layer List */}
          <div className="flex-1 overflow-y-auto p-2 space-y-1">
            {layers.length === 0 ? (
              <div className="text-center py-8">
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  No layers yet
                </p>
                <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
                  Add a layer to get started
                </p>
              </div>
            ) : (
              layers
                .sort((a, b) => a.order - b.order)
                .map((layer) => (
                  <LayerItem
                    key={layer.id}
                    layer={layer}
                    isActive={layer.id === activeLayer}
                    onSelect={setActiveLayer}
                    onToggleVisibility={toggleLayerVisibility}
                    onToggleLock={toggleLayerLock}
                    onColorChange={(id, color) =>
                      updateLayer(id, { color })
                    }
                    onRename={(id, name) => updateLayer(id, { name })}
                    onDelete={removeLayer}
                    onDragStart={setDraggedLayerId}
                    onDragOver={() => {}}
                    onDrop={(targetId) => {
                      if (draggedLayerId && draggedLayerId !== targetId) {
                        reorderLayers(draggedLayerId, targetId);
                      }
                      setDraggedLayerId(null);
                    }}
                    className="group"
                  />
                ))
            )}
          </div>

          {/* Footer Actions */}
          {layers.length > 0 && (
            <div
              className="
                p-2 border-t border-gray-200 dark:border-gray-700
                flex gap-2
              "
            >
              <button
                onClick={() => {
                  if (activeLayer) {
                    duplicateLayer(activeLayer);
                  }
                }}
                disabled={!activeLayer}
                className="
                  flex-1 px-2 py-1.5 text-xs
                  bg-gray-100 dark:bg-gray-800
                  hover:bg-gray-200 dark:hover:bg-gray-700
                  disabled:opacity-50 disabled:cursor-not-allowed
                  text-gray-700 dark:text-gray-300
                  rounded transition-colors
                "
                title="Duplicate selected layer"
              >
                Duplicate
              </button>
              <button
                onClick={() => {
                  if (activeLayer) {
                    removeLayer(activeLayer);
                  }
                }}
                disabled={!activeLayer || layers.length === 1}
                className="
                  flex-1 px-2 py-1.5 text-xs
                  bg-red-100 dark:bg-red-900
                  hover:bg-red-200 dark:hover:bg-red-800
                  disabled:opacity-50 disabled:cursor-not-allowed
                  text-red-700 dark:text-red-300
                  rounded transition-colors
                "
                title="Delete selected layer"
              >
                Delete
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
};
