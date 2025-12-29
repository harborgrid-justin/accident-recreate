/**
 * AccuScene Enterprise v0.3.0 - Comparison Widget
 * Before/after comparison with slider interface
 */

import React, { useState } from 'react';
import { WidgetProps, AnalyticsData } from '../types';

const ComparisonWidget: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  const [sliderPosition, setSliderPosition] = useState(50);
  const [compareMode, setCompareMode] = useState<'slider' | 'side-by-side'>('slider');

  // Mock comparison data - in production, this would come from actual before/after states
  const beforeData = {
    title: 'Initial State',
    metrics: [
      { label: 'Total Energy', value: 150000, unit: 'J' },
      { label: 'Max Speed', value: 65, unit: 'mph' },
      { label: 'Impact Force', value: 45000, unit: 'N' },
      { label: 'Damage Area', value: 2.5, unit: 'm²' },
    ],
  };

  const afterData = {
    title: 'Final State',
    metrics: [
      { label: 'Total Energy', value: 25000, unit: 'J' },
      { label: 'Max Speed', value: 15, unit: 'mph' },
      { label: 'Impact Force', value: 12000, unit: 'N' },
      { label: 'Damage Area', value: 4.2, unit: 'm²' },
    ],
  };

  // Calculate differences
  const differences = beforeData.metrics.map((before, index) => {
    const after = afterData.metrics[index];
    const change = after.value - before.value;
    const changePercent = ((change / before.value) * 100).toFixed(1);
    return {
      label: before.label,
      before: before.value,
      after: after.value,
      change,
      changePercent: parseFloat(changePercent),
      unit: before.unit,
    };
  });

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
      {/* Mode Selector */}
      <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'center' }}>
        <button
          onClick={() => setCompareMode('slider')}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: compareMode === 'slider' ? '#3b82f6' : '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
          }}
        >
          Slider View
        </button>
        <button
          onClick={() => setCompareMode('side-by-side')}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: compareMode === 'side-by-side' ? '#3b82f6' : '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
          }}
        >
          Side by Side
        </button>
      </div>

      {/* Comparison View */}
      {compareMode === 'slider' ? (
        <div
          style={{
            flex: 1,
            position: 'relative',
            backgroundColor: '#1e293b',
            borderRadius: '4px',
            border: '1px solid #334155',
            overflow: 'hidden',
          }}
        >
          {/* Before image (left side) */}
          <div
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              backgroundColor: '#0f172a',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: '#3b82f6',
              fontSize: '2rem',
              fontWeight: 600,
            }}
          >
            BEFORE
          </div>

          {/* After image (clipped) */}
          <div
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              backgroundColor: '#1e293b',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: '#10b981',
              fontSize: '2rem',
              fontWeight: 600,
              clipPath: `inset(0 ${100 - sliderPosition}% 0 0)`,
            }}
          >
            AFTER
          </div>

          {/* Slider line */}
          <div
            style={{
              position: 'absolute',
              top: 0,
              bottom: 0,
              left: `${sliderPosition}%`,
              width: '4px',
              backgroundColor: '#f1f5f9',
              cursor: 'ew-resize',
              boxShadow: '0 0 10px rgba(0, 0, 0, 0.5)',
            }}
          >
            {/* Slider handle */}
            <div
              style={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%)',
                width: '40px',
                height: '40px',
                backgroundColor: '#f1f5f9',
                borderRadius: '50%',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                boxShadow: '0 2px 8px rgba(0, 0, 0, 0.3)',
              }}
            >
              ⇔
            </div>
          </div>

          {/* Slider control */}
          <input
            type="range"
            min="0"
            max="100"
            value={sliderPosition}
            onChange={(e) => setSliderPosition(parseInt(e.target.value))}
            style={{
              position: 'absolute',
              bottom: '1rem',
              left: '10%',
              width: '80%',
              accentColor: '#3b82f6',
            }}
          />
        </div>
      ) : (
        <div
          style={{
            flex: 1,
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: '1rem',
          }}
        >
          {/* Before Panel */}
          <div
            style={{
              backgroundColor: '#1e293b',
              borderRadius: '4px',
              border: '2px solid #3b82f6',
              padding: '1rem',
              overflow: 'auto',
            }}
          >
            <h3
              style={{
                margin: '0 0 1rem 0',
                color: '#3b82f6',
                fontSize: '1rem',
                fontWeight: 600,
              }}
            >
              {beforeData.title}
            </h3>
            {beforeData.metrics.map((metric, index) => (
              <div
                key={index}
                style={{
                  padding: '0.75rem',
                  backgroundColor: '#0f172a',
                  borderRadius: '4px',
                  marginBottom: '0.5rem',
                }}
              >
                <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                  {metric.label}
                </div>
                <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#f1f5f9' }}>
                  {metric.value.toFixed(0)} {metric.unit}
                </div>
              </div>
            ))}
          </div>

          {/* After Panel */}
          <div
            style={{
              backgroundColor: '#1e293b',
              borderRadius: '4px',
              border: '2px solid #10b981',
              padding: '1rem',
              overflow: 'auto',
            }}
          >
            <h3
              style={{
                margin: '0 0 1rem 0',
                color: '#10b981',
                fontSize: '1rem',
                fontWeight: 600,
              }}
            >
              {afterData.title}
            </h3>
            {afterData.metrics.map((metric, index) => (
              <div
                key={index}
                style={{
                  padding: '0.75rem',
                  backgroundColor: '#0f172a',
                  borderRadius: '4px',
                  marginBottom: '0.5rem',
                }}
              >
                <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                  {metric.label}
                </div>
                <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#f1f5f9' }}>
                  {metric.value.toFixed(0)} {metric.unit}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Difference Summary */}
      <div
        style={{
          backgroundColor: '#1e293b',
          borderRadius: '4px',
          border: '1px solid #334155',
          padding: '1rem',
        }}
      >
        <h4
          style={{
            margin: '0 0 0.75rem 0',
            color: '#f1f5f9',
            fontSize: '0.875rem',
            fontWeight: 600,
          }}
        >
          Changes
        </h4>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))', gap: '0.75rem' }}>
          {differences.map((diff, index) => (
            <div
              key={index}
              style={{
                display: 'flex',
                flexDirection: 'column',
                gap: '0.25rem',
              }}
            >
              <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                {diff.label}
              </div>
              <div
                style={{
                  fontSize: '1rem',
                  fontWeight: 600,
                  color:
                    diff.change > 0
                      ? '#ef4444'
                      : diff.change < 0
                      ? '#10b981'
                      : '#94a3b8',
                }}
              >
                {diff.change > 0 ? '+' : ''}
                {diff.change.toFixed(0)} {diff.unit}
              </div>
              <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                ({diff.changePercent > 0 ? '+' : ''}
                {diff.changePercent}%)
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default ComparisonWidget;
