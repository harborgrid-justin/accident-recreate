/**
 * AccuScene Enterprise v0.3.0 - Scatter Plot Component
 * Scatter plot with optional regression line
 */

import React, { useMemo } from 'react';
import {
  ScatterChart as RechartsScatter,
  Scatter,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ZAxis,
  Line,
  ComposedChart,
} from 'recharts';
import { ChartConfig, DataSeries } from '../types';

interface ScatterPlotProps {
  data: any[];
  series: DataSeries[];
  config?: Partial<ChartConfig>;
  showRegression?: boolean;
  width?: string | number;
  height?: string | number;
}

const ScatterPlot: React.FC<ScatterPlotProps> = ({
  data,
  series,
  config = {},
  showRegression = false,
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

  // Calculate linear regression
  const regressionData = useMemo(() => {
    if (!showRegression || data.length === 0) return null;

    const xKey = series[0]?.id || 'x';
    const yKey = 'y';

    const xSum = data.reduce((sum, point) => sum + (point[xKey] || 0), 0);
    const ySum = data.reduce((sum, point) => sum + (point[yKey] || 0), 0);
    const n = data.length;

    const xMean = xSum / n;
    const yMean = ySum / n;

    let numerator = 0;
    let denominator = 0;

    data.forEach((point) => {
      const xDiff = (point[xKey] || 0) - xMean;
      const yDiff = (point[yKey] || 0) - yMean;
      numerator += xDiff * yDiff;
      denominator += xDiff * xDiff;
    });

    const slope = denominator !== 0 ? numerator / denominator : 0;
    const intercept = yMean - slope * xMean;

    // Generate regression line points
    const xValues = data.map((p) => p[xKey] || 0);
    const minX = Math.min(...xValues);
    const maxX = Math.max(...xValues);

    return [
      { [xKey]: minX, regression: slope * minX + intercept },
      { [xKey]: maxX, regression: slope * maxX + intercept },
    ];
  }, [data, series, showRegression]);

  // Custom tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (!active || !payload || payload.length === 0) return null;

    const data = payload[0].payload;

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
        <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
          {Object.entries(data)
            .filter(([key]) => key !== 'regression')
            .map(([key, value]: [string, any]) => (
              <div key={key}>
                <strong>{key}:</strong> {typeof value === 'number' ? value.toFixed(2) : value}
              </div>
            ))}
        </div>
      </div>
    );
  };

  return (
    <ResponsiveContainer width={width} height={height}>
      {showRegression && regressionData ? (
        <ComposedChart
          data={data}
          margin={{ top: 10, right: 30, left: 0, bottom: 0 }}
        >
          {config.xAxis?.grid !== false && (
            <CartesianGrid strokeDasharray="3 3" stroke="#334155" opacity={0.3} />
          )}

          <XAxis
            dataKey={series[0]?.id || 'x'}
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

          {config.tooltip?.show !== false && (
            <Tooltip content={<CustomTooltip />} cursor={{ strokeDasharray: '3 3' }} />
          )}

          {config.legend?.show !== false && (
            <Legend wrapperStyle={{ fontSize: '0.75rem' }} />
          )}

          {series
            .filter((s) => s.visible !== false)
            .map((s, index) => (
              <Scatter
                key={s.id}
                name={s.name}
                data={data}
                fill={s.color || colors[index % colors.length]}
                isAnimationActive={config.animations?.duration !== 0}
                animationDuration={config.animations?.duration || 1000}
              />
            ))}

          <Line
            data={regressionData}
            dataKey="regression"
            stroke="#ef4444"
            strokeWidth={2}
            strokeDasharray="5 5"
            dot={false}
            name="Regression"
          />
        </ComposedChart>
      ) : (
        <RechartsScatter margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
          {config.xAxis?.grid !== false && (
            <CartesianGrid strokeDasharray="3 3" stroke="#334155" opacity={0.3} />
          )}

          <XAxis
            type="number"
            dataKey={series[0]?.id || 'x'}
            stroke="#94a3b8"
            style={{ fontSize: '0.75rem' }}
            tickFormatter={config.xAxis?.format}
          />

          <YAxis
            type="number"
            dataKey="y"
            stroke="#94a3b8"
            style={{ fontSize: '0.75rem' }}
            tickFormatter={config.yAxis?.format}
          />

          <ZAxis range={[50, 400]} />

          {config.tooltip?.show !== false && (
            <Tooltip content={<CustomTooltip />} cursor={{ strokeDasharray: '3 3' }} />
          )}

          {config.legend?.show !== false && (
            <Legend wrapperStyle={{ fontSize: '0.75rem' }} />
          )}

          {series
            .filter((s) => s.visible !== false)
            .map((s, index) => (
              <Scatter
                key={s.id}
                name={s.name}
                data={data}
                fill={s.color || colors[index % colors.length]}
                isAnimationActive={config.animations?.duration !== 0}
                animationDuration={config.animations?.duration || 1000}
              />
            ))}
        </RechartsScatter>
      )}
    </ResponsiveContainer>
  );
};

export default ScatterPlot;
