/**
 * AccuScene Enterprise - Collaboration Client
 * v0.2.0
 *
 * Client-side collaboration manager for real-time editing
 */

import { v4 as uuidv4 } from 'uuid';
import {
  ClientId,
  UserId,
  RoomId,
  Operation,
  Message,
  MessageType,
  VectorClock,
  UserPresence,
  SceneState,
  ConnectionConfig,
  EventEmitter,
  EventHandler,
  UnsubscribeFn,
} from './types';
import { WebSocketTransport } from './transport/websocket';
import { RoomStateManager } from './room/state';
import { PresenceTracker } from './presence/tracker';
import { CursorManager } from './presence/cursor';
import { SelectionManager } from './presence/selection';
import { AwarenessStateManager } from './awareness/state';
import { OperationHistory } from './operations/history';
import { VectorClockManager } from './sync/vector-clock';
import { DifferentialSync } from './sync/diff';

export interface CollaborationClientConfig {
  url: string;
  userId: UserId;
  reconnect?: boolean;
  reconnectDelay?: number;
  maxReconnectAttempts?: number;
  heartbeatInterval?: number;
}

/**
 * Collaboration Client
 */
export class CollaborationClient implements EventEmitter {
  private clientId: ClientId;
  private userId: UserId;
  private config: CollaborationClientConfig;
  private transport: WebSocketTransport;
  private currentRoom: RoomId | null = null;
  private roomState: RoomStateManager | null = null;
  private presenceTracker: PresenceTracker;
  private cursorManager: CursorManager;
  private selectionManager: SelectionManager;
  private awarenessManager: AwarenessStateManager;
  private operationHistory: OperationHistory;
  private vectorClock: VectorClock;
  private eventHandlers: Map<string, Set<EventHandler>>;
  private pendingOperations: Operation[] = [];
  private connected: boolean = false;

  constructor(config: CollaborationClientConfig) {
    this.clientId = uuidv4();
    this.userId = config.userId;
    this.config = config;
    this.transport = new WebSocketTransport(this.clientId);
    this.presenceTracker = new PresenceTracker();
    this.cursorManager = new CursorManager();
    this.selectionManager = new SelectionManager();
    this.awarenessManager = new AwarenessStateManager();
    this.operationHistory = new OperationHistory();
    this.vectorClock = VectorClockManager.create(this.clientId);
    this.eventHandlers = new Map();

    this.setupTransport();
  }

  /**
   * Setup transport event handlers
   */
  private setupTransport(): void {
    this.transport.on('connected', () => {
      this.connected = true;
      this.emit('connected', { clientId: this.clientId });

      // Resync if we had a room
      if (this.currentRoom) {
        this.requestSync();
      }
    });

    this.transport.on('disconnected', () => {
      this.connected = false;
      this.emit('disconnected', { clientId: this.clientId });
    });

    this.transport.on('message', (data: any) => {
      this.handleMessage(data as Message);
    });

    this.transport.on('error', (error: any) => {
      this.emit('error', error);
    });
  }

  /**
   * Connect to collaboration server
   */
  async connect(): Promise<void> {
    const connectionConfig: ConnectionConfig = {
      url: `${this.config.url}?userId=${this.userId}`,
      reconnect: this.config.reconnect !== false,
      reconnectDelay: this.config.reconnectDelay || 1000,
      maxReconnectAttempts: this.config.maxReconnectAttempts || 5,
      heartbeatInterval: this.config.heartbeatInterval || 30000,
    };

    await this.transport.connect(connectionConfig);
  }

  /**
   * Disconnect from server
   */
  async disconnect(): Promise<void> {
    if (this.currentRoom) {
      await this.leaveRoom();
    }

    await this.transport.disconnect();
    this.connected = false;
  }

  /**
   * Join a collaboration room
   */
  async joinRoom(roomId: RoomId, sceneId: string): Promise<void> {
    if (!this.connected) {
      throw new Error('Not connected to server');
    }

    if (this.currentRoom) {
      await this.leaveRoom();
    }

    this.currentRoom = roomId;
    this.roomState = new RoomStateManager(roomId, this.clientId);

    await this.transport.send({
      type: MessageType.JOIN_ROOM,
      id: uuidv4(),
      clientId: this.clientId,
      timestamp: Date.now(),
      data: { roomId, sceneId, userId: this.userId },
    });
  }

  /**
   * Leave current room
   */
  async leaveRoom(): Promise<void> {
    if (!this.currentRoom) {
      return;
    }

    await this.transport.send({
      type: MessageType.LEAVE_ROOM,
      id: uuidv4(),
      clientId: this.clientId,
      roomId: this.currentRoom,
      timestamp: Date.now(),
      data: {},
    });

    this.currentRoom = null;
    this.roomState = null;
  }

  /**
   * Apply a local operation
   */
  applyOperation(operation: Operation): void {
    if (!this.roomState) {
      throw new Error('Not in a room');
    }

    // Update vector clock
    this.vectorClock = VectorClockManager.increment(this.vectorClock, this.clientId);
    operation.vectorClock = { ...this.vectorClock };
    operation.timestamp = {
      clientId: this.clientId,
      counter: this.vectorClock[this.clientId],
    };

    // Apply locally
    this.roomState.applyOperation(operation);
    this.operationHistory.add(operation);

    // Send to server
    if (this.connected) {
      this.transport.send({
        type: MessageType.OPERATION,
        id: uuidv4(),
        clientId: this.clientId,
        roomId: this.currentRoom!,
        timestamp: Date.now(),
        data: operation,
      }).catch((error) => {
        console.error('Failed to send operation:', error);
        this.pendingOperations.push(operation);
      });
    } else {
      this.pendingOperations.push(operation);
    }

    this.emit('operation', operation);
  }

  /**
   * Update presence
   */
  updatePresence(presence: Partial<UserPresence>): void {
    if (!this.currentRoom) {
      return;
    }

    this.presenceTracker.updatePresence(this.clientId, {
      ...presence,
      userId: this.userId,
      clientId: this.clientId,
    });

    if (this.connected) {
      this.transport.send({
        type: MessageType.PRESENCE_UPDATE,
        id: uuidv4(),
        clientId: this.clientId,
        roomId: this.currentRoom,
        timestamp: Date.now(),
        data: presence,
      }).catch((error) => {
        console.error('Failed to send presence update:', error);
      });
    }
  }

  /**
   * Update cursor position
   */
  updateCursor(x: number, y: number): void {
    this.updatePresence({
      cursor: { x, y, timestamp: Date.now() },
    });
  }

  /**
   * Update selection
   */
  updateSelection(objectIds: string[]): void {
    this.updatePresence({
      selection: { objectIds, timestamp: Date.now() },
    });
  }

  /**
   * Request sync from server
   */
  private async requestSync(): Promise<void> {
    if (!this.currentRoom || !this.connected) {
      return;
    }

    await this.transport.send({
      type: MessageType.SYNC_REQUEST,
      id: uuidv4(),
      clientId: this.clientId,
      roomId: this.currentRoom,
      timestamp: Date.now(),
      data: {
        clientId: this.clientId,
        vectorClock: this.vectorClock,
      },
    });
  }

  /**
   * Handle incoming message
   */
  private handleMessage(message: Message): void {
    switch (message.type) {
      case MessageType.CONNECT:
        this.handleConnect(message);
        break;

      case MessageType.ROOM_STATE:
        this.handleRoomState(message);
        break;

      case MessageType.OPERATION:
        this.handleRemoteOperation(message);
        break;

      case MessageType.OPERATION_ACK:
        this.handleOperationAck(message);
        break;

      case MessageType.SYNC_RESPONSE:
        this.handleSyncResponse(message);
        break;

      case MessageType.PRESENCE_UPDATE:
        this.handlePresenceUpdate(message);
        break;

      case MessageType.AWARENESS_UPDATE:
        this.handleAwarenessUpdate(message);
        break;

      case MessageType.PONG:
        // Heartbeat response
        break;

      case MessageType.ERROR:
        this.handleError(message);
        break;

      default:
        console.warn(`Unknown message type: ${message.type}`);
    }
  }

  /**
   * Handle connection message
   */
  private handleConnect(message: Message): void {
    const { clientId } = message.data as any;
    if (clientId) {
      this.clientId = clientId;
    }
  }

  /**
   * Handle room state
   */
  private handleRoomState(message: Message): void {
    const { state, vectorClock, clients, presences } = message.data as any;

    if (this.roomState) {
      // Initialize state
      const sceneState = {
        objects: new Map(Object.entries(state.objects || {})),
        annotations: new Map(Object.entries(state.annotations || {})),
        measurements: new Map(Object.entries(state.measurements || {})),
        properties: state.properties || {},
      };

      // Would need to properly initialize from state
      this.vectorClock = vectorClock;

      this.emit('room_state', { state: sceneState, vectorClock });
    }

    // Send pending operations
    this.sendPendingOperations();
  }

  /**
   * Handle remote operation
   */
  private handleRemoteOperation(message: Message): void {
    const operation = message.data as Operation;

    if (this.roomState) {
      // Apply remote operation
      this.roomState.applyOperation(operation);
      this.vectorClock = VectorClockManager.merge(this.vectorClock, operation.vectorClock);

      this.emit('remote_operation', operation);
    }
  }

  /**
   * Handle operation acknowledgment
   */
  private handleOperationAck(message: Message): void {
    const { operationId } = message.data as any;
    this.emit('operation_ack', { operationId });
  }

  /**
   * Handle sync response
   */
  private handleSyncResponse(message: Message): void {
    const { operations, vectorClock, snapshot } = message.data as any;

    if (this.roomState) {
      // Apply missing operations
      for (const operation of operations) {
        this.roomState.applyOperation(operation);
      }

      this.vectorClock = VectorClockManager.merge(this.vectorClock, vectorClock);

      this.emit('synced', { operations, vectorClock, snapshot });
    }
  }

  /**
   * Handle presence update
   */
  private handlePresenceUpdate(message: Message): void {
    const presence = message.data as Partial<UserPresence>;

    if (presence.clientId) {
      this.presenceTracker.updatePresence(presence.clientId, presence);

      if (presence.cursor) {
        this.cursorManager.updateCursor(presence.clientId, presence.cursor);
      }

      if (presence.selection) {
        this.selectionManager.updateSelection(presence.clientId, presence.selection.objectIds);
      }
    }

    this.emit('presence_update', presence);
  }

  /**
   * Handle awareness update
   */
  private handleAwarenessUpdate(message: Message): void {
    const update = message.data as any;

    if (update.clientId) {
      this.awarenessManager.setState(update.clientId, update);
    }

    this.emit('awareness_update', update);
  }

  /**
   * Handle error
   */
  private handleError(message: Message): void {
    const error = message.data as any;
    this.emit('error', error);
  }

  /**
   * Send pending operations
   */
  private async sendPendingOperations(): Promise<void> {
    if (this.pendingOperations.length === 0 || !this.connected) {
      return;
    }

    const operations = [...this.pendingOperations];
    this.pendingOperations = [];

    try {
      await this.transport.send({
        type: MessageType.OPERATION_BATCH,
        id: uuidv4(),
        clientId: this.clientId,
        roomId: this.currentRoom!,
        timestamp: Date.now(),
        data: operations,
      });
    } catch (error) {
      console.error('Failed to send pending operations:', error);
      this.pendingOperations.push(...operations);
    }
  }

  /**
   * Get current scene state
   */
  getSceneState(): SceneState | null {
    return this.roomState ? this.roomState.getSceneState() : null;
  }

  /**
   * Get all presences
   */
  getPresences(): UserPresence[] {
    return this.presenceTracker.getAllPresences();
  }

  /**
   * Undo last operation
   */
  undo(): Operation | null {
    const operation = this.operationHistory.undo(this.clientId);
    if (operation && this.connected) {
      this.applyOperation(operation);
    }
    return operation;
  }

  /**
   * Redo last undone operation
   */
  redo(): Operation | null {
    const operation = this.operationHistory.redo(this.clientId);
    if (operation && this.connected) {
      this.applyOperation(operation);
    }
    return operation;
  }

  /**
   * Register event handler
   */
  on(event: string, handler: EventHandler): UnsubscribeFn {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event)!.add(handler);

    return () => this.off(event, handler);
  }

  /**
   * Unregister event handler
   */
  off(event: string, handler: EventHandler): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      handlers.delete(handler);
    }
  }

  /**
   * Emit event
   */
  emit(event: string, data: unknown): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(data);
        } catch (error) {
          console.error(`Error in event handler for ${event}:`, error);
        }
      }
    }
  }

  /**
   * Register one-time event handler
   */
  once(event: string, handler: EventHandler): UnsubscribeFn {
    const wrapper = (data: unknown) => {
      handler(data);
      this.off(event, wrapper);
    };

    return this.on(event, wrapper);
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Get client ID
   */
  getClientId(): ClientId {
    return this.clientId;
  }

  /**
   * Get user ID
   */
  getUserId(): UserId {
    return this.userId;
  }

  /**
   * Get current room ID
   */
  getRoomId(): RoomId | null {
    return this.currentRoom;
  }
}
