/**
 * AccuScene Enterprise v0.3.0 - CAD Status Bar Component
 */

import React from 'react';
import { CADToolType, Point2D, ViewportState, SnapSettings } from './types';

interface CADStatusBarProps {
  activeTool: CADToolType;
  cursorPosition: Point2D | null;
  viewport: ViewportState;
  snapSettings: SnapSettings;
  selectedObjectCount?: number;
  onToggleSnap?: () => void;
  onToggleGrid?: () => void;
  className?: string;
}

export const CADStatusBar: React.FC<CADStatusBarProps> = ({
  activeTool,
  cursorPosition,
  viewport,
  snapSettings,
  selectedObjectCount = 0,
  onToggleSnap,
  onToggleGrid,
  className = '',
}) => {
  const formatCoordinate = (value: number): string => {
    return value.toFixed(2);
  };

  const formatZoom = (zoom: number): string => {
    return `${(zoom * 100).toFixed(0)}%`;
  };

  return (
    <div
      className={`
        fixed bottom-0 left-0 right-0
        h-8
        bg-gray-100 dark:bg-gray-800
        border-t border-gray-200 dark:border-gray-700
        flex items-center justify-between
        px-4 gap-4
        text-xs
        z-30
        ${className}
      `}
    >
      {/* Left Section - Tool & Selection Info */}
      <div className="flex items-center gap-4">
        {/* Active Tool */}
        <div className="flex items-center gap-2">
          <span className="text-gray-500 dark:text-gray-400">Tool:</span>
          <span className="font-medium text-gray-700 dark:text-gray-300">
            {activeTool.charAt(0).toUpperCase() + activeTool.slice(1).replace('_', ' ')}
          </span>
        </div>

        {/* Divider */}
        <div className="w-px h-4 bg-gray-300 dark:bg-gray-600" />

        {/* Selection Count */}
        <div className="flex items-center gap-2">
          <span className="text-gray-500 dark:text-gray-400">Selected:</span>
          <span className="font-medium text-gray-700 dark:text-gray-300">
            {selectedObjectCount}
          </span>
        </div>
      </div>

      {/* Center Section - Cursor Coordinates */}
      <div className="flex items-center gap-4">
        {cursorPosition ? (
          <>
            <div className="flex items-center gap-2">
              <span className="text-gray-500 dark:text-gray-400">X:</span>
              <span className="font-mono font-medium text-gray-700 dark:text-gray-300 min-w-[60px]">
                {formatCoordinate(cursorPosition.x)}
              </span>
            </div>

            <div className="flex items-center gap-2">
              <span className="text-gray-500 dark:text-gray-400">Y:</span>
              <span className="font-mono font-medium text-gray-700 dark:text-gray-300 min-w-[60px]">
                {formatCoordinate(cursorPosition.y)}
              </span>
            </div>
          </>
        ) : (
          <span className="text-gray-400 dark:text-gray-500">
            Move cursor over canvas
          </span>
        )}
      </div>

      {/* Right Section - View & Settings */}
      <div className="flex items-center gap-4">
        {/* Zoom Level */}
        <div className="flex items-center gap-2">
          <span className="text-gray-500 dark:text-gray-400">Zoom:</span>
          <span className="font-mono font-medium text-gray-700 dark:text-gray-300 min-w-[50px]">
            {formatZoom(viewport.zoom)}
          </span>
        </div>

        {/* Divider */}
        <div className="w-px h-4 bg-gray-300 dark:bg-gray-600" />

        {/* Grid Toggle */}
        <button
          onClick={onToggleGrid}
          className={`
            px-2 py-1 rounded
            transition-colors
            ${
              viewport.rotation === 0
                ? 'bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300'
                : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400'
            }
            hover:bg-blue-200 dark:hover:bg-blue-800
          `}
          title="Toggle grid"
        >
          <span className="flex items-center gap-1">
            <span>⊞</span>
            <span>Grid</span>
          </span>
        </button>

        {/* Snap Toggle */}
        <button
          onClick={onToggleSnap}
          className={`
            px-2 py-1 rounded
            transition-colors
            ${
              snapSettings.enabled
                ? 'bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300'
                : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400'
            }
            hover:bg-green-200 dark:hover:bg-green-800
          `}
          title={`Snap ${snapSettings.enabled ? 'ON' : 'OFF'}`}
        >
          <span className="flex items-center gap-1">
            <span>🧲</span>
            <span>Snap</span>
          </span>
        </button>

        {/* Divider */}
        <div className="w-px h-4 bg-gray-300 dark:bg-gray-600" />

        {/* Pan Position (for reference) */}
        <div className="flex items-center gap-2 text-gray-400 dark:text-gray-500">
          <span>Pan:</span>
          <span className="font-mono text-xs">
            ({formatCoordinate(viewport.panX)}, {formatCoordinate(viewport.panY)})
          </span>
        </div>

        {/* Rotation (if not 0) */}
        {viewport.rotation !== 0 && (
          <>
            <div className="w-px h-4 bg-gray-300 dark:bg-gray-600" />
            <div className="flex items-center gap-2">
              <span className="text-gray-500 dark:text-gray-400">Rotation:</span>
              <span className="font-mono font-medium text-gray-700 dark:text-gray-300">
                {viewport.rotation.toFixed(1)}°
              </span>
            </div>
          </>
        )}
      </div>
    </div>
  );
};
