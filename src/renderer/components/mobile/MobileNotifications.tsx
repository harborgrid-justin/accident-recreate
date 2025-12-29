/**
 * AccuScene Enterprise v0.3.0
 * Mobile Notifications Component
 *
 * In-app notification toasts for mobile
 */

import React, { useState, useEffect, CSSProperties, ReactNode } from 'react';
import { Notification, NotificationVariant } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface MobileNotificationsProps {
  position?: 'top' | 'bottom';
  maxNotifications?: number;
  className?: string;
}

/**
 * Notification toast manager
 * Displays stacked notifications with auto-dismiss
 *
 * @example
 * ```tsx
 * const { addNotification } = useMobileNotifications();
 *
 * addNotification({
 *   id: '1',
 *   message: 'Changes saved successfully',
 *   variant: 'success',
 *   duration: 3000,
 * });
 * ```
 */
export const MobileNotifications: React.FC<MobileNotificationsProps> = ({
  position = 'top',
  maxNotifications = 3,
  className = '',
}) => {
  const [notifications, setNotifications] = useState<Notification[]>([]);

  const addNotification = (notification: Notification) => {
    setNotifications((prev) => {
      const updated = [notification, ...prev].slice(0, maxNotifications);
      return updated;
    });

    // Haptic feedback based on variant
    switch (notification.variant) {
      case 'success':
        HapticFeedback.success();
        break;
      case 'error':
        HapticFeedback.error();
        break;
      case 'warning':
        HapticFeedback.warning();
        break;
      default:
        HapticFeedback.light();
    }

    // Auto-dismiss if duration is set
    if (notification.duration && notification.duration > 0) {
      setTimeout(() => {
        removeNotification(notification.id);
      }, notification.duration);
    }
  };

  const removeNotification = (id: string) => {
    setNotifications((prev) => prev.filter((n) => n.id !== id));
  };

  const handleDismiss = (id: string) => {
    HapticFeedback.light();
    removeNotification(id);
  };

  const containerStyles: CSSProperties = {
    position: 'fixed',
    [position]: '1rem',
    left: '1rem',
    right: '1rem',
    zIndex: 10000,
    display: 'flex',
    flexDirection: 'column',
    gap: '0.75rem',
    pointerEvents: 'none',
  };

  return (
    <div
      className={`mobile-notifications ${className}`}
      style={containerStyles}
      data-testid="mobile-notifications"
    >
      {notifications.map((notification) => (
        <NotificationToast
          key={notification.id}
          notification={notification}
          onDismiss={handleDismiss}
          position={position}
        />
      ))}
    </div>
  );
};

interface NotificationToastProps {
  notification: Notification;
  onDismiss: (id: string) => void;
  position: 'top' | 'bottom';
}

const NotificationToast: React.FC<NotificationToastProps> = ({
  notification,
  onDismiss,
  position,
}) => {
  const getVariantColor = (variant?: NotificationVariant): string => {
    switch (variant) {
      case 'success':
        return '#4CAF50';
      case 'error':
        return '#FF3B30';
      case 'warning':
        return '#FF9500';
      case 'info':
      default:
        return '#007AFF';
    }
  };

  const toastStyles: CSSProperties = {
    backgroundColor: '#ffffff',
    borderRadius: '12px',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
    padding: '1rem',
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    pointerEvents: 'auto',
    animation: `slideIn${position === 'top' ? 'Down' : 'Up'} 0.3s ease-out`,
    borderLeft: `4px solid ${getVariantColor(notification.variant)}`,
  };

  const iconContainerStyles: CSSProperties = {
    width: '32px',
    height: '32px',
    borderRadius: '50%',
    backgroundColor: `${getVariantColor(notification.variant)}20`,
    color: getVariantColor(notification.variant),
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '1.125rem',
    flexShrink: 0,
  };

  const contentStyles: CSSProperties = {
    flex: 1,
    fontSize: '0.875rem',
    lineHeight: 1.4,
    color: '#000000',
  };

  const actionButtonStyles: CSSProperties = {
    background: 'none',
    border: 'none',
    color: getVariantColor(notification.variant),
    fontSize: '0.875rem',
    fontWeight: 600,
    cursor: 'pointer',
    padding: '0.25rem 0.5rem',
    flexShrink: 0,
  };

  const dismissButtonStyles: CSSProperties = {
    background: 'none',
    border: 'none',
    color: '#8E8E93',
    fontSize: '1.125rem',
    cursor: 'pointer',
    padding: '0.25rem',
    lineHeight: 1,
    flexShrink: 0,
  };

  const getIcon = (): string => {
    if (notification.icon) return '';

    switch (notification.variant) {
      case 'success':
        return '✓';
      case 'error':
        return '✕';
      case 'warning':
        return '⚠';
      case 'info':
      default:
        return 'ℹ';
    }
  };

  return (
    <div
      className={`notification-toast notification-toast--${notification.variant || 'info'}`}
      style={toastStyles}
      role="alert"
      aria-live="polite"
    >
      <div className="notification-toast__icon" style={iconContainerStyles}>
        {notification.icon || getIcon()}
      </div>

      <div className="notification-toast__content" style={contentStyles}>
        {notification.message}
      </div>

      {notification.action && (
        <button
          className="notification-toast__action"
          style={actionButtonStyles}
          onClick={notification.action.onClick}
          type="button"
        >
          {notification.action.label}
        </button>
      )}

      {notification.dismissible !== false && (
        <button
          className="notification-toast__dismiss"
          style={dismissButtonStyles}
          onClick={() => onDismiss(notification.id)}
          type="button"
          aria-label="Dismiss notification"
        >
          ✕
        </button>
      )}

      <style>{`
        @keyframes slideInDown {
          from {
            transform: translateY(-100%);
            opacity: 0;
          }
          to {
            transform: translateY(0);
            opacity: 1;
          }
        }

        @keyframes slideInUp {
          from {
            transform: translateY(100%);
            opacity: 0;
          }
          to {
            transform: translateY(0);
            opacity: 1;
          }
        }

        .notification-toast__action:hover,
        .notification-toast__dismiss:hover {
          opacity: 0.7;
        }

        .notification-toast__action:active,
        .notification-toast__dismiss:active {
          transform: scale(0.95);
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .notification-toast {
            background-color: #1c1c1e;
          }

          .notification-toast__content {
            color: #ffffff;
          }

          .notification-toast__dismiss {
            color: #98989d;
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .notification-toast {
            animation: none !important;
          }
        }
      `}</style>
    </div>
  );
};

// Export hook for easy usage
export function useMobileNotifications() {
  const [notifications, setNotifications] = useState<Notification[]>([]);

  const addNotification = (
    message: string,
    variant: NotificationVariant = 'info',
    duration: number = 3000
  ) => {
    const notification: Notification = {
      id: `${Date.now()}-${Math.random()}`,
      message,
      variant,
      duration,
      timestamp: Date.now(),
    };

    setNotifications((prev) => [notification, ...prev]);

    return notification.id;
  };

  const removeNotification = (id: string) => {
    setNotifications((prev) => prev.filter((n) => n.id !== id));
  };

  return {
    notifications,
    addNotification,
    removeNotification,
    success: (message: string, duration?: number) =>
      addNotification(message, 'success', duration),
    error: (message: string, duration?: number) =>
      addNotification(message, 'error', duration),
    warning: (message: string, duration?: number) =>
      addNotification(message, 'warning', duration),
    info: (message: string, duration?: number) =>
      addNotification(message, 'info', duration),
  };
}

export default MobileNotifications;
