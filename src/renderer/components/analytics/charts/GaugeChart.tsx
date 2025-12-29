/**
 * AccuScene Enterprise v0.3.0 - Gauge Chart Component
 * Speedometer-style gauge chart for single value display
 */

import React from 'react';
import { ChartConfig } from '../types';

interface GaugeChartProps {
  value: number;
  min?: number;
  max?: number;
  label?: string;
  unit?: string;
  thresholds?: {
    low: number;
    medium: number;
    high: number;
  };
  config?: Partial<ChartConfig>;
  width?: string | number;
  height?: string | number;
}

const GaugeChart: React.FC<GaugeChartProps> = ({
  value,
  min = 0,
  max = 100,
  label = '',
  unit = '',
  thresholds,
  config = {},
  width = '100%',
  height = '100%',
}) => {
  // Calculate angle for the needle (0-180 degrees)
  const percentage = Math.max(0, Math.min(1, (value - min) / (max - min)));
  const angle = percentage * 180 - 90; // -90 to 90 degrees

  // Get color based on thresholds
  const getColor = () => {
    if (!thresholds) return '#3b82f6';

    const percent = percentage * 100;
    if (percent >= thresholds.high) return '#ef4444'; // Red
    if (percent >= thresholds.medium) return '#f59e0b'; // Orange
    if (percent >= thresholds.low) return '#10b981'; // Green
    return '#64748b'; // Gray
  };

  const color = getColor();

  return (
    <div
      style={{
        width,
        height,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '1rem',
      }}
    >
      <svg
        viewBox="0 0 200 120"
        style={{
          width: '100%',
          height: 'auto',
          maxWidth: '300px',
        }}
      >
        {/* Background arc */}
        <path
          d="M 20 100 A 80 80 0 0 1 180 100"
          fill="none"
          stroke="#1e293b"
          strokeWidth="20"
          strokeLinecap="round"
        />

        {/* Colored segments */}
        {thresholds ? (
          <>
            {/* Low (green) */}
            <path
              d={`M 20 100 A 80 80 0 0 1 ${20 + 160 * (thresholds.low / 100)} ${
                100 - 80 * Math.sin((thresholds.low / 100) * Math.PI)
              }`}
              fill="none"
              stroke="#10b981"
              strokeWidth="20"
              strokeLinecap="round"
              opacity={0.3}
            />

            {/* Medium (orange) */}
            <path
              d={`M ${20 + 160 * (thresholds.low / 100)} ${
                100 - 80 * Math.sin((thresholds.low / 100) * Math.PI)
              } A 80 80 0 0 1 ${20 + 160 * (thresholds.medium / 100)} ${
                100 - 80 * Math.sin((thresholds.medium / 100) * Math.PI)
              }`}
              fill="none"
              stroke="#f59e0b"
              strokeWidth="20"
              strokeLinecap="round"
              opacity={0.3}
            />

            {/* High (red) */}
            <path
              d={`M ${20 + 160 * (thresholds.medium / 100)} ${
                100 - 80 * Math.sin((thresholds.medium / 100) * Math.PI)
              } A 80 80 0 0 1 180 100`}
              fill="none"
              stroke="#ef4444"
              strokeWidth="20"
              strokeLinecap="round"
              opacity={0.3}
            />
          </>
        ) : (
          /* Single color arc */
          <path
            d="M 20 100 A 80 80 0 0 1 180 100"
            fill="none"
            stroke={color}
            strokeWidth="20"
            strokeLinecap="round"
            opacity={0.3}
          />
        )}

        {/* Value arc */}
        <path
          d={`M 20 100 A 80 80 0 0 ${percentage > 0.5 ? 1 : 0} 1 ${
            100 + 80 * Math.cos(angle * (Math.PI / 180))
          } ${100 + 80 * Math.sin(angle * (Math.PI / 180))}`}
          fill="none"
          stroke={color}
          strokeWidth="20"
          strokeLinecap="round"
        />

        {/* Center dot */}
        <circle cx="100" cy="100" r="8" fill="#334155" />

        {/* Needle */}
        <line
          x1="100"
          y1="100"
          x2={100 + 70 * Math.cos(angle * (Math.PI / 180))}
          y2={100 + 70 * Math.sin(angle * (Math.PI / 180))}
          stroke={color}
          strokeWidth="3"
          strokeLinecap="round"
        />
        <circle cx="100" cy="100" r="5" fill={color} />

        {/* Min label */}
        <text
          x="20"
          y="115"
          fill="#94a3b8"
          fontSize="10"
          textAnchor="start"
        >
          {min}
        </text>

        {/* Max label */}
        <text
          x="180"
          y="115"
          fill="#94a3b8"
          fontSize="10"
          textAnchor="end"
        >
          {max}
        </text>
      </svg>

      {/* Value display */}
      <div
        style={{
          marginTop: '1rem',
          textAlign: 'center',
        }}
      >
        <div
          style={{
            fontSize: '2.5rem',
            fontWeight: 700,
            color,
            lineHeight: 1,
          }}
        >
          {value.toFixed(1)}
          {unit && (
            <span style={{ fontSize: '1rem', marginLeft: '0.25rem' }}>
              {unit}
            </span>
          )}
        </div>
        {label && (
          <div
            style={{
              fontSize: '0.875rem',
              color: '#94a3b8',
              marginTop: '0.5rem',
            }}
          >
            {label}
          </div>
        )}
      </div>

      {/* Threshold labels */}
      {thresholds && (
        <div
          style={{
            display: 'flex',
            gap: '1rem',
            marginTop: '1rem',
            fontSize: '0.75rem',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                backgroundColor: '#10b981',
                borderRadius: '2px',
              }}
            />
            <span style={{ color: '#94a3b8' }}>Low (&lt;{thresholds.low}%)</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                backgroundColor: '#f59e0b',
                borderRadius: '2px',
              }}
            />
            <span style={{ color: '#94a3b8' }}>
              Medium ({thresholds.low}-{thresholds.medium}%)
            </span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                backgroundColor: '#ef4444',
                borderRadius: '2px',
              }}
            />
            <span style={{ color: '#94a3b8' }}>High (&gt;{thresholds.medium}%)</span>
          </div>
        </div>
      )}
    </div>
  );
};

export default GaugeChart;
