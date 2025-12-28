/**
 * StatCard Component - Card displaying statistics
 */

import React from 'react';

export interface StatCardProps {
  title: string;
  value: string | number;
  icon?: string;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  color?: 'blue' | 'green' | 'orange' | 'red' | 'purple';
  onClick?: () => void;
}

export const StatCard: React.FC<StatCardProps> = ({
  title,
  value,
  icon,
  trend,
  color = 'blue',
  onClick,
}) => {
  return (
    <div
      className={`stat-card stat-card-${color} ${onClick ? 'stat-card-clickable' : ''}`}
      onClick={onClick}
    >
      <div className="stat-card-content">
        <div className="stat-card-header">
          <span className="stat-card-title">{title}</span>
          {icon && <span className="stat-card-icon">{icon}</span>}
        </div>

        <div className="stat-card-value">{value}</div>

        {trend && (
          <div className="stat-card-trend">
            <span className={`stat-card-trend-value ${trend.isPositive ? 'positive' : 'negative'}`}>
              {trend.isPositive ? '▲' : '▼'} {Math.abs(trend.value)}%
            </span>
            <span className="stat-card-trend-label">vs last month</span>
          </div>
        )}
      </div>
    </div>
  );
};

export default StatCard;
