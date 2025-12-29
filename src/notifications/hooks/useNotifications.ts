// Custom hook for notifications with WebSocket support

import { useEffect, useState, useCallback, useRef } from 'react';
import {
  Notification,
  NotificationFilter,
  NotificationStats,
  NotificationPreferences,
  WebSocketMessage,
} from '../types';
import { notificationService } from '../services/NotificationService';

export interface UseNotificationsOptions {
  autoConnect?: boolean;
  autoRefresh?: boolean;
  refreshInterval?: number;
  pageSize?: number;
}

export interface UseNotificationsReturn {
  notifications: Notification[];
  unreadCount: number;
  stats: NotificationStats | null;
  preferences: NotificationPreferences | null;
  loading: boolean;
  error: string | null;
  wsConnected: boolean;
  hasMore: boolean;
  page: number;
  // Methods
  refresh: () => Promise<void>;
  loadMore: () => Promise<void>;
  markRead: (id: string) => Promise<void>;
  markUnread: (id: string) => Promise<void>;
  markAllRead: () => Promise<void>;
  archive: (id: string) => Promise<void>;
  deleteNotification: (id: string) => Promise<void>;
  updatePreferences: (preferences: Partial<NotificationPreferences>) => Promise<void>;
  setFilter: (filter: NotificationFilter) => void;
}

export function useNotifications(options: UseNotificationsOptions = {}): UseNotificationsReturn {
  const {
    autoConnect = true,
    autoRefresh = false,
    refreshInterval = 60000, // 1 minute
    pageSize = 20,
  } = options;

  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [stats, setStats] = useState<NotificationStats | null>(null);
  const [preferences, setPreferences] = useState<NotificationPreferences | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [wsConnected, setWsConnected] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [page, setPage] = useState(1);
  const [filter, setFilter] = useState<NotificationFilter | undefined>();

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const refreshIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Fetch notifications
  const fetchNotifications = useCallback(
    async (currentPage: number, append: boolean = false) => {
      try {
        setLoading(true);
        setError(null);

        const data = await notificationService.getNotifications(currentPage, pageSize, filter);

        if (append) {
          setNotifications((prev) => [...prev, ...data.notifications]);
        } else {
          setNotifications(data.notifications);
        }

        setHasMore(data.hasMore);
        setUnreadCount(data.notifications.filter((n) => !n.read).length);
      } catch (err) {
        setError((err as Error).message);
      } finally {
        setLoading(false);
      }
    },
    [pageSize, filter]
  );

  // Fetch stats
  const fetchStats = useCallback(async () => {
    try {
      const statsData = await notificationService.getStats();
      setStats(statsData);
      setUnreadCount(statsData.unread);
    } catch (err) {
      console.error('Error fetching stats:', err);
    }
  }, []);

  // Fetch preferences
  const fetchPreferences = useCallback(async () => {
    try {
      const prefsData = await notificationService.getPreferences();
      setPreferences(prefsData);
    } catch (err) {
      console.error('Error fetching preferences:', err);
    }
  }, []);

  // Refresh all data
  const refresh = useCallback(async () => {
    setPage(1);
    await Promise.all([fetchNotifications(1, false), fetchStats(), fetchPreferences()]);
  }, [fetchNotifications, fetchStats, fetchPreferences]);

  // Load more notifications
  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;
    const nextPage = page + 1;
    setPage(nextPage);
    await fetchNotifications(nextPage, true);
  }, [hasMore, loading, page, fetchNotifications]);

  // Mark notification as read
  const markRead = useCallback(
    async (id: string) => {
      try {
        await notificationService.markRead(id);
        setNotifications((prev) =>
          prev.map((n) => (n.id === id ? { ...n, read: true, readAt: new Date().toISOString() } : n))
        );
        setUnreadCount((prev) => Math.max(0, prev - 1));
        await fetchStats();
      } catch (err) {
        console.error('Error marking notification as read:', err);
        throw err;
      }
    },
    [fetchStats]
  );

  // Mark notification as unread
  const markUnread = useCallback(
    async (id: string) => {
      try {
        await notificationService.markUnread(id);
        setNotifications((prev) =>
          prev.map((n) => (n.id === id ? { ...n, read: false, readAt: undefined } : n))
        );
        setUnreadCount((prev) => prev + 1);
        await fetchStats();
      } catch (err) {
        console.error('Error marking notification as unread:', err);
        throw err;
      }
    },
    [fetchStats]
  );

  // Mark all as read
  const markAllRead = useCallback(async () => {
    try {
      await notificationService.markAllRead();
      setNotifications((prev) =>
        prev.map((n) => ({ ...n, read: true, readAt: new Date().toISOString() }))
      );
      setUnreadCount(0);
      await fetchStats();
    } catch (err) {
      console.error('Error marking all as read:', err);
      throw err;
    }
  }, [fetchStats]);

  // Archive notification
  const archive = useCallback(
    async (id: string) => {
      try {
        await notificationService.archive(id);
        setNotifications((prev) => prev.filter((n) => n.id !== id));
        await fetchStats();
      } catch (err) {
        console.error('Error archiving notification:', err);
        throw err;
      }
    },
    [fetchStats]
  );

  // Delete notification
  const deleteNotification = useCallback(
    async (id: string) => {
      try {
        await notificationService.delete(id);
        setNotifications((prev) => prev.filter((n) => n.id !== id));
        await fetchStats();
      } catch (err) {
        console.error('Error deleting notification:', err);
        throw err;
      }
    },
    [fetchStats]
  );

  // Update preferences
  const updatePreferences = useCallback(
    async (newPreferences: Partial<NotificationPreferences>) => {
      try {
        await notificationService.updatePreferences(newPreferences);
        await fetchPreferences();
      } catch (err) {
        console.error('Error updating preferences:', err);
        throw err;
      }
    },
    [fetchPreferences]
  );

  // WebSocket connection
  const connectWebSocket = useCallback(() => {
    try {
      const wsUrl = notificationService.getWebSocketUrl();
      const ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        console.log('WebSocket connected');
        setWsConnected(true);
        reconnectAttempts.current = 0;
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);

          switch (message.type) {
            case 'notification':
              setNotifications((prev) => [message.payload, ...prev]);
              setUnreadCount((prev) => (message.payload.read ? prev : prev + 1));

              // Show browser notification
              if ('Notification' in window && Notification.permission === 'granted') {
                new Notification(message.payload.title, {
                  body: message.payload.message,
                  icon: '/logo192.png',
                  tag: message.payload.id,
                });
              }
              break;

            case 'read':
              setNotifications((prev) =>
                prev.map((n) =>
                  n.id === message.payload.id
                    ? { ...n, read: true, readAt: new Date().toISOString() }
                    : n
                )
              );
              setUnreadCount((prev) => Math.max(0, prev - 1));
              break;

            case 'archived':
            case 'deleted':
              setNotifications((prev) => prev.filter((n) => n.id !== message.payload.id));
              break;

            case 'stats':
              setStats(message.payload);
              setUnreadCount(message.payload.unread);
              break;
          }
        } catch (err) {
          console.error('Error processing WebSocket message:', err);
        }
      };

      ws.onerror = (err) => {
        console.error('WebSocket error:', err);
        setWsConnected(false);
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setWsConnected(false);

        // Attempt to reconnect with exponential backoff
        if (autoConnect && reconnectAttempts.current < 10) {
          const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.current), 30000);
          reconnectTimeoutRef.current = setTimeout(() => {
            reconnectAttempts.current++;
            connectWebSocket();
          }, delay);
        }
      };

      wsRef.current = ws;
    } catch (err) {
      console.error('Error connecting WebSocket:', err);
    }
  }, [autoConnect]);

  // Initialize
  useEffect(() => {
    refresh();

    // Request browser notification permission
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }

    // Connect WebSocket
    if (autoConnect) {
      connectWebSocket();
    }

    // Setup auto-refresh
    if (autoRefresh) {
      refreshIntervalRef.current = setInterval(refresh, refreshInterval);
    }

    return () => {
      // Cleanup
      if (wsRef.current) {
        wsRef.current.close();
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (refreshIntervalRef.current) {
        clearInterval(refreshIntervalRef.current);
      }
    };
  }, [autoConnect, autoRefresh, refreshInterval, connectWebSocket, refresh]);

  // Refresh when filter changes
  useEffect(() => {
    setPage(1);
    fetchNotifications(1, false);
  }, [filter, fetchNotifications]);

  return {
    notifications,
    unreadCount,
    stats,
    preferences,
    loading,
    error,
    wsConnected,
    hasMore,
    page,
    refresh,
    loadMore,
    markRead,
    markUnread,
    markAllRead,
    archive,
    deleteNotification,
    updatePreferences,
    setFilter,
  };
}
