/**
 * AccuScene Enterprise - Grow-Only Counter
 * v0.2.0
 *
 * G-Counter CRDT for distributed counting (increment only)
 */

import { ClientId, VectorClock, CRDT, CRDTOperation, CRDTState } from '../types';

export interface GCounterOperation extends CRDTOperation {
  type: 'g-counter:increment';
  data: {
    amount: number;
  };
}

export interface GCounterState extends CRDTState {
  data: {
    counters: Map<ClientId, number>;
  };
}

/**
 * Grow-Only Counter
 * Supports only increments, guarantees convergence
 */
export class GCounter implements CRDT<number, GCounterOperation> {
  public state: GCounterState;
  private clientId: ClientId;

  constructor(clientId: ClientId) {
    this.clientId = clientId;
    this.state = {
      vectorClock: { [clientId]: 0 },
      data: {
        counters: new Map<ClientId, number>(),
      },
    };
  }

  /**
   * Increment the counter
   */
  increment(amount: number = 1): GCounterOperation {
    if (amount < 0) {
      throw new Error('G-Counter only supports positive increments');
    }

    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    const currentValue = this.state.data.counters.get(this.clientId) || 0;
    this.state.data.counters.set(this.clientId, currentValue + amount);

    return {
      id: `${this.clientId}-${counter}`,
      type: 'g-counter:increment',
      clientId: this.clientId,
      timestamp: {
        clientId: this.clientId,
        counter,
      },
      vectorClock: { ...this.state.vectorClock },
      data: { amount },
    };
  }

  /**
   * Apply an operation to the counter
   */
  apply(operation: GCounterOperation): void {
    // Update vector clock
    const opCounter = operation.vectorClock[operation.clientId] || 0;
    const currentCounter = this.state.vectorClock[operation.clientId] || 0;
    this.state.vectorClock[operation.clientId] = Math.max(opCounter, currentCounter);

    // Apply increment
    const currentValue = this.state.data.counters.get(operation.clientId) || 0;
    this.state.data.counters.set(operation.clientId, currentValue + operation.data.amount);
  }

  /**
   * Merge with another G-Counter
   */
  merge(other: GCounter): void {
    // Merge vector clocks
    for (const [clientId, counter] of Object.entries(other.state.vectorClock)) {
      const currentCounter = this.state.vectorClock[clientId] || 0;
      this.state.vectorClock[clientId] = Math.max(counter, currentCounter);
    }

    // Merge counters by taking maximum for each client
    for (const [clientId, value] of other.state.data.counters.entries()) {
      const currentValue = this.state.data.counters.get(clientId) || 0;
      this.state.data.counters.set(clientId, Math.max(value, currentValue));
    }
  }

  /**
   * Get the current counter value (sum of all client counters)
   */
  getValue(): number {
    let total = 0;
    for (const value of this.state.data.counters.values()) {
      total += value;
    }
    return total;
  }

  /**
   * Get the value for a specific client
   */
  getClientValue(clientId: ClientId): number {
    return this.state.data.counters.get(clientId) || 0;
  }

  /**
   * Clone the counter
   */
  clone(): GCounter {
    const cloned = new GCounter(this.clientId);
    cloned.state = {
      vectorClock: { ...this.state.vectorClock },
      data: {
        counters: new Map(this.state.data.counters),
      },
    };
    return cloned;
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      vectorClock: { ...this.state.vectorClock },
      counters: Object.fromEntries(this.state.data.counters),
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: { vectorClock: VectorClock; counters: Record<ClientId, number> }): void {
    this.state = {
      vectorClock: { ...data.vectorClock },
      data: {
        counters: new Map(Object.entries(data.counters)),
      },
    };
  }
}
