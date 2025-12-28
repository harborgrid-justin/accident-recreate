/**
 * Timeline Component
 * Simulation timeline with playback controls
 */

import React, { useRef, useEffect } from 'react';
import { TimelineProps } from '../types';
import './Timeline.css';

export const Timeline: React.FC<TimelineProps> = ({
  duration,
  currentTime,
  events,
  playing,
  playbackSpeed,
  onTimeChange,
  onPlayingChange,
  onSpeedChange,
}) => {
  const timelineRef = useRef<HTMLDivElement>(null);
  const isDraggingRef = useRef(false);

  const handleMouseDown = (e: React.MouseEvent) => {
    isDraggingRef.current = true;
    handleTimelineClick(e);
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDraggingRef.current) return;
    handleTimelineClick(e as any);
  };

  const handleMouseUp = () => {
    isDraggingRef.current = false;
  };

  const handleTimelineClick = (e: React.MouseEvent | MouseEvent) => {
    if (!timelineRef.current) return;

    const rect = timelineRef.current.getBoundingClientRect();
    const x = (e as MouseEvent).clientX - rect.left;
    const percentage = Math.max(0, Math.min(1, x / rect.width));
    const newTime = percentage * duration;

    onTimeChange(newTime);
  };

  useEffect(() => {
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  const progress = (currentTime / duration) * 100;

  return (
    <div className="simulation-timeline">
      <div
        ref={timelineRef}
        className="timeline-track"
        onMouseDown={handleMouseDown}
      >
        <div className="timeline-progress" style={{ width: `${progress}%` }} />
        <div
          className="timeline-handle"
          style={{ left: `${progress}%` }}
        />

        {/* Event markers */}
        {events.map((event, index) => {
          const eventPosition = (event.timestamp / duration) * 100;
          return (
            <div
              key={index}
              className="timeline-event-marker"
              style={{
                left: `${eventPosition}%`,
                backgroundColor: getEventColor(event.type),
              }}
              title={`${event.type} at ${event.timestamp.toFixed(2)}s`}
            />
          );
        })}
      </div>

      <div className="timeline-labels">
        <span>0s</span>
        <span>{(duration / 2).toFixed(1)}s</span>
        <span>{duration.toFixed(1)}s</span>
      </div>
    </div>
  );
};

function getEventColor(eventType: string): string {
  switch (eventType) {
    case 'collision':
      return '#ff0000';
    case 'brake':
      return '#ffaa00';
    case 'acceleration':
      return '#00ff00';
    case 'swerve':
      return '#00aaff';
    default:
      return '#ffffff';
  }
}

export default Timeline;
