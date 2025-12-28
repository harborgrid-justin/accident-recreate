/**
 * AccuScene Offline Sync
 *
 * Comprehensive offline synchronization system for AccuScene Enterprise
 * with CRDT-based conflict resolution and distributed versioning.
 *
 * @module offline
 */

// Core exports
export * from './types';
export { SyncManager } from './SyncManager';
export { OperationQueue } from './OperationQueue';
export { NetworkDetector } from './NetworkDetector';
export { ConflictResolver } from './ConflictResolver';
export { IndexedDBStore } from './IndexedDBStore';

// React hooks
export { useOfflineSync } from './hooks/useOfflineSync';
export { useNetworkStatus } from './hooks/useNetworkStatus';

// React components
export { SyncStatusIndicator } from './components/SyncStatusIndicator';
export { OfflineBanner } from './components/OfflineBanner';
export { ConflictModal } from './components/ConflictModal';

// Version
export const VERSION = '0.2.5';
