// React context for notifications

import React, { createContext, useContext, useEffect, useReducer, useRef, ReactNode } from 'react';
import { Notification, NotificationStats, WebSocketMessage } from '../types';
import { notificationService } from '../services/NotificationService';

interface NotificationState {
  notifications: Notification[];
  unreadCount: number;
  stats: NotificationStats | null;
  loading: boolean;
  error: string | null;
  wsConnected: boolean;
}

type NotificationAction =
  | { type: 'SET_NOTIFICATIONS'; payload: Notification[] }
  | { type: 'ADD_NOTIFICATION'; payload: Notification }
  | { type: 'UPDATE_NOTIFICATION'; payload: Notification }
  | { type: 'REMOVE_NOTIFICATION'; payload: string }
  | { type: 'SET_STATS'; payload: NotificationStats }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SET_WS_CONNECTED'; payload: boolean }
  | { type: 'MARK_READ'; payload: string }
  | { type: 'MARK_ALL_READ' }
  | { type: 'ARCHIVE'; payload: string };

const initialState: NotificationState = {
  notifications: [],
  unreadCount: 0,
  stats: null,
  loading: false,
  error: null,
  wsConnected: false,
};

function notificationReducer(state: NotificationState, action: NotificationAction): NotificationState {
  switch (action.type) {
    case 'SET_NOTIFICATIONS':
      return {
        ...state,
        notifications: action.payload,
        unreadCount: action.payload.filter((n) => !n.read).length,
        loading: false,
      };

    case 'ADD_NOTIFICATION':
      return {
        ...state,
        notifications: [action.payload, ...state.notifications],
        unreadCount: action.payload.read ? state.unreadCount : state.unreadCount + 1,
      };

    case 'UPDATE_NOTIFICATION':
      return {
        ...state,
        notifications: state.notifications.map((n) =>
          n.id === action.payload.id ? action.payload : n
        ),
        unreadCount: state.notifications
          .map((n) => (n.id === action.payload.id ? action.payload : n))
          .filter((n) => !n.read).length,
      };

    case 'REMOVE_NOTIFICATION':
      const removedNotif = state.notifications.find((n) => n.id === action.payload);
      return {
        ...state,
        notifications: state.notifications.filter((n) => n.id !== action.payload),
        unreadCount: removedNotif && !removedNotif.read ? state.unreadCount - 1 : state.unreadCount,
      };

    case 'MARK_READ':
      return {
        ...state,
        notifications: state.notifications.map((n) =>
          n.id === action.payload ? { ...n, read: true, readAt: new Date().toISOString() } : n
        ),
        unreadCount: Math.max(0, state.unreadCount - 1),
      };

    case 'MARK_ALL_READ':
      return {
        ...state,
        notifications: state.notifications.map((n) => ({ ...n, read: true, readAt: new Date().toISOString() })),
        unreadCount: 0,
      };

    case 'ARCHIVE':
      const archivedNotif = state.notifications.find((n) => n.id === action.payload);
      return {
        ...state,
        notifications: state.notifications.filter((n) => n.id !== action.payload),
        unreadCount: archivedNotif && !archivedNotif.read ? state.unreadCount - 1 : state.unreadCount,
      };

    case 'SET_STATS':
      return {
        ...state,
        stats: action.payload,
        unreadCount: action.payload.unread,
      };

    case 'SET_LOADING':
      return { ...state, loading: action.payload };

    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };

    case 'SET_WS_CONNECTED':
      return { ...state, wsConnected: action.payload };

    default:
      return state;
  }
}

interface NotificationContextValue extends NotificationState {
  refreshNotifications: () => Promise<void>;
  markRead: (id: string) => Promise<void>;
  markAllRead: () => Promise<void>;
  archive: (id: string) => Promise<void>;
  deleteNotification: (id: string) => Promise<void>;
  executeAction: (notificationId: string, actionId: string) => Promise<void>;
}

const NotificationContext = createContext<NotificationContextValue | undefined>(undefined);

interface NotificationProviderProps {
  children: ReactNode;
  autoConnect?: boolean;
}

export function NotificationProvider({ children, autoConnect = true }: NotificationProviderProps) {
  const [state, dispatch] = useReducer(notificationReducer, initialState);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);

  // Load initial notifications
  const refreshNotifications = async () => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      const data = await notificationService.getNotifications(1, 50);
      dispatch({ type: 'SET_NOTIFICATIONS', payload: data.notifications });

      // Also fetch stats
      const stats = await notificationService.getStats();
      dispatch({ type: 'SET_STATS', payload: stats });

      dispatch({ type: 'SET_ERROR', payload: null });
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: (error as Error).message });
    }
  };

  // WebSocket connection
  const connectWebSocket = () => {
    try {
      const wsUrl = notificationService.getWebSocketUrl();
      const ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        console.log('WebSocket connected');
        dispatch({ type: 'SET_WS_CONNECTED', payload: true });
        reconnectAttempts.current = 0;
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);

          switch (message.type) {
            case 'notification':
              dispatch({ type: 'ADD_NOTIFICATION', payload: message.payload });
              // Show browser notification if permitted
              if (Notification.permission === 'granted') {
                new Notification(message.payload.title, {
                  body: message.payload.message,
                  icon: '/logo192.png',
                });
              }
              break;

            case 'read':
              dispatch({ type: 'MARK_READ', payload: message.payload.id });
              break;

            case 'archived':
            case 'deleted':
              dispatch({ type: 'REMOVE_NOTIFICATION', payload: message.payload.id });
              break;

            case 'stats':
              dispatch({ type: 'SET_STATS', payload: message.payload });
              break;
          }
        } catch (error) {
          console.error('Error processing WebSocket message:', error);
        }
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        dispatch({ type: 'SET_WS_CONNECTED', payload: false });
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        dispatch({ type: 'SET_WS_CONNECTED', payload: false });

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
    } catch (error) {
      console.error('Error connecting WebSocket:', error);
    }
  };

  useEffect(() => {
    // Load initial data
    refreshNotifications();

    // Request browser notification permission
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }

    // Connect WebSocket
    if (autoConnect) {
      connectWebSocket();
    }

    return () => {
      // Cleanup
      if (wsRef.current) {
        wsRef.current.close();
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
    };
  }, [autoConnect]);

  const markRead = async (id: string) => {
    try {
      await notificationService.markRead(id);
      dispatch({ type: 'MARK_READ', payload: id });
    } catch (error) {
      console.error('Error marking notification as read:', error);
      throw error;
    }
  };

  const markAllRead = async () => {
    try {
      await notificationService.markAllRead();
      dispatch({ type: 'MARK_ALL_READ' });
    } catch (error) {
      console.error('Error marking all notifications as read:', error);
      throw error;
    }
  };

  const archive = async (id: string) => {
    try {
      await notificationService.archive(id);
      dispatch({ type: 'ARCHIVE', payload: id });
    } catch (error) {
      console.error('Error archiving notification:', error);
      throw error;
    }
  };

  const deleteNotification = async (id: string) => {
    try {
      await notificationService.delete(id);
      dispatch({ type: 'REMOVE_NOTIFICATION', payload: id });
    } catch (error) {
      console.error('Error deleting notification:', error);
      throw error;
    }
  };

  const executeAction = async (notificationId: string, actionId: string) => {
    try {
      await notificationService.executeAction(notificationId, actionId);
      // Mark as read after executing action
      await markRead(notificationId);
    } catch (error) {
      console.error('Error executing notification action:', error);
      throw error;
    }
  };

  const value: NotificationContextValue = {
    ...state,
    refreshNotifications,
    markRead,
    markAllRead,
    archive,
    deleteNotification,
    executeAction,
  };

  return <NotificationContext.Provider value={value}>{children}</NotificationContext.Provider>;
}

export function useNotificationContext() {
  const context = useContext(NotificationContext);
  if (context === undefined) {
    throw new Error('useNotificationContext must be used within a NotificationProvider');
  }
  return context;
}
