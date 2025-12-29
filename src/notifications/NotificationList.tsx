// Notification list component

import React, { useState } from 'react';
import { Notification, NotificationLevel, NotificationCategory } from './types';
import './NotificationList.css';

export interface NotificationListProps {
  notifications: Notification[];
  loading?: boolean;
  hasMore?: boolean;
  onLoadMore?: () => void;
  onRead?: (id: string) => void;
  onUnread?: (id: string) => void;
  onArchive?: (id: string) => void;
  onDelete?: (id: string) => void;
  onAction?: (notificationId: string, actionId: string) => void;
  emptyMessage?: string;
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

interface NotificationItemProps {
  notification: Notification;
  onRead?: (id: string) => void;
  onUnread?: (id: string) => void;
  onArchive?: (id: string) => void;
  onDelete?: (id: string) => void;
  onAction?: (notificationId: string, actionId: string) => void;
}

const NotificationItem: React.FC<NotificationItemProps> = ({
  notification,
  onRead,
  onUnread,
  onArchive,
  onDelete,
  onAction,
}) => {
  const [showActions, setShowActions] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);

  const handleClick = () => {
    if (!notification.read && onRead) {
      onRead(notification.id);
    }
    setIsExpanded(!isExpanded);
  };

  const handleActionClick = (actionId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (onAction) {
      onAction(notification.id, actionId);
    }
  };

  const handleToggleRead = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (notification.read && onUnread) {
      onUnread(notification.id);
    } else if (!notification.read && onRead) {
      onRead(notification.id);
    }
  };

  const handleArchive = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onArchive) {
      onArchive(notification.id);
    }
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onDelete && window.confirm('Are you sure you want to delete this notification?')) {
      onDelete(notification.id);
    }
  };

  const icon = levelIcons[notification.level];
  const color = levelColors[notification.level];
  const timeAgo = getTimeAgo(new Date(notification.createdAt));

  return (
    <div
      className={`notification-item ${notification.read ? '' : 'notification-item-unread'} ${
        isExpanded ? 'notification-item-expanded' : ''
      }`}
      onClick={handleClick}
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => setShowActions(false)}
    >
      <div className="notification-item-indicator" style={{ backgroundColor: color }} />

      <div className="notification-item-icon" style={{ color }}>
        {icon}
      </div>

      <div className="notification-item-content">
        <div className="notification-item-header">
          <h4 className="notification-item-title">{notification.title}</h4>
          <span className="notification-item-time">{timeAgo}</span>
        </div>

        <p className={`notification-item-message ${isExpanded ? '' : 'notification-item-message-truncated'}`}>
          {notification.htmlMessage ? (
            <span dangerouslySetInnerHTML={{ __html: notification.htmlMessage }} />
          ) : (
            notification.message
          )}
        </p>

        {notification.sender && (
          <div className="notification-item-sender">
            {notification.sender.avatarUrl && (
              <img
                src={notification.sender.avatarUrl}
                alt={notification.sender.name}
                className="notification-item-avatar"
              />
            )}
            <span className="notification-item-sender-name">{notification.sender.name}</span>
          </div>
        )}

        {isExpanded && notification.actions.length > 0 && (
          <div className="notification-item-actions-list">
            {notification.actions.map((action) => (
              <button
                key={action.id}
                className="notification-item-action-btn"
                onClick={(e) => handleActionClick(action.id, e)}
              >
                {action.label}
              </button>
            ))}
          </div>
        )}

        <div className="notification-item-footer">
          <span className="notification-item-category">{notification.category}</span>
          {notification.priority > 2 && (
            <span className="notification-item-priority">
              Priority: {['Low', 'Normal', 'High', 'Urgent', 'Critical'][notification.priority - 1]}
            </span>
          )}
        </div>
      </div>

      {showActions && (
        <div className="notification-item-quick-actions">
          <button
            className="notification-item-quick-action"
            onClick={handleToggleRead}
            title={notification.read ? 'Mark as unread' : 'Mark as read'}
          >
            {notification.read ? '📭' : '📬'}
          </button>
          <button
            className="notification-item-quick-action"
            onClick={handleArchive}
            title="Archive"
          >
            🗄️
          </button>
          <button
            className="notification-item-quick-action notification-item-quick-action-delete"
            onClick={handleDelete}
            title="Delete"
          >
            🗑️
          </button>
        </div>
      )}
    </div>
  );
};

export const NotificationList: React.FC<NotificationListProps> = ({
  notifications,
  loading = false,
  hasMore = false,
  onLoadMore,
  onRead,
  onUnread,
  onArchive,
  onDelete,
  onAction,
  emptyMessage = 'No notifications',
}) => {
  if (notifications.length === 0 && !loading) {
    return (
      <div className="notification-list-empty">
        <div className="notification-list-empty-icon">🔔</div>
        <p className="notification-list-empty-message">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div className="notification-list">
      <div className="notification-list-items">
        {notifications.map((notification) => (
          <NotificationItem
            key={notification.id}
            notification={notification}
            onRead={onRead}
            onUnread={onUnread}
            onArchive={onArchive}
            onDelete={onDelete}
            onAction={onAction}
          />
        ))}
      </div>

      {loading && (
        <div className="notification-list-loading">
          <div className="notification-list-spinner" />
          <span>Loading notifications...</span>
        </div>
      )}

      {hasMore && !loading && (
        <button className="notification-list-load-more" onClick={onLoadMore}>
          Load More
        </button>
      )}
    </div>
  );
};

// Helper function to calculate time ago
function getTimeAgo(date: Date): string {
  const seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000);

  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;
  if (seconds < 2592000) return `${Math.floor(seconds / 604800)}w ago`;

  return date.toLocaleDateString();
}

export default NotificationList;
