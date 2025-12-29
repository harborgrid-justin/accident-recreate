/**
 * Operation Queue
 * Priority queue for pending sync operations
 */

import { v4 as uuidv4 } from 'uuid';
import {
  SyncOperation,
  OperationType,
  Priority,
  Version,
} from './types';
import { IndexedDBStore } from './IndexedDBStore';

export class OperationQueue {
  private operations: Map<string, SyncOperation> = new Map();
  private completed: Set<string> = new Set();
  private store: IndexedDBStore;
  private maxSize: number;

  constructor(store: IndexedDBStore, maxSize: number = 10000) {
    this.store = store;
    this.maxSize = maxSize;
  }

  /**
   * Load operations from storage
   */
  async load(): Promise<void> {
    const operations = await this.store.getPendingOperations();
    operations.forEach((op) => {
      this.operations.set(op.id, op);
    });
  }

  /**
   * Enqueue an operation
   */
  async enqueue(
    entityId: string,
    entityType: string,
    operationType: OperationType,
    data: any,
    version: Version,
    priority: Priority = Priority.Normal
  ): Promise<string> {
    if (this.operations.size >= this.maxSize) {
      throw new Error('Operation queue is full');
    }

    const operation: SyncOperation = {
      id: uuidv4(),
      entityId,
      entityType,
      operationType,
      data,
      version,
      priority,
      queuedAt: new Date().toISOString(),
      retryCount: 0,
      dependencies: [],
      tags: [],
    };

    this.operations.set(operation.id, operation);
    await this.store.storeOperation(operation);

    return operation.id;
  }

  /**
   * Dequeue the next ready operation
   */
  dequeue(): SyncOperation | null {
    // Find highest priority operation with satisfied dependencies
    const sorted = Array.from(this.operations.values()).sort((a, b) => {
      if (a.priority !== b.priority) {
        return b.priority - a.priority;
      }
      return new Date(a.queuedAt).getTime() - new Date(b.queuedAt).getTime();
    });

    for (const op of sorted) {
      // Check if all dependencies are completed
      const depsSatisfied = op.dependencies.every((depId) =>
        this.completed.has(depId)
      );

      if (depsSatisfied) {
        this.operations.delete(op.id);
        return op;
      }
    }

    return null;
  }

  /**
   * Peek at next operation without removing
   */
  peek(): SyncOperation | null {
    const sorted = Array.from(this.operations.values()).sort((a, b) => {
      if (a.priority !== b.priority) {
        return b.priority - a.priority;
      }
      return new Date(a.queuedAt).getTime() - new Date(b.queuedAt).getTime();
    });

    return sorted[0] || null;
  }

  /**
   * Mark operation as completed
   */
  async markCompleted(operationId: string): Promise<void> {
    this.completed.add(operationId);
    this.operations.delete(operationId);
    await this.store.deleteOperation(operationId);

    // Clean up old completed IDs (keep last 1000)
    if (this.completed.size > 1000) {
      const toKeep = Array.from(this.completed).slice(-500);
      this.completed = new Set(toKeep);
    }
  }

  /**
   * Get operation by ID
   */
  get(operationId: string): SyncOperation | null {
    return this.operations.get(operationId) || null;
  }

  /**
   * Update an operation
   */
  async update(operation: SyncOperation): Promise<void> {
    this.operations.set(operation.id, operation);
    await this.store.storeOperation(operation);
  }

  /**
   * Remove an operation
   */
  async remove(operationId: string): Promise<void> {
    this.operations.delete(operationId);
    await this.store.deleteOperation(operationId);
  }

  /**
   * Get queue size
   */
  size(): number {
    return this.operations.size;
  }

  /**
   * Check if queue is empty
   */
  isEmpty(): boolean {
    return this.operations.size === 0;
  }

  /**
   * Get all pending operations
   */
  getPending(): SyncOperation[] {
    return Array.from(this.operations.values());
  }

  /**
   * Get operations for specific entity
   */
  getForEntity(entityId: string): SyncOperation[] {
    return Array.from(this.operations.values()).filter(
      (op) => op.entityId === entityId
    );
  }

  /**
   * Get operations with tag
   */
  getWithTag(tag: string): SyncOperation[] {
    return Array.from(this.operations.values()).filter((op) =>
      op.tags.includes(tag)
    );
  }

  /**
   * Clear all operations
   */
  async clear(): Promise<void> {
    const operationIds = Array.from(this.operations.keys());
    this.operations.clear();

    for (const id of operationIds) {
      await this.store.deleteOperation(id);
    }
  }

  /**
   * Add dependency to operation
   */
  async addDependency(operationId: string, dependencyId: string): Promise<void> {
    const operation = this.operations.get(operationId);
    if (operation) {
      if (!operation.dependencies.includes(dependencyId)) {
        operation.dependencies.push(dependencyId);
        await this.update(operation);
      }
    }
  }

  /**
   * Add tag to operation
   */
  async addTag(operationId: string, tag: string): Promise<void> {
    const operation = this.operations.get(operationId);
    if (operation) {
      if (!operation.tags.includes(tag)) {
        operation.tags.push(tag);
        await this.update(operation);
      }
    }
  }

  /**
   * Get queue statistics
   */
  getStats() {
    const byPriority: Record<Priority, number> = {
      [Priority.Low]: 0,
      [Priority.Normal]: 0,
      [Priority.High]: 0,
      [Priority.Critical]: 0,
    };

    const byType: Record<OperationType, number> = {
      [OperationType.Create]: 0,
      [OperationType.Update]: 0,
      [OperationType.Delete]: 0,
      [OperationType.Patch]: 0,
    };

    for (const op of this.operations.values()) {
      byPriority[op.priority]++;
      byType[op.operationType]++;
    }

    return {
      pending: this.operations.size,
      completed: this.completed.size,
      byPriority,
      byType,
    };
  }
}
