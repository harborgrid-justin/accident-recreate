/**
 * AccuScene Enterprise v0.3.0 - Measurement Overlay Component
 */

import React from 'react';
import { Measurement, Point2D } from './types';

interface MeasurementOverlayProps {
  measurement: Measurement;
  scale?: number;
  onRemove?: (id: string) => void;
  className?: string;
}

export const MeasurementOverlay: React.FC<MeasurementOverlayProps> = ({
  measurement,
  scale = 1,
  onRemove,
  className = '',
}) => {
  const color = measurement.color || '#ef4444';

  const renderDistanceMeasurement = () => {
    if (measurement.points.length < 2) return null;

    const [p1, p2] = measurement.points;
    const midpoint = {
      x: (p1.x + p2.x) / 2,
      y: (p1.y + p2.y) / 2,
    };

    return (
      <g>
        {/* Line */}
        <line
          x1={p1.x}
          y1={p1.y}
          x2={p2.x}
          y2={p2.y}
          stroke={color}
          strokeWidth={2 / scale}
          strokeDasharray="5,5"
        />

        {/* Endpoints */}
        <circle cx={p1.x} cy={p1.y} r={4 / scale} fill={color} />
        <circle cx={p2.x} cy={p2.y} r={4 / scale} fill={color} />

        {/* Label */}
        <g transform={`translate(${midpoint.x}, ${midpoint.y})`}>
          <rect
            x={-40 / scale}
            y={-12 / scale}
            width={80 / scale}
            height={24 / scale}
            fill="white"
            stroke={color}
            strokeWidth={1 / scale}
            rx={4 / scale}
          />
          <text
            x={0}
            y={4 / scale}
            textAnchor="middle"
            fontSize={12 / scale}
            fill={color}
            fontWeight="bold"
          >
            {measurement.value.toFixed(2)} {measurement.unit}
          </text>
        </g>
      </g>
    );
  };

  const renderAngleMeasurement = () => {
    if (measurement.points.length < 3) return null;

    const [p1, p2, p3] = measurement.points;
    const radius = 30 / scale;

    // Calculate angles
    const angle1 = Math.atan2(p1.y - p2.y, p1.x - p2.x);
    const angle2 = Math.atan2(p3.y - p2.y, p3.x - p2.x);

    // Create arc path
    const startAngle = angle1;
    const endAngle = angle2;
    const largeArc = Math.abs(endAngle - startAngle) > Math.PI ? 1 : 0;

    const arcStart = {
      x: p2.x + radius * Math.cos(startAngle),
      y: p2.y + radius * Math.sin(startAngle),
    };
    const arcEnd = {
      x: p2.x + radius * Math.cos(endAngle),
      y: p2.y + radius * Math.sin(endAngle),
    };

    const arcPath = `M ${arcStart.x} ${arcStart.y} A ${radius} ${radius} 0 ${largeArc} 1 ${arcEnd.x} ${arcEnd.y}`;

    // Label position (middle of arc)
    const midAngle = (startAngle + endAngle) / 2;
    const labelPos = {
      x: p2.x + (radius + 20 / scale) * Math.cos(midAngle),
      y: p2.y + (radius + 20 / scale) * Math.sin(midAngle),
    };

    return (
      <g>
        {/* Lines */}
        <line
          x1={p2.x}
          y1={p2.y}
          x2={p1.x}
          y2={p1.y}
          stroke={color}
          strokeWidth={1 / scale}
          opacity={0.5}
        />
        <line
          x1={p2.x}
          y1={p2.y}
          x2={p3.x}
          y2={p3.y}
          stroke={color}
          strokeWidth={1 / scale}
          opacity={0.5}
        />

        {/* Arc */}
        <path
          d={arcPath}
          stroke={color}
          strokeWidth={2 / scale}
          fill="none"
        />

        {/* Vertex point */}
        <circle cx={p2.x} cy={p2.y} r={4 / scale} fill={color} />

        {/* Label */}
        <g transform={`translate(${labelPos.x}, ${labelPos.y})`}>
          <rect
            x={-30 / scale}
            y={-12 / scale}
            width={60 / scale}
            height={24 / scale}
            fill="white"
            stroke={color}
            strokeWidth={1 / scale}
            rx={4 / scale}
          />
          <text
            x={0}
            y={4 / scale}
            textAnchor="middle"
            fontSize={12 / scale}
            fill={color}
            fontWeight="bold"
          >
            {measurement.value.toFixed(1)}°
          </text>
        </g>
      </g>
    );
  };

  const renderAreaMeasurement = () => {
    if (measurement.points.length < 3) return null;

    // Calculate centroid
    const centroid = measurement.points.reduce(
      (acc, p) => ({ x: acc.x + p.x, y: acc.y + p.y }),
      { x: 0, y: 0 }
    );
    centroid.x /= measurement.points.length;
    centroid.y /= measurement.points.length;

    // Create polygon path
    const pathData =
      'M ' +
      measurement.points.map((p) => `${p.x},${p.y}`).join(' L ') +
      ' Z';

    return (
      <g>
        {/* Polygon outline */}
        <path
          d={pathData}
          stroke={color}
          strokeWidth={2 / scale}
          fill={color}
          fillOpacity={0.1}
          strokeDasharray="5,5"
        />

        {/* Vertices */}
        {measurement.points.map((p, i) => (
          <circle key={i} cx={p.x} cy={p.y} r={4 / scale} fill={color} />
        ))}

        {/* Label at centroid */}
        <g transform={`translate(${centroid.x}, ${centroid.y})`}>
          <rect
            x={-50 / scale}
            y={-12 / scale}
            width={100 / scale}
            height={24 / scale}
            fill="white"
            stroke={color}
            strokeWidth={1 / scale}
            rx={4 / scale}
          />
          <text
            x={0}
            y={4 / scale}
            textAnchor="middle"
            fontSize={12 / scale}
            fill={color}
            fontWeight="bold"
          >
            {measurement.value.toFixed(2)} {measurement.unit}²
          </text>
        </g>
      </g>
    );
  };

  const renderMeasurement = () => {
    switch (measurement.type) {
      case 'distance':
        return renderDistanceMeasurement();
      case 'angle':
        return renderAngleMeasurement();
      case 'area':
        return renderAreaMeasurement();
      default:
        return null;
    }
  };

  return (
    <g className={className}>
      {renderMeasurement()}

      {/* Remove button (shown on first point) */}
      {onRemove && measurement.points.length > 0 && (
        <g
          transform={`translate(${measurement.points[0].x}, ${measurement.points[0].y})`}
          onClick={() => onRemove(measurement.id)}
          style={{ cursor: 'pointer' }}
        >
          <circle
            cx={0}
            cy={-15 / scale}
            r={8 / scale}
            fill="white"
            stroke="#ef4444"
            strokeWidth={1 / scale}
          />
          <text
            x={0}
            y={-11 / scale}
            textAnchor="middle"
            fontSize={12 / scale}
            fill="#ef4444"
            fontWeight="bold"
          >
            ×
          </text>
        </g>
      )}
    </g>
  );
};
