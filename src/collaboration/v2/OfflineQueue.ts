/**
 * AccuScene Enterprise v0.3.0 - Offline Queue
 *
 * Queue operations while offline with conflict detection
 */

import { EventEmitter } from 'events';
import { OfflineOperation, Operation } from './types';

export class OfflineQueue extends EventEmitter {
  private queue: OfflineOperation[] = [];
  private maxOperations: number;
  private persistToStorage: boolean;

  constructor(config: { maxOperations: number; persistToStorage: boolean }) {
    super();
    this.maxOperations = config.maxOperations;
    this.persistToStorage = config.persistToStorage;

    if (this.persistToStorage) {
      this.loadFromStorage();
    }
  }

  async enqueue(operation: OfflineOperation): Promise<void> {
    if (this.queue.length >= this.maxOperations) {
      throw new Error('Offline queue is full');
    }

    this.queue.push(operation);

    if (this.persistToStorage) {
      await this.saveToStorage();
    }

    this.emit('enqueued', operation);
  }

  async dequeue(operationId: string): Promise<void> {
    const index = this.queue.findIndex(op => op.id === operationId);

    if (index !== -1) {
      const [operation] = this.queue.splice(index, 1);

      if (this.persistToStorage) {
        await this.saveToStorage();
      }

      this.emit('dequeued', operation);
    }
  }

  async getAll(): Promise<OfflineOperation[]> {
    return [...this.queue];
  }

  async clear(): Promise<void> {
    this.queue = [];

    if (this.persistToStorage) {
      await this.saveToStorage();
    }

    this.emit('cleared');
  }

  getCount(): number {
    return this.queue.length;
  }

  private async saveToStorage(): Promise<void> {
    try {
      localStorage.setItem('offline-queue', JSON.stringify(this.queue));
    } catch (error) {
      console.error('Failed to save offline queue:', error);
    }
  }

  private loadFromStorage(): void {
    try {
      const data = localStorage.getItem('offline-queue');
      if (data) {
        this.queue = JSON.parse(data);
      }
    } catch (error) {
      console.error('Failed to load offline queue:', error);
    }
  }
}
