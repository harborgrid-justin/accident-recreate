/**
 * Sync Manager
 * Main orchestrator for offline sync operations
 */

import { v4 as uuidv4 } from 'uuid';
import {
  OfflineConfig,
  SyncStatus,
  SyncStats,
  SyncEvent,
  SyncCallback,
  OperationType,
  Priority,
  Version,
  VectorClock,
  Conflict,
} from './types';
import { IndexedDBStore } from './IndexedDBStore';
import { OperationQueue } from './OperationQueue';
import { NetworkDetector } from './NetworkDetector';
import { ConflictResolver } from './ConflictResolver';

export class SyncManager {
  private config: OfflineConfig;
  private store: IndexedDBStore;
  private queue: OperationQueue;
  private network: NetworkDetector;
  private resolver: ConflictResolver;
  private status: SyncStatus = SyncStatus.Idle;
  private stats: SyncStats;
  private callbacks: Set<SyncCallback> = new Set();
  private syncInterval: NodeJS.Timeout | null = null;
  private vectorClock: VectorClock;

  constructor(config: Partial<OfflineConfig> = {}) {
    this.config = {
      nodeId: config.nodeId || uuidv4(),
      autoSync: config.autoSync ?? true,
      syncIntervalMs: config.syncIntervalMs || 30000,
      batchSize: config.batchSize || 100,
      maxPendingOperations: config.maxPendingOperations || 10000,
      deltaEncoding: config.deltaEncoding ?? true,
      compression: config.compression ?? true,
      conflictResolution: config.conflictResolution || 'last_write_wins',
      apiEndpoint: config.apiEndpoint || 'https://api.accuscene.com',
      retryAttempts: config.retryAttempts || 5,
      retryDelayMs: config.retryDelayMs || 1000,
    };

    this.store = new IndexedDBStore();
    this.queue = new OperationQueue(this.store, this.config.maxPendingOperations);
    this.network = new NetworkDetector(this.config.apiEndpoint);
    this.resolver = new ConflictResolver(this.config.conflictResolution);

    this.stats = {
      totalSynced: 0,
      pending: 0,
      failed: 0,
      conflicts: 0,
      conflictsResolved: 0,
      bytesUploaded: 0,
      bytesDownloaded: 0,
    };

    this.vectorClock = { clocks: {} };
    this.vectorClock.clocks[this.config.nodeId] = 0;
  }

  /**
   * Initialize sync manager
   */
  async initialize(): Promise<void> {
    await this.queue.load();
    this.stats.pending = this.queue.size();

    // Start network monitoring
    this.network.start();

    // Setup auto-sync
    if (this.config.autoSync) {
      this.startAutoSync();
    }

    // Listen to network changes
    this.network.addListener((state) => {
      if (state === 'online' && this.status === SyncStatus.WaitingForNetwork) {
        this.sync();
      }
    });
  }

  /**
   * Start auto-sync interval
   */
  private startAutoSync(): void {
    if (this.syncInterval) {
      clearInterval(this.syncInterval);
    }

    this.syncInterval = setInterval(() => {
      if (this.network.isOnline() && this.status === SyncStatus.Idle) {
        this.sync();
      }
    }, this.config.syncIntervalMs);
  }

  /**
   * Stop auto-sync
   */
  stopAutoSync(): void {
    if (this.syncInterval) {
      clearInterval(this.syncInterval);
      this.syncInterval = null;
    }
  }

  /**
   * Enqueue a create operation
   */
  async create(
    entityType: string,
    entityId: string,
    data: any,
    priority: Priority = Priority.Normal
  ): Promise<string> {
    const version = this.createVersion(data);

    const operationId = await this.queue.enqueue(
      entityId,
      entityType,
      OperationType.Create,
      data,
      version,
      priority
    );

    this.stats.pending = this.queue.size();

    // Store locally
    await this.store.putRecord({
      entityId,
      entityType,
      data,
      version,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      deleted: false,
    });

    // Trigger sync if online
    if (this.network.isOnline() && this.config.autoSync) {
      this.sync();
    }

    return operationId;
  }

  /**
   * Enqueue an update operation
   */
  async update(
    entityType: string,
    entityId: string,
    data: any,
    priority: Priority = Priority.Normal
  ): Promise<string> {
    const version = this.createVersion(data);

    const operationId = await this.queue.enqueue(
      entityId,
      entityType,
      OperationType.Update,
      data,
      version,
      priority
    );

    this.stats.pending = this.queue.size();

    // Update locally
    await this.store.putRecord({
      entityId,
      entityType,
      data,
      version,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      deleted: false,
    });

    // Trigger sync if online
    if (this.network.isOnline() && this.config.autoSync) {
      this.sync();
    }

    return operationId;
  }

  /**
   * Enqueue a delete operation
   */
  async delete(
    entityType: string,
    entityId: string,
    priority: Priority = Priority.Normal
  ): Promise<string> {
    const version = this.createVersion({});

    const operationId = await this.queue.enqueue(
      entityId,
      entityType,
      OperationType.Delete,
      {},
      version,
      priority
    );

    this.stats.pending = this.queue.size();

    // Mark as deleted locally
    await this.store.deleteRecord(entityId, entityType);

    // Trigger sync if online
    if (this.network.isOnline() && this.config.autoSync) {
      this.sync();
    }

    return operationId;
  }

  /**
   * Start sync process
   */
  async sync(): Promise<void> {
    if (!this.network.isOnline()) {
      this.status = SyncStatus.WaitingForNetwork;
      this.emitEvent({ type: 'started', timestamp: new Date().toISOString() });
      return;
    }

    if (this.status === SyncStatus.Syncing) {
      return; // Already syncing
    }

    this.status = SyncStatus.Syncing;
    this.emitEvent({ type: 'started', timestamp: new Date().toISOString() });

    const startTime = Date.now();

    try {
      await this.syncLoop();

      this.status = SyncStatus.Completed;
      const duration = Date.now() - startTime;

      this.stats.lastSync = new Date().toISOString();
      this.stats.lastSyncDurationMs = duration;

      this.emitEvent({
        type: 'completed',
        timestamp: new Date().toISOString(),
        data: { duration },
      });
    } catch (error) {
      this.status = SyncStatus.Failed;
      this.emitEvent({
        type: 'failed',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  }

  /**
   * Main sync loop
   */
  private async syncLoop(): Promise<void> {
    const batchSize = this.config.batchSize;
    const batch = [];

    while (!this.queue.isEmpty() && batch.length < batchSize) {
      const operation = this.queue.dequeue();
      if (operation) {
        batch.push(operation);
      }
    }

    if (batch.length === 0) {
      return;
    }

    // Process batch
    for (const operation of batch) {
      try {
        await this.syncOperation(operation);
        await this.queue.markCompleted(operation.id);
        this.stats.totalSynced++;
        this.stats.pending = this.queue.size();

        this.emitEvent({
          type: 'progress',
          timestamp: new Date().toISOString(),
          data: { operation, stats: this.stats },
        });
      } catch (error) {
        if (error instanceof Error && error.message.includes('conflict')) {
          this.stats.conflicts++;
          this.emitEvent({
            type: 'conflict',
            timestamp: new Date().toISOString(),
            data: { operation },
          });
        } else {
          this.stats.failed++;
          operation.retryCount++;
          operation.lastError = error instanceof Error ? error.message : 'Unknown error';

          if (operation.retryCount < this.config.retryAttempts) {
            await this.queue.update(operation);
          }
        }
      }
    }

    // Continue if more operations
    if (!this.queue.isEmpty()) {
      await this.syncLoop();
    }
  }

  /**
   * Sync a single operation
   */
  private async syncOperation(operation: any): Promise<void> {
    // Simulate API call (replace with actual implementation)
    const endpoint = `${this.config.apiEndpoint}/sync/${operation.entityType}/${operation.entityId}`;

    const response = await fetch(endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(operation),
    });

    if (!response.ok) {
      throw new Error(`Sync failed: ${response.statusText}`);
    }

    const result = await response.json();

    // Check for conflicts
    if (result.conflict) {
      const conflict: Conflict = {
        entityId: operation.entityId,
        entityType: operation.entityType,
        localVersion: operation.version,
        localData: operation.data,
        remoteVersion: result.version,
        remoteData: result.data,
        detectedAt: new Date().toISOString(),
      };

      const resolution = await this.resolver.resolve(conflict);
      this.stats.conflictsResolved++;

      // Update local with resolved data
      await this.store.putRecord({
        entityId: operation.entityId,
        entityType: operation.entityType,
        data: resolution.data,
        version: resolution.version,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        deleted: false,
      });
    }

    this.stats.bytesUploaded += JSON.stringify(operation).length;
    this.stats.bytesDownloaded += JSON.stringify(result).length;
  }

  /**
   * Create a new version
   */
  private createVersion(data: any): Version {
    // Increment vector clock
    this.vectorClock.clocks[this.config.nodeId]++;

    return {
      clock: { ...this.vectorClock },
      nodeId: this.config.nodeId,
      timestamp: new Date().toISOString(),
      contentHash: this.hashData(data),
    };
  }

  /**
   * Hash data
   */
  private hashData(data: any): string {
    const str = JSON.stringify(data);
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }
    return Math.abs(hash).toString(16);
  }

  /**
   * Emit sync event
   */
  private emitEvent(event: SyncEvent): void {
    this.callbacks.forEach((callback) => callback(event));
  }

  /**
   * Add event listener
   */
  addEventListener(callback: SyncCallback): () => void {
    this.callbacks.add(callback);
    return () => this.callbacks.delete(callback);
  }

  /**
   * Get current status
   */
  getStatus(): SyncStatus {
    return this.status;
  }

  /**
   * Get statistics
   */
  getStats(): SyncStats {
    return { ...this.stats };
  }

  /**
   * Get network detector
   */
  getNetworkDetector(): NetworkDetector {
    return this.network;
  }

  /**
   * Get conflict resolver
   */
  getConflictResolver(): ConflictResolver {
    return this.resolver;
  }

  /**
   * Pause syncing
   */
  pause(): void {
    this.status = SyncStatus.Paused;
    this.stopAutoSync();
  }

  /**
   * Resume syncing
   */
  resume(): void {
    if (this.status === SyncStatus.Paused) {
      this.status = SyncStatus.Idle;
      if (this.config.autoSync) {
        this.startAutoSync();
      }
      this.sync();
    }
  }

  /**
   * Clear all pending operations
   */
  async clearQueue(): Promise<void> {
    await this.queue.clear();
    this.stats.pending = 0;
  }

  /**
   * Shutdown sync manager
   */
  shutdown(): void {
    this.stopAutoSync();
    this.network.stop();
    this.store.close();
  }
}
