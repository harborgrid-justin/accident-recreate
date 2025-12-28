/**
 * AccuScene Enterprise - Last-Writer-Wins Map
 * v0.2.0
 *
 * LWW-Map CRDT for distributed key-value storage with conflict resolution
 */

import { ClientId, Timestamp, VectorClock, CRDT, CRDTOperation, CRDTState } from '../types';

export interface LWWMapOperation extends CRDTOperation {
  type: 'lww-map:set' | 'lww-map:delete';
  data: {
    key: string;
    value?: unknown;
  };
}

export interface LWWMapEntry<V> {
  value: V | null;
  timestamp: Timestamp;
  deleted: boolean;
}

export interface LWWMapState extends CRDTState {
  data: {
    entries: Map<string, LWWMapEntry<unknown>>;
  };
}

/**
 * Last-Writer-Wins Map
 * Distributed key-value map with LWW conflict resolution
 */
export class LWWMap<K extends string, V> implements CRDT<Map<K, V>, LWWMapOperation> {
  public state: LWWMapState;
  private clientId: ClientId;

  constructor(clientId: ClientId) {
    this.clientId = clientId;
    this.state = {
      vectorClock: { [clientId]: 0 },
      data: {
        entries: new Map<string, LWWMapEntry<unknown>>(),
      },
    };
  }

  /**
   * Set a key-value pair
   */
  set(key: K, value: V): LWWMapOperation {
    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    const timestamp: Timestamp = {
      clientId: this.clientId,
      counter,
    };

    this.state.data.entries.set(key, {
      value,
      timestamp,
      deleted: false,
    });

    return {
      id: `${this.clientId}-${counter}`,
      type: 'lww-map:set',
      clientId: this.clientId,
      timestamp,
      vectorClock: { ...this.state.vectorClock },
      data: { key, value },
    };
  }

  /**
   * Delete a key
   */
  delete(key: K): LWWMapOperation {
    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    const timestamp: Timestamp = {
      clientId: this.clientId,
      counter,
    };

    const existing = this.state.data.entries.get(key);
    if (existing) {
      this.state.data.entries.set(key, {
        value: null,
        timestamp,
        deleted: true,
      });
    }

    return {
      id: `${this.clientId}-${counter}`,
      type: 'lww-map:delete',
      clientId: this.clientId,
      timestamp,
      vectorClock: { ...this.state.vectorClock },
      data: { key },
    };
  }

  /**
   * Apply an operation to the map
   */
  apply(operation: LWWMapOperation): void {
    // Update vector clock
    const opCounter = operation.vectorClock[operation.clientId] || 0;
    const currentCounter = this.state.vectorClock[operation.clientId] || 0;
    this.state.vectorClock[operation.clientId] = Math.max(opCounter, currentCounter);

    const key = operation.data.key;
    const existing = this.state.data.entries.get(key);

    // Apply if timestamp is newer or entry doesn't exist
    if (!existing || this.isNewer(operation.timestamp, existing.timestamp)) {
      if (operation.type === 'lww-map:set') {
        this.state.data.entries.set(key, {
          value: operation.data.value ?? null,
          timestamp: operation.timestamp,
          deleted: false,
        });
      } else {
        this.state.data.entries.set(key, {
          value: null,
          timestamp: operation.timestamp,
          deleted: true,
        });
      }
    }
  }

  /**
   * Merge with another LWW-Map
   */
  merge(other: LWWMap<K, V>): void {
    // Merge vector clocks
    for (const [clientId, counter] of Object.entries(other.state.vectorClock)) {
      const currentCounter = this.state.vectorClock[clientId] || 0;
      this.state.vectorClock[clientId] = Math.max(counter, currentCounter);
    }

    // Merge entries
    for (const [key, otherEntry] of other.state.data.entries.entries()) {
      const thisEntry = this.state.data.entries.get(key);

      if (!thisEntry || this.isNewer(otherEntry.timestamp, thisEntry.timestamp)) {
        this.state.data.entries.set(key, { ...otherEntry });
      }
    }
  }

  /**
   * Get a value by key
   */
  get(key: K): V | undefined {
    const entry = this.state.data.entries.get(key);
    if (entry && !entry.deleted) {
      return entry.value as V;
    }
    return undefined;
  }

  /**
   * Check if a key exists
   */
  has(key: K): boolean {
    const entry = this.state.data.entries.get(key);
    return entry ? !entry.deleted : false;
  }

  /**
   * Get all keys
   */
  keys(): K[] {
    const keys: K[] = [];
    for (const [key, entry] of this.state.data.entries.entries()) {
      if (!entry.deleted) {
        keys.push(key as K);
      }
    }
    return keys;
  }

  /**
   * Get all values
   */
  values(): V[] {
    const values: V[] = [];
    for (const entry of this.state.data.entries.values()) {
      if (!entry.deleted && entry.value !== null) {
        values.push(entry.value as V);
      }
    }
    return values;
  }

  /**
   * Get all entries
   */
  entries(): Array<[K, V]> {
    const entries: Array<[K, V]> = [];
    for (const [key, entry] of this.state.data.entries.entries()) {
      if (!entry.deleted && entry.value !== null) {
        entries.push([key as K, entry.value as V]);
      }
    }
    return entries;
  }

  /**
   * Get the size of the map
   */
  size(): number {
    let count = 0;
    for (const entry of this.state.data.entries.values()) {
      if (!entry.deleted) {
        count++;
      }
    }
    return count;
  }

  /**
   * Get the current map value
   */
  getValue(): Map<K, V> {
    const result = new Map<K, V>();
    for (const [key, entry] of this.state.data.entries.entries()) {
      if (!entry.deleted && entry.value !== null) {
        result.set(key as K, entry.value as V);
      }
    }
    return result;
  }

  /**
   * Clone the map
   */
  clone(): LWWMap<K, V> {
    const cloned = new LWWMap<K, V>(this.clientId);
    cloned.state = {
      vectorClock: { ...this.state.vectorClock },
      data: {
        entries: new Map(
          Array.from(this.state.data.entries.entries()).map(([k, v]) => [
            k,
            { ...v, timestamp: { ...v.timestamp } },
          ])
        ),
      },
    };
    return cloned;
  }

  /**
   * Compare timestamps to determine which is newer
   */
  private isNewer(ts1: Timestamp, ts2: Timestamp): boolean {
    if (ts1.counter !== ts2.counter) {
      return ts1.counter > ts2.counter;
    }
    // Tie-break using clientId for deterministic ordering
    return ts1.clientId > ts2.clientId;
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      vectorClock: { ...this.state.vectorClock },
      entries: Object.fromEntries(this.state.data.entries),
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: { vectorClock: VectorClock; entries: Record<string, LWWMapEntry<V>> }): void {
    this.state = {
      vectorClock: { ...data.vectorClock },
      data: {
        entries: new Map(Object.entries(data.entries)),
      },
    };
  }
}
