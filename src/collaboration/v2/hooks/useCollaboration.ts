/**
 * AccuScene Enterprise v0.3.0 - useCollaboration Hook
 *
 * Main React hook for collaboration features
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { CollaborationEngine } from '../CollaborationEngine';
import {
  User,
  SessionId,
  Operation,
  CollaborationConfig,
  ConnectionStatus,
  PerformanceMetrics,
  SessionSettings
} from '../types';

interface UseCollaborationOptions {
  config: CollaborationConfig;
  user: User;
  sessionId: SessionId;
  settings?: Partial<SessionSettings>;
  autoConnect?: boolean;
}

export const useCollaboration = ({
  config,
  user,
  sessionId,
  settings,
  autoConnect = true
}: UseCollaborationOptions) => {
  const engineRef = useRef<CollaborationEngine | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>(ConnectionStatus.DISCONNECTED);
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null);
  const [error, setError] = useState<Error | null>(null);

  // Initialize engine
  useEffect(() => {
    if (!engineRef.current) {
      engineRef.current = new CollaborationEngine(config);

      // Set up event listeners
      engineRef.current.on('stateChange', ({ to }) => {
        console.log('Collaboration state changed:', to);
      });

      engineRef.current.on('collaborationEvent', (event) => {
        console.log('Collaboration event:', event);
      });

      engineRef.current.on('metricsUpdate', (newMetrics) => {
        setMetrics(newMetrics);
      });
    }

    return () => {
      if (engineRef.current) {
        engineRef.current.removeAllListeners();
      }
    };
  }, [config]);

  // Auto-connect
  useEffect(() => {
    if (autoConnect && engineRef.current && !isInitialized) {
      initialize();
    }
  }, [autoConnect, isInitialized]);

  // Track connection status
  useEffect(() => {
    if (engineRef.current) {
      const updateStatus = () => {
        setConnectionStatus(engineRef.current!.getConnectionStatus());
      };

      const interval = setInterval(updateStatus, 1000);
      return () => clearInterval(interval);
    }
  }, [isInitialized]);

  const initialize = useCallback(async () => {
    if (!engineRef.current || isInitialized) return;

    try {
      await engineRef.current.initialize(user, sessionId, settings);
      setIsInitialized(true);
      setError(null);
    } catch (err) {
      setError(err as Error);
      console.error('Failed to initialize collaboration:', err);
    }
  }, [user, sessionId, settings, isInitialized]);

  const shutdown = useCallback(async () => {
    if (!engineRef.current || !isInitialized) return;

    try {
      await engineRef.current.shutdown();
      setIsInitialized(false);
    } catch (err) {
      setError(err as Error);
      console.error('Failed to shutdown collaboration:', err);
    }
  }, [isInitialized]);

  const applyOperation = useCallback(async (operation: Operation) => {
    if (!engineRef.current || !isInitialized) {
      throw new Error('Collaboration not initialized');
    }

    try {
      await engineRef.current.applyOperation(operation);
    } catch (err) {
      setError(err as Error);
      throw err;
    }
  }, [isInitialized]);

  return {
    engine: engineRef.current,
    isInitialized,
    connectionStatus,
    metrics,
    error,
    initialize,
    shutdown,
    applyOperation,
    presenceManager: engineRef.current?.getPresenceManager(),
    versionControl: engineRef.current?.getVersionControl(),
    conflictResolver: engineRef.current?.getConflictResolver(),
    permissionManager: engineRef.current?.getPermissionManager(),
    auditLogger: engineRef.current?.getAuditLogger()
  };
};
