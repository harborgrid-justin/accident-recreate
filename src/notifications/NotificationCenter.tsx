// Main notification center component

import React, { useState } from 'react';
import { NotificationList } from './NotificationList';
import { NotificationBadge } from './NotificationBadge';
import { useNotifications } from './hooks/useNotifications';
import { NotificationFilter, NotificationLevel, NotificationCategory } from './types';
import './NotificationCenter.css';

export interface NotificationCenterProps {
  position?: 'left' | 'right';
  maxHeight?: number;
  showFilters?: boolean;
  showStats?: boolean;
}

export const NotificationCenter: React.FC<NotificationCenterProps> = ({
  position = 'right',
  maxHeight = 600,
  showFilters = true,
  showStats = true,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<'all' | 'unread'>('all');
  const [selectedLevel, setSelectedLevel] = useState<NotificationLevel | 'all'>('all');
  const [selectedCategory, setSelectedCategory] = useState<NotificationCategory | 'all'>('all');

  const {
    notifications,
    unreadCount,
    stats,
    loading,
    error,
    wsConnected,
    hasMore,
    refresh,
    loadMore,
    markRead,
    markUnread,
    markAllRead,
    archive,
    deleteNotification,
    setFilter,
  } = useNotifications({
    autoConnect: true,
    autoRefresh: false,
  });

  const toggleOpen = () => {
    setIsOpen(!isOpen);
  };

  const handleTabChange = (tab: 'all' | 'unread') => {
    setActiveTab(tab);
    updateFilter({ read: tab === 'all' ? undefined : false });
  };

  const handleLevelFilter = (level: NotificationLevel | 'all') => {
    setSelectedLevel(level);
    updateFilter({ level: level === 'all' ? undefined : [level] });
  };

  const handleCategoryFilter = (category: NotificationCategory | 'all') => {
    setSelectedCategory(category);
    updateFilter({ category: category === 'all' ? undefined : [category] });
  };

  const updateFilter = (updates: Partial<NotificationFilter>) => {
    const newFilter: NotificationFilter = {
      level: selectedLevel !== 'all' ? [selectedLevel] : undefined,
      category: selectedCategory !== 'all' ? [selectedCategory] : undefined,
      read: activeTab === 'unread' ? false : undefined,
      ...updates,
    };
    setFilter(newFilter);
  };

  const handleMarkAllRead = async () => {
    try {
      await markAllRead();
    } catch (err) {
      console.error('Error marking all as read:', err);
    }
  };

  const filteredNotifications = notifications;

  return (
    <div className={`notification-center notification-center-${position}`}>
      {/* Trigger Button */}
      <button
        className="notification-center-trigger"
        onClick={toggleOpen}
        aria-label="Open notifications"
      >
        <NotificationBadge count={unreadCount} max={99}>
          <span className="notification-center-bell">🔔</span>
        </NotificationBadge>
        {wsConnected && (
          <span className="notification-center-status-indicator" title="Connected" />
        )}
      </button>

      {/* Dropdown Panel */}
      {isOpen && (
        <>
          <div className="notification-center-backdrop" onClick={toggleOpen} />
          <div className="notification-center-panel" style={{ maxHeight }}>
            {/* Header */}
            <div className="notification-center-header">
              <h3 className="notification-center-title">Notifications</h3>
              <div className="notification-center-header-actions">
                <button
                  className="notification-center-refresh"
                  onClick={refresh}
                  title="Refresh"
                  disabled={loading}
                >
                  🔄
                </button>
                <button
                  className="notification-center-close"
                  onClick={toggleOpen}
                  aria-label="Close"
                >
                  ×
                </button>
              </div>
            </div>

            {/* Tabs */}
            <div className="notification-center-tabs">
              <button
                className={`notification-center-tab ${activeTab === 'all' ? 'active' : ''}`}
                onClick={() => handleTabChange('all')}
              >
                All
                {stats && <span className="notification-center-tab-count">({stats.total})</span>}
              </button>
              <button
                className={`notification-center-tab ${activeTab === 'unread' ? 'active' : ''}`}
                onClick={() => handleTabChange('unread')}
              >
                Unread
                <NotificationBadge count={unreadCount} max={99} size="small" />
              </button>
            </div>

            {/* Filters */}
            {showFilters && (
              <div className="notification-center-filters">
                <select
                  className="notification-center-filter"
                  value={selectedLevel}
                  onChange={(e) => handleLevelFilter(e.target.value as NotificationLevel | 'all')}
                >
                  <option value="all">All Levels</option>
                  <option value={NotificationLevel.Info}>Info</option>
                  <option value={NotificationLevel.Success}>Success</option>
                  <option value={NotificationLevel.Warning}>Warning</option>
                  <option value={NotificationLevel.Error}>Error</option>
                  <option value={NotificationLevel.Alert}>Alert</option>
                </select>

                <select
                  className="notification-center-filter"
                  value={selectedCategory}
                  onChange={(e) =>
                    handleCategoryFilter(e.target.value as NotificationCategory | 'all')
                  }
                >
                  <option value="all">All Categories</option>
                  <option value={NotificationCategory.System}>System</option>
                  <option value={NotificationCategory.Case}>Case</option>
                  <option value={NotificationCategory.Collaboration}>Collaboration</option>
                  <option value={NotificationCategory.Analysis}>Analysis</option>
                  <option value={NotificationCategory.Report}>Report</option>
                  <option value={NotificationCategory.Security}>Security</option>
                </select>
              </div>
            )}

            {/* Actions Bar */}
            {unreadCount > 0 && (
              <div className="notification-center-actions-bar">
                <button
                  className="notification-center-mark-all-read"
                  onClick={handleMarkAllRead}
                >
                  Mark all as read
                </button>
              </div>
            )}

            {/* Stats */}
            {showStats && stats && (
              <div className="notification-center-stats">
                <div className="notification-center-stat">
                  <span className="notification-center-stat-label">Total:</span>
                  <span className="notification-center-stat-value">{stats.total}</span>
                </div>
                <div className="notification-center-stat">
                  <span className="notification-center-stat-label">Unread:</span>
                  <span className="notification-center-stat-value">{stats.unread}</span>
                </div>
              </div>
            )}

            {/* Error Message */}
            {error && (
              <div className="notification-center-error">
                <span className="notification-center-error-icon">⚠️</span>
                <span className="notification-center-error-message">{error}</span>
              </div>
            )}

            {/* Notification List */}
            <div className="notification-center-list-container">
              <NotificationList
                notifications={filteredNotifications}
                loading={loading}
                hasMore={hasMore}
                onLoadMore={loadMore}
                onRead={markRead}
                onUnread={markUnread}
                onArchive={archive}
                onDelete={deleteNotification}
                emptyMessage={
                  activeTab === 'unread'
                    ? 'No unread notifications'
                    : 'No notifications yet'
                }
              />
            </div>

            {/* Footer */}
            <div className="notification-center-footer">
              <a href="/notifications/settings" className="notification-center-settings-link">
                ⚙️ Notification Settings
              </a>
            </div>
          </div>
        </>
      )}
    </div>
  );
};

export default NotificationCenter;
