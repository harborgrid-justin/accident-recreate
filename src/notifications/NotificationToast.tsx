// Toast notification component

import React, { useEffect, useState } from 'react';
import { Notification, NotificationLevel } from './types';
import './NotificationToast.css';

export interface NotificationToastProps {
  notification: Notification;
  onClose: () => void;
  onAction?: (actionId: string) => void;
  autoClose?: boolean;
  autoCloseDelay?: number;
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center';
}

const levelIcons: Record<NotificationLevel, string> = {
  [NotificationLevel.Info]: 'ℹ️',
  [NotificationLevel.Success]: '✅',
  [NotificationLevel.Warning]: '⚠️',
  [NotificationLevel.Error]: '❌',
  [NotificationLevel.Alert]: '🚨',
};

const levelColors: Record<NotificationLevel, string> = {
  [NotificationLevel.Info]: '#3b82f6',
  [NotificationLevel.Success]: '#10b981',
  [NotificationLevel.Warning]: '#f59e0b',
  [NotificationLevel.Error]: '#ef4444',
  [NotificationLevel.Alert]: '#dc2626',
};

export const NotificationToast: React.FC<NotificationToastProps> = ({
  notification,
  onClose,
  onAction,
  autoClose = true,
  autoCloseDelay = 5000,
  position = 'top-right',
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const [isExiting, setIsExiting] = useState(false);

  useEffect(() => {
    // Animate in
    const timer = setTimeout(() => setIsVisible(true), 10);

    // Auto close
    let closeTimer: NodeJS.Timeout;
    if (autoClose) {
      closeTimer = setTimeout(() => {
        handleClose();
      }, autoCloseDelay);
    }

    return () => {
      clearTimeout(timer);
      if (closeTimer) clearTimeout(closeTimer);
    };
  }, [autoClose, autoCloseDelay]);

  const handleClose = () => {
    setIsExiting(true);
    setTimeout(() => {
      onClose();
    }, 300); // Animation duration
  };

  const handleActionClick = (actionId: string) => {
    if (onAction) {
      onAction(actionId);
    }
    handleClose();
  };

  const icon = levelIcons[notification.level];
  const color = levelColors[notification.level];

  return (
    <div
      className={`notification-toast notification-toast-${position} ${
        isVisible ? 'notification-toast-visible' : ''
      } ${isExiting ? 'notification-toast-exiting' : ''}`}
      style={{ borderLeftColor: color }}
    >
      <div className="notification-toast-icon" style={{ color }}>
        {icon}
      </div>

      <div className="notification-toast-content">
        <div className="notification-toast-header">
          <h4 className="notification-toast-title">{notification.title}</h4>
          <button
            className="notification-toast-close"
            onClick={handleClose}
            aria-label="Close notification"
          >
            ×
          </button>
        </div>

        <p className="notification-toast-message">
          {notification.htmlMessage ? (
            <span dangerouslySetInnerHTML={{ __html: notification.htmlMessage }} />
          ) : (
            notification.message
          )}
        </p>

        {notification.sender && (
          <div className="notification-toast-sender">
            {notification.sender.avatarUrl && (
              <img
                src={notification.sender.avatarUrl}
                alt={notification.sender.name}
                className="notification-toast-avatar"
              />
            )}
            <span className="notification-toast-sender-name">{notification.sender.name}</span>
          </div>
        )}

        {notification.actions.length > 0 && (
          <div className="notification-toast-actions">
            {notification.actions.map((action) => (
              <button
                key={action.id}
                className="notification-toast-action-btn"
                onClick={() => handleActionClick(action.id)}
              >
                {action.label}
              </button>
            ))}
          </div>
        )}

        <div className="notification-toast-footer">
          <span className="notification-toast-time">
            {new Date(notification.createdAt).toLocaleTimeString()}
          </span>
          <span className="notification-toast-category">{notification.category}</span>
        </div>
      </div>

      {!autoClose && (
        <div className="notification-toast-progress-container">
          <div
            className="notification-toast-progress"
            style={{
              animationDuration: `${autoCloseDelay}ms`,
              backgroundColor: color,
            }}
          />
        </div>
      )}
    </div>
  );
};

export interface ToastContainerProps {
  notifications: Notification[];
  onClose: (id: string) => void;
  onAction?: (notificationId: string, actionId: string) => void;
  position?: NotificationToastProps['position'];
  maxToasts?: number;
}

export const ToastContainer: React.FC<ToastContainerProps> = ({
  notifications,
  onClose,
  onAction,
  position = 'top-right',
  maxToasts = 5,
}) => {
  const visibleNotifications = notifications.slice(0, maxToasts);

  return (
    <div className={`notification-toast-container notification-toast-container-${position}`}>
      {visibleNotifications.map((notification) => (
        <NotificationToast
          key={notification.id}
          notification={notification}
          onClose={() => onClose(notification.id)}
          onAction={(actionId) => onAction && onAction(notification.id, actionId)}
          position={position}
        />
      ))}
    </div>
  );
};

export default NotificationToast;
