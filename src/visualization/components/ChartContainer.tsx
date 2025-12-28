import React, { useRef, useState, useEffect } from 'react';
import { ChartConfig } from '../types';
import { DEFAULT_CHART_CONFIG } from '../index';

interface ChartContainerProps {
  children: React.ReactNode;
  config?: Partial<ChartConfig>;
  className?: string;
  style?: React.CSSProperties;
  title?: string;
  subtitle?: string;
  loading?: boolean;
  error?: Error | null;
}

/**
 * Responsive container for charts with loading and error states
 */
const ChartContainer: React.FC<ChartContainerProps> = ({
  children,
  config = {},
  className = '',
  style = {},
  title,
  subtitle,
  loading = false,
  error = null,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({
    width: config.width || DEFAULT_CHART_CONFIG.width,
    height: config.height || DEFAULT_CHART_CONFIG.height,
  });

  const mergedConfig = { ...DEFAULT_CHART_CONFIG, ...config };
  const isResponsive = mergedConfig.responsive !== false;

  useEffect(() => {
    if (!containerRef.current || !isResponsive) return;

    const updateDimensions = () => {
      if (!containerRef.current) return;
      const { width } = containerRef.current.getBoundingClientRect();

      setDimensions({
        width,
        height: config.height || DEFAULT_CHART_CONFIG.height,
      });
    };

    updateDimensions();

    const resizeObserver = new ResizeObserver(updateDimensions);
    resizeObserver.observe(containerRef.current);

    return () => resizeObserver.disconnect();
  }, [config.height, isResponsive]);

  const containerStyle: React.CSSProperties = {
    position: 'relative',
    width: isResponsive ? '100%' : dimensions.width,
    height: dimensions.height,
    backgroundColor: mergedConfig.colors.background,
    fontFamily: mergedConfig.font.family,
    ...style,
  };

  return (
    <div
      ref={containerRef}
      className={`accuscene-chart-container ${className}`}
      style={containerStyle}
    >
      {title && (
        <div
          style={{
            padding: '16px 16px 8px',
            borderBottom: `1px solid ${mergedConfig.colors.grid}`,
          }}
        >
          <h3
            style={{
              margin: 0,
              fontSize: '20px',
              fontWeight: 600,
              color: mergedConfig.colors.text,
            }}
          >
            {title}
          </h3>
          {subtitle && (
            <p
              style={{
                margin: '4px 0 0',
                fontSize: '14px',
                color: mergedConfig.colors.axis,
              }}
            >
              {subtitle}
            </p>
          )}
        </div>
      )}

      {loading && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'rgba(255, 255, 255, 0.8)',
            zIndex: 10,
          }}
        >
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: '12px',
            }}
          >
            <div
              style={{
                width: '40px',
                height: '40px',
                border: `4px solid ${mergedConfig.colors.grid}`,
                borderTopColor: mergedConfig.colors.primary[0],
                borderRadius: '50%',
                animation: 'spin 1s linear infinite',
              }}
            />
            <span style={{ color: mergedConfig.colors.text }}>Loading...</span>
          </div>
        </div>
      )}

      {error && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'rgba(255, 255, 255, 0.95)',
            zIndex: 10,
          }}
        >
          <div
            style={{
              padding: '24px',
              maxWidth: '400px',
              textAlign: 'center',
            }}
          >
            <div
              style={{
                fontSize: '48px',
                marginBottom: '16px',
              }}
            >
              ⚠️
            </div>
            <h4
              style={{
                margin: '0 0 8px',
                color: '#DC2626',
                fontSize: '18px',
                fontWeight: 600,
              }}
            >
              Error Loading Chart
            </h4>
            <p
              style={{
                margin: 0,
                color: mergedConfig.colors.axis,
                fontSize: '14px',
              }}
            >
              {error.message}
            </p>
          </div>
        </div>
      )}

      {!loading && !error && children}

      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
};

export default ChartContainer;
