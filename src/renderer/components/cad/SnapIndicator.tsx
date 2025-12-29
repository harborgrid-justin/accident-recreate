/**
 * AccuScene Enterprise v0.3.0 - Snap Indicator Component
 */

import React from 'react';
import { SnapPoint, SnapType } from './types';

interface SnapIndicatorProps {
  snapPoint: SnapPoint | null;
  scale?: number;
  className?: string;
}

export const SnapIndicator: React.FC<SnapIndicatorProps> = ({
  snapPoint,
  scale = 1,
  className = '',
}) => {
  if (!snapPoint) return null;

  const getSnapIcon = (type: SnapType): { shape: JSX.Element; color: string } => {
    const size = 12 / scale;
    const lineWidth = 2 / scale;

    switch (type) {
      case SnapType.GRID:
        return {
          color: '#10b981',
          shape: (
            <circle
              cx={snapPoint.point.x}
              cy={snapPoint.point.y}
              r={size / 2}
              fill="none"
              stroke="#10b981"
              strokeWidth={lineWidth}
            />
          ),
        };

      case SnapType.ENDPOINT:
        return {
          color: '#ef4444',
          shape: (
            <rect
              x={snapPoint.point.x - size / 2}
              y={snapPoint.point.y - size / 2}
              width={size}
              height={size}
              fill="none"
              stroke="#ef4444"
              strokeWidth={lineWidth}
            />
          ),
        };

      case SnapType.MIDPOINT:
        return {
          color: '#3b82f6',
          shape: (
            <g>
              <line
                x1={snapPoint.point.x - size / 2}
                y1={snapPoint.point.y}
                x2={snapPoint.point.x + size / 2}
                y2={snapPoint.point.y}
                stroke="#3b82f6"
                strokeWidth={lineWidth}
              />
              <line
                x1={snapPoint.point.x}
                y1={snapPoint.point.y - size / 2}
                x2={snapPoint.point.x}
                y2={snapPoint.point.y + size / 2}
                stroke="#3b82f6"
                strokeWidth={lineWidth}
              />
            </g>
          ),
        };

      case SnapType.CENTER:
        return {
          color: '#f59e0b',
          shape: (
            <g>
              <circle
                cx={snapPoint.point.x}
                cy={snapPoint.point.y}
                r={size / 2}
                fill="none"
                stroke="#f59e0b"
                strokeWidth={lineWidth}
              />
              <circle
                cx={snapPoint.point.x}
                cy={snapPoint.point.y}
                r={size / 4}
                fill="none"
                stroke="#f59e0b"
                strokeWidth={lineWidth}
              />
            </g>
          ),
        };

      case SnapType.INTERSECTION:
        return {
          color: '#8b5cf6',
          shape: (
            <g>
              <line
                x1={snapPoint.point.x - size / 2}
                y1={snapPoint.point.y - size / 2}
                x2={snapPoint.point.x + size / 2}
                y2={snapPoint.point.y + size / 2}
                stroke="#8b5cf6"
                strokeWidth={lineWidth}
              />
              <line
                x1={snapPoint.point.x - size / 2}
                y1={snapPoint.point.y + size / 2}
                x2={snapPoint.point.x + size / 2}
                y2={snapPoint.point.y - size / 2}
                stroke="#8b5cf6"
                strokeWidth={lineWidth}
              />
            </g>
          ),
        };

      case SnapType.PERPENDICULAR:
        return {
          color: '#06b6d4',
          shape: (
            <g>
              <line
                x1={snapPoint.point.x - size / 2}
                y1={snapPoint.point.y}
                x2={snapPoint.point.x + size / 2}
                y2={snapPoint.point.y}
                stroke="#06b6d4"
                strokeWidth={lineWidth}
              />
              <line
                x1={snapPoint.point.x}
                y1={snapPoint.point.y - size / 2}
                x2={snapPoint.point.x}
                y2={snapPoint.point.y + size / 2}
                stroke="#06b6d4"
                strokeWidth={lineWidth}
              />
              <rect
                x={snapPoint.point.x - size / 4}
                y={snapPoint.point.y - size / 4}
                width={size / 2}
                height={size / 2}
                fill="none"
                stroke="#06b6d4"
                strokeWidth={lineWidth}
              />
            </g>
          ),
        };

      case SnapType.TANGENT:
        return {
          color: '#ec4899',
          shape: (
            <g>
              <circle
                cx={snapPoint.point.x}
                cy={snapPoint.point.y}
                r={size / 2}
                fill="none"
                stroke="#ec4899"
                strokeWidth={lineWidth}
              />
              <line
                x1={snapPoint.point.x - size / 2}
                y1={snapPoint.point.y + size / 2}
                x2={snapPoint.point.x + size / 2}
                y2={snapPoint.point.y + size / 2}
                stroke="#ec4899"
                strokeWidth={lineWidth}
              />
            </g>
          ),
        };

      case SnapType.QUADRANT:
        return {
          color: '#14b8a6',
          shape: (
            <g>
              <circle
                cx={snapPoint.point.x}
                cy={snapPoint.point.y}
                r={size / 2}
                fill="none"
                stroke="#14b8a6"
                strokeWidth={lineWidth}
              />
              <circle
                cx={snapPoint.point.x}
                cy={snapPoint.point.y}
                r={size / 6}
                fill="#14b8a6"
              />
            </g>
          ),
        };

      case SnapType.NEAREST:
      default:
        return {
          color: '#6366f1',
          shape: (
            <circle
              cx={snapPoint.point.x}
              cy={snapPoint.point.y}
              r={size / 3}
              fill="#6366f1"
              opacity={0.5}
            />
          ),
        };
    }
  };

  const { shape, color } = getSnapIcon(snapPoint.type);

  return (
    <g className={className}>
      {/* Snap indicator */}
      {shape}

      {/* Pulse animation */}
      <circle
        cx={snapPoint.point.x}
        cy={snapPoint.point.y}
        r={8 / scale}
        fill="none"
        stroke={color}
        strokeWidth={1 / scale}
        opacity={0.3}
      >
        <animate
          attributeName="r"
          from={8 / scale}
          to={16 / scale}
          dur="1s"
          repeatCount="indefinite"
        />
        <animate
          attributeName="opacity"
          from={0.3}
          to={0}
          dur="1s"
          repeatCount="indefinite"
        />
      </circle>

      {/* Tooltip */}
      <g transform={`translate(${snapPoint.point.x}, ${snapPoint.point.y - 20 / scale})`}>
        <rect
          x={-30 / scale}
          y={-10 / scale}
          width={60 / scale}
          height={20 / scale}
          fill="white"
          stroke={color}
          strokeWidth={1 / scale}
          rx={4 / scale}
          opacity={0.9}
        />
        <text
          x={0}
          y={2 / scale}
          textAnchor="middle"
          fontSize={10 / scale}
          fill={color}
          fontWeight="bold"
        >
          {snapPoint.type.toUpperCase()}
        </text>
      </g>
    </g>
  );
};
