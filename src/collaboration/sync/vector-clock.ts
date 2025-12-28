/**
 * AccuScene Enterprise - Vector Clock Implementation
 * v0.2.0
 *
 * Vector clock for tracking causality in distributed systems
 */

import { ClientId, VectorClock } from '../types';

/**
 * Vector Clock utilities for distributed synchronization
 */
export class VectorClockManager {
  /**
   * Create a new vector clock
   */
  static create(clientId: ClientId): VectorClock {
    return { [clientId]: 0 };
  }

  /**
   * Increment the counter for a client
   */
  static increment(clock: VectorClock, clientId: ClientId): VectorClock {
    return {
      ...clock,
      [clientId]: (clock[clientId] || 0) + 1,
    };
  }

  /**
   * Merge two vector clocks (take maximum of each counter)
   */
  static merge(clock1: VectorClock, clock2: VectorClock): VectorClock {
    const merged: VectorClock = { ...clock1 };

    for (const [clientId, counter] of Object.entries(clock2)) {
      merged[clientId] = Math.max(merged[clientId] || 0, counter);
    }

    return merged;
  }

  /**
   * Compare two vector clocks
   * Returns:
   *  - 'equal' if clocks are identical
   *  - 'before' if clock1 happened before clock2
   *  - 'after' if clock1 happened after clock2
   *  - 'concurrent' if events are concurrent
   */
  static compare(
    clock1: VectorClock,
    clock2: VectorClock
  ): 'equal' | 'before' | 'after' | 'concurrent' {
    const allClients = new Set([...Object.keys(clock1), ...Object.keys(clock2)]);

    let hasGreater = false;
    let hasLess = false;

    for (const clientId of allClients) {
      const count1 = clock1[clientId] || 0;
      const count2 = clock2[clientId] || 0;

      if (count1 > count2) {
        hasGreater = true;
      } else if (count1 < count2) {
        hasLess = true;
      }

      // Early exit for concurrent
      if (hasGreater && hasLess) {
        return 'concurrent';
      }
    }

    if (!hasGreater && !hasLess) {
      return 'equal';
    }
    if (hasGreater && !hasLess) {
      return 'after';
    }
    return 'before';
  }

  /**
   * Check if clock1 happened before clock2
   */
  static happensBefore(clock1: VectorClock, clock2: VectorClock): boolean {
    return this.compare(clock1, clock2) === 'before';
  }

  /**
   * Check if clock1 happened after clock2
   */
  static happensAfter(clock1: VectorClock, clock2: VectorClock): boolean {
    return this.compare(clock1, clock2) === 'after';
  }

  /**
   * Check if two clocks are concurrent
   */
  static areConcurrent(clock1: VectorClock, clock2: VectorClock): boolean {
    return this.compare(clock1, clock2) === 'concurrent';
  }

  /**
   * Check if two clocks are equal
   */
  static areEqual(clock1: VectorClock, clock2: VectorClock): boolean {
    return this.compare(clock1, clock2) === 'equal';
  }

  /**
   * Get the counter value for a specific client
   */
  static getCounter(clock: VectorClock, clientId: ClientId): number {
    return clock[clientId] || 0;
  }

  /**
   * Set the counter value for a specific client
   */
  static setCounter(clock: VectorClock, clientId: ClientId, counter: number): VectorClock {
    return {
      ...clock,
      [clientId]: counter,
    };
  }

  /**
   * Get all client IDs in the vector clock
   */
  static getClients(clock: VectorClock): ClientId[] {
    return Object.keys(clock);
  }

  /**
   * Get the maximum counter value in the clock
   */
  static getMaxCounter(clock: VectorClock): number {
    return Math.max(0, ...Object.values(clock));
  }

  /**
   * Get the sum of all counters
   */
  static getSum(clock: VectorClock): number {
    return Object.values(clock).reduce((sum, count) => sum + count, 0);
  }

  /**
   * Clone a vector clock
   */
  static clone(clock: VectorClock): VectorClock {
    return { ...clock };
  }

  /**
   * Check if a vector clock is empty
   */
  static isEmpty(clock: VectorClock): boolean {
    return Object.keys(clock).length === 0 || this.getMaxCounter(clock) === 0;
  }

  /**
   * Create a compact string representation of the clock
   */
  static toString(clock: VectorClock): string {
    const entries = Object.entries(clock)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([clientId, counter]) => `${clientId}:${counter}`);
    return `{${entries.join(', ')}}`;
  }

  /**
   * Parse a string representation back to a vector clock
   */
  static fromString(str: string): VectorClock {
    const clock: VectorClock = {};
    const content = str.slice(1, -1); // Remove { }

    if (content.trim()) {
      const entries = content.split(',').map(e => e.trim());
      for (const entry of entries) {
        const [clientId, counterStr] = entry.split(':');
        if (clientId && counterStr) {
          clock[clientId.trim()] = parseInt(counterStr.trim(), 10);
        }
      }
    }

    return clock;
  }

  /**
   * Calculate the difference between two vector clocks
   * Returns the operations in clock2 that are not in clock1
   */
  static diff(clock1: VectorClock, clock2: VectorClock): VectorClock {
    const diff: VectorClock = {};

    for (const [clientId, counter2] of Object.entries(clock2)) {
      const counter1 = clock1[clientId] || 0;
      if (counter2 > counter1) {
        diff[clientId] = counter2 - counter1;
      }
    }

    return diff;
  }

  /**
   * Check if clock1 is a subset of clock2
   * (all counters in clock1 are <= corresponding counters in clock2)
   */
  static isSubset(clock1: VectorClock, clock2: VectorClock): boolean {
    for (const [clientId, counter1] of Object.entries(clock1)) {
      const counter2 = clock2[clientId] || 0;
      if (counter1 > counter2) {
        return false;
      }
    }
    return true;
  }

  /**
   * Compact the vector clock by removing zero counters
   */
  static compact(clock: VectorClock): VectorClock {
    const compacted: VectorClock = {};

    for (const [clientId, counter] of Object.entries(clock)) {
      if (counter > 0) {
        compacted[clientId] = counter;
      }
    }

    return compacted;
  }
}
