/**
 * AccuScene Enterprise - Replicated Growable Array
 * v0.2.0
 *
 * RGA CRDT for distributed list/sequence operations
 */

import { ClientId, Timestamp, VectorClock, CRDT, CRDTOperation, CRDTState } from '../types';

export interface RGAOperation extends CRDTOperation {
  type: 'rga:insert' | 'rga:delete';
  data: {
    position?: number;
    element?: unknown;
    id?: string;
    afterId?: string | null;
  };
}

export interface RGAElement<T> {
  id: string;
  value: T;
  timestamp: Timestamp;
  deleted: boolean;
}

export interface RGAState extends CRDTState {
  data: {
    elements: RGAElement<unknown>[];
  };
}

/**
 * Replicated Growable Array
 * Distributed sequence/list with insert and delete operations
 */
export class RGA<T> implements CRDT<T[], RGAOperation> {
  public state: RGAState;
  private clientId: ClientId;

  constructor(clientId: ClientId) {
    this.clientId = clientId;
    this.state = {
      vectorClock: { [clientId]: 0 },
      data: {
        elements: [],
      },
    };
  }

  /**
   * Insert an element at a position
   */
  insert(position: number, element: T): RGAOperation {
    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    const timestamp: Timestamp = {
      clientId: this.clientId,
      counter,
    };

    const id = `${this.clientId}-${counter}`;

    // Find the element before the insertion position
    const visibleElements = this.state.data.elements.filter(e => !e.deleted);
    const afterId = position > 0 && visibleElements[position - 1]
      ? visibleElements[position - 1].id
      : null;

    const newElement: RGAElement<T> = {
      id,
      value: element,
      timestamp,
      deleted: false,
    };

    // Find insertion index
    let insertIndex = 0;
    if (afterId) {
      const afterIndex = this.state.data.elements.findIndex(e => e.id === afterId);
      if (afterIndex !== -1) {
        insertIndex = afterIndex + 1;
      }
    }

    this.state.data.elements.splice(insertIndex, 0, newElement as RGAElement<unknown>);

    return {
      id,
      type: 'rga:insert',
      clientId: this.clientId,
      timestamp,
      vectorClock: { ...this.state.vectorClock },
      data: { position, element, id, afterId },
    };
  }

  /**
   * Delete an element at a position
   */
  delete(position: number): RGAOperation {
    const counter = (this.state.vectorClock[this.clientId] || 0) + 1;
    this.state.vectorClock[this.clientId] = counter;

    const timestamp: Timestamp = {
      clientId: this.clientId,
      counter,
    };

    const visibleElements = this.state.data.elements.filter(e => !e.deleted);
    const elementToDelete = visibleElements[position];

    if (elementToDelete) {
      const index = this.state.data.elements.findIndex(e => e.id === elementToDelete.id);
      if (index !== -1) {
        this.state.data.elements[index].deleted = true;
      }
    }

    return {
      id: `${this.clientId}-${counter}`,
      type: 'rga:delete',
      clientId: this.clientId,
      timestamp,
      vectorClock: { ...this.state.vectorClock },
      data: { position, id: elementToDelete?.id },
    };
  }

  /**
   * Apply an operation to the array
   */
  apply(operation: RGAOperation): void {
    // Update vector clock
    const opCounter = operation.vectorClock[operation.clientId] || 0;
    const currentCounter = this.state.vectorClock[operation.clientId] || 0;
    this.state.vectorClock[operation.clientId] = Math.max(opCounter, currentCounter);

    if (operation.type === 'rga:insert') {
      const { id, element, afterId } = operation.data;

      // Check if element already exists
      if (id && this.state.data.elements.some(e => e.id === id)) {
        return;
      }

      const newElement: RGAElement<unknown> = {
        id: id!,
        value: element,
        timestamp: operation.timestamp,
        deleted: false,
      };

      // Find insertion index
      let insertIndex = 0;
      if (afterId) {
        const afterIndex = this.state.data.elements.findIndex(e => e.id === afterId);
        if (afterIndex !== -1) {
          insertIndex = afterIndex + 1;

          // Handle concurrent inserts after same element
          while (
            insertIndex < this.state.data.elements.length &&
            this.state.data.elements[insertIndex].timestamp.counter < operation.timestamp.counter
          ) {
            insertIndex++;
          }
        }
      }

      this.state.data.elements.splice(insertIndex, 0, newElement);

    } else if (operation.type === 'rga:delete') {
      const { id } = operation.data;
      const index = this.state.data.elements.findIndex(e => e.id === id);
      if (index !== -1) {
        this.state.data.elements[index].deleted = true;
      }
    }
  }

  /**
   * Merge with another RGA
   */
  merge(other: RGA<T>): void {
    // Merge vector clocks
    for (const [clientId, counter] of Object.entries(other.state.vectorClock)) {
      const currentCounter = this.state.vectorClock[clientId] || 0;
      this.state.vectorClock[clientId] = Math.max(counter, currentCounter);
    }

    // Merge elements
    for (const otherElement of other.state.data.elements) {
      const existingIndex = this.state.data.elements.findIndex(e => e.id === otherElement.id);

      if (existingIndex === -1) {
        // Element doesn't exist, find correct position to insert
        let insertIndex = 0;

        for (let i = 0; i < this.state.data.elements.length; i++) {
          if (this.compareTimestamps(otherElement.timestamp, this.state.data.elements[i].timestamp) > 0) {
            insertIndex = i + 1;
          } else {
            break;
          }
        }

        this.state.data.elements.splice(insertIndex, 0, { ...otherElement });
      } else {
        // Element exists, merge deleted state
        this.state.data.elements[existingIndex].deleted =
          this.state.data.elements[existingIndex].deleted || otherElement.deleted;
      }
    }
  }

  /**
   * Get the current array value (only non-deleted elements)
   */
  getValue(): T[] {
    return this.state.data.elements
      .filter(e => !e.deleted)
      .map(e => e.value as T);
  }

  /**
   * Get element at position
   */
  get(position: number): T | undefined {
    const visibleElements = this.state.data.elements.filter(e => !e.deleted);
    return visibleElements[position]?.value as T | undefined;
  }

  /**
   * Get the length of the array
   */
  length(): number {
    return this.state.data.elements.filter(e => !e.deleted).length;
  }

  /**
   * Clone the array
   */
  clone(): RGA<T> {
    const cloned = new RGA<T>(this.clientId);
    cloned.state = {
      vectorClock: { ...this.state.vectorClock },
      data: {
        elements: this.state.data.elements.map(e => ({
          ...e,
          timestamp: { ...e.timestamp },
        })),
      },
    };
    return cloned;
  }

  /**
   * Compare timestamps
   */
  private compareTimestamps(ts1: Timestamp, ts2: Timestamp): number {
    if (ts1.counter !== ts2.counter) {
      return ts1.counter - ts2.counter;
    }
    return ts1.clientId.localeCompare(ts2.clientId);
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      vectorClock: { ...this.state.vectorClock },
      elements: this.state.data.elements,
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: { vectorClock: VectorClock; elements: RGAElement<T>[] }): void {
    this.state = {
      vectorClock: { ...data.vectorClock },
      data: {
        elements: data.elements.map(e => ({ ...e })),
      },
    };
  }
}
