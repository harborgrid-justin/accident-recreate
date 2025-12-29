/**
 * AccuScene Enterprise v0.3.0 - Timeline Widget
 * Interactive event timeline with scrubbing and playback controls
 */

import React, { useState, useMemo, useCallback } from 'react';
import { WidgetProps, AnalyticsData, TimelineEvent } from '../types';

const TimelineWidget: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  const [currentTime, setCurrentTime] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [playbackSpeed, setPlaybackSpeed] = useState(1);

  // Get all timeline events
  const events = useMemo(() => {
    return data?.timeline || [];
  }, [data]);

  // Calculate timeline range
  const timeRange = useMemo(() => {
    if (events.length === 0) return { start: 0, end: 10 };

    return {
      start: Math.min(...events.map((e) => e.timestamp)),
      end: Math.max(...events.map((e) => e.timestamp)),
    };
  }, [events]);

  // Get events at current time
  const currentEvents = useMemo(() => {
    return events.filter(
      (e) => Math.abs(e.timestamp - currentTime) < 0.1
    );
  }, [events, currentTime]);

  // Playback control
  const handlePlayPause = useCallback(() => {
    setIsPlaying(!isPlaying);
  }, [isPlaying]);

  // Get event color
  const getEventColor = (type: string, severity?: string) => {
    if (severity === 'critical') return '#ef4444';
    if (severity === 'high') return '#f59e0b';

    switch (type) {
      case 'impact':
        return '#ef4444';
      case 'vehicle-action':
        return '#3b82f6';
      case 'environment':
        return '#10b981';
      case 'annotation':
        return '#8b5cf6';
      default:
        return '#64748b';
    }
  };

  if (events.length === 0) {
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
        No timeline events available
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
      {/* Current Event Display */}
      <div
        style={{
          backgroundColor: '#1e293b',
          padding: '1rem',
          borderRadius: '4px',
          border: '1px solid #334155',
          minHeight: '80px',
        }}
      >
        {currentEvents.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
            {currentEvents.map((event) => (
              <div key={event.id}>
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '0.5rem',
                    marginBottom: '0.25rem',
                  }}
                >
                  <div
                    style={{
                      width: '12px',
                      height: '12px',
                      borderRadius: '50%',
                      backgroundColor: getEventColor(event.type, event.severity),
                    }}
                  />
                  <span style={{ fontWeight: 600, color: '#f1f5f9' }}>
                    {event.title}
                  </span>
                  <span style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                    @ {event.timestamp.toFixed(2)}s
                  </span>
                </div>
                {event.description && (
                  <div style={{ fontSize: '0.875rem', color: '#cbd5e1', marginLeft: '1.75rem' }}>
                    {event.description}
                  </div>
                )}
              </div>
            ))}
          </div>
        ) : (
          <div style={{ color: '#94a3b8', textAlign: 'center' }}>
            No events at current time
          </div>
        )}
      </div>

      {/* Timeline Visualization */}
      <div
        style={{
          flex: 1,
          position: 'relative',
          backgroundColor: '#1e293b',
          borderRadius: '4px',
          border: '1px solid #334155',
          padding: '1rem',
          overflow: 'auto',
        }}
      >
        <svg width="100%" height="100%" style={{ minHeight: '200px' }}>
          {/* Timeline axis */}
          <line
            x1="50"
            y1="100"
            x2="95%"
            y2="100"
            stroke="#475569"
            strokeWidth="2"
          />

          {/* Event markers */}
          {events.map((event, index) => {
            const x =
              50 +
              ((event.timestamp - timeRange.start) /
                (timeRange.end - timeRange.start)) *
                (window.innerWidth * 0.85);
            const y = 100 - (index % 3) * 40;

            return (
              <g key={event.id}>
                {/* Event line */}
                <line
                  x1={x}
                  y1={y}
                  x2={x}
                  y2="100"
                  stroke={getEventColor(event.type, event.severity)}
                  strokeWidth="2"
                  strokeDasharray="3,3"
                />
                {/* Event marker */}
                <circle
                  cx={x}
                  cy={y}
                  r="6"
                  fill={getEventColor(event.type, event.severity)}
                  stroke="#f1f5f9"
                  strokeWidth="2"
                />
                {/* Event label */}
                <text
                  x={x}
                  y={y - 15}
                  fill="#f1f5f9"
                  fontSize="10"
                  textAnchor="middle"
                >
                  {event.title}
                </text>
              </g>
            );
          })}

          {/* Current time indicator */}
          <line
            x1={
              50 +
              ((currentTime - timeRange.start) /
                (timeRange.end - timeRange.start)) *
                (window.innerWidth * 0.85)
            }
            y1="0"
            x2={
              50 +
              ((currentTime - timeRange.start) /
                (timeRange.end - timeRange.start)) *
                (window.innerWidth * 0.85)
            }
            y2="150"
            stroke="#3b82f6"
            strokeWidth="2"
          />
        </svg>
      </div>

      {/* Timeline Scrubber */}
      <div>
        <input
          type="range"
          min={timeRange.start}
          max={timeRange.end}
          step="0.01"
          value={currentTime}
          onChange={(e) => setCurrentTime(parseFloat(e.target.value))}
          style={{
            width: '100%',
            accentColor: '#3b82f6',
          }}
        />
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            fontSize: '0.75rem',
            color: '#94a3b8',
            marginTop: '0.25rem',
          }}
        >
          <span>{timeRange.start.toFixed(2)}s</span>
          <span style={{ fontWeight: 600, color: '#3b82f6' }}>
            {currentTime.toFixed(2)}s
          </span>
          <span>{timeRange.end.toFixed(2)}s</span>
        </div>
      </div>

      {/* Playback Controls */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          gap: '0.5rem',
        }}
      >
        <button
          onClick={() => setCurrentTime(timeRange.start)}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
          }}
        >
          ⏮️ Start
        </button>
        <button
          onClick={handlePlayPause}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: '#3b82f6',
            color: '#f1f5f9',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
          }}
        >
          {isPlaying ? '⏸️ Pause' : '▶️ Play'}
        </button>
        <button
          onClick={() => setCurrentTime(timeRange.end)}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
          }}
        >
          ⏭️ End
        </button>
        <select
          value={playbackSpeed}
          onChange={(e) => setPlaybackSpeed(parseFloat(e.target.value))}
          style={{
            padding: '0.5rem',
            backgroundColor: '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
            borderRadius: '4px',
            fontSize: '0.875rem',
          }}
        >
          <option value="0.5">0.5x</option>
          <option value="1">1x</option>
          <option value="2">2x</option>
          <option value="4">4x</option>
        </select>
      </div>
    </div>
  );
};

export default TimelineWidget;
