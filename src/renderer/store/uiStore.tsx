/**
 * UI Store - Manages UI state (modals, notifications, theme, etc.)
 */

import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';

export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  message: string;
  duration?: number;
}

export interface Modal {
  id: string;
  type: string;
  data?: any;
}

interface UIState {
  theme: 'light' | 'dark';
  sidebarOpen: boolean;
  notifications: Notification[];
  modals: Modal[];
  loading: boolean;
  loadingMessage: string;
}

interface UIContextValue extends UIState {
  setTheme: (theme: 'light' | 'dark') => void;
  toggleTheme: () => void;
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
  showNotification: (
    type: Notification['type'],
    message: string,
    duration?: number
  ) => void;
  hideNotification: (id: string) => void;
  clearNotifications: () => void;
  openModal: (type: string, data?: any) => string;
  closeModal: (id: string) => void;
  closeAllModals: () => void;
  setLoading: (loading: boolean, message?: string) => void;
}

const UIContext = createContext<UIContextValue | undefined>(undefined);

export const UIProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [state, setState] = useState<UIState>({
    theme: (localStorage.getItem('theme') as 'light' | 'dark') || 'light',
    sidebarOpen: true,
    notifications: [],
    modals: [],
    loading: false,
    loadingMessage: '',
  });

  const setTheme = useCallback((theme: 'light' | 'dark') => {
    localStorage.setItem('theme', theme);
    document.documentElement.setAttribute('data-theme', theme);
    setState((prev) => ({ ...prev, theme }));
  }, []);

  const toggleTheme = useCallback(() => {
    setState((prev) => {
      const newTheme = prev.theme === 'light' ? 'dark' : 'light';
      localStorage.setItem('theme', newTheme);
      document.documentElement.setAttribute('data-theme', newTheme);
      return { ...prev, theme: newTheme };
    });
  }, []);

  const toggleSidebar = useCallback(() => {
    setState((prev) => ({ ...prev, sidebarOpen: !prev.sidebarOpen }));
  }, []);

  const setSidebarOpen = useCallback((open: boolean) => {
    setState((prev) => ({ ...prev, sidebarOpen: open }));
  }, []);

  const showNotification = useCallback(
    (type: Notification['type'], message: string, duration: number = 5000) => {
      const id = `notification-${Date.now()}-${Math.random()}`;
      const notification: Notification = { id, type, message, duration };

      setState((prev) => ({
        ...prev,
        notifications: [...prev.notifications, notification],
      }));

      if (duration > 0) {
        setTimeout(() => {
          setState((prev) => ({
            ...prev,
            notifications: prev.notifications.filter((n) => n.id !== id),
          }));
        }, duration);
      }
    },
    []
  );

  const hideNotification = useCallback((id: string) => {
    setState((prev) => ({
      ...prev,
      notifications: prev.notifications.filter((n) => n.id !== id),
    }));
  }, []);

  const clearNotifications = useCallback(() => {
    setState((prev) => ({ ...prev, notifications: [] }));
  }, []);

  const openModal = useCallback((type: string, data?: any): string => {
    const id = `modal-${Date.now()}-${Math.random()}`;
    const modal: Modal = { id, type, data };

    setState((prev) => ({
      ...prev,
      modals: [...prev.modals, modal],
    }));

    return id;
  }, []);

  const closeModal = useCallback((id: string) => {
    setState((prev) => ({
      ...prev,
      modals: prev.modals.filter((m) => m.id !== id),
    }));
  }, []);

  const closeAllModals = useCallback(() => {
    setState((prev) => ({ ...prev, modals: [] }));
  }, []);

  const setLoading = useCallback((loading: boolean, message: string = '') => {
    setState((prev) => ({ ...prev, loading, loadingMessage: message }));
  }, []);

  const value: UIContextValue = {
    ...state,
    setTheme,
    toggleTheme,
    toggleSidebar,
    setSidebarOpen,
    showNotification,
    hideNotification,
    clearNotifications,
    openModal,
    closeModal,
    closeAllModals,
    setLoading,
  };

  return <UIContext.Provider value={value}>{children}</UIContext.Provider>;
};

export const useUIStore = (): UIContextValue => {
  const context = useContext(UIContext);
  if (context === undefined) {
    throw new Error('useUIStore must be used within a UIProvider');
  }
  return context;
};
