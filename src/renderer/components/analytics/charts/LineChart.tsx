/**
 * AccuScene Enterprise v0.3.0 - Line Chart Component
 * Configurable line chart with multiple series support
 */

import React from 'react';
import {
  LineChart as RechartsLine,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ReferenceLine,
} from 'recharts';
import { ChartConfig, DataSeries } from '../types';

interface LineChartProps {
  data: any[];
  series: DataSeries[];
  config?: Partial<ChartConfig>;
  width?: string | number;
  height?: string | number;
}

const LineChart: React.FC<LineChartProps> = ({
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
  const CustomTooltip = ({ active, payload, label }: any) => {
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
          {label}
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
            {config.xAxis?.format
              ? config.xAxis.format(entry.value)
              : entry.value.toFixed(2)}
          </div>
        ))}
      </div>
    );
  };

  return (
    <ResponsiveContainer width={width} height={height}>
      <RechartsLine
        data={data}
        margin={config.tooltip?.show ? { top: 10, right: 30, left: 0, bottom: 0 } : undefined}
      >
        {config.xAxis?.grid !== false && (
          <CartesianGrid strokeDasharray="3 3" stroke="#334155" opacity={0.3} />
        )}

        <XAxis
          dataKey={config.xAxis?.label || 'x'}
          stroke="#94a3b8"
          style={{ fontSize: '0.75rem' }}
          tickFormatter={config.xAxis?.format}
          label={
            config.xAxis?.label
              ? {
                  value: config.xAxis.label,
                  position: 'insideBottom',
                  offset: -5,
                  fill: '#94a3b8',
                }
              : undefined
          }
        />

        <YAxis
          stroke="#94a3b8"
          style={{ fontSize: '0.75rem' }}
          tickFormatter={config.yAxis?.format}
          domain={
            config.yAxis?.min !== undefined || config.yAxis?.max !== undefined
              ? [config.yAxis?.min || 'auto', config.yAxis?.max || 'auto']
              : undefined
          }
          label={
            config.yAxis?.label
              ? {
                  value: config.yAxis.label,
                  angle: -90,
                  position: 'insideLeft',
                  fill: '#94a3b8',
                }
              : undefined
          }
        />

        {config.tooltip?.show !== false && <Tooltip content={<CustomTooltip />} />}

        {config.legend?.show !== false && (
          <Legend
            wrapperStyle={{ fontSize: '0.75rem' }}
            iconType="line"
            align={config.legend?.align}
          />
        )}

        {/* Render series */}
        {series
          .filter((s) => s.visible !== false)
          .map((s, index) => (
            <Line
              key={s.id}
              type="monotone"
              dataKey={s.id}
              name={s.name}
              stroke={s.color || colors[index % colors.length]}
              strokeWidth={2}
              dot={{ r: 3 }}
              activeDot={{ r: 5 }}
              isAnimationActive={config.animations?.duration !== 0}
              animationDuration={config.animations?.duration || 1000}
              animationEasing={config.animations?.easing || 'ease-out'}
            />
          ))}
      </RechartsLine>
    </ResponsiveContainer>
  );
};

export default LineChart;
