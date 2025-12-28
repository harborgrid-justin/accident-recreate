/**
 * AccuScene Enterprise - Operation Journal
 * v0.2.0
 *
 * Journal operations for replay and recovery
 */

import { RoomId, Operation, JournalEntry } from '../types';
import { v4 as uuidv4 } from 'uuid';

/**
 * Operation Journal Manager
 */
export class OperationJournal {
  private journal: Map<RoomId, JournalEntry[]>;
  private maxEntries: number;

  constructor(maxEntries: number = 10000) {
    this.journal = new Map();
    this.maxEntries = maxEntries;
  }

  /**
   * Append an operation to the journal
   */
  append(roomId: RoomId, operation: Operation, applied: boolean = false): JournalEntry {
    const entry: JournalEntry = {
      id: uuidv4(),
      roomId,
      operation,
      timestamp: Date.now(),
      applied,
    };

    if (!this.journal.has(roomId)) {
      this.journal.set(roomId, []);
    }

    const roomJournal = this.journal.get(roomId)!;
    roomJournal.push(entry);

    // Trim if too large
    if (roomJournal.length > this.maxEntries) {
      roomJournal.shift();
    }

    return entry;
  }

  /**
   * Append multiple operations
   */
  appendMultiple(roomId: RoomId, operations: Operation[]): JournalEntry[] {
    return operations.map(op => this.append(roomId, op));
  }

  /**
   * Mark an entry as applied
   */
  markApplied(roomId: RoomId, entryId: string): boolean {
    const roomJournal = this.journal.get(roomId);
    if (!roomJournal) {
      return false;
    }

    const entry = roomJournal.find(e => e.id === entryId);
    if (entry) {
      entry.applied = true;
      return true;
    }

    return false;
  }

  /**
   * Get all journal entries for a room
   */
  getEntries(roomId: RoomId): JournalEntry[] {
    return this.journal.get(roomId) || [];
  }

  /**
   * Get unapplied entries
   */
  getUnappliedEntries(roomId: RoomId): JournalEntry[] {
    const roomJournal = this.journal.get(roomId);
    return roomJournal ? roomJournal.filter(e => !e.applied) : [];
  }

  /**
   * Get entries in a time range
   */
  getEntriesInRange(roomId: RoomId, startTime: number, endTime: number): JournalEntry[] {
    const roomJournal = this.journal.get(roomId);
    return roomJournal
      ? roomJournal.filter(e => e.timestamp >= startTime && e.timestamp <= endTime)
      : [];
  }

  /**
   * Get entries since a timestamp
   */
  getEntriesSince(roomId: RoomId, timestamp: number): JournalEntry[] {
    const roomJournal = this.journal.get(roomId);
    return roomJournal ? roomJournal.filter(e => e.timestamp > timestamp) : [];
  }

  /**
   * Get operations from journal
   */
  getOperations(roomId: RoomId): Operation[] {
    const roomJournal = this.journal.get(roomId);
    return roomJournal ? roomJournal.map(e => e.operation) : [];
  }

  /**
   * Replay journal entries
   */
  replay(
    roomId: RoomId,
    applyFn: (operation: Operation) => void,
    filter?: (entry: JournalEntry) => boolean
  ): number {
    const roomJournal = this.journal.get(roomId);
    if (!roomJournal) {
      return 0;
    }

    let count = 0;
    const entries = filter ? roomJournal.filter(filter) : roomJournal;

    for (const entry of entries) {
      applyFn(entry.operation);
      entry.applied = true;
      count++;
    }

    return count;
  }

  /**
   * Clear journal for a room
   */
  clearRoom(roomId: RoomId): void {
    this.journal.delete(roomId);
  }

  /**
   * Compact journal by removing old applied entries
   */
  compact(roomId: RoomId, keepUnapplied: boolean = true): number {
    const roomJournal = this.journal.get(roomId);
    if (!roomJournal) {
      return 0;
    }

    const before = roomJournal.length;

    if (keepUnapplied) {
      // Keep recent entries and all unapplied
      const recent = roomJournal.slice(-1000);
      const unapplied = roomJournal.filter(e => !e.applied);

      const combined = new Map<string, JournalEntry>();
      for (const entry of [...recent, ...unapplied]) {
        combined.set(entry.id, entry);
      }

      const compacted = Array.from(combined.values()).sort(
        (a, b) => a.timestamp - b.timestamp
      );

      this.journal.set(roomId, compacted);
    } else {
      // Keep only recent entries
      this.journal.set(roomId, roomJournal.slice(-1000));
    }

    return before - this.journal.get(roomId)!.length;
  }

  /**
   * Prune old entries
   */
  prune(maxAge: number = 86400000): number {
    const now = Date.now();
    let pruned = 0;

    for (const [roomId, roomJournal] of this.journal.entries()) {
      const before = roomJournal.length;

      const filtered = roomJournal.filter(
        e => !e.applied || now - e.timestamp < maxAge
      );

      this.journal.set(roomId, filtered);
      pruned += before - filtered.length;
    }

    return pruned;
  }

  /**
   * Get journal size
   */
  getSize(roomId?: RoomId): number {
    if (roomId) {
      return this.journal.get(roomId)?.length || 0;
    }

    let total = 0;
    for (const roomJournal of this.journal.values()) {
      total += roomJournal.length;
    }
    return total;
  }

  /**
   * Get storage size estimate
   */
  getStorageSize(): number {
    let total = 0;

    for (const roomJournal of this.journal.values()) {
      for (const entry of roomJournal) {
        total += JSON.stringify(entry).length;
      }
    }

    return total;
  }

  /**
   * Clear all journals
   */
  clear(): void {
    this.journal.clear();
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      journal: Array.from(this.journal.entries()),
      maxEntries: this.maxEntries,
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: { journal: Array<[RoomId, JournalEntry[]]>; maxEntries: number }): void {
    this.journal = new Map(data.journal);
    this.maxEntries = data.maxEntries;
  }
}
