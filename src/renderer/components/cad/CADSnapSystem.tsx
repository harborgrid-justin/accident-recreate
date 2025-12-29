/**
 * AccuScene Enterprise v0.3.0 - CAD Snap System Component
 */

import React, { useState, useCallback } from 'react';
import { SnapIndicator } from './SnapIndicator';
import { useSnapPoint } from './hooks/useSnapPoint';
import { Point2D, SnapType } from './types';

interface CADSnapSystemProps {
  currentPoint: Point2D | null;
  objects?: any[];
  scale?: number;
  onSnapPointChange?: (point: Point2D | null) => void;
  className?: string;
}

export const CADSnapSystem: React.FC<CADSnapSystemProps> = ({
  currentPoint,
  objects = [],
  scale = 1,
  onSnapPointChange,
  className = '',
}) => {
  const {
    snapSettings,
    updateSnapSettings,
    findSnapPoint,
    toggleSnap,
    toggleSnapType,
  } = useSnapPoint();

  const [showSettings, setShowSettings] = useState(false);

  const snapPoint = currentPoint ? findSnapPoint(currentPoint, objects) : null;

  // Notify parent of snap point changes
  React.useEffect(() => {
    if (snapPoint && onSnapPointChange) {
      onSnapPointChange(snapPoint.point);
    } else if (!snapPoint && onSnapPointChange) {
      onSnapPointChange(currentPoint);
    }
  }, [snapPoint, currentPoint, onSnapPointChange]);

  const snapTypes = [
    { type: SnapType.GRID, label: 'Grid', icon: '⊞' },
    { type: SnapType.ENDPOINT, label: 'Endpoint', icon: '□' },
    { type: SnapType.MIDPOINT, label: 'Midpoint', icon: '╋' },
    { type: SnapType.CENTER, label: 'Center', icon: '◎' },
    { type: SnapType.INTERSECTION, label: 'Intersection', icon: '✕' },
    { type: SnapType.PERPENDICULAR, label: 'Perpendicular', icon: '⊥' },
    { type: SnapType.TANGENT, label: 'Tangent', icon: '◠' },
    { type: SnapType.QUADRANT, label: 'Quadrant', icon: '◴' },
    { type: SnapType.NEAREST, label: 'Nearest', icon: '•' },
  ];

  return (
    <div className={`relative ${className}`}>
      {/* Snap indicator on canvas */}
      <svg
        className="absolute inset-0 pointer-events-none"
        style={{ width: '100%', height: '100%' }}
      >
        <SnapIndicator snapPoint={snapPoint} scale={scale} />

        {/* Magnetic guide lines */}
        {snapPoint && snapSettings.enabled && (
          <g opacity={0.3}>
            {/* Horizontal guide */}
            <line
              x1={0}
              y1={snapPoint.point.y}
              x2={10000}
              y2={snapPoint.point.y}
              stroke="#3b82f6"
              strokeWidth={1 / scale}
              strokeDasharray="5,5"
            />
            {/* Vertical guide */}
            <line
              x1={snapPoint.point.x}
              y1={0}
              x2={snapPoint.point.x}
              y2={10000}
              stroke="#3b82f6"
              strokeWidth={1 / scale}
              strokeDasharray="5,5"
            />
          </g>
        )}
      </svg>

      {/* Snap Settings Panel */}
      <div className="absolute top-4 left-4 pointer-events-auto">
        {/* Toggle Button */}
        <button
          onClick={() => setShowSettings(!showSettings)}
          className={`
            px-3 py-2 rounded-lg shadow-lg
            transition-all duration-200
            ${
              snapSettings.enabled
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
            }
            hover:shadow-xl
          `}
          title="Snap settings"
        >
          <span className="flex items-center gap-2">
            <span>🧲</span>
            <span className="text-sm font-medium">
              Snap {snapSettings.enabled ? 'ON' : 'OFF'}
            </span>
          </span>
        </button>

        {/* Settings Panel */}
        {showSettings && (
          <div
            className="
              mt-2 p-3
              bg-white dark:bg-gray-900
              border border-gray-200 dark:border-gray-700
              rounded-lg shadow-xl
              min-w-[250px]
            "
          >
            {/* Enable/Disable Snap */}
            <div className="mb-3 pb-3 border-b border-gray-200 dark:border-gray-700">
              <label className="flex items-center justify-between cursor-pointer">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  Enable Snap
                </span>
                <button
                  onClick={toggleSnap}
                  className={`
                    relative w-12 h-6 rounded-full transition-colors
                    ${snapSettings.enabled ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'}
                  `}
                >
                  <div
                    className={`
                      absolute w-5 h-5 bg-white rounded-full top-0.5
                      transition-transform duration-200
                      ${snapSettings.enabled ? 'left-6' : 'left-0.5'}
                    `}
                  />
                </button>
              </label>
            </div>

            {/* Grid Snap */}
            <div className="mb-3">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={snapSettings.gridSnap}
                  onChange={(e) =>
                    updateSnapSettings({ gridSnap: e.target.checked })
                  }
                  disabled={!snapSettings.enabled}
                  className="w-4 h-4 rounded"
                />
                <span className="text-sm text-gray-700 dark:text-gray-300">
                  Grid Snap
                </span>
              </label>
              {snapSettings.gridSnap && (
                <div className="mt-2 ml-6">
                  <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                    Grid Size: {snapSettings.gridSize}px
                  </label>
                  <input
                    type="range"
                    min="5"
                    max="100"
                    step="5"
                    value={snapSettings.gridSize}
                    onChange={(e) =>
                      updateSnapSettings({ gridSize: parseInt(e.target.value) })
                    }
                    className="w-full"
                  />
                </div>
              )}
            </div>

            {/* Object Snap */}
            <div className="mb-3">
              <label className="flex items-center gap-2 cursor-pointer mb-2">
                <input
                  type="checkbox"
                  checked={snapSettings.objectSnap}
                  onChange={(e) =>
                    updateSnapSettings({ objectSnap: e.target.checked })
                  }
                  disabled={!snapSettings.enabled}
                  className="w-4 h-4 rounded"
                />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  Object Snap
                </span>
              </label>

              {/* Snap Types */}
              {snapSettings.objectSnap && (
                <div className="ml-6 space-y-1.5">
                  {snapTypes.map(({ type, label, icon }) => (
                    <label
                      key={type}
                      className="flex items-center gap-2 cursor-pointer"
                    >
                      <input
                        type="checkbox"
                        checked={snapSettings.snapTypes.includes(type)}
                        onChange={() => toggleSnapType(type)}
                        disabled={!snapSettings.enabled}
                        className="w-3.5 h-3.5 rounded"
                      />
                      <span className="text-xs text-gray-600 dark:text-gray-400">
                        {icon} {label}
                      </span>
                    </label>
                  ))}
                </div>
              )}
            </div>

            {/* Snap Distance */}
            <div>
              <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                Snap Distance: {snapSettings.snapDistance}px
              </label>
              <input
                type="range"
                min="5"
                max="50"
                step="5"
                value={snapSettings.snapDistance}
                onChange={(e) =>
                  updateSnapSettings({ snapDistance: parseInt(e.target.value) })
                }
                disabled={!snapSettings.enabled}
                className="w-full"
              />
            </div>
          </div>
        )}
      </div>

      {/* Current Snap Status */}
      {snapPoint && snapSettings.enabled && (
        <div
          className="
            absolute bottom-4 left-4
            px-3 py-2
            bg-black bg-opacity-75 text-white
            rounded text-xs
            pointer-events-none
          "
        >
          Snap: {snapPoint.type} ({snapPoint.point.x.toFixed(0)},{' '}
          {snapPoint.point.y.toFixed(0)})
        </div>
      )}
    </div>
  );
};
