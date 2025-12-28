/**
 * AccuScene Enterprise - State Snapshots
 * v0.2.0
 *
 * Create and manage state snapshots for efficient synchronization
 */

import { createHash } from 'crypto';
import { RoomId, StateSnapshot, SceneState, VectorClock } from '../types';
import { v4 as uuidv4 } from 'uuid';

/**
 * Snapshot Manager
 */
export class SnapshotManager {
  private snapshots: Map<RoomId, StateSnapshot[]>;
  private maxSnapshots: number;
  private snapshotInterval: number;
  private lastSnapshot: Map<RoomId, number>;

  constructor(maxSnapshots: number = 10, snapshotInterval: number = 300000) {
    this.snapshots = new Map();
    this.maxSnapshots = maxSnapshots;
    this.snapshotInterval = snapshotInterval;
    this.lastSnapshot = new Map();
  }

  /**
   * Create a snapshot of the current state
   */
  createSnapshot(
    roomId: RoomId,
    state: SceneState,
    vectorClock: VectorClock
  ): StateSnapshot {
    const now = Date.now();

    const snapshot: StateSnapshot = {
      id: uuidv4(),
      roomId,
      vectorClock: { ...vectorClock },
      state: this.cloneState(state),
      timestamp: now,
      checksum: this.calculateChecksum(state),
    };

    // Store snapshot
    if (!this.snapshots.has(roomId)) {
      this.snapshots.set(roomId, []);
    }

    const roomSnapshots = this.snapshots.get(roomId)!;
    roomSnapshots.push(snapshot);

    // Trim old snapshots
    if (roomSnapshots.length > this.maxSnapshots) {
      roomSnapshots.shift();
    }

    this.lastSnapshot.set(roomId, now);

    return snapshot;
  }

  /**
   * Get the latest snapshot for a room
   */
  getLatestSnapshot(roomId: RoomId): StateSnapshot | undefined {
    const roomSnapshots = this.snapshots.get(roomId);
    if (!roomSnapshots || roomSnapshots.length === 0) {
      return undefined;
    }

    return roomSnapshots[roomSnapshots.length - 1];
  }

  /**
   * Get snapshot by ID
   */
  getSnapshot(roomId: RoomId, snapshotId: string): StateSnapshot | undefined {
    const roomSnapshots = this.snapshots.get(roomId);
    return roomSnapshots?.find(s => s.id === snapshotId);
  }

  /**
   * Get all snapshots for a room
   */
  getSnapshots(roomId: RoomId): StateSnapshot[] {
    return this.snapshots.get(roomId) || [];
  }

  /**
   * Get snapshot at or before a timestamp
   */
  getSnapshotAt(roomId: RoomId, timestamp: number): StateSnapshot | undefined {
    const roomSnapshots = this.snapshots.get(roomId);
    if (!roomSnapshots) {
      return undefined;
    }

    // Find the latest snapshot before or at the timestamp
    let result: StateSnapshot | undefined;

    for (const snapshot of roomSnapshots) {
      if (snapshot.timestamp <= timestamp) {
        if (!result || snapshot.timestamp > result.timestamp) {
          result = snapshot;
        }
      }
    }

    return result;
  }

  /**
   * Check if it's time to create a snapshot
   */
  shouldCreateSnapshot(roomId: RoomId): boolean {
    const lastTime = this.lastSnapshot.get(roomId);
    if (!lastTime) {
      return true;
    }

    return Date.now() - lastTime >= this.snapshotInterval;
  }

  /**
   * Verify snapshot integrity
   */
  verifySnapshot(snapshot: StateSnapshot): boolean {
    const checksum = this.calculateChecksum(snapshot.state);
    return checksum === snapshot.checksum;
  }

  /**
   * Calculate checksum for state
   */
  private calculateChecksum(state: SceneState): string {
    const data = JSON.stringify({
      objects: Array.from(state.objects.entries()).sort(),
      annotations: Array.from(state.annotations.entries()).sort(),
      measurements: Array.from(state.measurements.entries()).sort(),
      properties: Object.entries(state.properties).sort(),
    });

    return createHash('sha256').update(data).digest('hex');
  }

  /**
   * Clone state deeply
   */
  private cloneState(state: SceneState): SceneState {
    return {
      objects: new Map(
        Array.from(state.objects.entries()).map(([k, v]) => [k, { ...v }])
      ),
      annotations: new Map(
        Array.from(state.annotations.entries()).map(([k, v]) => [k, { ...v }])
      ),
      measurements: new Map(
        Array.from(state.measurements.entries()).map(([k, v]) => [k, { ...v }])
      ),
      properties: { ...state.properties },
    };
  }

  /**
   * Delete snapshots for a room
   */
  deleteRoomSnapshots(roomId: RoomId): void {
    this.snapshots.delete(roomId);
    this.lastSnapshot.delete(roomId);
  }

  /**
   * Delete old snapshots
   */
  pruneSnapshots(maxAge: number = 86400000): number {
    const now = Date.now();
    let pruned = 0;

    for (const [roomId, roomSnapshots] of this.snapshots.entries()) {
      const before = roomSnapshots.length;

      // Keep at least one snapshot
      const filtered = roomSnapshots.filter(
        (s, i) =>
          now - s.timestamp < maxAge || i === roomSnapshots.length - 1
      );

      this.snapshots.set(roomId, filtered);
      pruned += before - filtered.length;
    }

    return pruned;
  }

  /**
   * Get storage size estimate
   */
  getStorageSize(): number {
    let total = 0;

    for (const roomSnapshots of this.snapshots.values()) {
      for (const snapshot of roomSnapshots) {
        total += JSON.stringify(snapshot).length;
      }
    }

    return total;
  }

  /**
   * Get snapshot count
   */
  getSnapshotCount(roomId?: RoomId): number {
    if (roomId) {
      return this.snapshots.get(roomId)?.length || 0;
    }

    let total = 0;
    for (const roomSnapshots of this.snapshots.values()) {
      total += roomSnapshots.length;
    }
    return total;
  }

  /**
   * Clear all snapshots
   */
  clear(): void {
    this.snapshots.clear();
    this.lastSnapshot.clear();
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      snapshots: Array.from(this.snapshots.entries()).map(([roomId, snaps]) => [
        roomId,
        snaps.map(s => ({
          ...s,
          state: {
            objects: Array.from(s.state.objects.entries()),
            annotations: Array.from(s.state.annotations.entries()),
            measurements: Array.from(s.state.measurements.entries()),
            properties: s.state.properties,
          },
        })),
      ]),
      maxSnapshots: this.maxSnapshots,
      snapshotInterval: this.snapshotInterval,
      lastSnapshot: Array.from(this.lastSnapshot.entries()),
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: any): void {
    this.snapshots = new Map(
      data.snapshots.map(([roomId, snaps]: [RoomId, any[]]) => [
        roomId,
        snaps.map(s => ({
          ...s,
          state: {
            objects: new Map(s.state.objects),
            annotations: new Map(s.state.annotations),
            measurements: new Map(s.state.measurements),
            properties: s.state.properties,
          },
        })),
      ])
    );
    this.maxSnapshots = data.maxSnapshots;
    this.snapshotInterval = data.snapshotInterval;
    this.lastSnapshot = new Map(data.lastSnapshot);
  }
}
