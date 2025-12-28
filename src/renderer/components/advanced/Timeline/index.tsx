/**
 * Timeline Component
 * Timeline visualization with events and keyframes
 */

import React, { useRef, useState } from 'react';
import { TimelineComponentProps } from '../types';
import './Timeline.css';

export const Timeline: React.FC<TimelineComponentProps> = ({
  events,
  currentTime,
  duration,
  onTimeChange,
  onEventClick,
  showKeyframes = true,
  editable = false,
}) => {
  const timelineRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);

  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    updateTime(e);
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging) return;
    updateTime(e as any);
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  const updateTime = (e: MouseEvent | React.MouseEvent) => {
    if (!timelineRef.current) return;

    const rect = timelineRef.current.getBoundingClientRect();
    const x = (e as MouseEvent).clientX - rect.left;
    const percentage = Math.max(0, Math.min(1, x / rect.width));
    const newTime = percentage * duration;

    onTimeChange(newTime);
  };

  React.useEffect(() => {
    if (isDragging) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);

      return () => {
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [isDragging]);

  const progress = (currentTime / duration) * 100;

  return (
    <div className="timeline-component">
      <div className="timeline-header">
        <span className="timeline-current-time">{currentTime.toFixed(2)}s</span>
        <span className="timeline-duration">/ {duration.toFixed(2)}s</span>
      </div>

      <div
        ref={timelineRef}
        className="timeline-main"
        onMouseDown={handleMouseDown}
      >
        {/* Background track */}
        <div className="timeline-bg" />

        {/* Progress indicator */}
        <div
          className="timeline-progress-line"
          style={{ left: `${progress}%` }}
        />

        {/* Event markers */}
        {events.map(event => {
          const eventPos = (event.timestamp / duration) * 100;
          return (
            <div
              key={event.id}
              className="timeline-event"
              style={{
                left: `${eventPos}%`,
                backgroundColor: event.color || '#0088ff',
              }}
              onClick={(e) => {
                e.stopPropagation();
                onEventClick?.(event);
              }}
              title={event.title}
            >
              <div className="event-tooltip">
                <strong>{event.title}</strong>
                {event.description && <p>{event.description}</p>}
                <small>{event.timestamp.toFixed(2)}s</small>
              </div>
            </div>
          );
        })}

        {/* Time markers */}
        {Array.from({ length: 11 }, (_, i) => {
          const timePos = (i / 10) * 100;
          const time = (i / 10) * duration;
          return (
            <div
              key={i}
              className="timeline-marker"
              style={{ left: `${timePos}%` }}
            >
              <div className="marker-label">{time.toFixed(1)}s</div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default Timeline;
