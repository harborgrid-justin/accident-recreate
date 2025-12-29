/**
 * AccuScene Enterprise v0.3.0 - Radar Chart Component
 * Multi-axis radar chart for comparative analysis
 */

import React from 'react';
import {
  RadarChart as RechartsRadar,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  Radar,
  Legend,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { ChartConfig, DataSeries } from '../types';

interface RadarChartProps {
  data: any[];
  series: DataSeries[];
  config?: Partial<ChartConfig>;
  width?: string | number;
  height?: string | number;
}

const RadarChart: React.FC<RadarChartProps> = ({
  data,
  series,
  config = {},
  width = '100%',
  height = '100%',
}) => {
  const colors = [
    '#3b82f6',
    '#ef4444',
    '#10b981',
    '#f59e0b',
    '#8b5cf6',
    '#ec4899',
    '#06b6d4',
    '#84cc16',
  ];

  // Custom tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (!active || !payload || payload.length === 0) return null;

    return (
      <div
        style={{
          backgroundColor: 'rgba(30, 41, 59, 0.95)',
          border: '1px solid #334155',
          borderRadius: '4px',
          padding: '0.75rem',
          boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
        }}
      >
        <div
          style={{
            color: '#94a3b8',
            fontSize: '0.75rem',
            marginBottom: '0.5rem',
          }}
        >
          {payload[0]?.payload?.subject}
        </div>
        {payload.map((entry: any, index: number) => (
          <div
            key={index}
            style={{
              color: entry.color,
              fontSize: '0.875rem',
              marginBottom: '0.25rem',
            }}
          >
            <strong>{entry.name}:</strong>{' '}
            {entry.value.toFixed(2)}
          </div>
        ))}
      </div>
    );
  };

  return (
    <ResponsiveContainer width={width} height={height}>
      <RechartsRadar data={data}>
        <PolarGrid stroke="#334155" />

        <PolarAngleAxis
          dataKey="subject"
          stroke="#94a3b8"
          style={{ fontSize: '0.75rem' }}
        />

        <PolarRadiusAxis
          angle={90}
          domain={[0, config.yAxis?.max || 'auto']}
          stroke="#94a3b8"
          style={{ fontSize: '0.75rem' }}
        />

        {config.tooltip?.show !== false && <Tooltip content={<CustomTooltip />} />}

        {config.legend?.show !== false && (
          <Legend
            wrapperStyle={{ fontSize: '0.75rem' }}
            iconType="circle"
            align={config.legend?.align}
          />
        )}

        {series
          .filter((s) => s.visible !== false)
          .map((s, index) => (
            <Radar
              key={s.id}
              name={s.name}
              dataKey={s.id}
              stroke={s.color || colors[index % colors.length]}
              fill={s.color || colors[index % colors.length]}
              fillOpacity={0.3}
              isAnimationActive={config.animations?.duration !== 0}
              animationDuration={config.animations?.duration || 1000}
            />
          ))}
      </RechartsRadar>
    </ResponsiveContainer>
  );
};

export default RadarChart;
