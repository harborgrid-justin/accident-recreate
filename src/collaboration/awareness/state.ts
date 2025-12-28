/**
 * AccuScene Enterprise - Awareness State Management
 * v0.2.0
 *
 * Manages awareness state for collaborative editing
 */

import { ClientId, AwarenessState, AwarenessUpdate, UserMetadata, UserPresence } from '../types';

/**
 * Awareness State Manager
 */
export class AwarenessStateManager {
  private states: Map<ClientId, AwarenessState>;
  private updateListeners: Set<(update: AwarenessUpdate) => void>;

  constructor() {
    this.states = new Map();
    this.updateListeners = new Set();
  }

  /**
   * Set awareness state for a client
   */
  setState(clientId: ClientId, state: Partial<AwarenessState>): void {
    const existing = this.states.get(clientId);
    const now = Date.now();

    const newState: AwarenessState = {
      clientId,
      user: state.user || existing?.user || { name: 'Unknown', color: '#000000' },
      presence: state.presence || existing?.presence || {
        userId: clientId,
        clientId,
        sessionId: clientId,
        online: true,
        lastSeen: now,
      },
      lastUpdate: now,
    };

    const wasNew = !existing;
    this.states.set(clientId, newState);

    // Notify listeners
    this.notifyUpdate({
      added: wasNew ? [clientId] : [],
      updated: wasNew ? [] : [clientId],
      removed: [],
    });
  }

  /**
   * Update user metadata
   */
  updateUser(clientId: ClientId, user: Partial<UserMetadata>): void {
    const state = this.states.get(clientId);
    if (state) {
      state.user = { ...state.user, ...user };
      state.lastUpdate = Date.now();
      this.states.set(clientId, state);

      this.notifyUpdate({
        added: [],
        updated: [clientId],
        removed: [],
      });
    }
  }

  /**
   * Update presence
   */
  updatePresence(clientId: ClientId, presence: Partial<UserPresence>): void {
    const state = this.states.get(clientId);
    if (state) {
      state.presence = { ...state.presence, ...presence };
      state.lastUpdate = Date.now();
      this.states.set(clientId, state);

      this.notifyUpdate({
        added: [],
        updated: [clientId],
        removed: [],
      });
    }
  }

  /**
   * Get awareness state for a client
   */
  getState(clientId: ClientId): AwarenessState | undefined {
    return this.states.get(clientId);
  }

  /**
   * Get all awareness states
   */
  getAllStates(): AwarenessState[] {
    return Array.from(this.states.values());
  }

  /**
   * Get online states
   */
  getOnlineStates(): AwarenessState[] {
    return this.getAllStates().filter(state => state.presence.online);
  }

  /**
   * Remove awareness state
   */
  removeState(clientId: ClientId): void {
    if (this.states.delete(clientId)) {
      this.notifyUpdate({
        added: [],
        updated: [],
        removed: [clientId],
      });
    }
  }

  /**
   * Clear stale states
   */
  clearStaleStates(maxAge: number = 30000): ClientId[] {
    const now = Date.now();
    const stale: ClientId[] = [];

    for (const [clientId, state] of this.states.entries()) {
      if (now - state.lastUpdate > maxAge) {
        this.states.delete(clientId);
        stale.push(clientId);
      }
    }

    if (stale.length > 0) {
      this.notifyUpdate({
        added: [],
        updated: [],
        removed: stale,
      });
    }

    return stale;
  }

  /**
   * Subscribe to awareness updates
   */
  onUpdate(listener: (update: AwarenessUpdate) => void): () => void {
    this.updateListeners.add(listener);
    return () => this.updateListeners.delete(listener);
  }

  /**
   * Notify all listeners of an update
   */
  private notifyUpdate(update: AwarenessUpdate): void {
    for (const listener of this.updateListeners) {
      try {
        listener(update);
      } catch (error) {
        console.error('Error in awareness update listener:', error);
      }
    }
  }

  /**
   * Get client count
   */
  getClientCount(): number {
    return this.states.size;
  }

  /**
   * Get online client count
   */
  getOnlineCount(): number {
    return this.getOnlineStates().length;
  }

  /**
   * Clear all states
   */
  clear(): void {
    const removed = Array.from(this.states.keys());
    this.states.clear();

    if (removed.length > 0) {
      this.notifyUpdate({
        added: [],
        updated: [],
        removed,
      });
    }
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      states: Array.from(this.states.entries()),
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: { states: Array<[ClientId, AwarenessState]> }): void {
    const previousClients = new Set(this.states.keys());
    this.states = new Map(data.states);
    const currentClients = new Set(this.states.keys());

    const added: ClientId[] = [];
    const updated: ClientId[] = [];
    const removed: ClientId[] = [];

    for (const client of currentClients) {
      if (!previousClients.has(client)) {
        added.push(client);
      } else {
        updated.push(client);
      }
    }

    for (const client of previousClients) {
      if (!currentClients.has(client)) {
        removed.push(client);
      }
    }

    if (added.length > 0 || updated.length > 0 || removed.length > 0) {
      this.notifyUpdate({ added, updated, removed });
    }
  }
}
