/**
 * AccuScene Enterprise v0.3.0 - Presence Manager
 *
 * Manages user presence, cursors, selections, and activity status
 */

import { EventEmitter } from 'events';
import {
  Presence,
  User,
  UserId,
  SessionId,
  CursorPosition,
  Selection,
  Viewport,
  UserStatus
} from './types';

interface PresenceUpdate {
  userId: UserId;
  field: keyof Presence;
  value: any;
  timestamp: number;
}

export class PresenceManager extends EventEmitter {
  private presences: Map<UserId, Presence> = new Map();
  private currentUser: User | null = null;
  private sessionId: SessionId | null = null;
  private activityTimeouts: Map<UserId, NodeJS.Timeout> = new Map();
  private updateThrottle: Map<string, number> = new Map();

  // Configuration
  private readonly CURSOR_THROTTLE_MS = 50; // 20 updates/sec max
  private readonly ACTIVITY_TIMEOUT_MS = 30000; // 30 seconds
  private readonly AWAY_TIMEOUT_MS = 300000; // 5 minutes
  private readonly UPDATE_BATCH_SIZE = 10;

  private pendingUpdates: PresenceUpdate[] = [];
  private updateInterval: NodeJS.Timeout | null = null;

  constructor() {
    super();
  }

  // ============================================================================
  // Public API
  // ============================================================================

  async initialize(user: User, sessionId: SessionId): Promise<void> {
    this.currentUser = user;
    this.sessionId = sessionId;

    // Set initial presence for current user
    const initialPresence: Presence = {
      userId: user.id,
      user,
      cursor: null,
      selection: null,
      viewport: null,
      status: UserStatus.ONLINE,
      lastActivity: Date.now(),
      isTyping: false
    };

    this.presences.set(user.id, initialPresence);

    // Start update batch processor
    this.startUpdateProcessor();

    // Start activity monitoring
    this.startActivityMonitoring();
  }

  destroy(): void {
    this.stopUpdateProcessor();
    this.stopActivityMonitoring();
    this.presences.clear();
    this.activityTimeouts.clear();
    this.updateThrottle.clear();
    this.removeAllListeners();
  }

  updateCursor(userId: UserId, cursor: CursorPosition | null): void {
    // Throttle cursor updates
    const throttleKey = `cursor-${userId}`;
    if (this.shouldThrottle(throttleKey, this.CURSOR_THROTTLE_MS)) {
      return;
    }

    this.queueUpdate({
      userId,
      field: 'cursor',
      value: cursor,
      timestamp: Date.now()
    });

    this.updateActivity(userId);
  }

  updateSelection(userId: UserId, selection: Selection | null): void {
    this.queueUpdate({
      userId,
      field: 'selection',
      value: selection,
      timestamp: Date.now()
    });

    this.updateActivity(userId);
  }

  updateViewport(userId: UserId, viewport: Viewport | null): void {
    // Throttle viewport updates
    const throttleKey = `viewport-${userId}`;
    if (this.shouldThrottle(throttleKey, 200)) {
      return;
    }

    this.queueUpdate({
      userId,
      field: 'viewport',
      value: viewport,
      timestamp: Date.now()
    });

    this.updateActivity(userId);
  }

  updateStatus(userId: UserId, status: UserStatus): void {
    this.queueUpdate({
      userId,
      field: 'status',
      value: status,
      timestamp: Date.now()
    });

    this.emit('statusChange', { userId, status });
  }

  setTyping(userId: UserId, isTyping: boolean): void {
    this.queueUpdate({
      userId,
      field: 'isTyping',
      value: isTyping,
      timestamp: Date.now()
    });

    if (isTyping) {
      this.updateActivity(userId);
    }
  }

  setCurrentTool(userId: UserId, tool: string | undefined): void {
    this.queueUpdate({
      userId,
      field: 'currentTool',
      value: tool,
      timestamp: Date.now()
    });
  }

  updatePresence(userId: UserId, updates: Partial<Presence>): void {
    const presence = this.presences.get(userId);

    if (!presence) {
      console.warn(`No presence found for user ${userId}`);
      return;
    }

    const updated: Presence = {
      ...presence,
      ...updates,
      lastActivity: Date.now()
    };

    this.presences.set(userId, updated);
    this.emit('presenceUpdate', { userId, presence: updated });

    this.updateActivity(userId);
  }

  addUser(user: User): void {
    const presence: Presence = {
      userId: user.id,
      user,
      cursor: null,
      selection: null,
      viewport: null,
      status: UserStatus.ONLINE,
      lastActivity: Date.now(),
      isTyping: false
    };

    this.presences.set(user.id, presence);
    this.emit('userJoined', { user, presence });
    this.startActivityTimeout(user.id);
  }

  removeUser(userId: UserId): void {
    const presence = this.presences.get(userId);

    if (presence) {
      this.presences.delete(userId);
      this.clearActivityTimeout(userId);
      this.emit('userLeft', { userId, presence });
    }
  }

  getPresence(userId: UserId): Presence | null {
    return this.presences.get(userId) || null;
  }

  getAllPresences(): Presence[] {
    return Array.from(this.presences.values());
  }

  getActiveUsers(): User[] {
    return Array.from(this.presences.values())
      .filter(p => p.status === UserStatus.ONLINE || p.status === UserStatus.BUSY)
      .map(p => p.user);
  }

  getActiveUserCount(): number {
    return this.getActiveUsers().length;
  }

  getUsersWithSelection(): Presence[] {
    return Array.from(this.presences.values())
      .filter(p => p.selection !== null && p.selection.objectIds.length > 0);
  }

  getUsersViewingObject(objectId: string): User[] {
    return Array.from(this.presences.values())
      .filter(p => {
        return p.selection?.objectIds.includes(objectId) ||
               p.cursor !== null; // Rough approximation
      })
      .map(p => p.user);
  }

  isUserActive(userId: UserId): boolean {
    const presence = this.presences.get(userId);
    if (!presence) return false;

    const timeSinceActivity = Date.now() - presence.lastActivity;
    return timeSinceActivity < this.ACTIVITY_TIMEOUT_MS;
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private queueUpdate(update: PresenceUpdate): void {
    this.pendingUpdates.push(update);

    // Apply update immediately to local state
    this.applyUpdate(update);

    // Emit immediately for local feedback
    const presence = this.presences.get(update.userId);
    if (presence) {
      this.emit('presenceUpdate', { userId: update.userId, presence });
    }
  }

  private applyUpdate(update: PresenceUpdate): void {
    const presence = this.presences.get(update.userId);

    if (!presence) {
      console.warn(`Cannot apply update: no presence for user ${update.userId}`);
      return;
    }

    (presence as any)[update.field] = update.value;
    presence.lastActivity = update.timestamp;

    this.presences.set(update.userId, presence);
  }

  private startUpdateProcessor(): void {
    this.updateInterval = setInterval(() => {
      this.processPendingUpdates();
    }, 100); // Process every 100ms
  }

  private stopUpdateProcessor(): void {
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
  }

  private processPendingUpdates(): void {
    if (this.pendingUpdates.length === 0) return;

    // Take batch of updates
    const batch = this.pendingUpdates.splice(0, this.UPDATE_BATCH_SIZE);

    // Emit batch update event (for network sync)
    this.emit('batchUpdate', { updates: batch });
  }

  private startActivityMonitoring(): void {
    setInterval(() => {
      this.checkUserActivity();
    }, 10000); // Check every 10 seconds
  }

  private stopActivityMonitoring(): void {
    for (const timeout of this.activityTimeouts.values()) {
      clearTimeout(timeout);
    }
    this.activityTimeouts.clear();
  }

  private checkUserActivity(): void {
    const now = Date.now();

    for (const [userId, presence] of this.presences.entries()) {
      const timeSinceActivity = now - presence.lastActivity;

      // Update status based on inactivity
      if (timeSinceActivity > this.AWAY_TIMEOUT_MS && presence.status === UserStatus.ONLINE) {
        this.updateStatus(userId, UserStatus.AWAY);
      } else if (timeSinceActivity < this.AWAY_TIMEOUT_MS && presence.status === UserStatus.AWAY) {
        this.updateStatus(userId, UserStatus.ONLINE);
      }
    }
  }

  private updateActivity(userId: UserId): void {
    const presence = this.presences.get(userId);
    if (presence) {
      presence.lastActivity = Date.now();
      this.presences.set(userId, presence);

      // Reset activity timeout
      this.resetActivityTimeout(userId);
    }
  }

  private startActivityTimeout(userId: UserId): void {
    this.clearActivityTimeout(userId);

    const timeout = setTimeout(() => {
      this.updateStatus(userId, UserStatus.AWAY);
    }, this.AWAY_TIMEOUT_MS);

    this.activityTimeouts.set(userId, timeout);
  }

  private resetActivityTimeout(userId: UserId): void {
    this.startActivityTimeout(userId);
  }

  private clearActivityTimeout(userId: UserId): void {
    const timeout = this.activityTimeouts.get(userId);
    if (timeout) {
      clearTimeout(timeout);
      this.activityTimeouts.delete(userId);
    }
  }

  private shouldThrottle(key: string, intervalMs: number): boolean {
    const lastUpdate = this.updateThrottle.get(key) || 0;
    const now = Date.now();

    if (now - lastUpdate < intervalMs) {
      return true;
    }

    this.updateThrottle.set(key, now);
    return false;
  }

  // ============================================================================
  // Statistics & Analytics
  // ============================================================================

  getPresenceStats() {
    const presences = Array.from(this.presences.values());

    return {
      total: presences.length,
      online: presences.filter(p => p.status === UserStatus.ONLINE).length,
      away: presences.filter(p => p.status === UserStatus.AWAY).length,
      busy: presences.filter(p => p.status === UserStatus.BUSY).length,
      offline: presences.filter(p => p.status === UserStatus.OFFLINE).length,
      withCursor: presences.filter(p => p.cursor !== null).length,
      withSelection: presences.filter(p => p.selection !== null).length,
      typing: presences.filter(p => p.isTyping).length
    };
  }

  getCursorDensityMap(): Map<string, number> {
    // Create a grid-based density map of cursor positions
    const density = new Map<string, number>();
    const gridSize = 100; // pixels

    for (const presence of this.presences.values()) {
      if (presence.cursor) {
        const gridX = Math.floor(presence.cursor.x / gridSize);
        const gridY = Math.floor(presence.cursor.y / gridSize);
        const key = `${gridX},${gridY}`;

        density.set(key, (density.get(key) || 0) + 1);
      }
    }

    return density;
  }

  getSelectionOverlap(): Map<string, UserId[]> {
    // Find which objects are selected by multiple users
    const overlap = new Map<string, UserId[]>();

    for (const presence of this.presences.values()) {
      if (presence.selection) {
        for (const objectId of presence.selection.objectIds) {
          const users = overlap.get(objectId) || [];
          users.push(presence.userId);
          overlap.set(objectId, users);
        }
      }
    }

    // Filter to only overlapping selections
    for (const [objectId, users] of overlap.entries()) {
      if (users.length < 2) {
        overlap.delete(objectId);
      }
    }

    return overlap;
  }
}
