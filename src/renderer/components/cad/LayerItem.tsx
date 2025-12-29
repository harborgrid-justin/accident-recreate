/**
 * AccuScene Enterprise v0.3.0 - Layer Item Component
 */

import React, { useState, DragEvent } from 'react';
import { Layer } from './types';

interface LayerItemProps {
  layer: Layer;
  isActive: boolean;
  onSelect: (id: string) => void;
  onToggleVisibility: (id: string) => void;
  onToggleLock: (id: string) => void;
  onColorChange: (id: string, color: string) => void;
  onRename: (id: string, name: string) => void;
  onDelete: (id: string) => void;
  onDragStart: (id: string) => void;
  onDragOver: (id: string) => void;
  onDrop: (targetId: string) => void;
  className?: string;
}

export const LayerItem: React.FC<LayerItemProps> = ({
  layer,
  isActive,
  onSelect,
  onToggleVisibility,
  onToggleLock,
  onColorChange,
  onRename,
  onDelete,
  onDragStart,
  onDragOver,
  onDrop,
  className = '',
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const [editName, setEditName] = useState(layer.name);
  const [isDragging, setIsDragging] = useState(false);

  const handleDragStart = (e: DragEvent) => {
    setIsDragging(true);
    onDragStart(layer.id);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragEnd = () => {
    setIsDragging(false);
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    onDragOver(layer.id);
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    onDrop(layer.id);
  };

  const handleDoubleClick = () => {
    if (!layer.locked) {
      setIsEditing(true);
      setEditName(layer.name);
    }
  };

  const handleBlur = () => {
    setIsEditing(false);
    if (editName.trim() && editName !== layer.name) {
      onRename(layer.id, editName.trim());
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleBlur();
    } else if (e.key === 'Escape') {
      setIsEditing(false);
      setEditName(layer.name);
    }
  };

  return (
    <div
      draggable
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
      onClick={() => onSelect(layer.id)}
      onDoubleClick={handleDoubleClick}
      className={`
        flex items-center gap-2 px-2 py-1.5
        rounded cursor-pointer
        transition-all duration-150
        ${
          isActive
            ? 'bg-blue-100 dark:bg-blue-900 border border-blue-300 dark:border-blue-700'
            : 'bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-750'
        }
        ${isDragging ? 'opacity-50' : 'opacity-100'}
        ${layer.locked ? 'cursor-not-allowed' : ''}
        ${className}
      `}
    >
      {/* Drag Handle */}
      <div
        className="
          text-gray-400 dark:text-gray-500 cursor-grab active:cursor-grabbing
          text-xs
        "
      >
        ⋮⋮
      </div>

      {/* Visibility Toggle */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          onToggleVisibility(layer.id);
        }}
        className="
          w-5 h-5 flex items-center justify-center
          text-gray-600 dark:text-gray-400
          hover:bg-gray-200 dark:hover:bg-gray-700
          rounded transition-colors
        "
        aria-label={layer.visible ? 'Hide layer' : 'Show layer'}
        title={layer.visible ? 'Hide layer' : 'Show layer'}
      >
        {layer.visible ? '👁' : '🚫'}
      </button>

      {/* Lock Toggle */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          onToggleLock(layer.id);
        }}
        className="
          w-5 h-5 flex items-center justify-center
          text-gray-600 dark:text-gray-400
          hover:bg-gray-200 dark:hover:bg-gray-700
          rounded transition-colors
        "
        aria-label={layer.locked ? 'Unlock layer' : 'Lock layer'}
        title={layer.locked ? 'Unlock layer' : 'Lock layer'}
      >
        {layer.locked ? '🔒' : '🔓'}
      </button>

      {/* Color Indicator */}
      <input
        type="color"
        value={layer.color}
        onChange={(e) => {
          e.stopPropagation();
          onColorChange(layer.id, e.target.value);
        }}
        onClick={(e) => e.stopPropagation()}
        disabled={layer.locked}
        className="
          w-6 h-6 rounded border border-gray-300 dark:border-gray-600
          cursor-pointer disabled:cursor-not-allowed
        "
        aria-label="Layer color"
        title="Layer color"
      />

      {/* Layer Name */}
      {isEditing ? (
        <input
          type="text"
          value={editName}
          onChange={(e) => setEditName(e.target.value)}
          onBlur={handleBlur}
          onKeyDown={handleKeyDown}
          onClick={(e) => e.stopPropagation()}
          autoFocus
          className="
            flex-1 px-1 py-0.5 text-sm
            bg-white dark:bg-gray-700
            border border-blue-500
            rounded focus:outline-none
          "
        />
      ) : (
        <span
          className={`
            flex-1 text-sm truncate
            ${layer.locked ? 'text-gray-400 dark:text-gray-500' : 'text-gray-700 dark:text-gray-300'}
          `}
        >
          {layer.name}
        </span>
      )}

      {/* Object Count */}
      <span
        className="
          text-xs text-gray-400 dark:text-gray-500
          min-w-[2rem] text-right
        "
      >
        {layer.objectCount}
      </span>

      {/* Delete Button */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          if (confirm(`Delete layer "${layer.name}"?`)) {
            onDelete(layer.id);
          }
        }}
        className="
          w-5 h-5 flex items-center justify-center
          text-red-500 hover:bg-red-100 dark:hover:bg-red-900
          rounded transition-colors
          opacity-0 group-hover:opacity-100
        "
        aria-label="Delete layer"
        title="Delete layer"
      >
        ×
      </button>
    </div>
  );
};
