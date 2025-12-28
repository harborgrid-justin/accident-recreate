/**
 * Offline Sync Type Definitions
 * TypeScript interfaces for offline synchronization system
 */

export enum SyncStatus {
  Idle = 'idle',
  Syncing = 'syncing',
  Completed = 'completed',
  Failed = 'failed',
  Paused = 'paused',
  WaitingForNetwork = 'waiting_for_network',
}

export enum NetworkState {
  Online = 'online',
  Degraded = 'degraded',
  Offline = 'offline',
  Unknown = 'unknown',
}

export enum OperationType {
  Create = 'create',
  Update = 'update',
  Delete = 'delete',
  Patch = 'patch',
}

export enum Priority {
  Low = 0,
  Normal = 1,
  High = 2,
  Critical = 3,
}

export enum ConflictResolution {
  LastWriteWins = 'last_write_wins',
  FirstWriteWins = 'first_write_wins',
  ServerWins = 'server_wins',
  ClientWins = 'client_wins',
  Manual = 'manual',
  OperationalTransform = 'operational_transform',
  Custom = 'custom',
}

/** Vector clock for distributed versioning */
export interface VectorClock {
  clocks: Record<string, number>;
}

/** Version information */
export interface Version {
  clock: VectorClock;
  nodeId: string;
  timestamp: string;
  contentHash: string;
}

/** Sync operation */
export interface SyncOperation {
  id: string;
  entityId: string;
  entityType: string;
  operationType: OperationType;
  data: any;
  version: Version;
  priority: Priority;
  queuedAt: string;
  retryCount: number;
  lastError?: string;
  dependencies: string[];
  tags: string[];
}

/** Conflict between local and remote versions */
export interface Conflict {
  entityId: string;
  entityType: string;
  localVersion: Version;
  localData: any;
  remoteVersion: Version;
  remoteData: any;
  detectedAt: string;
}

/** Resolution result */
export interface ResolutionResult {
  data: any;
  version: Version;
  strategy: ConflictResolution;
  manual: boolean;
  metadata: Record<string, any>;
}

/** Sync statistics */
export interface SyncStats {
  totalSynced: number;
  pending: number;
  failed: number;
  conflicts: number;
  conflictsResolved: number;
  bytesUploaded: number;
  bytesDownloaded: number;
  lastSync?: string;
  lastSyncDurationMs?: number;
}

/** Network quality metrics */
export interface NetworkQuality {
  latencyMs: number;
  bandwidthBps: number;
  packetLoss: number;
  score: number;
}

/** Offline configuration */
export interface OfflineConfig {
  nodeId: string;
  autoSync: boolean;
  syncIntervalMs: number;
  batchSize: number;
  maxPendingOperations: number;
  deltaEncoding: boolean;
  compression: boolean;
  conflictResolution: ConflictResolution;
  apiEndpoint: string;
  retryAttempts: number;
  retryDelayMs: number;
}

/** IndexedDB schema */
export interface StorageRecord {
  entityId: string;
  entityType: string;
  data: any;
  version: Version;
  createdAt: string;
  updatedAt: string;
  deleted: boolean;
}

/** Sync event */
export interface SyncEvent {
  type: 'started' | 'progress' | 'completed' | 'failed' | 'conflict';
  timestamp: string;
  data?: any;
  error?: string;
}

/** Offline state */
export interface OfflineState {
  isOnline: boolean;
  networkState: NetworkState;
  networkQuality?: NetworkQuality;
  syncStatus: SyncStatus;
  syncStats: SyncStats;
  pendingOperations: number;
  conflicts: Conflict[];
}

/** Sync callback */
export type SyncCallback = (event: SyncEvent) => void;

/** Conflict resolver function */
export type ConflictResolverFn = (conflict: Conflict) => Promise<ResolutionResult>;
