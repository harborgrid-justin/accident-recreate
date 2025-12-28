/**
 * AccuScene Enterprise - Selection Awareness
 * v0.2.0
 *
 * Track user selections for collaborative editing awareness
 */

import { ClientId, ObjectId, SelectionState } from '../types';

export interface SelectionRange {
  start: number;
  end: number;
}

export interface ExtendedSelectionState extends SelectionState {
  clientId: ClientId;
  type?: 'objects' | 'text' | 'area';
  range?: SelectionRange;
  area?: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
}

/**
 * Selection Awareness Manager
 */
export class SelectionManager {
  private selections: Map<ClientId, ExtendedSelectionState>;

  constructor() {
    this.selections = new Map();
  }

  /**
   * Update user selection
   */
  updateSelection(
    clientId: ClientId,
    objectIds: ObjectId[],
    type: 'objects' | 'text' | 'area' = 'objects',
    range?: SelectionRange,
    area?: { x: number; y: number; width: number; height: number }
  ): ExtendedSelectionState {
    const selection: ExtendedSelectionState = {
      clientId,
      objectIds,
      timestamp: Date.now(),
      type,
      range,
      area,
    };

    this.selections.set(clientId, selection);
    return selection;
  }

  /**
   * Get user selection
   */
  getSelection(clientId: ClientId): ExtendedSelectionState | undefined {
    return this.selections.get(clientId);
  }

  /**
   * Get all selections
   */
  getAllSelections(): ExtendedSelectionState[] {
    return Array.from(this.selections.values());
  }

  /**
   * Clear user selection
   */
  clearSelection(clientId: ClientId): void {
    this.selections.delete(clientId);
  }

  /**
   * Get users selecting a specific object
   */
  getUsersSelectingObject(objectId: ObjectId): ClientId[] {
    const users: ClientId[] = [];

    for (const [clientId, selection] of this.selections.entries()) {
      if (selection.objectIds.includes(objectId)) {
        users.push(clientId);
      }
    }

    return users;
  }

  /**
   * Get objects selected by a user
   */
  getSelectedObjects(clientId: ClientId): ObjectId[] {
    const selection = this.selections.get(clientId);
    return selection?.objectIds || [];
  }

  /**
   * Check if an object is selected by any user
   */
  isObjectSelected(objectId: ObjectId): boolean {
    for (const selection of this.selections.values()) {
      if (selection.objectIds.includes(objectId)) {
        return true;
      }
    }
    return false;
  }

  /**
   * Check if an object is selected by a specific user
   */
  isObjectSelectedByUser(objectId: ObjectId, clientId: ClientId): boolean {
    const selection = this.selections.get(clientId);
    return selection?.objectIds.includes(objectId) || false;
  }

  /**
   * Get conflicting selections (multiple users selecting same object)
   */
  getConflictingSelections(): Map<ObjectId, ClientId[]> {
    const conflicts = new Map<ObjectId, ClientId[]>();

    for (const [clientId, selection] of this.selections.entries()) {
      for (const objectId of selection.objectIds) {
        if (!conflicts.has(objectId)) {
          conflicts.set(objectId, []);
        }
        conflicts.get(objectId)!.push(clientId);
      }
    }

    // Keep only objects selected by multiple users
    for (const [objectId, users] of conflicts.entries()) {
      if (users.length <= 1) {
        conflicts.delete(objectId);
      }
    }

    return conflicts;
  }

  /**
   * Get selections overlapping with an area
   */
  getSelectionsInArea(
    x: number,
    y: number,
    width: number,
    height: number
  ): ExtendedSelectionState[] {
    const overlapping: ExtendedSelectionState[] = [];

    for (const selection of this.selections.values()) {
      if (selection.area) {
        if (this.areasOverlap(
          { x, y, width, height },
          selection.area
        )) {
          overlapping.push(selection);
        }
      }
    }

    return overlapping;
  }

  /**
   * Check if two areas overlap
   */
  private areasOverlap(
    area1: { x: number; y: number; width: number; height: number },
    area2: { x: number; y: number; width: number; height: number }
  ): boolean {
    return !(
      area1.x + area1.width < area2.x ||
      area2.x + area2.width < area1.x ||
      area1.y + area1.height < area2.y ||
      area2.y + area2.height < area1.y
    );
  }

  /**
   * Get selection count
   */
  getCount(): number {
    return this.selections.size;
  }

  /**
   * Get total selected objects count
   */
  getTotalSelectedObjects(): number {
    const objectIds = new Set<ObjectId>();

    for (const selection of this.selections.values()) {
      for (const objectId of selection.objectIds) {
        objectIds.add(objectId);
      }
    }

    return objectIds.size;
  }

  /**
   * Clear all selections
   */
  clear(): void {
    this.selections.clear();
  }

  /**
   * Clear stale selections (not updated recently)
   */
  clearStaleSelections(maxAge: number = 300000): ClientId[] {
    const now = Date.now();
    const stale: ClientId[] = [];

    for (const [clientId, selection] of this.selections.entries()) {
      if (now - selection.timestamp > maxAge) {
        this.selections.delete(clientId);
        stale.push(clientId);
      }
    }

    return stale;
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      selections: Array.from(this.selections.entries()),
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: { selections: Array<[ClientId, ExtendedSelectionState]> }): void {
    this.selections = new Map(data.selections);
  }
}
