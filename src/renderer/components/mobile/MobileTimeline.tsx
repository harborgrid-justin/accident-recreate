/**
 * AccuScene Enterprise v0.3.0
 * Mobile Timeline Component
 *
 * Touch-friendly timeline scrubber for mobile playback
 */

import React, { useState, useRef, useCallback, CSSProperties } from 'react';
import { TimelineProps, TimelineMarker } from './types';
import { HapticFeedback } from './HapticFeedback';

/**
 * Touch-optimized timeline scrubber
 * Supports dragging, markers, and playback controls
 *
 * @example
 * ```tsx
 * <MobileTimeline
 *   duration={120}
 *   currentTime={45}
 *   markers={keyframes}
 *   onSeek={(time) => setCurrentTime(time)}
 *   playing={isPlaying}
 *   onPlay={() => setPlaying(true)}
 *   onPause={() => setPlaying(false)}
 * />
 * ```
 */
export const MobileTimeline: React.FC<TimelineProps> = ({
  duration,
  currentTime,
  markers = [],
  onSeek,
  onPlay,
  onPause,
  playing = false,
  fps = 30,
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const [previewTime, setPreviewTime] = useState<number | null>(null);
  const timelineRef = useRef<HTMLDivElement>(null);

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const getTimeFromPosition = useCallback(
    (clientX: number): number => {
      if (!timelineRef.current) return 0;

      const rect = timelineRef.current.getBoundingClientRect();
      const percent = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
      return percent * duration;
    },
    [duration]
  );

  const handleTouchStart = (e: React.TouchEvent) => {
    const touch = e.touches[0];
    const time = getTimeFromPosition(touch.clientX);

    setIsDragging(true);
    setPreviewTime(time);
    onSeek(time);
    HapticFeedback.light();
  };

  const handleTouchMove = (e: React.TouchEvent) => {
    if (!isDragging) return;

    const touch = e.touches[0];
    const time = getTimeFromPosition(touch.clientX);

    setPreviewTime(time);
    onSeek(time);

    // Haptic feedback at markers
    markers.forEach((marker) => {
      if (Math.abs(time - marker.time) < 0.5) {
        HapticFeedback.snap();
      }
    });
  };

  const handleTouchEnd = () => {
    setIsDragging(false);
    setPreviewTime(null);
    HapticFeedback.light();
  };

  const handlePlayPause = () => {
    if (playing) {
      onPause?.();
      HapticFeedback.medium();
    } else {
      onPlay?.();
      HapticFeedback.medium();
    }
  };

  const progressPercent = (currentTime / duration) * 100;

  const containerStyles: CSSProperties = {
    width: '100%',
    padding: '1rem',
    backgroundColor: '#ffffff',
    borderRadius: '12px',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)',
  };

  const controlsStyles: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
    marginBottom: '1rem',
  };

  const playButtonStyles: CSSProperties = {
    width: '44px',
    height: '44px',
    borderRadius: '50%',
    border: 'none',
    backgroundColor: '#007AFF',
    color: '#ffffff',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    cursor: 'pointer',
    fontSize: '1.25rem',
    transition: 'transform 0.2s ease',
    flexShrink: 0,
  };

  const timeDisplayStyles: CSSProperties = {
    fontSize: '0.875rem',
    fontWeight: 500,
    color: '#000000',
    flex: 1,
    textAlign: 'right',
  };

  const trackContainerStyles: CSSProperties = {
    position: 'relative',
    width: '100%',
    height: '44px',
    display: 'flex',
    alignItems: 'center',
    touchAction: 'none',
  };

  const trackStyles: CSSProperties = {
    position: 'relative',
    width: '100%',
    height: '4px',
    backgroundColor: '#e0e0e0',
    borderRadius: '2px',
    cursor: 'pointer',
  };

  const progressStyles: CSSProperties = {
    position: 'absolute',
    top: 0,
    left: 0,
    height: '100%',
    width: `${progressPercent}%`,
    backgroundColor: '#007AFF',
    borderRadius: '2px',
    transition: isDragging ? 'none' : 'width 0.1s linear',
  };

  const thumbStyles: CSSProperties = {
    position: 'absolute',
    top: '50%',
    left: `${progressPercent}%`,
    transform: 'translate(-50%, -50%)',
    width: '20px',
    height: '20px',
    backgroundColor: '#007AFF',
    borderRadius: '50%',
    border: '2px solid #ffffff',
    boxShadow: '0 2px 4px rgba(0, 0, 0, 0.2)',
    cursor: 'grab',
    transition: isDragging ? 'none' : 'left 0.1s linear',
  };

  const markerStyles = (marker: TimelineMarker): CSSProperties => ({
    position: 'absolute',
    left: `${(marker.time / duration) * 100}%`,
    top: '50%',
    transform: 'translate(-50%, -50%)',
    width: '8px',
    height: '8px',
    backgroundColor: marker.color || '#FF9500',
    borderRadius: '50%',
    pointerEvents: 'none',
  });

  const previewStyles: CSSProperties = {
    position: 'absolute',
    bottom: '100%',
    left: previewTime !== null ? `${(previewTime / duration) * 100}%` : '0',
    transform: 'translateX(-50%)',
    marginBottom: '0.5rem',
    padding: '0.25rem 0.5rem',
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    color: '#ffffff',
    fontSize: '0.75rem',
    borderRadius: '4px',
    whiteSpace: 'nowrap',
    pointerEvents: 'none',
    opacity: previewTime !== null ? 1 : 0,
    transition: 'opacity 0.2s ease',
  };

  return (
    <div
      className="mobile-timeline"
      style={containerStyles}
      data-testid="mobile-timeline"
    >
      {/* Controls */}
      <div className="mobile-timeline__controls" style={controlsStyles}>
        <button
          className="mobile-timeline__play-button"
          style={playButtonStyles}
          onClick={handlePlayPause}
          type="button"
          aria-label={playing ? 'Pause' : 'Play'}
        >
          {playing ? '⏸' : '▶'}
        </button>

        <div className="mobile-timeline__time-display" style={timeDisplayStyles}>
          {formatTime(currentTime)} / {formatTime(duration)}
        </div>
      </div>

      {/* Timeline track */}
      <div
        ref={timelineRef}
        className="mobile-timeline__track-container"
        style={trackContainerStyles}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        onTouchCancel={handleTouchEnd}
      >
        {/* Preview tooltip */}
        <div className="mobile-timeline__preview" style={previewStyles}>
          {previewTime !== null && formatTime(previewTime)}
        </div>

        <div className="mobile-timeline__track" style={trackStyles}>
          {/* Progress bar */}
          <div className="mobile-timeline__progress" style={progressStyles} />

          {/* Markers */}
          {markers.map((marker) => (
            <div
              key={marker.id}
              className="mobile-timeline__marker"
              style={markerStyles(marker)}
              title={marker.label}
            />
          ))}

          {/* Scrubber thumb */}
          <div className="mobile-timeline__thumb" style={thumbStyles} />
        </div>
      </div>

      <style>{`
        .mobile-timeline__play-button:active {
          transform: scale(0.95);
        }

        .mobile-timeline__play-button:focus-visible {
          outline: 2px solid #007AFF;
          outline-offset: 2px;
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .mobile-timeline {
            background-color: #1c1c1e;
          }

          .mobile-timeline__time-display {
            color: #ffffff;
          }

          .mobile-timeline__track {
            background-color: #48484a;
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .mobile-timeline__progress,
          .mobile-timeline__thumb {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default MobileTimeline;
