/**
 * AccuScene Enterprise - Room Manager
 * v0.2.0
 *
 * Manage collaboration rooms and their lifecycle
 */

import { RoomId, ClientId, RoomState, ClientInfo, RoomConfig, RoomError } from '../types';
import { VectorClockManager } from '../sync/vector-clock';

/**
 * Room Manager
 */
export class RoomManager {
  private rooms: Map<RoomId, RoomState>;
  private defaultConfig: RoomConfig;

  constructor(defaultConfig?: Partial<RoomConfig>) {
    this.rooms = new Map();
    this.defaultConfig = {
      maxClients: 50,
      autoSnapshot: true,
      snapshotInterval: 300000, // 5 minutes
      operationHistoryLimit: 1000,
      persistenceEnabled: true,
      ...defaultConfig,
    };
  }

  /**
   * Create a new room
   */
  createRoom(
    roomId: RoomId,
    sceneId: string,
    name?: string,
    config?: Partial<RoomConfig>
  ): RoomState {
    if (this.rooms.has(roomId)) {
      throw new RoomError(`Room ${roomId} already exists`);
    }

    const now = Date.now();
    const room: RoomState = {
      id: roomId,
      name: name || `Room ${roomId}`,
      sceneId,
      clients: new Map(),
      operations: [],
      vectorClock: {},
      createdAt: now,
      updatedAt: now,
    };

    this.rooms.set(roomId, room);
    return room;
  }

  /**
   * Get a room by ID
   */
  getRoom(roomId: RoomId): RoomState | undefined {
    return this.rooms.get(roomId);
  }

  /**
   * Delete a room
   */
  deleteRoom(roomId: RoomId): boolean {
    return this.rooms.delete(roomId);
  }

  /**
   * Add client to room
   */
  addClient(roomId: RoomId, clientInfo: ClientInfo): void {
    const room = this.rooms.get(roomId);
    if (!room) {
      throw new RoomError(`Room ${roomId} not found`);
    }

    // Check max clients
    if (
      this.defaultConfig.maxClients &&
      room.clients.size >= this.defaultConfig.maxClients
    ) {
      throw new RoomError(`Room ${roomId} is full`);
    }

    room.clients.set(clientInfo.clientId, clientInfo);
    room.updatedAt = Date.now();

    // Initialize vector clock for new client
    if (!room.vectorClock[clientInfo.clientId]) {
      room.vectorClock[clientInfo.clientId] = 0;
    }
  }

  /**
   * Remove client from room
   */
  removeClient(roomId: RoomId, clientId: ClientId): void {
    const room = this.rooms.get(roomId);
    if (!room) {
      throw new RoomError(`Room ${roomId} not found`);
    }

    room.clients.delete(clientId);
    room.updatedAt = Date.now();
  }

  /**
   * Get client info
   */
  getClient(roomId: RoomId, clientId: ClientId): ClientInfo | undefined {
    const room = this.rooms.get(roomId);
    return room?.clients.get(clientId);
  }

  /**
   * Get all clients in a room
   */
  getClients(roomId: RoomId): ClientInfo[] {
    const room = this.rooms.get(roomId);
    return room ? Array.from(room.clients.values()) : [];
  }

  /**
   * Update client activity
   */
  updateClientActivity(roomId: RoomId, clientId: ClientId): void {
    const room = this.rooms.get(roomId);
    if (!room) {
      throw new RoomError(`Room ${roomId} not found`);
    }

    const client = room.clients.get(clientId);
    if (client) {
      client.lastActivity = Date.now();
      room.updatedAt = Date.now();
    }
  }

  /**
   * Check if room exists
   */
  hasRoom(roomId: RoomId): boolean {
    return this.rooms.has(roomId);
  }

  /**
   * Get all rooms
   */
  getAllRooms(): RoomState[] {
    return Array.from(this.rooms.values());
  }

  /**
   * Get active rooms (with clients)
   */
  getActiveRooms(): RoomState[] {
    return this.getAllRooms().filter(room => room.clients.size > 0);
  }

  /**
   * Get empty rooms
   */
  getEmptyRooms(): RoomState[] {
    return this.getAllRooms().filter(room => room.clients.size === 0);
  }

  /**
   * Get room count
   */
  getRoomCount(): number {
    return this.rooms.size;
  }

  /**
   * Get total client count across all rooms
   */
  getTotalClientCount(): number {
    return this.getAllRooms().reduce((total, room) => total + room.clients.size, 0);
  }

  /**
   * Clean up empty rooms
   */
  cleanupEmptyRooms(maxAge: number = 3600000): RoomId[] {
    const now = Date.now();
    const removed: RoomId[] = [];

    for (const [roomId, room] of this.rooms.entries()) {
      if (room.clients.size === 0 && now - room.updatedAt > maxAge) {
        this.rooms.delete(roomId);
        removed.push(roomId);
      }
    }

    return removed;
  }

  /**
   * Clean up inactive clients
   */
  cleanupInactiveClients(roomId: RoomId, maxInactivity: number = 300000): ClientId[] {
    const room = this.rooms.get(roomId);
    if (!room) {
      return [];
    }

    const now = Date.now();
    const removed: ClientId[] = [];

    for (const [clientId, client] of room.clients.entries()) {
      if (now - client.lastActivity > maxInactivity) {
        room.clients.delete(clientId);
        removed.push(clientId);
      }
    }

    if (removed.length > 0) {
      room.updatedAt = now;
    }

    return removed;
  }

  /**
   * Get room statistics
   */
  getRoomStats(roomId: RoomId): Record<string, unknown> | null {
    const room = this.rooms.get(roomId);
    if (!room) {
      return null;
    }

    return {
      id: room.id,
      name: room.name,
      sceneId: room.sceneId,
      clientCount: room.clients.size,
      operationCount: room.operations.length,
      createdAt: room.createdAt,
      updatedAt: room.updatedAt,
      age: Date.now() - room.createdAt,
    };
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      rooms: Array.from(this.rooms.entries()).map(([id, room]) => [
        id,
        {
          ...room,
          clients: Array.from(room.clients.entries()),
        },
      ]),
      defaultConfig: this.defaultConfig,
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: {
    rooms: Array<[RoomId, any]>;
    defaultConfig: RoomConfig;
  }): void {
    this.rooms = new Map(
      data.rooms.map(([id, room]) => [
        id,
        {
          ...room,
          clients: new Map(room.clients),
        },
      ])
    );
    this.defaultConfig = data.defaultConfig;
  }
}
