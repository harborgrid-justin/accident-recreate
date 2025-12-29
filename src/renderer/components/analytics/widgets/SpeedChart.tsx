/**
 * AccuScene Enterprise v0.3.0 - Speed Chart Widget
 * Speed vs time chart with impact annotations and critical event markers
 */

import React, { useMemo } from 'react';
import { LineChart as RechartsLine, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, ReferenceLine, Area, ComposedChart } from 'recharts';
import { WidgetProps, AnalyticsData } from '../types';

const SpeedChart: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  // Process speed data from vehicles
  const chartData = useMemo(() => {
    if (!data?.vehicles || data.vehicles.length === 0) return [];

    // Get the maximum number of data points across all vehicles
    const maxLength = Math.max(
      ...data.vehicles.map((v) => v.speed?.length || 0)
    );

    // Merge all vehicle speed data into time-indexed points
    const merged: any[] = [];
    for (let i = 0; i < maxLength; i++) {
      const point: any = { time: 0 };
      data.vehicles.forEach((vehicle) => {
        if (vehicle.speed && vehicle.speed[i]) {
          point.time = vehicle.speed[i].timestamp;
          point[vehicle.name] = vehicle.speed[i].value;
        }
      });
      merged.push(point);
    }

    return merged;
  }, [data]);

  // Find impact events for annotations
  const impactAnnotations = useMemo(() => {
    if (!data?.impacts) return [];
    return data.impacts.map((impact) => ({
      x: impact.timestamp,
      label: impact.type,
      severity: impact.severity,
    }));
  }, [data]);

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
        <div style={{ color: '#94a3b8', fontSize: '0.75rem', marginBottom: '0.5rem' }}>
          Time: {label.toFixed(2)}s
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
            <strong>{entry.name}:</strong> {entry.value.toFixed(1)} mph
          </div>
        ))}
      </div>
    );
  };

  // Color palette for vehicles
  const colors = ['#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6', '#ec4899'];

  if (!data?.vehicles || data.vehicles.length === 0) {
    return (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#94a3b8',
        }}
      >
        No speed data available
      </div>
    );
  }

  return (
    <div style={{ width: '100%', height: '100%' }}>
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart
          data={chartData}
          margin={{ top: 10, right: 30, left: 0, bottom: 0 }}
        >
          <defs>
            {data.vehicles.map((vehicle, index) => (
              <linearGradient
                key={vehicle.id}
                id={`gradient-${vehicle.id}`}
                x1="0"
                y1="0"
                x2="0"
                y2="1"
              >
                <stop
                  offset="5%"
                  stopColor={colors[index % colors.length]}
                  stopOpacity={0.3}
                />
                <stop
                  offset="95%"
                  stopColor={colors[index % colors.length]}
                  stopOpacity={0}
                />
              </linearGradient>
            ))}
          </defs>

          <CartesianGrid strokeDasharray="3 3" stroke="#334155" opacity={0.3} />

          <XAxis
            dataKey="time"
            stroke="#94a3b8"
            style={{ fontSize: '0.75rem' }}
            label={{ value: 'Time (s)', position: 'insideBottom', offset: -5, fill: '#94a3b8' }}
            tickFormatter={(value) => value.toFixed(1)}
          />

          <YAxis
            stroke="#94a3b8"
            style={{ fontSize: '0.75rem' }}
            label={{ value: 'Speed (mph)', angle: -90, position: 'insideLeft', fill: '#94a3b8' }}
          />

          <Tooltip content={<CustomTooltip />} />

          <Legend
            wrapperStyle={{ fontSize: '0.75rem' }}
            iconType="line"
          />

          {/* Impact event markers */}
          {impactAnnotations.map((annotation, index) => (
            <ReferenceLine
              key={index}
              x={annotation.x}
              stroke={annotation.severity > 0.7 ? '#ef4444' : '#f59e0b'}
              strokeDasharray="3 3"
              label={{
                value: annotation.label,
                fill: '#f1f5f9',
                fontSize: 10,
                position: 'top',
              }}
            />
          ))}

          {/* Vehicle speed lines with area fill */}
          {data.vehicles.map((vehicle, index) => (
            <React.Fragment key={vehicle.id}>
              <Area
                type="monotone"
                dataKey={vehicle.name}
                fill={`url(#gradient-${vehicle.id})`}
                stroke="none"
                isAnimationActive={true}
                animationDuration={1000}
                animationEasing="ease-out"
              />
              <Line
                type="monotone"
                dataKey={vehicle.name}
                stroke={colors[index % colors.length]}
                strokeWidth={2}
                dot={{ r: 3, fill: colors[index % colors.length] }}
                activeDot={{ r: 5 }}
                isAnimationActive={true}
                animationDuration={1000}
                animationEasing="ease-out"
              />
            </React.Fragment>
          ))}
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
};

export default SpeedChart;
