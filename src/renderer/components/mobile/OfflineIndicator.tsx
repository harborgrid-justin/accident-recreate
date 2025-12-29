/**
 * AccuScene Enterprise v0.3.0
 * Offline Indicator Component
 *
 * Visual indicator for offline/online status
 */

import React, { useState, useEffect, CSSProperties } from 'react';
import { HapticFeedback } from './HapticFeedback';

export interface OfflineIndicatorProps {
  position?: 'top' | 'bottom';
  showOnlineMessage?: boolean;
  onlineMessageDuration?: number;
  className?: string;
}

/**
 * Offline status indicator
 * Shows banner when connection is lost
 *
 * @example
 * ```tsx
 * <OfflineIndicator
 *   position="top"
 *   showOnlineMessage
 *   onlineMessageDuration={3000}
 * />
 * ```
 */
export const OfflineIndicator: React.FC<OfflineIndicatorProps> = ({
  position = 'top',
  showOnlineMessage = true,
  onlineMessageDuration = 3000,
  className = '',
}) => {
  const [isOnline, setIsOnline] = useState(navigator.onLine);
  const [showOnlineNotification, setShowOnlineNotification] = useState(false);

  useEffect(() => {
    const handleOnline = () => {
      setIsOnline(true);
      if (showOnlineMessage) {
        setShowOnlineNotification(true);
        HapticFeedback.success();
        setTimeout(() => {
          setShowOnlineNotification(false);
        }, onlineMessageDuration);
      }
    };

    const handleOffline = () => {
      setIsOnline(false);
      HapticFeedback.warning();
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [showOnlineMessage, onlineMessageDuration]);

  const shouldShow = !isOnline || showOnlineNotification;

  if (!shouldShow) return null;

  const containerStyles: CSSProperties = {
    position: 'fixed',
    [position]: 0,
    left: 0,
    right: 0,
    padding: '0.75rem 1rem',
    backgroundColor: isOnline ? '#4CAF50' : '#FF9500',
    color: '#ffffff',
    textAlign: 'center',
    fontSize: '0.875rem',
    fontWeight: 500,
    zIndex: 9999,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '0.5rem',
    boxShadow: position === 'top' ? '0 2px 8px rgba(0, 0, 0, 0.15)' : '0 -2px 8px rgba(0, 0, 0, 0.15)',
    animation: 'slideIn 0.3s ease-out',
  };

  const iconStyles: CSSProperties = {
    fontSize: '1.125rem',
  };

  return (
    <div
      className={`offline-indicator ${className}`}
      style={containerStyles}
      role="alert"
      aria-live="polite"
      data-testid="offline-indicator"
    >
      <span style={iconStyles}>
        {isOnline ? '✓' : '⚠'}
      </span>
      <span>
        {isOnline ? 'Back online' : 'No internet connection'}
      </span>

      <style>{`
        @keyframes slideIn {
          from {
            transform: translateY(${position === 'top' ? '-100%' : '100%'});
          }
          to {
            transform: translateY(0);
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .offline-indicator {
            animation: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default OfflineIndicator;
