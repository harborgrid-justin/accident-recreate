/**
 * Layers Panel Component
 * Layer hierarchy and management
 */

import React, { useState } from 'react';
import { LayersProps, Layer } from '../types';
import './Layers.css';

export const Layers: React.FC<LayersProps> = ({
  layers,
  selectedLayer,
  onLayerSelect,
  onLayerToggle,
  onLayerLock,
  onLayerAdd,
  onLayerRemove,
  onLayerRename,
}) => {
  const [editingLayer, setEditingLayer] = useState<string | null>(null);
  const [expandedLayers, setExpandedLayers] = useState<Set<string>>(new Set());

  const toggleExpanded = (layerId: string) => {
    const newExpanded = new Set(expandedLayers);
    if (newExpanded.has(layerId)) {
      newExpanded.delete(layerId);
    } else {
      newExpanded.add(layerId);
    }
    setExpandedLayers(newExpanded);
  };

  const renderLayer = (layer: Layer, depth: number = 0) => {
    const isSelected = selectedLayer === layer.id;
    const isExpanded = expandedLayers.has(layer.id);
    const hasChildren = layer.children && layer.children.length > 0;

    return (
      <div key={layer.id} className="layer-item-wrapper">
        <div
          className={`layer-item ${isSelected ? 'selected' : ''}`}
          style={{ paddingLeft: `${depth * 20 + 12}px` }}
        >
          {hasChildren && (
            <button
              className="layer-expand-btn"
              onClick={() => toggleExpanded(layer.id)}
            >
              {isExpanded ? '▼' : '▶'}
            </button>
          )}

          <div className="layer-type-icon" title={layer.type}>
            {getLayerIcon(layer.type)}
          </div>

          {editingLayer === layer.id ? (
            <input
              type="text"
              defaultValue={layer.name}
              autoFocus
              onBlur={(e) => {
                onLayerRename?.(layer.id, e.target.value);
                setEditingLayer(null);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  onLayerRename?.(layer.id, e.currentTarget.value);
                  setEditingLayer(null);
                } else if (e.key === 'Escape') {
                  setEditingLayer(null);
                }
              }}
              className="layer-name-input"
            />
          ) : (
            <div
              className="layer-name"
              onClick={() => onLayerSelect(layer.id)}
              onDoubleClick={() => setEditingLayer(layer.id)}
            >
              {layer.name}
            </div>
          )}

          <div className="layer-actions">
            <button
              className={`layer-visibility-btn ${!layer.visible ? 'hidden' : ''}`}
              onClick={() => onLayerToggle(layer.id, !layer.visible)}
              title={layer.visible ? 'Hide layer' : 'Show layer'}
            >
              {layer.visible ? '👁' : '👁‍🗨'}
            </button>

            <button
              className={`layer-lock-btn ${layer.locked ? 'locked' : ''}`}
              onClick={() => onLayerLock(layer.id, !layer.locked)}
              title={layer.locked ? 'Unlock layer' : 'Lock layer'}
            >
              {layer.locked ? '🔒' : '🔓'}
            </button>

            {onLayerRemove && (
              <button
                className="layer-remove-btn"
                onClick={() => onLayerRemove(layer.id)}
                title="Remove layer"
              >
                ×
              </button>
            )}
          </div>
        </div>

        {hasChildren && isExpanded && (
          <div className="layer-children">
            {layer.children!.map(child => renderLayer(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="layers-panel">
      <div className="panel-header">
        <h3>Layers</h3>
        {onLayerAdd && (
          <button className="add-layer-btn" onClick={() => onLayerAdd()}>
            +
          </button>
        )}
      </div>

      <div className="layers-list">
        {layers.length === 0 ? (
          <div className="layers-empty">No layers</div>
        ) : (
          layers.map(layer => renderLayer(layer))
        )}
      </div>
    </div>
  );
};

function getLayerIcon(type: string): string {
  switch (type) {
    case 'vehicle':
      return '🚗';
    case 'environment':
      return '🌍';
    case 'annotation':
      return '📝';
    case 'measurement':
      return '📏';
    default:
      return '📄';
  }
}

export default Layers;
