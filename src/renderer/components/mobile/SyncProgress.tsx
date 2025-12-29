/**
 * AccuScene Enterprise v0.3.0
 * Sync Progress Component
 *
 * Background sync progress indicator
 */

import React, { CSSProperties } from 'react';
import { SyncProgress as SyncProgressType } from './types';

export interface SyncProgressProps {
  progress: SyncProgressType;
  onCancel?: () => void;
  position?: 'top' | 'bottom';
  compact?: boolean;
  className?: string;
}

/**
 * Sync progress indicator
 * Shows background synchronization status
 *
 * @example
 * ```tsx
 * <SyncProgress
 *   progress={{
 *     total: 100,
 *     completed: 45,
 *     failed: 2,
 *     inProgress: 5,
 *     percentage: 45,
 *   }}
 *   onCancel={() => cancelSync()}
 *   position="bottom"
 * />
 * ```
 */
export const SyncProgress: React.FC<SyncProgressProps> = ({
  progress,
  onCancel,
  position = 'bottom',
  compact = false,
  className = '',
}) => {
  const formatTime = (seconds?: number): string => {
    if (!seconds) return '';
    if (seconds < 60) return `${Math.round(seconds)}s`;
    const mins = Math.floor(seconds / 60);
    const secs = Math.round(seconds % 60);
    return `${mins}m ${secs}s`;
  };

  const containerStyles: CSSProperties = {
    position: 'fixed',
    [position]: '1rem',
    left: '1rem',
    right: '1rem',
    backgroundColor: '#ffffff',
    borderRadius: '12px',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
    padding: compact ? '0.75rem 1rem' : '1rem 1.25rem',
    zIndex: 1000,
    animation: 'slideUp 0.3s ease-out',
  };

  const headerStyles: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: compact ? '0.5rem' : '0.75rem',
  };

  const titleStyles: CSSProperties = {
    fontSize: compact ? '0.875rem' : '1rem',
    fontWeight: 600,
    color: '#000000',
  };

  const cancelButtonStyles: CSSProperties = {
    background: 'none',
    border: 'none',
    color: '#8E8E93',
    fontSize: '1.25rem',
    cursor: 'pointer',
    padding: '0.25rem',
    lineHeight: 1,
  };

  const progressBarContainerStyles: CSSProperties = {
    width: '100%',
    height: compact ? '4px' : '6px',
    backgroundColor: '#e0e0e0',
    borderRadius: '3px',
    overflow: 'hidden',
    marginBottom: compact ? '0.5rem' : '0.75rem',
  };

  const progressBarStyles: CSSProperties = {
    height: '100%',
    width: `${progress.percentage}%`,
    backgroundColor: progress.failed > 0 ? '#FF9500' : '#007AFF',
    borderRadius: '3px',
    transition: 'width 0.3s ease',
  };

  const statsStyles: CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: compact ? '0.75rem' : '0.875rem',
    color: '#8E8E93',
  };

  const statItemStyles: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: '0.25rem',
  };

  return (
    <div
      className={`sync-progress ${className}`}
      style={containerStyles}
      role="status"
      aria-live="polite"
      data-testid="sync-progress"
    >
      <div className="sync-progress__header" style={headerStyles}>
        <div className="sync-progress__title" style={titleStyles}>
          Syncing {progress.completed} of {progress.total}
        </div>
        {onCancel && (
          <button
            className="sync-progress__cancel"
            style={cancelButtonStyles}
            onClick={onCancel}
            type="button"
            aria-label="Cancel sync"
          >
            ✕
          </button>
        )}
      </div>

      <div className="sync-progress__bar-container" style={progressBarContainerStyles}>
        <div className="sync-progress__bar" style={progressBarStyles} />
      </div>

      {!compact && (
        <div className="sync-progress__stats" style={statsStyles}>
          <div style={statItemStyles}>
            <span>{Math.round(progress.percentage)}%</span>
          </div>

          {progress.inProgress > 0 && (
            <div style={statItemStyles}>
              <span>⏳ {progress.inProgress} in progress</span>
            </div>
          )}

          {progress.failed > 0 && (
            <div style={{ ...statItemStyles, color: '#FF3B30' }}>
              <span>⚠ {progress.failed} failed</span>
            </div>
          )}

          {progress.estimatedTimeRemaining && (
            <div style={statItemStyles}>
              <span>{formatTime(progress.estimatedTimeRemaining)} remaining</span>
            </div>
          )}
        </div>
      )}

      <style>{`
        @keyframes slideUp {
          from {
            transform: translateY(100%);
            opacity: 0;
          }
          to {
            transform: translateY(0);
            opacity: 1;
          }
        }

        .sync-progress__cancel:hover {
          color: #000000;
        }

        .sync-progress__cancel:active {
          transform: scale(0.95);
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .sync-progress {
            background-color: #1c1c1e;
          }

          .sync-progress__title {
            color: #ffffff;
          }

          .sync-progress__bar-container {
            background-color: #48484a;
          }

          .sync-progress__stats {
            color: #98989d;
          }

          .sync-progress__cancel:hover {
            color: #ffffff;
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .sync-progress {
            animation: none !important;
          }

          .sync-progress__bar {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default SyncProgress;
