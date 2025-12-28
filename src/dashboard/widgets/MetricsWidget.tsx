/**
 * Metrics Widget Component
 *
 * Displays key performance indicators and metrics with trend indicators
 */

import React, { useMemo } from 'react';
import { useWidget } from '../hooks/useDashboard';
import {
  MetricValue,
  MetricsLayout,
  TrendDirection,
  MetricFormat,
} from '../types';

/**
 * Metrics widget props
 */
interface MetricsWidgetProps {
  widgetId: string;
}

/**
 * Format metric value
 */
function formatMetricValue(metric: MetricValue): string {
  const { value, format } = metric;

  switch (format) {
    case MetricFormat.Number:
      if (Math.abs(value) >= 1_000_000) {
        return `${(value / 1_000_000).toFixed(2)}M`;
      } else if (Math.abs(value) >= 1_000) {
        return `${(value / 1_000).toFixed(2)}K`;
      }
      return value.toFixed(2);

    case MetricFormat.Percentage:
      return `${value.toFixed(1)}%`;

    case MetricFormat.Currency:
      return `$${value.toFixed(2)}`;

    case MetricFormat.Duration:
      if (value >= 3600) {
        return `${(value / 3600).toFixed(1)}h`;
      } else if (value >= 60) {
        return `${(value / 60).toFixed(1)}m`;
      }
      return `${value.toFixed(0)}s`;

    case MetricFormat.Bytes:
      if (value >= 1_073_741_824) {
        return `${(value / 1_073_741_824).toFixed(2)} GB`;
      } else if (value >= 1_048_576) {
        return `${(value / 1_048_576).toFixed(2)} MB`;
      } else if (value >= 1024) {
        return `${(value / 1024).toFixed(2)} KB`;
      }
      return `${value.toFixed(0)} B`;

    default:
      return value.toString();
  }
}

/**
 * Trend icon component
 */
const TrendIcon: React.FC<{ direction: TrendDirection }> = ({ direction }) => {
  const style: React.CSSProperties = {
    width: 16,
    height: 16,
    display: 'inline-block',
    marginLeft: 8,
  };

  if (direction === TrendDirection.Up) {
    return (
      <svg style={style} fill="currentColor" viewBox="0 0 24 24">
        <path d="M7 14l5-5 5 5z" />
      </svg>
    );
  } else if (direction === TrendDirection.Down) {
    return (
      <svg style={style} fill="currentColor" viewBox="0 0 24 24">
        <path d="M7 10l5 5 5-5z" />
      </svg>
    );
  }

  return (
    <svg style={style} fill="currentColor" viewBox="0 0 24 24">
      <path d="M19 12H5" />
    </svg>
  );
};

/**
 * Single metric component
 */
interface MetricCardProps {
  metric: MetricValue;
  showTrend: boolean;
}

const MetricCard: React.FC<MetricCardProps> = ({ metric, showTrend }) => {
  const trendColor = useMemo(() => {
    if (!metric.trend) return 'inherit';
    switch (metric.trend) {
      case TrendDirection.Up:
        return '#4caf50';
      case TrendDirection.Down:
        return '#f44336';
      default:
        return '#999';
    }
  }, [metric.trend]);

  const cardStyle: React.CSSProperties = {
    backgroundColor: metric.color || '#ffffff',
    borderRadius: 8,
    padding: 16,
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
    justifyContent: 'space-between',
  };

  const labelStyle: React.CSSProperties = {
    fontSize: 12,
    fontWeight: 500,
    textTransform: 'uppercase',
    opacity: 0.7,
    marginBottom: 8,
  };

  const valueStyle: React.CSSProperties = {
    fontSize: 32,
    fontWeight: 700,
    lineHeight: 1,
    marginBottom: 8,
  };

  const trendStyle: React.CSSProperties = {
    fontSize: 14,
    color: trendColor,
    display: 'flex',
    alignItems: 'center',
  };

  const targetStyle: React.CSSProperties = {
    fontSize: 12,
    opacity: 0.6,
    marginTop: 8,
  };

  return (
    <div style={cardStyle}>
      <div>
        <div style={labelStyle}>{metric.label}</div>
        <div style={valueStyle}>
          {formatMetricValue(metric)}
          {metric.unit && <span style={{ fontSize: 16, marginLeft: 4 }}>{metric.unit}</span>}
        </div>
      </div>

      {showTrend && metric.trend && metric.changePercent !== undefined && (
        <div style={trendStyle}>
          <TrendIcon direction={metric.trend} />
          <span style={{ marginLeft: 4 }}>
            {metric.changePercent > 0 ? '+' : ''}
            {metric.changePercent.toFixed(1)}%
          </span>
        </div>
      )}

      {metric.target !== undefined && (
        <div style={targetStyle}>
          Target: {formatMetricValue({ ...metric, value: metric.target })}
        </div>
      )}
    </div>
  );
};

/**
 * Metrics Widget Component
 */
const MetricsWidget: React.FC<MetricsWidgetProps> = ({ widgetId }) => {
  const { widget, isLoading, error } = useWidget(widgetId);

  const metricsData = useMemo(() => {
    if (!widget?.data?.data) return null;
    return widget.data.data as {
      metrics: MetricValue[];
      layout: MetricsLayout;
      showTrend: boolean;
    };
  }, [widget]);

  const containerStyle: React.CSSProperties = {
    width: '100%',
    height: '100%',
    backgroundColor: widget?.config.display.backgroundColor || '#ffffff',
    borderRadius: 8,
    padding: widget?.config.display.padding || 16,
    border: widget?.config.display.showBorder ? '1px solid #e0e0e0' : 'none',
    boxSizing: 'border-box',
    overflow: 'auto',
  };

  const headerStyle: React.CSSProperties = {
    marginBottom: 16,
    paddingBottom: widget?.config.display.showTitle ? 12 : 0,
    borderBottom: widget?.config.display.showTitle ? '1px solid #e0e0e0' : 'none',
  };

  const titleStyle: React.CSSProperties = {
    fontSize: 18,
    fontWeight: 600,
    margin: 0,
    color: widget?.config.display.textColor || 'inherit',
  };

  const gridStyle = useMemo<React.CSSProperties>(() => {
    if (!metricsData) return {};

    const layout = metricsData.layout;

    switch (layout) {
      case MetricsLayout.Single:
        return {
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
        };
      case MetricsLayout.Grid:
        return {
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
          gap: 16,
        };
      case MetricsLayout.Horizontal:
        return {
          display: 'flex',
          gap: 16,
          overflowX: 'auto',
        };
      case MetricsLayout.Vertical:
        return {
          display: 'flex',
          flexDirection: 'column',
          gap: 16,
        };
      default:
        return {
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
          gap: 16,
        };
    }
  }, [metricsData]);

  if (isLoading) {
    return (
      <div style={{ ...containerStyle, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <p>Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ ...containerStyle, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <p style={{ color: '#f44336' }}>Error: {error}</p>
      </div>
    );
  }

  if (!metricsData || !metricsData.metrics || metricsData.metrics.length === 0) {
    return (
      <div style={{ ...containerStyle, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <p style={{ opacity: 0.5 }}>No metrics data available</p>
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      {widget?.config.display.showTitle && (
        <div style={headerStyle}>
          <h3 style={titleStyle}>{widget.config.metadata.title}</h3>
          {widget.config.metadata.description && (
            <p style={{ margin: '4px 0 0', fontSize: 12, opacity: 0.6 }}>
              {widget.config.metadata.description}
            </p>
          )}
        </div>
      )}

      <div style={gridStyle}>
        {metricsData.metrics.map((metric, index) => (
          <MetricCard
            key={metric.label || index}
            metric={metric}
            showTrend={metricsData.showTrend}
          />
        ))}
      </div>
    </div>
  );
};

export default MetricsWidget;
