/**
 * AccuScene Enterprise - User Presence Tracker
 * v0.2.0
 *
 * Track and manage user presence in collaboration sessions
 */

import { ClientId, UserId, UserPresence, UserMetadata, SessionId } from '../types';

/**
 * User Presence Tracker
 */
export class PresenceTracker {
  private presences: Map<ClientId, UserPresence>;
  private lastHeartbeat: Map<ClientId, number>;
  private heartbeatInterval: number;
  private timeoutThreshold: number;

  constructor(heartbeatInterval: number = 5000, timeoutThreshold: number = 30000) {
    this.presences = new Map();
    this.lastHeartbeat = new Map();
    this.heartbeatInterval = heartbeatInterval;
    this.timeoutThreshold = timeoutThreshold;
  }

  /**
   * Add or update a user's presence
   */
  updatePresence(clientId: ClientId, presence: Partial<UserPresence>): UserPresence {
    const now = Date.now();
    const existing = this.presences.get(clientId);

    const updated: UserPresence = {
      userId: presence.userId || existing?.userId || clientId,
      clientId,
      sessionId: presence.sessionId || existing?.sessionId || clientId,
      online: true,
      lastSeen: now,
      cursor: presence.cursor || existing?.cursor,
      selection: presence.selection || existing?.selection,
      viewport: presence.viewport || existing?.viewport,
      metadata: presence.metadata || existing?.metadata,
    };

    this.presences.set(clientId, updated);
    this.lastHeartbeat.set(clientId, now);

    return updated;
  }

  /**
   * Get a user's presence
   */
  getPresence(clientId: ClientId): UserPresence | undefined {
    return this.presences.get(clientId);
  }

  /**
   * Get all presences
   */
  getAllPresences(): UserPresence[] {
    return Array.from(this.presences.values());
  }

  /**
   * Get online users
   */
  getOnlineUsers(): UserPresence[] {
    return this.getAllPresences().filter(p => p.online);
  }

  /**
   * Get offline users
   */
  getOfflineUsers(): UserPresence[] {
    return this.getAllPresences().filter(p => !p.online);
  }

  /**
   * Mark a user as offline
   */
  setOffline(clientId: ClientId): void {
    const presence = this.presences.get(clientId);
    if (presence) {
      presence.online = false;
      presence.lastSeen = Date.now();
      this.presences.set(clientId, presence);
    }
  }

  /**
   * Remove a user's presence
   */
  removePresence(clientId: ClientId): void {
    this.presences.delete(clientId);
    this.lastHeartbeat.delete(clientId);
  }

  /**
   * Update heartbeat for a client
   */
  heartbeat(clientId: ClientId): void {
    this.lastHeartbeat.set(clientId, Date.now());

    const presence = this.presences.get(clientId);
    if (presence) {
      presence.online = true;
      presence.lastSeen = Date.now();
    }
  }

  /**
   * Check for timed-out users and mark them offline
   */
  checkTimeouts(): ClientId[] {
    const now = Date.now();
    const timedOut: ClientId[] = [];

    for (const [clientId, lastBeat] of this.lastHeartbeat.entries()) {
      if (now - lastBeat > this.timeoutThreshold) {
        this.setOffline(clientId);
        timedOut.push(clientId);
      }
    }

    return timedOut;
  }

  /**
   * Start automatic timeout checking
   */
  startTimeoutChecker(callback?: (timedOut: ClientId[]) => void): NodeJS.Timeout {
    return setInterval(() => {
      const timedOut = this.checkTimeouts();
      if (timedOut.length > 0 && callback) {
        callback(timedOut);
      }
    }, this.heartbeatInterval);
  }

  /**
   * Get user count
   */
  getUserCount(): number {
    return this.presences.size;
  }

  /**
   * Get online user count
   */
  getOnlineCount(): number {
    return this.getOnlineUsers().length;
  }

  /**
   * Check if a user is online
   */
  isOnline(clientId: ClientId): boolean {
    const presence = this.presences.get(clientId);
    return presence?.online || false;
  }

  /**
   * Get users by userId (can have multiple clients)
   */
  getUserSessions(userId: UserId): UserPresence[] {
    return this.getAllPresences().filter(p => p.userId === userId);
  }

  /**
   * Clear all presences
   */
  clear(): void {
    this.presences.clear();
    this.lastHeartbeat.clear();
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      presences: Array.from(this.presences.entries()),
      lastHeartbeat: Array.from(this.lastHeartbeat.entries()),
      heartbeatInterval: this.heartbeatInterval,
      timeoutThreshold: this.timeoutThreshold,
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: {
    presences: Array<[ClientId, UserPresence]>;
    lastHeartbeat: Array<[ClientId, number]>;
    heartbeatInterval: number;
    timeoutThreshold: number;
  }): void {
    this.presences = new Map(data.presences);
    this.lastHeartbeat = new Map(data.lastHeartbeat);
    this.heartbeatInterval = data.heartbeatInterval;
    this.timeoutThreshold = data.timeoutThreshold;
  }
}
