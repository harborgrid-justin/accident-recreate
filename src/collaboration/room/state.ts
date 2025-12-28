/**
 * AccuScene Enterprise - Room State Management
 * v0.2.0
 *
 * Manage collaborative room state with CRDTs
 */

import { RoomId, Operation, VectorClock, SceneState, ObjectId } from '../types';
import { LWWMap } from '../crdt/lww-map';
import { VectorClockManager } from '../sync/vector-clock';

/**
 * Room State Manager
 */
export class RoomStateManager {
  private roomId: RoomId;
  private clientId: string;
  private vectorClock: VectorClock;
  private objects: LWWMap<string, any>;
  private annotations: LWWMap<string, any>;
  private measurements: LWWMap<string, any>;
  private properties: LWWMap<string, any>;
  private operations: Operation[];

  constructor(roomId: RoomId, clientId: string) {
    this.roomId = roomId;
    this.clientId = clientId;
    this.vectorClock = VectorClockManager.create(clientId);
    this.objects = new LWWMap(clientId);
    this.annotations = new LWWMap(clientId);
    this.measurements = new LWWMap(clientId);
    this.properties = new LWWMap(clientId);
    this.operations = [];
  }

  /**
   * Apply an operation to the state
   */
  applyOperation(operation: Operation): void {
    // Update vector clock
    this.vectorClock = VectorClockManager.merge(this.vectorClock, operation.vectorClock);

    // Store operation
    this.operations.push(operation);

    // Apply to appropriate CRDT
    const data = operation.data as any;

    if (operation.type.toString().startsWith('object:')) {
      if (operation.type.toString().includes('delete')) {
        this.objects.delete(data.objectId);
      } else {
        this.objects.set(data.objectId, data);
      }
    } else if (operation.type.toString().startsWith('annotation:')) {
      if (operation.type.toString().includes('delete')) {
        this.annotations.delete(data.annotationId);
      } else {
        this.annotations.set(data.annotationId, data);
      }
    } else if (operation.type.toString().startsWith('measurement:')) {
      if (operation.type.toString().includes('delete')) {
        this.measurements.delete(data.measurementId);
      } else {
        this.measurements.set(data.measurementId, data);
      }
    } else if (operation.type.toString().startsWith('scene:')) {
      if (data.properties) {
        for (const [key, value] of Object.entries(data.properties)) {
          this.properties.set(key, value);
        }
      }
    }
  }

  /**
   * Apply multiple operations
   */
  applyOperations(operations: Operation[]): void {
    for (const op of operations) {
      this.applyOperation(op);
    }
  }

  /**
   * Get the current scene state
   */
  getSceneState(): SceneState {
    return {
      objects: this.objects.getValue() as Map<ObjectId, any>,
      annotations: this.annotations.getValue() as Map<string, any>,
      measurements: this.measurements.getValue() as Map<string, any>,
      properties: Object.fromEntries(this.properties.entries()),
    };
  }

  /**
   * Get vector clock
   */
  getVectorClock(): VectorClock {
    return { ...this.vectorClock };
  }

  /**
   * Get all operations
   */
  getOperations(): Operation[] {
    return [...this.operations];
  }

  /**
   * Get operations since a vector clock
   */
  getOperationsSince(clock: VectorClock): Operation[] {
    return this.operations.filter(op => {
      const opCounter = op.vectorClock[op.clientId] || 0;
      const clockCounter = clock[op.clientId] || 0;
      return opCounter > clockCounter;
    });
  }

  /**
   * Merge state from another room
   */
  mergeState(other: RoomStateManager): void {
    // Merge vector clocks
    this.vectorClock = VectorClockManager.merge(this.vectorClock, other.vectorClock);

    // Merge CRDTs
    this.objects.merge(other.objects);
    this.annotations.merge(other.annotations);
    this.measurements.merge(other.measurements);
    this.properties.merge(other.properties);

    // Merge operations
    const existingIds = new Set(this.operations.map(op => op.id));
    for (const op of other.operations) {
      if (!existingIds.has(op.id)) {
        this.operations.push(op);
      }
    }

    // Sort operations by timestamp
    this.operations.sort((a, b) => a.timestamp.counter - b.timestamp.counter);
  }

  /**
   * Clear old operations (keep recent ones)
   */
  pruneOperations(keepCount: number = 1000): void {
    if (this.operations.length > keepCount) {
      this.operations = this.operations.slice(-keepCount);
    }
  }

  /**
   * Get object by ID
   */
  getObject(objectId: ObjectId): any | undefined {
    return this.objects.get(objectId);
  }

  /**
   * Get all objects
   */
  getObjects(): Map<ObjectId, any> {
    return this.objects.getValue() as Map<ObjectId, any>;
  }

  /**
   * Get annotation by ID
   */
  getAnnotation(annotationId: string): any | undefined {
    return this.annotations.get(annotationId);
  }

  /**
   * Get all annotations
   */
  getAnnotations(): Map<string, any> {
    return this.annotations.getValue() as Map<string, any>;
  }

  /**
   * Get measurement by ID
   */
  getMeasurement(measurementId: string): any | undefined {
    return this.measurements.get(measurementId);
  }

  /**
   * Get all measurements
   */
  getMeasurements(): Map<string, any> {
    return this.measurements.getValue() as Map<string, any>;
  }

  /**
   * Get property
   */
  getProperty(key: string): any | undefined {
    return this.properties.get(key);
  }

  /**
   * Get all properties
   */
  getProperties(): Record<string, any> {
    return Object.fromEntries(this.properties.entries());
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      roomId: this.roomId,
      clientId: this.clientId,
      vectorClock: this.vectorClock,
      objects: this.objects.toJSON(),
      annotations: this.annotations.toJSON(),
      measurements: this.measurements.toJSON(),
      properties: this.properties.toJSON(),
      operations: this.operations,
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: any): void {
    this.roomId = data.roomId;
    this.clientId = data.clientId;
    this.vectorClock = data.vectorClock;
    this.objects.fromJSON(data.objects);
    this.annotations.fromJSON(data.annotations);
    this.measurements.fromJSON(data.measurements);
    this.properties.fromJSON(data.properties);
    this.operations = data.operations;
  }
}
