/**
 * AccuScene Enterprise v0.3.0 - Bar Chart Component
 * Horizontal and vertical bar charts with stacking support
 */

import React from 'react';
import {
  BarChart as RechartsBar,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  Cell,
} from 'recharts';
import { ChartConfig, DataSeries } from '../types';

interface BarChartProps {
  data: any[];
  series: DataSeries[];
  config?: Partial<ChartConfig>;
  orientation?: 'vertical' | 'horizontal';
  stacked?: boolean;
  width?: string | number;
  height?: string | number;
}

const BarChart: React.FC<BarChartProps> = ({
  data,
  series,
  config = {},
  orientation = 'vertical',
  stacked = false,
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
            {config.yAxis?.format
              ? config.yAxis.format(entry.value)
              : entry.value.toFixed(2)}
          </div>
        ))}
      </div>
    );
  };

  return (
    <ResponsiveContainer width={width} height={height}>
      <RechartsBar
        data={data}
        layout={orientation === 'horizontal' ? 'vertical' : 'horizontal'}
        margin={{ top: 10, right: 30, left: 0, bottom: 0 }}
      >
        {config.xAxis?.grid !== false && (
          <CartesianGrid strokeDasharray="3 3" stroke="#334155" opacity={0.3} />
        )}

        <XAxis
          type={orientation === 'horizontal' ? 'number' : 'category'}
          dataKey={orientation === 'horizontal' ? undefined : (config.xAxis?.label || 'x')}
          stroke="#94a3b8"
          style={{ fontSize: '0.75rem' }}
          tickFormatter={config.xAxis?.format}
        />

        <YAxis
          type={orientation === 'horizontal' ? 'category' : 'number'}
          dataKey={orientation === 'horizontal' ? (config.xAxis?.label || 'x') : undefined}
          stroke="#94a3b8"
          style={{ fontSize: '0.75rem' }}
          tickFormatter={config.yAxis?.format}
          domain={
            config.yAxis?.min !== undefined || config.yAxis?.max !== undefined
              ? [config.yAxis?.min || 'auto', config.yAxis?.max || 'auto']
              : undefined
          }
        />

        {config.tooltip?.show !== false && <Tooltip content={<CustomTooltip />} />}

        {config.legend?.show !== false && (
          <Legend
            wrapperStyle={{ fontSize: '0.75rem' }}
            iconType="rect"
            align={config.legend?.align}
          />
        )}

        {/* Render series */}
        {series
          .filter((s) => s.visible !== false)
          .map((s, index) => (
            <Bar
              key={s.id}
              dataKey={s.id}
              name={s.name}
              fill={s.color || colors[index % colors.length]}
              stackId={stacked ? 'stack' : undefined}
              isAnimationActive={config.animations?.duration !== 0}
              animationDuration={config.animations?.duration || 1000}
              animationEasing={config.animations?.easing || 'ease-out'}
            />
          ))}
      </RechartsBar>
    </ResponsiveContainer>
  );
};

export default BarChart;
