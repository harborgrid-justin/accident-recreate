/**
 * Live Region Component
 *
 * ARIA live region for dynamic content announcements to screen readers.
 * Implements WCAG 2.1 Success Criterion 4.1.3 (Status Messages)
 */

import React, { useEffect, useRef, useState } from 'react';
import type { LiveRegionOptions } from '../types';

export interface LiveRegionProps {
  message?: string;
  priority?: 'polite' | 'assertive';
  atomic?: boolean;
  relevant?: 'additions' | 'removals' | 'text' | 'all';
  role?: 'status' | 'alert' | 'log' | 'timer';
  clearOnUnmount?: boolean;
  className?: string;
}

export const LiveRegion: React.FC<LiveRegionProps> = ({
  message = '',
  priority = 'polite',
  atomic = true,
  relevant = 'additions',
  role = 'status',
  clearOnUnmount = true,
  className = '',
}) => {
  const [currentMessage, setCurrentMessage] = useState(message);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (message) {
      // Clear any pending timeout
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }

      // Set message
      setCurrentMessage(message);

      // Auto-clear after announcement (optional)
      if (clearOnUnmount) {
        timeoutRef.current = setTimeout(() => {
          setCurrentMessage('');
        }, 5000);
      }
    }

    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [message, clearOnUnmount]);

  return (
    <div
      role={role}
      aria-live={priority}
      aria-atomic={atomic}
      aria-relevant={relevant}
      className={`live-region ${className}`}
      style={{
        position: 'absolute',
        width: '1px',
        height: '1px',
        padding: 0,
        margin: '-1px',
        overflow: 'hidden',
        clip: 'rect(0, 0, 0, 0)',
        whiteSpace: 'nowrap',
        border: 0,
      }}
    >
      {currentMessage}
    </div>
  );
};

/**
 * Global Live Region Manager
 *
 * Single instance to manage all announcements
 */
export const GlobalLiveRegion: React.FC = () => {
  const [announcements, setAnnouncements] = useState<{
    polite: string;
    assertive: string;
  }>({
    polite: '',
    assertive: '',
  });

  useEffect(() => {
    const handleAnnounce = (event: Event) => {
      const customEvent = event as CustomEvent<{
        message: string;
        priority: 'polite' | 'assertive';
      }>;
      const { message, priority } = customEvent.detail;

      setAnnouncements((prev) => ({
        ...prev,
        [priority]: message,
      }));

      // Clear after announcement
      setTimeout(() => {
        setAnnouncements((prev) => ({
          ...prev,
          [priority]: '',
        }));
      }, 100);
    };

    window.addEventListener('a11y-announce', handleAnnounce);
    return () => window.removeEventListener('a11y-announce', handleAnnounce);
  }, []);

  return (
    <>
      <LiveRegion
        message={announcements.polite}
        priority="polite"
        role="status"
      />
      <LiveRegion
        message={announcements.assertive}
        priority="assertive"
        role="alert"
      />
    </>
  );
};

export default LiveRegion;
