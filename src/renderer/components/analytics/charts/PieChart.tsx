/**
 * AccuScene Enterprise v0.3.0 - Pie Chart Component
 * Pie and donut charts with customizable appearance
 */

import React from 'react';
import {
  PieChart as RechartsPie,
  Pie,
  Cell,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { ChartConfig } from '../types';

interface PieChartProps {
  data: Array<{ name: string; value: number; color?: string }>;
  config?: Partial<ChartConfig>;
  donut?: boolean;
  innerRadius?: number;
  outerRadius?: number;
  width?: string | number;
  height?: string | number;
}

const PieChart: React.FC<PieChartProps> = ({
  data,
  config = {},
  donut = false,
  innerRadius = 0,
  outerRadius = 80,
  width = '100%',
  height = '100%',
}) => {
  const colors = config.colors || [
    '#3b82f6',
    '#ef4444',
    '#10b981',
    '#f59e0b',
    '#8b5cf6',
    '#ec4899',
    '#06b6d4',
    '#84cc16',
  ];

  // Custom label
  const renderLabel = (entry: any) => {
    const percent = ((entry.value / entry.payload.total) * 100).toFixed(1);
    return `${entry.name} (${percent}%)`;
  };

  // Custom tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (!active || !payload || payload.length === 0) return null;

    const data = payload[0].payload;
    const total = data.total || 0;
    const percent = total > 0 ? ((data.value / total) * 100).toFixed(1) : '0';

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
            color: data.fill,
            fontSize: '0.875rem',
            fontWeight: 600,
            marginBottom: '0.25rem',
          }}
        >
          {data.name}
        </div>
        <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
          <div>Value: {data.value.toFixed(2)}</div>
          <div>Percentage: {percent}%</div>
        </div>
      </div>
    );
  };

  // Calculate total for percentages
  const total = data.reduce((sum, item) => sum + item.value, 0);
  const dataWithTotal = data.map((item) => ({ ...item, total }));

  return (
    <ResponsiveContainer width={width} height={height}>
      <RechartsPie>
        <Pie
          data={dataWithTotal}
          cx="50%"
          cy="50%"
          labelLine={false}
          label={config.legend?.show !== false ? renderLabel : undefined}
          outerRadius={outerRadius}
          innerRadius={donut ? innerRadius || outerRadius * 0.6 : innerRadius}
          fill="#8884d8"
          dataKey="value"
          isAnimationActive={config.animations?.duration !== 0}
          animationDuration={config.animations?.duration || 1000}
          animationEasing={config.animations?.easing || 'ease-out'}
        >
          {dataWithTotal.map((entry, index) => (
            <Cell
              key={`cell-${index}`}
              fill={entry.color || colors[index % colors.length]}
            />
          ))}
        </Pie>

        {config.tooltip?.show !== false && <Tooltip content={<CustomTooltip />} />}

        {config.legend?.show !== false && (
          <Legend
            wrapperStyle={{ fontSize: '0.75rem' }}
            iconType="circle"
            align={config.legend?.align}
          />
        )}
      </RechartsPie>
    </ResponsiveContainer>
  );
};

export default PieChart;
