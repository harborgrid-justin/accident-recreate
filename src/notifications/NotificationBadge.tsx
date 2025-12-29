// Notification badge component showing unread count

import React from 'react';
import './NotificationBadge.css';

export interface NotificationBadgeProps {
  count: number;
  max?: number;
  dot?: boolean;
  showZero?: boolean;
  color?: 'primary' | 'success' | 'warning' | 'error' | 'info';
  size?: 'small' | 'medium' | 'large';
  className?: string;
  children?: React.ReactNode;
}

export const NotificationBadge: React.FC<NotificationBadgeProps> = ({
  count,
  max = 99,
  dot = false,
  showZero = false,
  color = 'error',
  size = 'medium',
  className = '',
  children,
}) => {
  const displayCount = count > max ? `${max}+` : count.toString();
  const shouldShow = count > 0 || showZero;

  if (!shouldShow && !children) {
    return null;
  }

  const badgeContent = dot ? null : displayCount;

  return (
    <div className={`notification-badge-wrapper ${className}`}>
      {children}
      {shouldShow && (
        <span
          className={`notification-badge notification-badge-${color} notification-badge-${size} ${
            dot ? 'notification-badge-dot' : ''
          }`}
          aria-label={`${count} unread notifications`}
        >
          {badgeContent}
        </span>
      )}
    </div>
  );
};

export default NotificationBadge;
