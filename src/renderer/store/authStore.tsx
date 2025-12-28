/**
 * Authentication Store - Manages user authentication state
 */

import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { api, AuthResponse } from '../services/api';

export interface User {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  role: string;
  isActive: boolean;
  fullName?: string;
}

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

export interface AuthContextValue extends AuthState {
  login: (email: string, password: string) => Promise<boolean>;
  logout: () => void;
  clearError: () => void;
  checkAuth: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [state, setState] = useState<AuthState>({
    user: null,
    token: api.getToken(),
    isAuthenticated: api.isAuthenticated(),
    isLoading: false,
    error: null,
  });

  const login = useCallback(async (email: string, password: string): Promise<boolean> => {
    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      const response = await api.login({ email, password });

      if (response.success && response.data) {
        const { token, user } = response.data;
        setState({
          user: {
            ...user,
            fullName: `${user.firstName} ${user.lastName}`,
          },
          token,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        });
        return true;
      } else {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: response.error || 'Login failed',
        }));
        return false;
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Login failed',
      }));
      return false;
    }
  }, []);

  const logout = useCallback(() => {
    api.logout();
    setState({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
    });
  }, []);

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  const checkAuth = useCallback(async () => {
    if (!api.isAuthenticated()) {
      setState((prev) => ({ ...prev, isAuthenticated: false, user: null }));
      return;
    }

    setState((prev) => ({ ...prev, isLoading: true }));

    try {
      const response = await api.getCurrentUser();
      if (response.success && response.data) {
        setState({
          user: {
            ...response.data,
            fullName: `${response.data.firstName} ${response.data.lastName}`,
          },
          token: api.getToken(),
          isAuthenticated: true,
          isLoading: false,
          error: null,
        });
      } else {
        // Token invalid, logout
        logout();
      }
    } catch (error) {
      logout();
    }
  }, [logout]);

  const value: AuthContextValue = {
    ...state,
    login,
    logout,
    clearError,
    checkAuth,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export const useAuthStore = (): AuthContextValue => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuthStore must be used within an AuthProvider');
  }
  return context;
};
