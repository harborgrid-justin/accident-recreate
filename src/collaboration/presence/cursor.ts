/**
 * AccuScene Enterprise - Cursor Position Tracking
 * v0.2.0
 *
 * Real-time cursor position tracking for collaborative editing
 */

import { ClientId, CursorPosition } from '../types';

export interface CursorStyle {
  color: string;
  size?: number;
  label?: string;
}

export interface CursorState extends CursorPosition {
  clientId: ClientId;
  style?: CursorStyle;
}

/**
 * Cursor Position Manager
 */
export class CursorManager {
  private cursors: Map<ClientId, CursorState>;
  private updateThrottle: number;
  private lastUpdate: Map<ClientId, number>;

  constructor(updateThrottle: number = 50) {
    this.cursors = new Map();
    this.lastUpdate = new Map();
    this.updateThrottle = updateThrottle;
  }

  /**
   * Update cursor position
   */
  updateCursor(clientId: ClientId, position: CursorPosition, style?: CursorStyle): boolean {
    const now = Date.now();
    const lastUpdateTime = this.lastUpdate.get(clientId) || 0;

    // Throttle updates
    if (now - lastUpdateTime < this.updateThrottle) {
      return false;
    }

    const cursor: CursorState = {
      clientId,
      x: position.x,
      y: position.y,
      timestamp: now,
      style,
    };

    this.cursors.set(clientId, cursor);
    this.lastUpdate.set(clientId, now);

    return true;
  }

  /**
   * Get cursor position
   */
  getCursor(clientId: ClientId): CursorState | undefined {
    return this.cursors.get(clientId);
  }

  /**
   * Get all cursors
   */
  getAllCursors(): CursorState[] {
    return Array.from(this.cursors.values());
  }

  /**
   * Remove cursor
   */
  removeCursor(clientId: ClientId): void {
    this.cursors.delete(clientId);
    this.lastUpdate.delete(clientId);
  }

  /**
   * Get cursors near a point
   */
  getCursorsNear(x: number, y: number, radius: number): CursorState[] {
    const nearby: CursorState[] = [];

    for (const cursor of this.cursors.values()) {
      const distance = Math.sqrt(
        Math.pow(cursor.x - x, 2) + Math.pow(cursor.y - y, 2)
      );

      if (distance <= radius) {
        nearby.push(cursor);
      }
    }

    return nearby;
  }

  /**
   * Get cursors in a rectangular area
   */
  getCursorsInArea(
    x: number,
    y: number,
    width: number,
    height: number
  ): CursorState[] {
    const inArea: CursorState[] = [];

    for (const cursor of this.cursors.values()) {
      if (
        cursor.x >= x &&
        cursor.x <= x + width &&
        cursor.y >= y &&
        cursor.y <= y + height
      ) {
        inArea.push(cursor);
      }
    }

    return inArea;
  }

  /**
   * Clear stale cursors (not updated recently)
   */
  clearStaleCursors(maxAge: number = 5000): ClientId[] {
    const now = Date.now();
    const stale: ClientId[] = [];

    for (const [clientId, cursor] of this.cursors.entries()) {
      if (now - cursor.timestamp > maxAge) {
        this.cursors.delete(clientId);
        this.lastUpdate.delete(clientId);
        stale.push(clientId);
      }
    }

    return stale;
  }

  /**
   * Get cursor count
   */
  getCount(): number {
    return this.cursors.size;
  }

  /**
   * Clear all cursors
   */
  clear(): void {
    this.cursors.clear();
    this.lastUpdate.clear();
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      cursors: Array.from(this.cursors.entries()),
      updateThrottle: this.updateThrottle,
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: {
    cursors: Array<[ClientId, CursorState]>;
    updateThrottle: number;
  }): void {
    this.cursors = new Map(data.cursors);
    this.updateThrottle = data.updateThrottle;
    this.lastUpdate.clear();
  }
}
