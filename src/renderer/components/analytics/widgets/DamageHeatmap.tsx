/**
 * AccuScene Enterprise v0.3.0 - Damage Heatmap Widget
 * Vehicle damage visualization with severity heatmap
 */

import React, { useMemo } from 'react';
import { WidgetProps, AnalyticsData } from '../types';

const DamageHeatmap: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  const [selectedVehicle, setSelectedVehicle] = React.useState<string | null>(
    null
  );

  // Get damage data for selected vehicle
  const damageData = useMemo(() => {
    if (!data?.vehicles || data.vehicles.length === 0) return null;

    const vehicle = selectedVehicle
      ? data.vehicles.find((v) => v.id === selectedVehicle)
      : data.vehicles[0];

    return vehicle?.damageProfile || [];
  }, [data, selectedVehicle]);

  // Calculate damage statistics
  const stats = useMemo(() => {
    if (!damageData || damageData.length === 0) return null;

    const totalArea = damageData.reduce((sum, d) => sum + d.area, 0);
    const avgSeverity =
      damageData.reduce((sum, d) => sum + d.severity, 0) / damageData.length;
    const maxDepth = Math.max(...damageData.map((d) => d.depth));
    const criticalDamage = damageData.filter((d) => d.severity > 0.7).length;

    return {
      count: damageData.length,
      totalArea,
      avgSeverity,
      maxDepth,
      criticalDamage,
    };
  }, [damageData]);

  // Get color based on severity
  const getSeverityColor = (severity: number): string => {
    const hue = (1 - severity) * 120; // Red (0) to Green (120)
    return `hsl(${hue}, 70%, 50%)`;
  };

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
        No damage data available
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
      {/* Vehicle Selector */}
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

      {/* Statistics */}
      {stats && (
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
            gap: '0.75rem',
          }}
        >
          <div
            style={{
              backgroundColor: '#1e293b',
              padding: '0.75rem',
              borderRadius: '4px',
              border: '1px solid #334155',
            }}
          >
            <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
              Damage Points
            </div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#f1f5f9' }}>
              {stats.count}
            </div>
          </div>
          <div
            style={{
              backgroundColor: '#1e293b',
              padding: '0.75rem',
              borderRadius: '4px',
              border: '1px solid #334155',
            }}
          >
            <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
              Total Area
            </div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#3b82f6' }}>
              {stats.totalArea.toFixed(1)}
              <span style={{ fontSize: '0.75rem', marginLeft: '0.25rem' }}>
                m²
              </span>
            </div>
          </div>
          <div
            style={{
              backgroundColor: '#1e293b',
              padding: '0.75rem',
              borderRadius: '4px',
              border: '1px solid #334155',
            }}
          >
            <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
              Max Depth
            </div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#ef4444' }}>
              {stats.maxDepth.toFixed(1)}
              <span style={{ fontSize: '0.75rem', marginLeft: '0.25rem' }}>
                cm
              </span>
            </div>
          </div>
          <div
            style={{
              backgroundColor: '#1e293b',
              padding: '0.75rem',
              borderRadius: '4px',
              border: '1px solid #334155',
            }}
          >
            <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
              Critical
            </div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#f59e0b' }}>
              {stats.criticalDamage}
            </div>
          </div>
        </div>
      )}

      {/* Heatmap Visualization */}
      <div
        style={{
          flex: 1,
          backgroundColor: '#1e293b',
          borderRadius: '4px',
          border: '1px solid #334155',
          padding: '1rem',
          overflow: 'auto',
          position: 'relative',
        }}
      >
        {damageData && damageData.length > 0 ? (
          <svg width="100%" height="100%" viewBox="0 0 400 200">
            {/* Vehicle outline (simplified top view) */}
            <rect
              x="100"
              y="50"
              width="200"
              height="100"
              fill="none"
              stroke="#475569"
              strokeWidth="2"
              rx="10"
            />
            <rect
              x="120"
              y="60"
              width="160"
              height="80"
              fill="none"
              stroke="#475569"
              strokeWidth="1"
              strokeDasharray="5,5"
            />

            {/* Damage points */}
            {damageData.map((damage) => {
              // Map 3D position to 2D top view
              const x = 200 + damage.location.x * 20;
              const y = 100 + damage.location.y * 20;
              const radius = Math.sqrt(damage.area) * 5;

              return (
                <g key={damage.id}>
                  <circle
                    cx={x}
                    cy={y}
                    r={radius}
                    fill={getSeverityColor(damage.severity)}
                    opacity={0.6}
                  />
                  <circle
                    cx={x}
                    cy={y}
                    r={radius}
                    fill="none"
                    stroke={getSeverityColor(damage.severity)}
                    strokeWidth="2"
                  />
                  <title>
                    {damage.type}: Severity {(damage.severity * 100).toFixed(0)}%,
                    Depth {damage.depth.toFixed(1)}cm
                  </title>
                </g>
              );
            })}
          </svg>
        ) : (
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
            No damage points recorded
          </div>
        )}
      </div>

      {/* Legend */}
      <div style={{ fontSize: '0.75rem' }}>
        <div
          style={{
            color: '#94a3b8',
            marginBottom: '0.5rem',
            fontWeight: 600,
          }}
        >
          Severity Scale
        </div>
        <div
          style={{
            display: 'flex',
            height: '20px',
            borderRadius: '4px',
            overflow: 'hidden',
          }}
        >
          {Array.from({ length: 10 }).map((_, i) => (
            <div
              key={i}
              style={{
                flex: 1,
                backgroundColor: getSeverityColor(i / 10),
              }}
            />
          ))}
        </div>
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            marginTop: '0.25rem',
            color: '#94a3b8',
          }}
        >
          <span>0% (Minor)</span>
          <span>100% (Critical)</span>
        </div>
      </div>
    </div>
  );
};

export default DamageHeatmap;
