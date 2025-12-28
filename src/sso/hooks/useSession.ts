/**
 * useSession Hook
 * Provides session state and management utilities
 */

import { useState, useEffect } from 'react';
import { useAuth } from '../AuthProvider';
import { Session } from '../types';
import SessionManager from '../SessionManager';

export interface UseSessionReturn {
  session: Session | null;
  isValid: boolean;
  isExpired: boolean;
  expiresIn: number | null;
  shouldRefresh: boolean;
  refresh: () => Promise<void>;
}

export const useSession = (): UseSessionReturn => {
  const { session, refreshSession } = useAuth();
  const [expiresIn, setExpiresIn] = useState<number | null>(null);

  useEffect(() => {
    if (!session) {
      setExpiresIn(null);
      return;
    }

    const updateExpiresIn = () => {
      const expiresAt = new Date(session.expires_at).getTime();
      const now = Date.now();
      const remaining = Math.max(0, expiresAt - now);
      setExpiresIn(remaining);
    };

    updateExpiresIn();
    const interval = setInterval(updateExpiresIn, 1000);

    return () => clearInterval(interval);
  }, [session]);

  const isExpired = session ? SessionManager.isSessionExpired(session) : false;
  const isValid = session ? !isExpired : false;
  const shouldRefresh = expiresIn !== null && expiresIn < 5 * 60 * 1000; // Less than 5 minutes

  return {
    session,
    isValid,
    isExpired,
    expiresIn,
    shouldRefresh,
    refresh: refreshSession,
  };
};

export default useSession;
