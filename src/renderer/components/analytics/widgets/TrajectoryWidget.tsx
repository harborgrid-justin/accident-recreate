/**
 * AccuScene Enterprise v0.3.0 - Trajectory Widget
 * 2D/3D vehicle trajectory visualization with path analysis
 */

import React, { useMemo, useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Scatter, ScatterChart } from 'recharts';
import { WidgetProps, AnalyticsData } from '../types';

const TrajectoryWidget: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  const [viewMode, setViewMode] = useState<'2d' | '3d'>('2d');
  const [selectedVehicle, setSelectedVehicle] = useState<string | null>(null);

  // Process trajectory data
  const trajectoryData = useMemo(() => {
    if (!data?.vehicles || data.vehicles.length === 0) return [];

    const vehicle = selectedVehicle
      ? data.vehicles.find((v) => v.id === selectedVehicle)
      : data.vehicles[0];

    if (!vehicle?.trajectory) return [];

    return vehicle.trajectory.map((point) => ({
      x: point.position.x,
      y: point.position.y,
      z: point.position.z,
      time: point.time,
      speed: Math.sqrt(
        point.velocity.x ** 2 + point.velocity.y ** 2 + point.velocity.z ** 2
      ),
      heading: point.heading,
    }));
  }, [data, selectedVehicle]);

  // Custom tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (!active || !payload || payload.length === 0) return null;

    const point = payload[0].payload;

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
          <div>Time: {point.time.toFixed(2)}s</div>
          <div>
            Position: ({point.x.toFixed(1)}, {point.y.toFixed(1)}, {point.z.toFixed(1)})
          </div>
          <div>Speed: {point.speed.toFixed(1)} m/s</div>
          <div>Heading: {((point.heading * 180) / Math.PI).toFixed(1)}°</div>
        </div>
      </div>
    );
  };

  const colors = ['#3b82f6', '#ef4444', '#10b981', '#f59e0b'];

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
        No trajectory data available
      </div>
    );
  }

  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        gap: '1rem',
      }}
    >
      {/* Controls */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <select
          value={selectedVehicle || data.vehicles[0].id}
          onChange={(e) => setSelectedVehicle(e.target.value)}
          style={{
            padding: '0.5rem',
            backgroundColor: '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
            borderRadius: '4px',
            fontSize: '0.875rem',
          }}
        >
          {data.vehicles.map((vehicle) => (
            <option key={vehicle.id} value={vehicle.id}>
              {vehicle.name}
            </option>
          ))}
        </select>

        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            onClick={() => setViewMode('2d')}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: viewMode === '2d' ? '#3b82f6' : '#1e293b',
              color: '#f1f5f9',
              border: '1px solid #334155',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '0.875rem',
            }}
          >
            2D
          </button>
          <button
            onClick={() => setViewMode('3d')}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: viewMode === '3d' ? '#3b82f6' : '#1e293b',
              color: '#f1f5f9',
              border: '1px solid #334155',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '0.875rem',
            }}
          >
            3D
          </button>
        </div>
      </div>

      {/* Trajectory Chart */}
      <div style={{ flex: 1, minHeight: 0 }}>
        <ResponsiveContainer width="100%" height="100%">
          <ScatterChart margin={{ top: 10, right: 30, bottom: 20, left: 10 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#334155" opacity={0.3} />
            <XAxis
              type="number"
              dataKey="x"
              name="X Position"
              stroke="#94a3b8"
              style={{ fontSize: '0.75rem' }}
              label={{
                value: 'X Position (m)',
                position: 'insideBottom',
                offset: -10,
                fill: '#94a3b8',
              }}
            />
            <YAxis
              type="number"
              dataKey="y"
              name="Y Position"
              stroke="#94a3b8"
              style={{ fontSize: '0.75rem' }}
              label={{
                value: 'Y Position (m)',
                angle: -90,
                position: 'insideLeft',
                fill: '#94a3b8',
              }}
            />
            <Tooltip content={<CustomTooltip />} cursor={{ strokeDasharray: '3 3' }} />
            <Scatter
              data={trajectoryData}
              fill="#3b82f6"
              line={{ stroke: '#3b82f6', strokeWidth: 2 }}
              isAnimationActive={true}
              animationDuration={1000}
            />
          </ScatterChart>
        </ResponsiveContainer>
      </div>

      {/* Statistics */}
      {trajectoryData.length > 0 && (
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(100px, 1fr))',
            gap: '0.75rem',
          }}
        >
          <div
            style={{
              backgroundColor: '#1e293b',
              padding: '0.5rem',
              borderRadius: '4px',
              border: '1px solid #334155',
            }}
          >
            <div style={{ fontSize: '0.65rem', color: '#94a3b8' }}>
              Duration
            </div>
            <div style={{ fontSize: '1rem', fontWeight: 600, color: '#f1f5f9' }}>
              {trajectoryData[trajectoryData.length - 1].time.toFixed(1)}s
            </div>
          </div>
          <div
            style={{
              backgroundColor: '#1e293b',
              padding: '0.5rem',
              borderRadius: '4px',
              border: '1px solid #334155',
            }}
          >
            <div style={{ fontSize: '0.65rem', color: '#94a3b8' }}>
              Max Speed
            </div>
            <div style={{ fontSize: '1rem', fontWeight: 600, color: '#3b82f6' }}>
              {Math.max(...trajectoryData.map((p) => p.speed)).toFixed(1)} m/s
            </div>
          </div>
          <div
            style={{
              backgroundColor: '#1e293b',
              padding: '0.5rem',
              borderRadius: '4px',
              border: '1px solid #334155',
            }}
          >
            <div style={{ fontSize: '0.65rem', color: '#94a3b8' }}>
              Distance
            </div>
            <div style={{ fontSize: '1rem', fontWeight: 600, color: '#10b981' }}>
              {calculateDistance(trajectoryData).toFixed(1)} m
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// Calculate total distance traveled
function calculateDistance(trajectory: any[]): number {
  let distance = 0;
  for (let i = 1; i < trajectory.length; i++) {
    const dx = trajectory[i].x - trajectory[i - 1].x;
    const dy = trajectory[i].y - trajectory[i - 1].y;
    const dz = trajectory[i].z - trajectory[i - 1].z;
    distance += Math.sqrt(dx ** 2 + dy ** 2 + dz ** 2);
  }
  return distance;
}

export default TrajectoryWidget;
