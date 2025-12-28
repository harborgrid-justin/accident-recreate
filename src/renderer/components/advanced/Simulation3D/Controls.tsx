/**
 * Controls Component
 * Playback control buttons
 */

import React from 'react';
import { ControlsProps } from '../types';
import './Controls.css';

export const Controls: React.FC<ControlsProps> = ({
  playing,
  playbackSpeed,
  onPlay,
  onPause,
  onStop,
  onSpeedChange,
  showFrameControls = false,
  onNextFrame,
  onPrevFrame,
}) => {
  const speedOptions = [0.25, 0.5, 1, 2, 4];

  return (
    <div className="simulation-controls">
      <div className="controls-group">
        <button
          className="control-button"
          onClick={onStop}
          title="Stop"
        >
          <StopIcon />
        </button>

        {playing ? (
          <button
            className="control-button primary"
            onClick={onPause}
            title="Pause"
          >
            <PauseIcon />
          </button>
        ) : (
          <button
            className="control-button primary"
            onClick={onPlay}
            title="Play"
          >
            <PlayIcon />
          </button>
        )}

        {showFrameControls && (
          <>
            <button
              className="control-button"
              onClick={onPrevFrame}
              title="Previous Frame"
            >
              <PrevFrameIcon />
            </button>
            <button
              className="control-button"
              onClick={onNextFrame}
              title="Next Frame"
            >
              <NextFrameIcon />
            </button>
          </>
        )}
      </div>

      <div className="controls-group">
        <label>Speed:</label>
        {speedOptions.map(speed => (
          <button
            key={speed}
            className={`speed-button ${playbackSpeed === speed ? 'active' : ''}`}
            onClick={() => onSpeedChange(speed)}
          >
            {speed}x
          </button>
        ))}
      </div>
    </div>
  );
};

// Icon components
const PlayIcon = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <path d="M4 2l10 6-10 6V2z" />
  </svg>
);

const PauseIcon = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <rect x="4" y="2" width="3" height="12" />
    <rect x="9" y="2" width="3" height="12" />
  </svg>
);

const StopIcon = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <rect x="3" y="3" width="10" height="10" />
  </svg>
);

const PrevFrameIcon = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <path d="M10 2l-6 6 6 6V2z" />
    <rect x="3" y="2" width="1" height="12" />
  </svg>
);

const NextFrameIcon = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <path d="M6 2l6 6-6 6V2z" />
    <rect x="12" y="2" width="1" height="12" />
  </svg>
);

export default Controls;
