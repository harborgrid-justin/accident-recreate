/**
 * AccuScene Enterprise - Observed-Remove Set
 * v0.2.0
 *
 * OR-Set CRDT for distributed set operations with add/remove
 */

import { ClientId, Timestamp, VectorClock, CRDT, CRDTOperation, CRDTState } from '../types';

export interface ORSetOperation extends CRDTOperation {
  type: 'or-set:add' | 'or-set:remove';
  data: {
    element: unknown;
    tag?: string;
  };
}

export interface ORSetState extends CRDTState {
  data: {
    elements: Map<string, Set<string>>; // element -> set of tags
  };
}

/**
 * Observed-Remove Set
 * Supports add and remove operations with conflict-free semantics
 */
export class ORSet<T> implements CRDT<Set<T>, ORSetOperation> {
  public state: ORSetState;
  private clientId: ClientId;

  constructor(clientId: ClientId) {
    this.clientId = clientId;
    this.state = {
      vectorClock: { [clientId]: 0 },
      data: {
        elements: new Map<string, Set<string>>(),
      },
    };
  }

  /**
   * Add an element to the set
   */
  add(element: T): ORSetOperation {
    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    const elementKey = this.serialize(element);
    const tag = `${this.clientId}-${counter}`;

    if (!this.state.data.elements.has(elementKey)) {
      this.state.data.elements.set(elementKey, new Set<string>());
    }
    this.state.data.elements.get(elementKey)!.add(tag);

    return {
      id: `${this.clientId}-${counter}`,
      type: 'or-set:add',
      clientId: this.clientId,
      timestamp: {
        clientId: this.clientId,
        counter,
      },
      vectorClock: { ...this.state.vectorClock },
      data: { element, tag },
    };
  }

  /**
   * Remove an element from the set
   */
  remove(element: T): ORSetOperation {
    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    const elementKey = this.serialize(element);
    const tags = this.state.data.elements.get(elementKey);

    // Remove all tags for this element
    if (tags) {
      this.state.data.elements.delete(elementKey);
    }

    return {
      id: `${this.clientId}-${counter}`,
      type: 'or-set:remove',
      clientId: this.clientId,
      timestamp: {
        clientId: this.clientId,
        counter,
      },
      vectorClock: { ...this.state.vectorClock },
      data: { element },
    };
  }

  /**
   * Apply an operation to the set
   */
  apply(operation: ORSetOperation): void {
    // Update vector clock
    const opCounter = operation.vectorClock[operation.clientId] || 0;
    const currentCounter = this.state.vectorClock[operation.clientId] || 0;
    this.state.vectorClock[operation.clientId] = Math.max(opCounter, currentCounter);

    const elementKey = this.serialize(operation.data.element);

    if (operation.type === 'or-set:add') {
      if (!this.state.data.elements.has(elementKey)) {
        this.state.data.elements.set(elementKey, new Set<string>());
      }
      if (operation.data.tag) {
        this.state.data.elements.get(elementKey)!.add(operation.data.tag);
      }
    } else if (operation.type === 'or-set:remove') {
      this.state.data.elements.delete(elementKey);
    }
  }

  /**
   * Merge with another OR-Set
   */
  merge(other: ORSet<T>): void {
    // Merge vector clocks
    for (const [clientId, counter] of Object.entries(other.state.vectorClock)) {
      const currentCounter = this.state.vectorClock[clientId] || 0;
      this.state.vectorClock[clientId] = Math.max(counter, currentCounter);
    }

    // Merge elements
    for (const [element, tags] of other.state.data.elements.entries()) {
      if (!this.state.data.elements.has(element)) {
        this.state.data.elements.set(element, new Set<string>());
      }
      const currentTags = this.state.data.elements.get(element)!;
      for (const tag of tags) {
        currentTags.add(tag);
      }
    }
  }

  /**
   * Get the current set value
   */
  getValue(): Set<T> {
    const result = new Set<T>();
    for (const [elementKey, tags] of this.state.data.elements.entries()) {
      if (tags.size > 0) {
        result.add(this.deserialize(elementKey));
      }
    }
    return result;
  }

  /**
   * Check if the set contains an element
   */
  has(element: T): boolean {
    const elementKey = this.serialize(element);
    const tags = this.state.data.elements.get(elementKey);
    return tags ? tags.size > 0 : false;
  }

  /**
   * Get the size of the set
   */
  size(): number {
    let count = 0;
    for (const tags of this.state.data.elements.values()) {
      if (tags.size > 0) {
        count++;
      }
    }
    return count;
  }

  /**
   * Clone the set
   */
  clone(): ORSet<T> {
    const cloned = new ORSet<T>(this.clientId);
    cloned.state = {
      vectorClock: { ...this.state.vectorClock },
      data: {
        elements: new Map(
          Array.from(this.state.data.elements.entries()).map(([k, v]) => [k, new Set(v)])
        ),
      },
    };
    return cloned;
  }

  /**
   * Serialize an element to a string key
   */
  private serialize(element: T): string {
    return JSON.stringify(element);
  }

  /**
   * Deserialize a string key back to an element
   */
  private deserialize(key: string): T {
    return JSON.parse(key) as T;
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      vectorClock: { ...this.state.vectorClock },
      elements: Object.fromEntries(
        Array.from(this.state.data.elements.entries()).map(([k, v]) => [k, Array.from(v)])
      ),
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: { vectorClock: VectorClock; elements: Record<string, string[]> }): void {
    this.state = {
      vectorClock: { ...data.vectorClock },
      data: {
        elements: new Map(
          Object.entries(data.elements).map(([k, v]) => [k, new Set(v)])
        ),
      },
    };
  }
}
