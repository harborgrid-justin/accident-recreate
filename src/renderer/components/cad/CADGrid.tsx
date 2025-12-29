/**
 * AccuScene Enterprise v0.3.0 - CAD Grid Component
 */

import React, { useMemo } from 'react';
import { ViewportState, GridSettings } from './types';

interface CADGridProps {
  viewport: ViewportState;
  settings?: GridSettings;
  className?: string;
}

const defaultSettings: GridSettings = {
  enabled: true,
  type: 'line',
  majorSpacing: 100,
  minorSpacing: 20,
  majorColor: '#cbd5e1',
  minorColor: '#e2e8f0',
  opacity: 0.5,
};

export const CADGrid: React.FC<CADGridProps> = ({
  viewport,
  settings = defaultSettings,
  className = '',
}) => {
  const {
    enabled,
    type,
    majorSpacing,
    minorSpacing,
    majorColor,
    minorColor,
    opacity,
  } = {
    ...defaultSettings,
    ...settings,
  };

  if (!enabled) return null;

  // Calculate visible grid range
  const gridRange = useMemo(() => {
    const padding = 100; // Extra padding to ensure grid covers viewport during pan
    return {
      minX: Math.floor((viewport.panX - padding) / minorSpacing) * minorSpacing,
      maxX: Math.ceil((viewport.panX + viewport.width / viewport.zoom + padding) / minorSpacing) * minorSpacing,
      minY: Math.floor((viewport.panY - padding) / minorSpacing) * minorSpacing,
      maxY: Math.ceil((viewport.panY + viewport.height / viewport.zoom + padding) / minorSpacing) * minorSpacing,
    };
  }, [viewport, minorSpacing]);

  // Generate grid lines
  const gridLines = useMemo(() => {
    const lines: Array<{
      x1: number;
      y1: number;
      x2: number;
      y2: number;
      isMajor: boolean;
    }> = [];

    // Vertical lines
    for (let x = gridRange.minX; x <= gridRange.maxX; x += minorSpacing) {
      const isMajor = x % majorSpacing === 0;
      const screenX = (x - viewport.panX) * viewport.zoom;

      lines.push({
        x1: screenX,
        y1: 0,
        x2: screenX,
        y2: viewport.height,
        isMajor,
      });
    }

    // Horizontal lines
    for (let y = gridRange.minY; y <= gridRange.maxY; y += minorSpacing) {
      const isMajor = y % majorSpacing === 0;
      const screenY = (y - viewport.panY) * viewport.zoom;

      lines.push({
        x1: 0,
        y1: screenY,
        x2: viewport.width,
        y2: screenY,
        isMajor,
      });
    }

    return lines;
  }, [viewport, gridRange, minorSpacing, majorSpacing]);

  // Generate grid dots
  const gridDots = useMemo(() => {
    const dots: Array<{ x: number; y: number; isMajor: boolean }> = [];

    for (let x = gridRange.minX; x <= gridRange.maxX; x += minorSpacing) {
      for (let y = gridRange.minY; y <= gridRange.maxY; y += minorSpacing) {
        const screenX = (x - viewport.panX) * viewport.zoom;
        const screenY = (y - viewport.panY) * viewport.zoom;

        const isMajor = x % majorSpacing === 0 && y % majorSpacing === 0;

        dots.push({
          x: screenX,
          y: screenY,
          isMajor,
        });
      }
    }

    return dots;
  }, [viewport, gridRange, minorSpacing, majorSpacing]);

  return (
    <svg
      className={`absolute inset-0 pointer-events-none ${className}`}
      style={{ width: '100%', height: '100%' }}
    >
      <defs>
        {/* Pattern for line grid */}
        {type === 'line' && (
          <pattern
            id="grid-pattern"
            width={minorSpacing * viewport.zoom}
            height={minorSpacing * viewport.zoom}
            patternUnits="userSpaceOnUse"
            x={(-(viewport.panX * viewport.zoom)) % (minorSpacing * viewport.zoom)}
            y={(-(viewport.panY * viewport.zoom)) % (minorSpacing * viewport.zoom)}
          >
            <line
              x1={0}
              y1={0}
              x2={0}
              y2={minorSpacing * viewport.zoom}
              stroke={minorColor}
              strokeWidth={1}
              opacity={opacity}
            />
            <line
              x1={0}
              y1={0}
              x2={minorSpacing * viewport.zoom}
              y2={0}
              stroke={minorColor}
              strokeWidth={1}
              opacity={opacity}
            />
          </pattern>
        )}
      </defs>

      {type === 'line' ? (
        <>
          {/* Minor grid lines via pattern */}
          <rect
            x={0}
            y={0}
            width="100%"
            height="100%"
            fill="url(#grid-pattern)"
          />

          {/* Major grid lines */}
          {gridLines
            .filter((line) => line.isMajor)
            .map((line, index) => (
              <line
                key={`major-${index}`}
                x1={line.x1}
                y1={line.y1}
                x2={line.x2}
                y2={line.y2}
                stroke={majorColor}
                strokeWidth={1.5}
                opacity={opacity}
              />
            ))}
        </>
      ) : (
        // Dot grid
        <>
          {gridDots.map((dot, index) => (
            <circle
              key={index}
              cx={dot.x}
              cy={dot.y}
              r={dot.isMajor ? 2 : 1}
              fill={dot.isMajor ? majorColor : minorColor}
              opacity={opacity}
            />
          ))}
        </>
      )}

      {/* Origin axes (optional) */}
      {viewport.panX <= 0 && viewport.panX + viewport.width / viewport.zoom >= 0 && (
        <line
          x1={-viewport.panX * viewport.zoom}
          y1={0}
          x2={-viewport.panX * viewport.zoom}
          y2={viewport.height}
          stroke="#3b82f6"
          strokeWidth={2}
          opacity={0.3}
        />
      )}
      {viewport.panY <= 0 && viewport.panY + viewport.height / viewport.zoom >= 0 && (
        <line
          x1={0}
          y1={-viewport.panY * viewport.zoom}
          x2={viewport.width}
          y2={-viewport.panY * viewport.zoom}
          stroke="#3b82f6"
          strokeWidth={2}
          opacity={0.3}
        />
      )}
    </svg>
  );
};
