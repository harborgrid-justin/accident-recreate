/**
 * Authentication Context Provider
 * Manages global authentication state and provides auth methods
 */

import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import {
  SSOUser,
  Session,
  AuthContextValue,
  TOTPSetup,
  UserMFAConfig,
  SSOError,
} from './types';
import AuthService from './services/AuthService';
import SessionManager from './SessionManager';

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export interface AuthProviderProps {
  children: React.ReactNode;
  autoRefresh?: boolean;
  refreshInterval?: number;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({
  children,
  autoRefresh = true,
  refreshInterval = 60000, // 1 minute
}) => {
  const [user, setUser] = useState<SSOUser | null>(null);
  const [session, setSession] = useState<Session | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // ============================================================================
  // Initialization
  // ============================================================================

  useEffect(() => {
    initializeAuth();
  }, []);

  const initializeAuth = async () => {
    try {
      setIsLoading(true);

      // Load from session storage
      const storedUser = SessionManager.getUser();
      const storedSession = SessionManager.getSession();

      if (storedUser && storedSession) {
        // Check if session is still valid
        if (!SessionManager.isSessionExpired(storedSession)) {
          setUser(storedUser);
          setSession(storedSession);

          // Validate with server
          try {
            await AuthService.validateSession();
          } catch (err) {
            // Session invalid on server, clear local state
            handleLogout();
          }
        } else {
          // Try to refresh
          try {
            await refreshSession();
          } catch (err) {
            handleLogout();
          }
        }
      }
    } catch (err) {
      console.error('Failed to initialize auth:', err);
      setError(err instanceof Error ? err.message : 'Initialization failed');
    } finally {
      setIsLoading(false);
    }
  };

  // ============================================================================
  // Auto-refresh Token
  // ============================================================================

  useEffect(() => {
    if (!autoRefresh || !user) return;

    const interval = setInterval(async () => {
      if (SessionManager.shouldRefreshToken()) {
        try {
          await refreshSession();
        } catch (err) {
          console.error('Auto-refresh failed:', err);
        }
      }
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [autoRefresh, refreshInterval, user]);

  // ============================================================================
  // Authentication Methods
  // ============================================================================

  const login = useCallback(async (provider: string, redirectUri?: string) => {
    try {
      setIsLoading(true);
      setError(null);

      const response = await AuthService.initiateLogin({
        provider,
        redirect_uri: redirectUri || window.location.origin + '/auth/callback',
      });

      // Redirect to SSO provider
      window.location.href = response.authorization_url;
    } catch (err) {
      const errorMessage = err instanceof SSOError ? err.message : 'Login failed';
      setError(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const loginWithLDAP = useCallback(
    async (provider: string, username: string, password: string) => {
      try {
        setIsLoading(true);
        setError(null);

        const result = await AuthService.loginWithLDAP({
          provider,
          username,
          password,
        });

        if (result.mfa_required) {
          // Don't set full auth state yet, MFA required
          setError('MFA verification required');
          return;
        }

        setUser(result.user);
        setSession({
          id: result.session_id,
          user_id: result.user.id,
          email: result.user.email,
          created_at: new Date().toISOString(),
          last_accessed: new Date().toISOString(),
          expires_at: result.expires_at,
          mfa_verified: !result.mfa_required,
          metadata: {},
        });
      } catch (err) {
        const errorMessage = err instanceof SSOError ? err.message : 'Login failed';
        setError(errorMessage);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  const logout = useCallback(async () => {
    try {
      setIsLoading(true);
      await AuthService.logout();
    } catch (err) {
      console.error('Logout error:', err);
    } finally {
      handleLogout();
    }
  }, []);

  const handleLogout = () => {
    setUser(null);
    setSession(null);
    setError(null);
    setIsLoading(false);
  };

  const refreshSession = useCallback(async () => {
    try {
      const result = await AuthService.refreshToken();

      setUser(result.user);
      setSession({
        id: result.session_id,
        user_id: result.user.id,
        email: result.user.email,
        created_at: new Date().toISOString(),
        last_accessed: new Date().toISOString(),
        expires_at: result.expires_at,
        mfa_verified: true,
        metadata: {},
      });
    } catch (err) {
      console.error('Token refresh failed:', err);
      throw err;
    }
  }, []);

  // ============================================================================
  // MFA Methods
  // ============================================================================

  const setupTOTP = useCallback(async (): Promise<TOTPSetup> => {
    try {
      return await AuthService.setupTOTP();
    } catch (err) {
      const errorMessage = err instanceof SSOError ? err.message : 'TOTP setup failed';
      setError(errorMessage);
      throw err;
    }
  }, []);

  const verifyTOTP = useCallback(
    async (code: string, mfaToken?: string) => {
      try {
        setIsLoading(true);
        setError(null);

        if (mfaToken) {
          // MFA verification during login
          const result = await AuthService.verifyTOTP(code, mfaToken);
          setUser(result.user);
          setSession({
            id: result.session_id,
            user_id: result.user.id,
            email: result.user.email,
            created_at: new Date().toISOString(),
            last_accessed: new Date().toISOString(),
            expires_at: result.expires_at,
            mfa_verified: true,
            metadata: {},
          });
        } else {
          // TOTP setup verification
          await AuthService.verifyTOTPSetup(code);
        }
      } catch (err) {
        const errorMessage = err instanceof SSOError ? err.message : 'TOTP verification failed';
        setError(errorMessage);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  const setupWebAuthn = useCallback(async (name: string) => {
    try {
      return await AuthService.startWebAuthnRegistration(name);
    } catch (err) {
      const errorMessage = err instanceof SSOError ? err.message : 'WebAuthn setup failed';
      setError(errorMessage);
      throw err;
    }
  }, []);

  const verifyWebAuthn = useCallback(
    async (credential: any, mfaToken?: string) => {
      try {
        setIsLoading(true);
        setError(null);

        if (mfaToken) {
          // WebAuthn verification during login
          const result = await AuthService.completeWebAuthnAuthentication(credential, mfaToken);
          setUser(result.user);
          setSession({
            id: result.session_id,
            user_id: result.user.id,
            email: result.user.email,
            created_at: new Date().toISOString(),
            last_accessed: new Date().toISOString(),
            expires_at: result.expires_at,
            mfa_verified: true,
            metadata: {},
          });
        } else {
          // WebAuthn registration
          await AuthService.completeWebAuthnRegistration(credential);
        }
      } catch (err) {
        const errorMessage =
          err instanceof SSOError ? err.message : 'WebAuthn verification failed';
        setError(errorMessage);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  const getMFAConfig = useCallback(async (): Promise<UserMFAConfig> => {
    try {
      return await AuthService.getMFAConfig();
    } catch (err) {
      const errorMessage = err instanceof SSOError ? err.message : 'Failed to get MFA config';
      setError(errorMessage);
      throw err;
    }
  }, []);

  // ============================================================================
  // Session Methods
  // ============================================================================

  const validateSession = useCallback(async (): Promise<boolean> => {
    try {
      await AuthService.validateSession();
      return true;
    } catch (err) {
      return false;
    }
  }, []);

  const getSession = useCallback(async (): Promise<Session | null> => {
    try {
      return await AuthService.getSession();
    } catch (err) {
      console.error('Failed to get session:', err);
      return null;
    }
  }, []);

  // ============================================================================
  // Context Value
  // ============================================================================

  const value: AuthContextValue = {
    user,
    session,
    isAuthenticated: !!user && !!session && !SessionManager.isSessionExpired(session),
    isLoading,
    error,
    login,
    loginWithLDAP,
    logout,
    refreshSession,
    setupTOTP,
    verifyTOTP,
    setupWebAuthn,
    verifyWebAuthn,
    getMFAConfig,
    validateSession,
    getSession,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

// ============================================================================
// useAuth Hook
// ============================================================================

export const useAuth = (): AuthContextValue => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

export default AuthProvider;
