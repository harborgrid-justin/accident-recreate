/**
 * AccuScene Enterprise v0.3.0 - Statistics Card Widget
 * KPI and statistics display with trends and comparisons
 */

import React, { useMemo } from 'react';
import { LineChart, Line, ResponsiveContainer } from 'recharts';
import { WidgetProps, AnalyticsData, StatisticValue } from '../types';

const StatisticsCard: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  // Calculate key statistics from data
  const statistics = useMemo((): StatisticValue[] => {
    if (!data) return [];

    const stats: StatisticValue[] = [];

    // Vehicle count
    if (data.vehicles) {
      stats.push({
        label: 'Vehicles',
        value: data.vehicles.length,
        icon: '🚗',
        format: 'number',
        color: '#3b82f6',
      });
    }

    // Total impacts
    if (data.impacts) {
      stats.push({
        label: 'Impact Events',
        value: data.impacts.length,
        icon: '💥',
        format: 'number',
        color: '#ef4444',
      });

      // Total energy
      const totalEnergy = data.impacts.reduce((sum, i) => sum + i.energy, 0);
      stats.push({
        label: 'Total Energy',
        value: totalEnergy,
        unit: 'J',
        icon: '⚡',
        format: 'number',
        precision: 0,
        color: '#f59e0b',
      });

      // Average severity
      const avgSeverity =
        data.impacts.reduce((sum, i) => sum + i.severity, 0) / data.impacts.length;
      stats.push({
        label: 'Avg Severity',
        value: avgSeverity * 100,
        unit: '%',
        icon: '📊',
        format: 'percentage',
        precision: 1,
        color: avgSeverity > 0.7 ? '#ef4444' : avgSeverity > 0.4 ? '#f59e0b' : '#10b981',
      });
    }

    // Force vectors
    if (data.forceVectors) {
      const maxForce = Math.max(...data.forceVectors.map((f) => f.magnitude), 0);
      stats.push({
        label: 'Max Force',
        value: maxForce,
        unit: 'N',
        icon: '🎯',
        format: 'number',
        precision: 0,
        color: '#8b5cf6',
      });
    }

    // Energy transfers
    if (data.energyTransfers) {
      const totalTransfer = data.energyTransfers.reduce(
        (sum, t) => sum + t.amount,
        0
      );
      stats.push({
        label: 'Energy Transfer',
        value: totalTransfer,
        unit: 'J',
        icon: '🔄',
        format: 'number',
        precision: 0,
        color: '#10b981',
      });

      // Average efficiency
      const avgEfficiency =
        data.energyTransfers.reduce((sum, t) => sum + t.efficiency, 0) /
        data.energyTransfers.length;
      stats.push({
        label: 'Avg Efficiency',
        value: avgEfficiency * 100,
        unit: '%',
        icon: '📈',
        format: 'percentage',
        precision: 1,
        color: '#3b82f6',
      });
    }

    return stats;
  }, [data]);

  // Format value based on type
  const formatValue = (stat: StatisticValue): string => {
    const value = typeof stat.value === 'number' ? stat.value : parseFloat(stat.value);
    const precision = stat.precision ?? 2;

    switch (stat.format) {
      case 'percentage':
        return `${value.toFixed(precision)}${stat.unit || '%'}`;
      case 'currency':
        return `$${value.toFixed(precision)}`;
      case 'time':
        return `${value.toFixed(precision)}${stat.unit || 's'}`;
      case 'distance':
        return `${value.toFixed(precision)}${stat.unit || 'm'}`;
      default:
        return `${value.toFixed(precision)}${stat.unit ? ' ' + stat.unit : ''}`;
    }
  };

  if (statistics.length === 0) {
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
        No statistics available
      </div>
    );
  }

  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: '1rem',
        overflow: 'auto',
      }}
    >
      {statistics.map((stat, index) => (
        <div
          key={index}
          style={{
            backgroundColor: '#1e293b',
            padding: '1rem',
            borderRadius: '8px',
            border: `1px solid #334155`,
            borderLeft: `4px solid ${stat.color || '#3b82f6'}`,
            display: 'flex',
            flexDirection: 'column',
            gap: '0.5rem',
            transition: 'transform 0.2s, box-shadow 0.2s',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = 'translateY(-2px)';
            e.currentTarget.style.boxShadow = '0 4px 12px rgba(0, 0, 0, 0.2)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = 'translateY(0)';
            e.currentTarget.style.boxShadow = 'none';
          }}
        >
          {/* Header */}
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span style={{ fontSize: '0.875rem', color: '#94a3b8' }}>
              {stat.label}
            </span>
            {stat.icon && <span style={{ fontSize: '1.25rem' }}>{stat.icon}</span>}
          </div>

          {/* Value */}
          <div
            style={{
              fontSize: '2rem',
              fontWeight: 700,
              color: stat.color || '#f1f5f9',
              lineHeight: 1,
            }}
          >
            {formatValue(stat)}
          </div>

          {/* Change indicator */}
          {stat.change !== undefined && (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '0.5rem',
                fontSize: '0.875rem',
              }}
            >
              <span
                style={{
                  color:
                    stat.changeType === 'increase'
                      ? '#10b981'
                      : stat.changeType === 'decrease'
                      ? '#ef4444'
                      : '#94a3b8',
                }}
              >
                {stat.changeType === 'increase' ? '↑' : stat.changeType === 'decrease' ? '↓' : '→'}{' '}
                {Math.abs(stat.change).toFixed(1)}%
              </span>
              <span style={{ color: '#94a3b8', fontSize: '0.75rem' }}>
                vs previous
              </span>
            </div>
          )}

          {/* Trend sparkline */}
          {stat.trend && stat.trend.length > 0 && (
            <div style={{ height: '40px', marginTop: '0.5rem' }}>
              <ResponsiveContainer width="100%" height="100%">
                <LineChart
                  data={stat.trend.map((value, i) => ({ value, index: i }))}
                  margin={{ top: 5, right: 0, left: 0, bottom: 5 }}
                >
                  <Line
                    type="monotone"
                    dataKey="value"
                    stroke={stat.color || '#3b82f6'}
                    strokeWidth={2}
                    dot={false}
                    isAnimationActive={false}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

export default StatisticsCard;
