/**
 * IndexedDB Storage Layer
 * Provides persistent local storage for offline data
 */

import { StorageRecord, SyncOperation } from './types';

const DB_NAME = 'accuscene_offline';
const DB_VERSION = 1;

const STORES = {
  RECORDS: 'records',
  OPERATIONS: 'operations',
  METADATA: 'metadata',
} as const;

export class IndexedDBStore {
  private db: IDBDatabase | null = null;
  private initPromise: Promise<void> | null = null;

  constructor() {
    this.initPromise = this.initialize();
  }

  /**
   * Initialize IndexedDB
   */
  private async initialize(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        reject(new Error(`Failed to open IndexedDB: ${request.error}`));
      };

      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Records store
        if (!db.objectStoreNames.contains(STORES.RECORDS)) {
          const recordStore = db.createObjectStore(STORES.RECORDS, {
            keyPath: ['entityId', 'entityType'],
          });
          recordStore.createIndex('entityType', 'entityType', { unique: false });
          recordStore.createIndex('updatedAt', 'updatedAt', { unique: false });
        }

        // Operations store
        if (!db.objectStoreNames.contains(STORES.OPERATIONS)) {
          const opStore = db.createObjectStore(STORES.OPERATIONS, {
            keyPath: 'id',
          });
          opStore.createIndex('priority', 'priority', { unique: false });
          opStore.createIndex('queuedAt', 'queuedAt', { unique: false });
          opStore.createIndex('entityId', 'entityId', { unique: false });
        }

        // Metadata store
        if (!db.objectStoreNames.contains(STORES.METADATA)) {
          db.createObjectStore(STORES.METADATA, { keyPath: 'key' });
        }
      };
    });
  }

  /**
   * Ensure database is initialized
   */
  private async ensureInitialized(): Promise<void> {
    if (this.initPromise) {
      await this.initPromise;
    }
  }

  /**
   * Store a record
   */
  async putRecord(record: StorageRecord): Promise<void> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORES.RECORDS], 'readwrite');
      const store = transaction.objectStore(STORES.RECORDS);
      const request = store.put(record);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Get a record
   */
  async getRecord(
    entityId: string,
    entityType: string
  ): Promise<StorageRecord | null> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORES.RECORDS], 'readonly');
      const store = transaction.objectStore(STORES.RECORDS);
      const request = store.get([entityId, entityType]);

      request.onsuccess = () => {
        const record = request.result as StorageRecord | undefined;
        resolve(record && !record.deleted ? record : null);
      };
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Delete a record (mark as deleted)
   */
  async deleteRecord(entityId: string, entityType: string): Promise<void> {
    await this.ensureInitialized();

    const record = await this.getRecord(entityId, entityType);
    if (record) {
      record.deleted = true;
      record.updatedAt = new Date().toISOString();
      await this.putRecord(record);
    }
  }

  /**
   * List records by type
   */
  async listRecords(
    entityType: string,
    limit?: number
  ): Promise<StorageRecord[]> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORES.RECORDS], 'readonly');
      const store = transaction.objectStore(STORES.RECORDS);
      const index = store.index('entityType');
      const request = index.openCursor(IDBKeyRange.only(entityType));

      const records: StorageRecord[] = [];
      let count = 0;

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result as IDBCursorWithValue;

        if (cursor && (!limit || count < limit)) {
          const record = cursor.value as StorageRecord;
          if (!record.deleted) {
            records.push(record);
            count++;
          }
          cursor.continue();
        } else {
          resolve(records);
        }
      };

      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Store a sync operation
   */
  async storeOperation(operation: SyncOperation): Promise<void> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORES.OPERATIONS], 'readwrite');
      const store = transaction.objectStore(STORES.OPERATIONS);
      const request = store.put(operation);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Get all pending operations
   */
  async getPendingOperations(): Promise<SyncOperation[]> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORES.OPERATIONS], 'readonly');
      const store = transaction.objectStore(STORES.OPERATIONS);
      const request = store.getAll();

      request.onsuccess = () => {
        const operations = request.result as SyncOperation[];
        // Sort by priority (desc) and queued time (asc)
        operations.sort((a, b) => {
          if (a.priority !== b.priority) {
            return b.priority - a.priority;
          }
          return new Date(a.queuedAt).getTime() - new Date(b.queuedAt).getTime();
        });
        resolve(operations);
      };
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Delete an operation
   */
  async deleteOperation(operationId: string): Promise<void> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORES.OPERATIONS], 'readwrite');
      const store = transaction.objectStore(STORES.OPERATIONS);
      const request = store.delete(operationId);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Get metadata
   */
  async getMetadata(key: string): Promise<any> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORES.METADATA], 'readonly');
      const store = transaction.objectStore(STORES.METADATA);
      const request = store.get(key);

      request.onsuccess = () => {
        const result = request.result;
        resolve(result ? result.value : null);
      };
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Set metadata
   */
  async setMetadata(key: string, value: any): Promise<void> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORES.METADATA], 'readwrite');
      const store = transaction.objectStore(STORES.METADATA);
      const request = store.put({ key, value, updatedAt: new Date().toISOString() });

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Clear all data
   */
  async clear(): Promise<void> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(
        [STORES.RECORDS, STORES.OPERATIONS, STORES.METADATA],
        'readwrite'
      );

      const promises = [
        new Promise((res, rej) => {
          const req = transaction.objectStore(STORES.RECORDS).clear();
          req.onsuccess = () => res(undefined);
          req.onerror = () => rej(req.error);
        }),
        new Promise((res, rej) => {
          const req = transaction.objectStore(STORES.OPERATIONS).clear();
          req.onsuccess = () => res(undefined);
          req.onerror = () => rej(req.error);
        }),
        new Promise((res, rej) => {
          const req = transaction.objectStore(STORES.METADATA).clear();
          req.onsuccess = () => res(undefined);
          req.onerror = () => rej(req.error);
        }),
      ];

      Promise.all(promises)
        .then(() => resolve())
        .catch(reject);
    });
  }

  /**
   * Get storage size estimate
   */
  async getSize(): Promise<number> {
    if ('storage' in navigator && 'estimate' in navigator.storage) {
      const estimate = await navigator.storage.estimate();
      return estimate.usage || 0;
    }
    return 0;
  }

  /**
   * Close database connection
   */
  close(): void {
    if (this.db) {
      this.db.close();
      this.db = null;
    }
  }
}
