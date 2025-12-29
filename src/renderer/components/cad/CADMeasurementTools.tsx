/**
 * AccuScene Enterprise v0.3.0 - CAD Measurement Tools Component
 */

import React from 'react';
import { MeasurementOverlay } from './MeasurementOverlay';
import { useMeasurement } from './hooks/useMeasurement';
import { CADToolType } from './types';

interface CADMeasurementToolsProps {
  activeTool: CADToolType;
  scale?: number;
  viewBox?: { x: number; y: number; width: number; height: number };
  className?: string;
}

export const CADMeasurementTools: React.FC<CADMeasurementToolsProps> = ({
  activeTool,
  scale = 1,
  viewBox = { x: 0, y: 0, width: 1000, height: 1000 },
  className = '',
}) => {
  const {
    measurements,
    removeMeasurement,
    clearMeasurements,
    formatValue,
  } = useMeasurement();

  const isMeasurementTool =
    activeTool === CADToolType.MEASURE_DISTANCE ||
    activeTool === CADToolType.MEASURE_ANGLE ||
    activeTool === CADToolType.MEASURE_AREA;

  return (
    <div className={`relative ${className}`}>
      {/* SVG Overlay for measurements */}
      <svg
        className="absolute inset-0 pointer-events-none"
        viewBox={`${viewBox.x} ${viewBox.y} ${viewBox.width} ${viewBox.height}`}
        style={{
          width: '100%',
          height: '100%',
        }}
      >
        {measurements.map((measurement) => (
          <MeasurementOverlay
            key={measurement.id}
            measurement={measurement}
            scale={scale}
            onRemove={removeMeasurement}
          />
        ))}
      </svg>

      {/* Measurement Panel */}
      {measurements.length > 0 && (
        <div
          className="
            absolute bottom-4 left-4
            bg-white dark:bg-gray-900
            border border-gray-200 dark:border-gray-700
            rounded-lg shadow-lg
            p-3 max-w-xs
            pointer-events-auto
          "
        >
          {/* Header */}
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
              Measurements
            </h3>
            <button
              onClick={clearMeasurements}
              className="
                px-2 py-1 text-xs
                bg-red-100 dark:bg-red-900
                text-red-700 dark:text-red-300
                hover:bg-red-200 dark:hover:bg-red-800
                rounded transition-colors
              "
              title="Clear all measurements"
            >
              Clear All
            </button>
          </div>

          {/* Measurement List */}
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {measurements.map((measurement, index) => (
              <div
                key={measurement.id}
                className="
                  flex items-center justify-between gap-2
                  p-2 bg-gray-50 dark:bg-gray-800
                  rounded
                "
              >
                <div className="flex-1 min-w-0">
                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    {measurement.type.charAt(0).toUpperCase() +
                      measurement.type.slice(1)}{' '}
                    #{index + 1}
                  </div>
                  <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                    {measurement.type === 'angle'
                      ? `${measurement.value.toFixed(1)}°`
                      : formatValue(
                          measurement.value,
                          measurement.type === 'area'
                            ? `${measurement.unit}²`
                            : measurement.unit
                        )}
                  </div>
                </div>
                <button
                  onClick={() => removeMeasurement(measurement.id)}
                  className="
                    w-6 h-6 flex items-center justify-center
                    text-red-500 hover:bg-red-100 dark:hover:bg-red-900
                    rounded transition-colors
                  "
                  title="Remove measurement"
                >
                  ×
                </button>
              </div>
            ))}
          </div>

          {/* Instructions */}
          {isMeasurementTool && (
            <div
              className="
                mt-2 pt-2 border-t border-gray-200 dark:border-gray-700
                text-xs text-gray-500 dark:text-gray-400
              "
            >
              {activeTool === CADToolType.MEASURE_DISTANCE &&
                'Click two points to measure distance'}
              {activeTool === CADToolType.MEASURE_ANGLE &&
                'Click three points to measure angle'}
              {activeTool === CADToolType.MEASURE_AREA &&
                'Click points to define area (double-click to finish)'}
            </div>
          )}
        </div>
      )}

      {/* Tool Instructions (when no measurements) */}
      {measurements.length === 0 && isMeasurementTool && (
        <div
          className="
            absolute bottom-4 left-4
            bg-blue-50 dark:bg-blue-900
            border border-blue-200 dark:border-blue-700
            rounded-lg shadow-lg
            p-3 max-w-xs
            pointer-events-none
          "
        >
          <div className="flex items-start gap-2">
            <span className="text-lg">📏</span>
            <div>
              <h3 className="text-sm font-semibold text-blue-900 dark:text-blue-100 mb-1">
                {activeTool === CADToolType.MEASURE_DISTANCE && 'Measure Distance'}
                {activeTool === CADToolType.MEASURE_ANGLE && 'Measure Angle'}
                {activeTool === CADToolType.MEASURE_AREA && 'Measure Area'}
              </h3>
              <p className="text-xs text-blue-700 dark:text-blue-300">
                {activeTool === CADToolType.MEASURE_DISTANCE &&
                  'Click two points to measure the distance between them'}
                {activeTool === CADToolType.MEASURE_ANGLE &&
                  'Click three points: first line start, vertex, second line end'}
                {activeTool === CADToolType.MEASURE_AREA &&
                  'Click to define polygon vertices, double-click to finish'}
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
