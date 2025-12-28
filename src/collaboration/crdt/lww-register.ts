/**
 * AccuScene Enterprise - Last-Writer-Wins Register
 * v0.2.0
 *
 * LWW-Register CRDT for conflict-free replicated values
 */

import { ClientId, Timestamp, VectorClock, CRDT, CRDTOperation, CRDTState } from '../types';

export interface LWWRegisterOperation extends CRDTOperation {
  type: 'lww-register:set';
  data: {
    value: unknown;
  };
}

export interface LWWRegisterState extends CRDTState {
  data: {
    value: unknown;
    timestamp: Timestamp;
  };
}

/**
 * Last-Writer-Wins Register
 * Resolves conflicts by keeping the value with the highest timestamp
 */
export class LWWRegister<T> implements CRDT<T, LWWRegisterOperation> {
  public state: LWWRegisterState;
  private clientId: ClientId;

  constructor(clientId: ClientId, initialValue?: T) {
    this.clientId = clientId;
    this.state = {
      vectorClock: { [clientId]: 0 },
      data: {
        value: initialValue ?? null,
        timestamp: {
          clientId,
          counter: 0,
        },
      },
    };
  }

  /**
   * Set the register value
   */
  set(value: T): LWWRegisterOperation {
    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    const timestamp: Timestamp = {
      clientId: this.clientId,
      counter,
    };

    this.state.data = {
      value,
      timestamp,
    };

    return {
      id: `${this.clientId}-${counter}`,
      type: 'lww-register:set',
      clientId: this.clientId,
      timestamp,
      vectorClock: { ...this.state.vectorClock },
      data: { value },
    };
  }

  /**
   * Apply an operation to the register
   */
  apply(operation: LWWRegisterOperation): void {
    // Update vector clock
    const opCounter = operation.vectorClock[operation.clientId] || 0;
    const currentCounter = this.state.vectorClock[operation.clientId] || 0;
    this.state.vectorClock[operation.clientId] = Math.max(opCounter, currentCounter);

    // Apply if timestamp is newer
    if (this.isNewer(operation.timestamp, this.state.data.timestamp)) {
      this.state.data = {
        value: operation.data.value,
        timestamp: operation.timestamp,
      };
    }
  }

  /**
   * Merge with another LWW-Register
   */
  merge(other: LWWRegister<T>): void {
    // Merge vector clocks
    for (const [clientId, counter] of Object.entries(other.state.vectorClock)) {
      const currentCounter = this.state.vectorClock[clientId] || 0;
      this.state.vectorClock[clientId] = Math.max(counter, currentCounter);
    }

    // Keep the value with the highest timestamp
    if (this.isNewer(other.state.data.timestamp, this.state.data.timestamp)) {
      this.state.data = { ...other.state.data };
    }
  }

  /**
   * Get the current value
   */
  getValue(): T {
    return this.state.data.value as T;
  }

  /**
   * Get the timestamp of the current value
   */
  getTimestamp(): Timestamp {
    return this.state.data.timestamp;
  }

  /**
   * Clone the register
   */
  clone(): LWWRegister<T> {
    const cloned = new LWWRegister<T>(this.clientId);
    cloned.state = {
      vectorClock: { ...this.state.vectorClock },
      data: {
        value: this.state.data.value,
        timestamp: { ...this.state.data.timestamp },
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
  toJSON(): LWWRegisterState {
    return {
      vectorClock: { ...this.state.vectorClock },
      data: {
        value: this.state.data.value,
        timestamp: { ...this.state.data.timestamp },
      },
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(state: LWWRegisterState): void {
    this.state = {
      vectorClock: { ...state.vectorClock },
      data: {
        value: state.data.value,
        timestamp: { ...state.data.timestamp },
      },
    };
  }
}
