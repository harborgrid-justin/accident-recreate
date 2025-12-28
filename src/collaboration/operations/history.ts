/**
 * AccuScene Enterprise - Operation History
 * v0.2.0
 *
 * Track and manage operation history with undo/redo support
 */

import { Operation, OperationType, ClientId } from '../types';

export interface HistoryEntry {
  operation: Operation;
  timestamp: number;
  undone: boolean;
}

/**
 * Operation History Manager
 */
export class OperationHistory {
  private history: HistoryEntry[];
  private undoStack: Operation[];
  private redoStack: Operation[];
  private maxHistorySize: number;

  constructor(maxHistorySize: number = 1000) {
    this.history = [];
    this.undoStack = [];
    this.redoStack = [];
    this.maxHistorySize = maxHistorySize;
  }

  /**
   * Add an operation to history
   */
  add(operation: Operation): void {
    const entry: HistoryEntry = {
      operation,
      timestamp: Date.now(),
      undone: false,
    };

    this.history.push(entry);

    // Trim history if too large
    if (this.history.length > this.maxHistorySize) {
      this.history.shift();
    }

    // Clear redo stack when new operation is added
    this.redoStack = [];
  }

  /**
   * Add multiple operations to history
   */
  addMultiple(operations: Operation[]): void {
    for (const op of operations) {
      this.add(op);
    }
  }

  /**
   * Undo the last operation
   */
  undo(clientId: ClientId): Operation | null {
    // Find the last non-undone operation from this client
    for (let i = this.history.length - 1; i >= 0; i--) {
      const entry = this.history[i];

      if (entry.operation.clientId === clientId && !entry.undone) {
        entry.undone = true;
        this.undoStack.push(entry.operation);
        return this.createInverseOperation(entry.operation);
      }
    }

    return null;
  }

  /**
   * Redo the last undone operation
   */
  redo(clientId: ClientId): Operation | null {
    if (this.undoStack.length === 0) {
      return null;
    }

    const operation = this.undoStack.pop()!;

    if (operation.clientId === clientId) {
      // Find the entry in history and mark as not undone
      const entry = this.history.find(
        e => e.operation.id === operation.id && e.undone
      );

      if (entry) {
        entry.undone = false;
      }

      this.redoStack.push(operation);
      return operation;
    }

    return null;
  }

  /**
   * Create an inverse operation for undo
   */
  private createInverseOperation(operation: Operation): Operation {
    const data = operation.data as any;

    // Inverse operations
    switch (operation.type) {
      case OperationType.OBJECT_CREATE:
        return {
          ...operation,
          type: OperationType.OBJECT_DELETE,
        };

      case OperationType.OBJECT_DELETE:
        return {
          ...operation,
          type: OperationType.OBJECT_CREATE,
        };

      case OperationType.ANNOTATION_CREATE:
        return {
          ...operation,
          type: OperationType.ANNOTATION_DELETE,
        };

      case OperationType.ANNOTATION_DELETE:
        return {
          ...operation,
          type: OperationType.ANNOTATION_CREATE,
        };

      case OperationType.MEASUREMENT_CREATE:
        return {
          ...operation,
          type: OperationType.MEASUREMENT_DELETE,
        };

      case OperationType.MEASUREMENT_DELETE:
        return {
          ...operation,
          type: OperationType.MEASUREMENT_CREATE,
        };

      default:
        // For updates, we would need to store the previous state
        // For now, just return a marker operation
        return {
          ...operation,
          type: OperationType.CUSTOM,
          data: { ...data, undo: true },
        };
    }
  }

  /**
   * Get all operations in history
   */
  getAll(): Operation[] {
    return this.history
      .filter(entry => !entry.undone)
      .map(entry => entry.operation);
  }

  /**
   * Get operations for a specific client
   */
  getForClient(clientId: ClientId): Operation[] {
    return this.history
      .filter(entry => entry.operation.clientId === clientId && !entry.undone)
      .map(entry => entry.operation);
  }

  /**
   * Get operations in a time range
   */
  getInRange(startTime: number, endTime: number): Operation[] {
    return this.history
      .filter(
        entry =>
          !entry.undone &&
          entry.timestamp >= startTime &&
          entry.timestamp <= endTime
      )
      .map(entry => entry.operation);
  }

  /**
   * Get recent operations
   */
  getRecent(count: number): Operation[] {
    return this.history
      .filter(entry => !entry.undone)
      .slice(-count)
      .map(entry => entry.operation);
  }

  /**
   * Clear history
   */
  clear(): void {
    this.history = [];
    this.undoStack = [];
    this.redoStack = [];
  }

  /**
   * Get history size
   */
  size(): number {
    return this.history.filter(entry => !entry.undone).length;
  }

  /**
   * Get total history size (including undone)
   */
  totalSize(): number {
    return this.history.length;
  }

  /**
   * Get undo stack size
   */
  undoSize(): number {
    return this.undoStack.length;
  }

  /**
   * Get redo stack size
   */
  redoSize(): number {
    return this.redoStack.length;
  }

  /**
   * Check if undo is available
   */
  canUndo(clientId: ClientId): boolean {
    return this.history.some(
      entry => entry.operation.clientId === clientId && !entry.undone
    );
  }

  /**
   * Check if redo is available
   */
  canRedo(clientId: ClientId): boolean {
    return this.undoStack.some(op => op.clientId === clientId);
  }

  /**
   * Export history for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      history: this.history,
      undoStack: this.undoStack,
      redoStack: this.redoStack,
      maxHistorySize: this.maxHistorySize,
    };
  }

  /**
   * Import history from serialized data
   */
  fromJSON(data: {
    history: HistoryEntry[];
    undoStack: Operation[];
    redoStack: Operation[];
    maxHistorySize: number;
  }): void {
    this.history = data.history;
    this.undoStack = data.undoStack;
    this.redoStack = data.redoStack;
    this.maxHistorySize = data.maxHistorySize;
  }
}
