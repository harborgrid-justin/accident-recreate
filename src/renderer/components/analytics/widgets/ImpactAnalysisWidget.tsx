/**
 * AccuScene Enterprise v0.3.0 - Impact Analysis Widget
 * Detailed analysis of impact points with severity visualization
 */

import React, { useMemo } from 'react';
import { ScatterChart, Scatter, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, ZAxis, Cell } from 'recharts';
import { WidgetProps, AnalyticsData, ImpactEvent } from '../types';

const ImpactAnalysisWidget: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  // Process impact data for visualization
  const impactData = useMemo(() => {
    if (!data?.impacts) return [];

    return data.impacts.map((impact) => ({
      x: impact.location.x,
      y: impact.location.y,
      z: impact.location.z,
      energy: impact.energy,
      severity: impact.severity,
      force: Math.sqrt(
        impact.force.x ** 2 + impact.force.y ** 2 + impact.force.z ** 2
      ),
      timestamp: impact.timestamp,
      type: impact.type,
      id: impact.id,
    }));
  }, [data]);

  // Calculate statistics
  const stats = useMemo(() => {
    if (impactData.length === 0) return null;

    const totalEnergy = impactData.reduce((sum, i) => sum + i.energy, 0);
    const avgSeverity =
      impactData.reduce((sum, i) => sum + i.severity, 0) / impactData.length;
    const maxForce = Math.max(...impactData.map((i) => i.force));
    const criticalImpacts = impactData.filter((i) => i.severity > 0.7).length;

    return {
      total: impactData.length,
      totalEnergy,
      avgSeverity,
      maxForce,
      criticalImpacts,
    };
  }, [impactData]);

  // Custom tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (!active || !payload || payload.length === 0) return null;

    const impact = payload[0].payload;

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
            fontWeight: 600,
            marginBottom: '0.5rem',
            color: '#f1f5f9',
          }}
        >
          {impact.type.charAt(0).toUpperCase() + impact.type.slice(1)} Impact
        </div>
        <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
          <div>Time: {impact.timestamp.toFixed(2)}s</div>
          <div>
            Position: ({impact.x.toFixed(1)}, {impact.y.toFixed(1)},{' '}
            {impact.z.toFixed(1)})
          </div>
          <div>Force: {impact.force.toFixed(0)} N</div>
          <div>Energy: {impact.energy.toFixed(0)} J</div>
          <div>
            Severity:{' '}
            <span
              style={{
                color:
                  impact.severity > 0.7
                    ? '#ef4444'
                    : impact.severity > 0.4
                    ? '#f59e0b'
                    : '#10b981',
              }}
            >
              {(impact.severity * 100).toFixed(0)}%
            </span>
          </div>
        </div>
      </div>
    );
  };

  // Get color based on severity
  const getSeverityColor = (severity: number) => {
    if (severity > 0.7) return '#ef4444'; // Critical
    if (severity > 0.4) return '#f59e0b'; // Warning
    return '#10b981'; // Normal
  };

  if (impactData.length === 0) {
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
        No impact data available
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
      {/* Statistics Cards */}
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
              Total Impacts
            </div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#f1f5f9' }}>
              {stats.total}
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
              Total Energy
            </div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#3b82f6' }}>
              {stats.totalEnergy.toFixed(0)}
              <span style={{ fontSize: '0.75rem', marginLeft: '0.25rem' }}>
                J
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
              Max Force
            </div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#ef4444' }}>
              {stats.maxForce.toFixed(0)}
              <span style={{ fontSize: '0.75rem', marginLeft: '0.25rem' }}>
                N
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
              {stats.criticalImpacts}
            </div>
          </div>
        </div>
      )}

      {/* Impact Scatter Plot */}
      <div style={{ flex: 1, minHeight: 0 }}>
        <ResponsiveContainer width="100%" height="100%">
          <ScatterChart margin={{ top: 10, right: 20, bottom: 20, left: 10 }}>
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
            <ZAxis
              type="number"
              dataKey="severity"
              range={[100, 1000]}
              name="Severity"
            />
            <Tooltip content={<CustomTooltip />} cursor={{ strokeDasharray: '3 3' }} />
            <Scatter
              name="Impacts"
              data={impactData}
              isAnimationActive={true}
              animationDuration={800}
            >
              {impactData.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={getSeverityColor(entry.severity)} />
              ))}
            </Scatter>
          </ScatterChart>
        </ResponsiveContainer>
      </div>

      {/* Legend */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          gap: '1rem',
          fontSize: '0.75rem',
          color: '#94a3b8',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <div
            style={{
              width: '12px',
              height: '12px',
              backgroundColor: '#10b981',
              borderRadius: '50%',
            }}
          />
          Low Severity
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <div
            style={{
              width: '12px',
              height: '12px',
              backgroundColor: '#f59e0b',
              borderRadius: '50%',
            }}
          />
          Medium Severity
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <div
            style={{
              width: '12px',
              height: '12px',
              backgroundColor: '#ef4444',
              borderRadius: '50%',
            }}
          />
          Critical Severity
        </div>
      </div>
    </div>
  );
};

export default ImpactAnalysisWidget;
