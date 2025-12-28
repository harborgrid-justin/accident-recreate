/**
 * AccuScene Enterprise - Positive-Negative Counter
 * v0.2.0
 *
 * PN-Counter CRDT for distributed counting (increment and decrement)
 */

import { ClientId, VectorClock, CRDT, CRDTOperation, CRDTState } from '../types';
import { GCounter } from './g-counter';

export interface PNCounterOperation extends CRDTOperation {
  type: 'pn-counter:increment' | 'pn-counter:decrement';
  data: {
    amount: number;
  };
}

export interface PNCounterState extends CRDTState {
  data: {
    positive: GCounter;
    negative: GCounter;
  };
}

/**
 * Positive-Negative Counter
 * Supports both increments and decrements using two G-Counters
 */
export class PNCounter implements CRDT<number, PNCounterOperation> {
  public state: PNCounterState;
  private clientId: ClientId;

  constructor(clientId: ClientId) {
    this.clientId = clientId;
    this.state = {
      vectorClock: { [clientId]: 0 },
      data: {
        positive: new GCounter(clientId),
        negative: new GCounter(clientId),
      },
    };
  }

  /**
   * Increment the counter
   */
  increment(amount: number = 1): PNCounterOperation {
    if (amount < 0) {
      throw new Error('Use decrement() for negative values');
    }

    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    this.state.data.positive.increment(amount);

    return {
      id: `${this.clientId}-${counter}`,
      type: 'pn-counter:increment',
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
   * Decrement the counter
   */
  decrement(amount: number = 1): PNCounterOperation {
    if (amount < 0) {
      throw new Error('Use increment() for positive values');
    }

    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    this.state.data.negative.increment(amount);

    return {
      id: `${this.clientId}-${counter}`,
      type: 'pn-counter:decrement',
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
  apply(operation: PNCounterOperation): void {
    // Update vector clock
    const opCounter = operation.vectorClock[operation.clientId] || 0;
    const currentCounter = this.state.vectorClock[operation.clientId] || 0;
    this.state.vectorClock[operation.clientId] = Math.max(opCounter, currentCounter);

    // Apply to appropriate G-Counter
    if (operation.type === 'pn-counter:increment') {
      this.state.data.positive.apply({
        ...operation,
        type: 'g-counter:increment',
      });
    } else {
      this.state.data.negative.apply({
        ...operation,
        type: 'g-counter:increment',
      });
    }
  }

  /**
   * Merge with another PN-Counter
   */
  merge(other: PNCounter): void {
    // Merge vector clocks
    for (const [clientId, counter] of Object.entries(other.state.vectorClock)) {
      const currentCounter = this.state.vectorClock[clientId] || 0;
      this.state.vectorClock[clientId] = Math.max(counter, currentCounter);
    }

    // Merge both G-Counters
    this.state.data.positive.merge(other.state.data.positive);
    this.state.data.negative.merge(other.state.data.negative);
  }

  /**
   * Get the current counter value (positive - negative)
   */
  getValue(): number {
    return this.state.data.positive.getValue() - this.state.data.negative.getValue();
  }

  /**
   * Get the positive component
   */
  getPositive(): number {
    return this.state.data.positive.getValue();
  }

  /**
   * Get the negative component
   */
  getNegative(): number {
    return this.state.data.negative.getValue();
  }

  /**
   * Clone the counter
   */
  clone(): PNCounter {
    const cloned = new PNCounter(this.clientId);
    cloned.state = {
      vectorClock: { ...this.state.vectorClock },
      data: {
        positive: this.state.data.positive.clone(),
        negative: this.state.data.negative.clone(),
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
      positive: this.state.data.positive.toJSON(),
      negative: this.state.data.negative.toJSON(),
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: { vectorClock: VectorClock; positive: unknown; negative: unknown }): void {
    this.state.vectorClock = { ...data.vectorClock };
    this.state.data.positive.fromJSON(data.positive as any);
    this.state.data.negative.fromJSON(data.negative as any);
  }
}
