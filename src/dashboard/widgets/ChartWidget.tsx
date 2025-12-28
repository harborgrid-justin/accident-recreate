/**
 * Chart Widget Component
 *
 * Displays interactive charts with support for various chart types
 * Uses D3.js for rendering (placeholder implementation)
 */

import React, { useMemo, useRef, useEffect, useState } from 'react';
import { useWidget } from '../hooks/useDashboard';
import { ChartType, DataSeries, ChartConfig as ChartConfigType } from '../types';

/**
 * Chart widget props
 */
interface ChartWidgetProps {
  widgetId: string;
}

/**
 * Simple SVG chart renderer (placeholder - in production use visx/d3)
 */
interface SimpleChartProps {
  config: ChartConfigType;
  width: number;
  height: number;
}

const SimpleChart: React.FC<SimpleChartProps> = ({ config, width, height }) => {
  const { chartType, series, xAxis, yAxis } = config;

  const margin = { top: 20, right: 30, bottom: 40, left: 50 };
  const chartWidth = width - margin.left - margin.right;
  const chartHeight = height - margin.top - margin.bottom;

  const allDataPoints = useMemo(() => {
    return series.flatMap((s) => s.data);
  }, [series]);

  const yExtent = useMemo(() => {
    const values = allDataPoints.map((d) => d.y);
    return {
      min: Math.min(...values, 0),
      max: Math.max(...values),
    };
  }, [allDataPoints]);

  const yScale = (value: number) => {
    const range = yExtent.max - yExtent.min;
    if (range === 0) return chartHeight / 2;
    return chartHeight - ((value - yExtent.min) / range) * chartHeight;
  };

  const xScale = (index: number, total: number) => {
    return (index / Math.max(1, total - 1)) * chartWidth;
  };

  const renderLineChart = (series: DataSeries, colorIndex: number) => {
    if (!series.visible || series.data.length === 0) return null;

    const color = series.color || config.colorPalette[colorIndex % config.colorPalette.length];
    const points = series.data
      .map((d, i) => `${xScale(i, series.data.length)},${yScale(d.y)}`)
      .join(' ');

    return (
      <g key={series.name}>
        <polyline
          points={points}
          fill="none"
          stroke={color}
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        {series.data.map((d, i) => (
          <circle
            key={i}
            cx={xScale(i, series.data.length)}
            cy={yScale(d.y)}
            r={4}
            fill={color}
          />
        ))}
      </g>
    );
  };

  const renderBarChart = (allSeries: DataSeries[]) => {
    const visibleSeries = allSeries.filter((s) => s.visible);
    if (visibleSeries.length === 0) return null;

    const maxDataPoints = Math.max(...visibleSeries.map((s) => s.data.length));
    const barGroupWidth = chartWidth / maxDataPoints;
    const barWidth = barGroupWidth / visibleSeries.length - 4;

    return (
      <g>
        {visibleSeries.map((series, seriesIndex) => {
          const color = series.color || config.colorPalette[seriesIndex % config.colorPalette.length];

          return series.data.map((d, dataIndex) => {
            const x = dataIndex * barGroupWidth + seriesIndex * (barWidth + 4);
            const barHeight = Math.abs(yScale(d.y) - yScale(0));
            const y = Math.min(yScale(d.y), yScale(0));

            return (
              <rect
                key={`${series.name}-${dataIndex}`}
                x={x}
                y={y}
                width={barWidth}
                height={barHeight}
                fill={color}
                rx={2}
              />
            );
          });
        })}
      </g>
    );
  };

  return (
    <svg width={width} height={height}>
      <g transform={`translate(${margin.left},${margin.top})`}>
        {/* Y-axis grid */}
        {yAxis.showGrid && (
          <g opacity={0.1}>
            {[0, 0.25, 0.5, 0.75, 1].map((percent) => {
              const y = chartHeight * (1 - percent);
              return (
                <line
                  key={percent}
                  x1={0}
                  y1={y}
                  x2={chartWidth}
                  y2={y}
                  stroke="currentColor"
                  strokeWidth={1}
                />
              );
            })}
          </g>
        )}

        {/* Chart content */}
        {chartType === ChartType.Line && series.map((s, i) => renderLineChart(s, i))}
        {chartType === ChartType.Bar && renderBarChart(series)}
        {(chartType === ChartType.Area || chartType === ChartType.Scatter) && (
          <text x={chartWidth / 2} y={chartHeight / 2} textAnchor="middle" opacity={0.3}>
            {chartType} chart (placeholder)
          </text>
        )}

        {/* X-axis */}
        {xAxis.showAxis && (
          <line
            x1={0}
            y1={chartHeight}
            x2={chartWidth}
            y2={chartHeight}
            stroke="currentColor"
            strokeWidth={1}
            opacity={0.2}
          />
        )}

        {/* Y-axis */}
        {yAxis.showAxis && (
          <line
            x1={0}
            y1={0}
            x2={0}
            y2={chartHeight}
            stroke="currentColor"
            strokeWidth={1}
            opacity={0.2}
          />
        )}

        {/* Y-axis labels */}
        {yAxis.showAxis && (
          <g>
            {[0, 0.25, 0.5, 0.75, 1].map((percent) => {
              const value = yExtent.min + (yExtent.max - yExtent.min) * percent;
              const y = chartHeight * (1 - percent);
              return (
                <text
                  key={percent}
                  x={-10}
                  y={y}
                  textAnchor="end"
                  fontSize={10}
                  dominantBaseline="middle"
                  opacity={0.6}
                >
                  {value.toFixed(0)}
                </text>
              );
            })}
          </g>
        )}
      </g>

      {/* Axis labels */}
      {xAxis.label && (
        <text
          x={width / 2}
          y={height - 5}
          textAnchor="middle"
          fontSize={12}
          opacity={0.7}
        >
          {xAxis.label}
        </text>
      )}
      {yAxis.label && (
        <text
          transform={`translate(15, ${height / 2}) rotate(-90)`}
          textAnchor="middle"
          fontSize={12}
          opacity={0.7}
        >
          {yAxis.label}
        </text>
      )}
    </svg>
  );
};

/**
 * Chart legend component
 */
interface ChartLegendProps {
  series: DataSeries[];
  colorPalette: string[];
  onToggleSeries?: (seriesName: string) => void;
}

const ChartLegend: React.FC<ChartLegendProps> = ({ series, colorPalette, onToggleSeries }) => {
  const legendStyle: React.CSSProperties = {
    display: 'flex',
    flexWrap: 'wrap',
    gap: 16,
    padding: '12px 0',
    justifyContent: 'center',
  };

  const legendItemStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    fontSize: 12,
    cursor: onToggleSeries ? 'pointer' : 'default',
  };

  return (
    <div style={legendStyle}>
      {series.map((s, index) => {
        const color = s.color || colorPalette[index % colorPalette.length];
        return (
          <div
            key={s.name}
            style={legendItemStyle}
            onClick={() => onToggleSeries?.(s.name)}
          >
            <div
              style={{
                width: 12,
                height: 12,
                backgroundColor: color,
                borderRadius: 2,
                opacity: s.visible ? 1 : 0.3,
              }}
            />
            <span style={{ opacity: s.visible ? 1 : 0.5 }}>{s.name}</span>
          </div>
        );
      })}
    </div>
  );
};

/**
 * Chart Widget Component
 */
const ChartWidget: React.FC<ChartWidgetProps> = ({ widgetId }) => {
  const { widget, isLoading, error } = useWidget(widgetId);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 400, height: 300 });

  const chartConfig = useMemo(() => {
    if (!widget?.data?.data) return null;
    return widget.data.data as ChartConfigType;
  }, [widget]);

  useEffect(() => {
    if (!containerRef.current) return;

    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) {
        const { width, height } = entry.contentRect;
        setDimensions({ width, height: Math.max(height - 100, 200) });
      }
    });

    resizeObserver.observe(containerRef.current);
    return () => resizeObserver.disconnect();
  }, []);

  const containerStyle: React.CSSProperties = {
    width: '100%',
    height: '100%',
    backgroundColor: widget?.config.display.backgroundColor || '#ffffff',
    borderRadius: 8,
    padding: widget?.config.display.padding || 16,
    border: widget?.config.display.showBorder ? '1px solid #e0e0e0' : 'none',
    boxSizing: 'border-box',
    overflow: 'hidden',
    display: 'flex',
    flexDirection: 'column',
  };

  const headerStyle: React.CSSProperties = {
    marginBottom: 12,
    paddingBottom: widget?.config.display.showTitle ? 12 : 0,
    borderBottom: widget?.config.display.showTitle ? '1px solid #e0e0e0' : 'none',
  };

  const titleStyle: React.CSSProperties = {
    fontSize: 18,
    fontWeight: 600,
    margin: 0,
    color: widget?.config.display.textColor || 'inherit',
  };

  if (isLoading) {
    return (
      <div style={{ ...containerStyle, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <p>Loading chart...</p>
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

  if (!chartConfig || !chartConfig.series || chartConfig.series.length === 0) {
    return (
      <div style={{ ...containerStyle, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <p style={{ opacity: 0.5 }}>No chart data available</p>
      </div>
    );
  }

  return (
    <div ref={containerRef} style={containerStyle}>
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

      <div style={{ flex: 1, minHeight: 0, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <SimpleChart
          config={chartConfig}
          width={dimensions.width - 32}
          height={dimensions.height}
        />
      </div>

      {chartConfig.legend.show && (
        <ChartLegend
          series={chartConfig.series}
          colorPalette={chartConfig.colorPalette}
        />
      )}
    </div>
  );
};

export default ChartWidget;
