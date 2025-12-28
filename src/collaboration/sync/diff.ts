/**
 * AccuScene Enterprise - Differential Synchronization
 * v0.2.0
 *
 * Differential sync for efficient state synchronization
 */

import { Operation, VectorClock, DiffResult, SyncRequest, SyncResponse } from '../types';
import { VectorClockManager } from './vector-clock';

/**
 * Differential Synchronization Engine
 */
export class DifferentialSync {
  /**
   * Calculate differences between two operation sets
   */
  static diff(
    localOps: Operation[],
    remoteOps: Operation[],
    localClock: VectorClock,
    remoteClock: VectorClock
  ): DiffResult {
    const localOpIds = new Set(localOps.map(op => op.id));
    const remoteOpIds = new Set(remoteOps.map(op => op.id));

    const missing: Operation[] = [];
    const conflicting: Operation[] = [];

    // Find operations in remote but not in local
    for (const remoteOp of remoteOps) {
      if (!localOpIds.has(remoteOp.id)) {
        // Check if this operation should be included based on vector clocks
        const remoteCounter = remoteOp.vectorClock[remoteOp.clientId] || 0;
        const localCounter = localClock[remoteOp.clientId] || 0;

        if (remoteCounter > localCounter) {
          missing.push(remoteOp);
        }
      }
    }

    // Find conflicting operations (same timestamp range but different data)
    for (const remoteOp of remoteOps) {
      const localOp = localOps.find(op => op.id === remoteOp.id);

      if (localOp && !this.areOperationsEqual(localOp, remoteOp)) {
        conflicting.push(remoteOp);
      }
    }

    return { missing, conflicting };
  }

  /**
   * Calculate operations needed to sync from one state to another
   */
  static calculateSyncOps(
    localOps: Operation[],
    localClock: VectorClock,
    remoteClock: VectorClock
  ): Operation[] {
    const syncOps: Operation[] = [];

    // Find operations that remote doesn't have yet
    for (const op of localOps) {
      const opCounter = op.vectorClock[op.clientId] || 0;
      const remoteCounter = remoteClock[op.clientId] || 0;

      if (opCounter > remoteCounter) {
        syncOps.push(op);
      }
    }

    return syncOps.sort((a, b) => a.timestamp.counter - b.timestamp.counter);
  }

  /**
   * Process a sync request and generate a response
   */
  static processSyncRequest(
    request: SyncRequest,
    localOps: Operation[],
    localClock: VectorClock
  ): SyncResponse {
    const operations = this.calculateSyncOps(localOps, localClock, request.vectorClock);

    return {
      operations,
      vectorClock: localClock,
    };
  }

  /**
   * Apply sync response to local state
   */
  static applySyncResponse(
    response: SyncResponse,
    localOps: Operation[],
    localClock: VectorClock
  ): {
    operations: Operation[];
    vectorClock: VectorClock;
  } {
    const localOpIds = new Set(localOps.map(op => op.id));
    const newOps: Operation[] = [];

    // Add new operations
    for (const op of response.operations) {
      if (!localOpIds.has(op.id)) {
        newOps.push(op);
      }
    }

    // Merge operations and sort by timestamp
    const mergedOps = [...localOps, ...newOps].sort(
      (a, b) => a.timestamp.counter - b.timestamp.counter
    );

    // Merge vector clocks
    const mergedClock = VectorClockManager.merge(localClock, response.vectorClock);

    return {
      operations: mergedOps,
      vectorClock: mergedClock,
    };
  }

  /**
   * Check if two operations are equal
   */
  private static areOperationsEqual(op1: Operation, op2: Operation): boolean {
    return (
      op1.id === op2.id &&
      op1.type === op2.type &&
      op1.clientId === op2.clientId &&
      op1.timestamp.counter === op2.timestamp.counter &&
      JSON.stringify(op1.data) === JSON.stringify(op2.data)
    );
  }

  /**
   * Partition operations by vector clock range
   */
  static partitionOperations(
    operations: Operation[],
    startClock: VectorClock,
    endClock: VectorClock
  ): Operation[] {
    return operations.filter(op => {
      const opCounter = op.vectorClock[op.clientId] || 0;
      const startCounter = startClock[op.clientId] || 0;
      const endCounter = endClock[op.clientId] || 0;

      return opCounter > startCounter && opCounter <= endCounter;
    });
  }

  /**
   * Calculate the delta between two vector clocks
   */
  static calculateDelta(from: VectorClock, to: VectorClock): VectorClock {
    return VectorClockManager.diff(from, to);
  }

  /**
   * Optimize operation list by removing redundant operations
   */
  static optimize(operations: Operation[]): Operation[] {
    const optimized: Operation[] = [];
    const seen = new Map<string, Operation>();

    // Process operations in reverse order to keep latest
    for (let i = operations.length - 1; i >= 0; i--) {
      const op = operations[i];
      const key = this.getOperationKey(op);

      if (!seen.has(key)) {
        seen.set(key, op);
      }
    }

    // Convert back to array and sort by timestamp
    for (const op of seen.values()) {
      optimized.push(op);
    }

    return optimized.sort((a, b) => a.timestamp.counter - b.timestamp.counter);
  }

  /**
   * Get a unique key for an operation based on its target
   */
  private static getOperationKey(op: Operation): string {
    // For object operations, use the object ID
    if (op.type.startsWith('object:')) {
      return `object:${(op.data as any).objectId}`;
    }

    // For annotation operations, use the annotation ID
    if (op.type.startsWith('annotation:')) {
      return `annotation:${(op.data as any).annotationId}`;
    }

    // For measurement operations, use the measurement ID
    if (op.type.startsWith('measurement:')) {
      return `measurement:${(op.data as any).measurementId}`;
    }

    // Default to operation ID
    return op.id;
  }

  /**
   * Calculate bandwidth estimate for sync
   */
  static estimateSyncSize(operations: Operation[]): number {
    return operations.reduce((total, op) => {
      // Rough estimate: operation metadata + data size
      const metadata = 200; // bytes for ID, type, timestamp, etc.
      const data = JSON.stringify(op.data).length;
      return total + metadata + data;
    }, 0);
  }

  /**
   * Batch operations for efficient transmission
   */
  static batchOperations(
    operations: Operation[],
    maxBatchSize: number = 100,
    maxBatchBytes: number = 100000
  ): Operation[][] {
    const batches: Operation[][] = [];
    let currentBatch: Operation[] = [];
    let currentSize = 0;

    for (const op of operations) {
      const opSize = JSON.stringify(op).length;

      if (
        currentBatch.length >= maxBatchSize ||
        currentSize + opSize > maxBatchBytes
      ) {
        if (currentBatch.length > 0) {
          batches.push(currentBatch);
        }
        currentBatch = [op];
        currentSize = opSize;
      } else {
        currentBatch.push(op);
        currentSize += opSize;
      }
    }

    if (currentBatch.length > 0) {
      batches.push(currentBatch);
    }

    return batches;
  }

  /**
   * Compress operations by removing redundant information
   */
  static compress(operations: Operation[]): Operation[] {
    // Remove operations that are superseded by later operations
    const compressed: Operation[] = [];
    const latestByTarget = new Map<string, Operation>();

    for (const op of operations) {
      const key = this.getOperationKey(op);
      const existing = latestByTarget.get(key);

      if (
        !existing ||
        op.timestamp.counter > existing.timestamp.counter
      ) {
        latestByTarget.set(key, op);
      }
    }

    return Array.from(latestByTarget.values()).sort(
      (a, b) => a.timestamp.counter - b.timestamp.counter
    );
  }
}
