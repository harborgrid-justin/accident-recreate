/**
 * useOfflineSync Hook
 * React hook for offline sync state management
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import {
  SyncStatus,
  SyncStats,
  SyncEvent,
  OfflineConfig,
  Priority,
} from '../types';
import { SyncManager } from '../SyncManager';

export interface UseOfflineSyncOptions {
  config?: Partial<OfflineConfig>;
  autoInitialize?: boolean;
}

export interface UseOfflineSyncReturn {
  // State
  status: SyncStatus;
  stats: SyncStats;
  isInitialized: boolean;
  isSyncing: boolean;
  hasPendingOperations: boolean;

  // Actions
  initialize: () => Promise<void>;
  sync: () => Promise<void>;
  pause: () => void;
  resume: () => void;
  clearQueue: () => Promise<void>;

  // Operations
  create: (
    entityType: string,
    entityId: string,
    data: any,
    priority?: Priority
  ) => Promise<string>;
  update: (
    entityType: string,
    entityId: string,
    data: any,
    priority?: Priority
  ) => Promise<string>;
  delete: (
    entityType: string,
    entityId: string,
    priority?: Priority
  ) => Promise<string>;

  // Manager access
  manager: SyncManager | null;
}

export function useOfflineSync(
  options: UseOfflineSyncOptions = {}
): UseOfflineSyncReturn {
  const { config, autoInitialize = true } = options;

  const managerRef = useRef<SyncManager | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [status, setStatus] = useState<SyncStatus>(SyncStatus.Idle);
  const [stats, setStats] = useState<SyncStats>({
    totalSynced: 0,
    pending: 0,
    failed: 0,
    conflicts: 0,
    conflictsResolved: 0,
    bytesUploaded: 0,
    bytesDownloaded: 0,
  });

  // Initialize manager
  const initialize = useCallback(async () => {
    if (managerRef.current) {
      return; // Already initialized
    }

    const manager = new SyncManager(config);
    managerRef.current = manager;

    // Listen to sync events
    manager.addEventListener((event: SyncEvent) => {
      switch (event.type) {
        case 'started':
          setStatus(SyncStatus.Syncing);
          break;

        case 'progress':
          if (event.data?.stats) {
            setStats(event.data.stats);
          }
          break;

        case 'completed':
          setStatus(SyncStatus.Completed);
          setTimeout(() => setStatus(SyncStatus.Idle), 1000);
          break;

        case 'failed':
          setStatus(SyncStatus.Failed);
          setTimeout(() => setStatus(SyncStatus.Idle), 3000);
          break;

        case 'conflict':
          // Handle conflict event
          break;
      }
    });

    await manager.initialize();
    setStats(manager.getStats());
    setStatus(manager.getStatus());
    setIsInitialized(true);
  }, [config]);

  // Auto-initialize
  useEffect(() => {
    if (autoInitialize) {
      initialize();
    }

    return () => {
      if (managerRef.current) {
        managerRef.current.shutdown();
        managerRef.current = null;
      }
    };
  }, [autoInitialize, initialize]);

  // Sync
  const sync = useCallback(async () => {
    if (!managerRef.current) {
      throw new Error('Sync manager not initialized');
    }
    await managerRef.current.sync();
  }, []);

  // Pause
  const pause = useCallback(() => {
    if (!managerRef.current) {
      throw new Error('Sync manager not initialized');
    }
    managerRef.current.pause();
    setStatus(SyncStatus.Paused);
  }, []);

  // Resume
  const resume = useCallback(() => {
    if (!managerRef.current) {
      throw new Error('Sync manager not initialized');
    }
    managerRef.current.resume();
    setStatus(SyncStatus.Idle);
  }, []);

  // Clear queue
  const clearQueue = useCallback(async () => {
    if (!managerRef.current) {
      throw new Error('Sync manager not initialized');
    }
    await managerRef.current.clearQueue();
    setStats(managerRef.current.getStats());
  }, []);

  // Create operation
  const create = useCallback(
    async (
      entityType: string,
      entityId: string,
      data: any,
      priority: Priority = Priority.Normal
    ) => {
      if (!managerRef.current) {
        throw new Error('Sync manager not initialized');
      }
      const operationId = await managerRef.current.create(
        entityType,
        entityId,
        data,
        priority
      );
      setStats(managerRef.current.getStats());
      return operationId;
    },
    []
  );

  // Update operation
  const update = useCallback(
    async (
      entityType: string,
      entityId: string,
      data: any,
      priority: Priority = Priority.Normal
    ) => {
      if (!managerRef.current) {
        throw new Error('Sync manager not initialized');
      }
      const operationId = await managerRef.current.update(
        entityType,
        entityId,
        data,
        priority
      );
      setStats(managerRef.current.getStats());
      return operationId;
    },
    []
  );

  // Delete operation
  const deleteOp = useCallback(
    async (
      entityType: string,
      entityId: string,
      priority: Priority = Priority.Normal
    ) => {
      if (!managerRef.current) {
        throw new Error('Sync manager not initialized');
      }
      const operationId = await managerRef.current.delete(
        entityType,
        entityId,
        priority
      );
      setStats(managerRef.current.getStats());
      return operationId;
    },
    []
  );

  return {
    // State
    status,
    stats,
    isInitialized,
    isSyncing: status === SyncStatus.Syncing,
    hasPendingOperations: stats.pending > 0,

    // Actions
    initialize,
    sync,
    pause,
    resume,
    clearQueue,

    // Operations
    create,
    update,
    delete: deleteOp,

    // Manager
    manager: managerRef.current,
  };
}
