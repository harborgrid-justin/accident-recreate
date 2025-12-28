import React, { useEffect, useRef, useState } from 'react';
import { TooltipData, ChartConfig } from '../types';
import { DEFAULT_CHART_CONFIG } from '../index';

interface ChartTooltipProps {
  data: TooltipData | null;
  config?: Partial<ChartConfig>;
  formatValue?: (value: number | string) => string;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * Rich tooltip component for charts
 */
const ChartTooltip: React.FC<ChartTooltipProps> = ({
  data,
  config = {},
  formatValue,
  className = '',
  style = {},
}) => {
  const tooltipRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ x: 0, y: 0 });

  const mergedConfig = { ...DEFAULT_CHART_CONFIG, ...config };

  useEffect(() => {
    if (!data || !tooltipRef.current) return;

    const tooltip = tooltipRef.current;
    const tooltipRect = tooltip.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    let x = data.x + 10;
    let y = data.y - 10;

    // Adjust position if tooltip goes off screen
    if (x + tooltipRect.width > viewportWidth) {
      x = data.x - tooltipRect.width - 10;
    }

    if (y + tooltipRect.height > viewportHeight) {
      y = data.y - tooltipRect.height - 10;
    }

    // Ensure tooltip doesn't go off the top or left
    x = Math.max(10, x);
    y = Math.max(10, y);

    setPosition({ x, y });
  }, [data]);

  if (!data) return null;

  const defaultFormatValue = (value: number | string): string => {
    if (typeof value === 'number') {
      return value.toLocaleString(undefined, {
        minimumFractionDigits: 0,
        maximumFractionDigits: 2,
      });
    }
    return String(value);
  };

  const formatFn = formatValue || defaultFormatValue;

  const tooltipStyle: React.CSSProperties = {
    position: 'fixed',
    left: `${position.x}px`,
    top: `${position.y}px`,
    pointerEvents: 'none',
    backgroundColor: 'rgba(255, 255, 255, 0.98)',
    border: `1px solid ${mergedConfig.colors.grid}`,
    borderRadius: '8px',
    padding: '12px',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
    fontFamily: mergedConfig.font.family,
    fontSize: `${mergedConfig.font.size}px`,
    color: mergedConfig.colors.text,
    zIndex: 1000,
    minWidth: '150px',
    maxWidth: '300px',
    animation: mergedConfig.animation.enabled ? 'fadeIn 0.15s ease-in' : undefined,
    ...style,
  };

  return (
    <>
      <div
        ref={tooltipRef}
        className={`accuscene-chart-tooltip ${className}`}
        style={tooltipStyle}
      >
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            marginBottom: '8px',
            paddingBottom: '8px',
            borderBottom: `1px solid ${mergedConfig.colors.grid}`,
          }}
        >
          <div
            style={{
              width: '12px',
              height: '12px',
              borderRadius: '50%',
              backgroundColor: data.color,
              marginRight: '8px',
            }}
          />
          <span style={{ fontWeight: 600, fontSize: '13px' }}>
            {data.series}
          </span>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
          {data.label && (
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: '16px' }}>
              <span style={{ color: mergedConfig.colors.axis }}>Label:</span>
              <span style={{ fontWeight: 500 }}>{data.label}</span>
            </div>
          )}

          <div style={{ display: 'flex', justifyContent: 'space-between', gap: '16px' }}>
            <span style={{ color: mergedConfig.colors.axis }}>Value:</span>
            <span style={{ fontWeight: 600, color: data.color }}>
              {formatFn(data.value)}
            </span>
          </div>
        </div>
      </div>

      <style>{`
        @keyframes fadeIn {
          from {
            opacity: 0;
            transform: translateY(-4px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
      `}</style>
    </>
  );
};

export default ChartTooltip;
