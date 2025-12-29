import React from 'react';
import { LegendItem, ChartConfig } from '../types';
import { DEFAULT_CHART_CONFIG } from '../index';

interface ChartLegendProps {
  items: LegendItem[];
  config?: Partial<ChartConfig>;
  onToggle?: (name: string) => void;
  orientation?: 'horizontal' | 'vertical';
  position?: 'top' | 'right' | 'bottom' | 'left';
  className?: string;
  style?: React.CSSProperties;
}

/**
 * Interactive legend component for charts
 */
const ChartLegend: React.FC<ChartLegendProps> = ({
  items,
  config = {},
  onToggle,
  orientation = 'horizontal',
  position = 'bottom',
  className = '',
  style = {},
}) => {
  const mergedConfig = { ...DEFAULT_CHART_CONFIG, ...config };

  const containerStyle: React.CSSProperties = {
    display: 'flex',
    flexDirection: orientation === 'horizontal' ? 'row' : 'column',
    flexWrap: 'wrap',
    gap: '16px',
    padding: '12px 16px',
    justifyContent: position === 'bottom' || position === 'top' ? 'center' : 'flex-start',
    alignItems: 'center',
    borderTop: position === 'bottom' ? `1px solid ${mergedConfig.colors.grid}` : undefined,
    borderRight: position === 'left' ? `1px solid ${mergedConfig.colors.grid}` : undefined,
    borderBottom: position === 'top' ? `1px solid ${mergedConfig.colors.grid}` : undefined,
    borderLeft: position === 'right' ? `1px solid ${mergedConfig.colors.grid}` : undefined,
    ...style,
  };

  const renderSymbol = (item: LegendItem) => {
    const symbolStyle: React.CSSProperties = {
      display: 'inline-block',
      width: '16px',
      height: '16px',
      marginRight: '8px',
      backgroundColor: item.visible ? item.color : 'transparent',
      border: `2px solid ${item.color}`,
      transition: 'all 0.2s ease',
    };

    switch (item.chartType) {
      case 'line':
        return (
          <div
            style={{
              ...symbolStyle,
              backgroundColor: 'transparent',
              borderTop: `3px solid ${item.color}`,
              borderBottom: 'none',
              borderLeft: 'none',
              borderRight: 'none',
            }}
          />
        );
      case 'area':
        return (
          <div
            style={{
              ...symbolStyle,
              opacity: item.visible ? 0.6 : 0.3,
            }}
          />
        );
      case 'scatter':
        return (
          <div
            style={{
              ...symbolStyle,
              borderRadius: '50%',
            }}
          />
        );
      case 'bar':
        return (
          <div
            style={{
              ...symbolStyle,
              borderRadius: '2px',
            }}
          />
        );
      default:
        return (
          <div
            style={{
              ...symbolStyle,
              borderRadius: '2px',
            }}
          />
        );
    }
  };

  return (
    <div className={`accuscene-chart-legend ${className}`} style={containerStyle}>
      {items.map((item) => (
        <button
          key={item.name}
          onClick={() => onToggle?.(item.name)}
          style={{
            display: 'flex',
            alignItems: 'center',
            padding: '4px 8px',
            background: 'none',
            border: 'none',
            cursor: onToggle ? 'pointer' : 'default',
            opacity: item.visible ? 1 : 0.4,
            transition: 'opacity 0.2s ease',
            fontFamily: mergedConfig.font.family,
            fontSize: `${mergedConfig.font.size}px`,
            color: mergedConfig.colors.text,
          }}
          onMouseEnter={(e) => {
            if (onToggle) {
              e.currentTarget.style.backgroundColor = mergedConfig.colors.grid;
              e.currentTarget.style.borderRadius = '4px';
            }
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
          }}
          aria-label={`${item.visible ? 'Hide' : 'Show'} ${item.name}`}
          type="button"
        >
          {renderSymbol(item)}
          <span>{item.name}</span>
        </button>
      ))}
    </div>
  );
};

export default ChartLegend;
