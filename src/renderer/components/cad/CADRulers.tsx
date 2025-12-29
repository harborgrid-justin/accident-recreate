/**
 * AccuScene Enterprise v0.3.0 - CAD Rulers Component
 */

import React, { useMemo } from 'react';
import { ViewportState, RulerSettings } from './types';

interface CADRulersProps {
  viewport: ViewportState;
  settings?: RulerSettings;
  className?: string;
}

const defaultSettings: RulerSettings = {
  enabled: true,
  size: 30,
  backgroundColor: '#f3f4f6',
  textColor: '#374151',
  lineColor: '#9ca3af',
  unit: 'px',
};

export const CADRulers: React.FC<CADRulersProps> = ({
  viewport,
  settings = defaultSettings,
  className = '',
}) => {
  const { enabled, size, backgroundColor, textColor, lineColor, unit } = {
    ...defaultSettings,
    ...settings,
  };

  if (!enabled) return null;

  // Calculate tick spacing based on zoom level
  const getTickSpacing = (zoom: number): { major: number; minor: number } => {
    const baseSpacing = 100;
    const spacing = baseSpacing / zoom;

    if (spacing < 20) {
      return { major: 100, minor: 20 };
    } else if (spacing < 50) {
      return { major: 100, minor: 25 };
    } else if (spacing < 100) {
      return { major: 100, minor: 50 };
    } else {
      return { major: 200, minor: 100 };
    }
  };

  const tickSpacing = getTickSpacing(viewport.zoom);

  // Generate horizontal ruler ticks
  const horizontalTicks = useMemo(() => {
    const ticks: Array<{ position: number; label?: string; isMajor: boolean }> = [];
    const startX = Math.floor((viewport.panX - 100) / tickSpacing.minor) * tickSpacing.minor;
    const endX = viewport.panX + viewport.width / viewport.zoom + 100;

    for (let x = startX; x <= endX; x += tickSpacing.minor) {
      const isMajor = x % tickSpacing.major === 0;
      const screenX = (x - viewport.panX) * viewport.zoom;

      if (screenX >= -size && screenX <= viewport.width) {
        ticks.push({
          position: screenX,
          label: isMajor ? `${x}${unit}` : undefined,
          isMajor,
        });
      }
    }

    return ticks;
  }, [viewport, tickSpacing, unit, size]);

  // Generate vertical ruler ticks
  const verticalTicks = useMemo(() => {
    const ticks: Array<{ position: number; label?: string; isMajor: boolean }> = [];
    const startY = Math.floor((viewport.panY - 100) / tickSpacing.minor) * tickSpacing.minor;
    const endY = viewport.panY + viewport.height / viewport.zoom + 100;

    for (let y = startY; y <= endY; y += tickSpacing.minor) {
      const isMajor = y % tickSpacing.major === 0;
      const screenY = (y - viewport.panY) * viewport.zoom;

      if (screenY >= -size && screenY <= viewport.height) {
        ticks.push({
          position: screenY,
          label: isMajor ? `${y}${unit}` : undefined,
          isMajor,
        });
      }
    }

    return ticks;
  }, [viewport, tickSpacing, unit, size]);

  return (
    <div className={className}>
      {/* Horizontal Ruler */}
      <div
        className="absolute top-0 left-0 right-0"
        style={{
          height: `${size}px`,
          backgroundColor,
          borderBottom: `1px solid ${lineColor}`,
          marginLeft: `${size}px`,
        }}
      >
        <svg width="100%" height={size}>
          {horizontalTicks.map((tick, index) => (
            <g key={index}>
              {/* Tick mark */}
              <line
                x1={tick.position}
                y1={size}
                x2={tick.position}
                y2={tick.isMajor ? size * 0.4 : size * 0.7}
                stroke={lineColor}
                strokeWidth={tick.isMajor ? 1.5 : 1}
              />
              {/* Label */}
              {tick.label && (
                <text
                  x={tick.position}
                  y={size * 0.3}
                  fontSize="10px"
                  fill={textColor}
                  textAnchor="middle"
                  dominantBaseline="middle"
                  fontFamily="monospace"
                >
                  {tick.label}
                </text>
              )}
            </g>
          ))}
        </svg>
      </div>

      {/* Vertical Ruler */}
      <div
        className="absolute top-0 left-0 bottom-0"
        style={{
          width: `${size}px`,
          backgroundColor,
          borderRight: `1px solid ${lineColor}`,
          marginTop: `${size}px`,
        }}
      >
        <svg width={size} height="100%">
          {verticalTicks.map((tick, index) => (
            <g key={index}>
              {/* Tick mark */}
              <line
                x1={size}
                y1={tick.position}
                x2={tick.isMajor ? size * 0.4 : size * 0.7}
                y2={tick.position}
                stroke={lineColor}
                strokeWidth={tick.isMajor ? 1.5 : 1}
              />
              {/* Label */}
              {tick.label && (
                <text
                  x={size * 0.3}
                  y={tick.position}
                  fontSize="10px"
                  fill={textColor}
                  textAnchor="middle"
                  dominantBaseline="middle"
                  transform={`rotate(-90 ${size * 0.3} ${tick.position})`}
                  fontFamily="monospace"
                >
                  {tick.label}
                </text>
              )}
            </g>
          ))}
        </svg>
      </div>

      {/* Corner box */}
      <div
        className="absolute top-0 left-0"
        style={{
          width: `${size}px`,
          height: `${size}px`,
          backgroundColor,
          borderRight: `1px solid ${lineColor}`,
          borderBottom: `1px solid ${lineColor}`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <span style={{ fontSize: '12px', color: textColor }}>📐</span>
      </div>
    </div>
  );
};
