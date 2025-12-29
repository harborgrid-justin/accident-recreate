/**
 * AccuScene Enterprise v0.3.0 - usePresence Hook
 *
 * React hook for user presence tracking
 */

import { useState, useEffect, useCallback } from 'react';
import { PresenceManager } from '../PresenceManager';
import {
  Presence,
  CursorPosition,
  Selection,
  Viewport,
  UserStatus,
  UserId
} from '../types';

export const usePresence = (presenceManager: PresenceManager | undefined) => {
  const [presences, setPresences] = useState<Presence[]>([]);
  const [activeUsers, setActiveUsers] = useState<number>(0);

  useEffect(() => {
    if (!presenceManager) return;

    const updatePresences = () => {
      setPresences(presenceManager.getAllPresences());
      setActiveUsers(presenceManager.getActiveUserCount());
    };

    // Initial load
    updatePresences();

    // Listen for updates
    presenceManager.on('presenceUpdate', updatePresences);
    presenceManager.on('userJoined', updatePresences);
    presenceManager.on('userLeft', updatePresences);

    return () => {
      presenceManager.off('presenceUpdate', updatePresences);
      presenceManager.off('userJoined', updatePresences);
      presenceManager.off('userLeft', updatePresences);
    };
  }, [presenceManager]);

  const updateCursor = useCallback((cursor: CursorPosition | null, userId: UserId) => {
    if (!presenceManager) return;
    presenceManager.updateCursor(userId, cursor);
  }, [presenceManager]);

  const updateSelection = useCallback((selection: Selection | null, userId: UserId) => {
    if (!presenceManager) return;
    presenceManager.updateSelection(userId, selection);
  }, [presenceManager]);

  const updateViewport = useCallback((viewport: Viewport | null, userId: UserId) => {
    if (!presenceManager) return;
    presenceManager.updateViewport(userId, viewport);
  }, [presenceManager]);

  const setTyping = useCallback((userId: UserId, isTyping: boolean) => {
    if (!presenceManager) return;
    presenceManager.setTyping(userId, isTyping);
  }, [presenceManager]);

  const getPresence = useCallback((userId: UserId): Presence | null => {
    if (!presenceManager) return null;
    return presenceManager.getPresence(userId);
  }, [presenceManager]);

  return {
    presences,
    activeUsers,
    updateCursor,
    updateSelection,
    updateViewport,
    setTyping,
    getPresence
  };
};
