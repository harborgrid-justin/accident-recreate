/**
 * AccuScene Enterprise - Collaboration Server
 * v0.2.0
 *
 * WebSocket server for real-time collaboration
 */

import WebSocket, { WebSocketServer } from 'ws';
import { IncomingMessage } from 'http';
import { v4 as uuidv4 } from 'uuid';
import {
  Message,
  MessageType,
  Operation,
  RoomId,
  ClientId,
  UserId,
  RoomPermission,
  SyncRequest,
  UserPresence,
  AwarenessUpdate,
} from './types';
import { RoomManager } from './room/manager';
import { PermissionManager } from './room/permissions';
import { RoomStateManager } from './room/state';
import { PresenceTracker } from './presence/tracker';
import { CursorManager } from './presence/cursor';
import { SelectionManager } from './presence/selection';
import { AwarenessStateManager } from './awareness/state';
import { SnapshotManager } from './persistence/snapshot';
import { OperationJournal } from './persistence/journal';
import { DifferentialSync } from './sync/diff';
import { ConflictResolutionManager } from './sync/conflict';
import { OperationHistory } from './operations/history';

interface ClientConnection {
  clientId: ClientId;
  userId: UserId;
  socket: WebSocket;
  roomId?: RoomId;
  lastActivity: number;
}

/**
 * Collaboration Server
 */
export class CollaborationServer {
  private wss: WebSocketServer;
  private clients: Map<ClientId, ClientConnection>;
  private roomManager: RoomManager;
  private permissionManager: PermissionManager;
  private roomStates: Map<RoomId, RoomStateManager>;
  private presenceTracker: PresenceTracker;
  private cursorManager: CursorManager;
  private selectionManager: SelectionManager;
  private awarenessManager: AwarenessStateManager;
  private snapshotManager: SnapshotManager;
  private operationJournal: OperationJournal;
  private conflictResolver: ConflictResolutionManager;
  private operationHistory: OperationHistory;

  constructor(port: number = 8080) {
    this.wss = new WebSocketServer({ port });
    this.clients = new Map();
    this.roomManager = new RoomManager();
    this.permissionManager = new PermissionManager();
    this.roomStates = new Map();
    this.presenceTracker = new PresenceTracker();
    this.cursorManager = new CursorManager();
    this.selectionManager = new SelectionManager();
    this.awarenessManager = new AwarenessStateManager();
    this.snapshotManager = new SnapshotManager();
    this.operationJournal = new OperationJournal();
    this.conflictResolver = new ConflictResolutionManager();
    this.operationHistory = new OperationHistory();

    this.setupServer();
    this.startCleanupTasks();
  }

  /**
   * Setup WebSocket server
   */
  private setupServer(): void {
    this.wss.on('connection', (socket: WebSocket, request: IncomingMessage) => {
      const clientId = uuidv4();
      const userId = this.extractUserId(request);

      const connection: ClientConnection = {
        clientId,
        userId,
        socket,
        lastActivity: Date.now(),
      };

      this.clients.set(clientId, connection);

      console.log(`Client connected: ${clientId} (User: ${userId})`);

      // Send connection confirmation
      this.sendMessage(socket, {
        type: MessageType.CONNECT,
        id: uuidv4(),
        clientId: 'server',
        timestamp: Date.now(),
        data: { clientId, userId },
      });

      // Setup message handlers
      socket.on('message', (data: WebSocket.RawData) => {
        this.handleMessage(clientId, data);
      });

      socket.on('close', () => {
        this.handleDisconnect(clientId);
      });

      socket.on('error', (error) => {
        console.error(`WebSocket error for client ${clientId}:`, error);
      });
    });

    console.log(`Collaboration server started on port ${this.wss.options.port}`);
  }

  /**
   * Handle incoming message
   */
  private handleMessage(clientId: ClientId, data: WebSocket.RawData): void {
    try {
      const message = JSON.parse(data.toString()) as Message;
      const connection = this.clients.get(clientId);

      if (!connection) {
        return;
      }

      connection.lastActivity = Date.now();

      switch (message.type) {
        case MessageType.JOIN_ROOM:
          this.handleJoinRoom(clientId, message);
          break;

        case MessageType.LEAVE_ROOM:
          this.handleLeaveRoom(clientId, message);
          break;

        case MessageType.OPERATION:
          this.handleOperation(clientId, message);
          break;

        case MessageType.OPERATION_BATCH:
          this.handleOperationBatch(clientId, message);
          break;

        case MessageType.SYNC_REQUEST:
          this.handleSyncRequest(clientId, message);
          break;

        case MessageType.PRESENCE_UPDATE:
          this.handlePresenceUpdate(clientId, message);
          break;

        case MessageType.AWARENESS_UPDATE:
          this.handleAwarenessUpdate(clientId, message);
          break;

        case MessageType.PING:
          this.handlePing(clientId, message);
          break;

        default:
          console.warn(`Unknown message type: ${message.type}`);
      }
    } catch (error) {
      console.error(`Error handling message from ${clientId}:`, error);
    }
  }

  /**
   * Handle join room request
   */
  private handleJoinRoom(clientId: ClientId, message: Message): void {
    const { roomId, sceneId, userId } = message.data as any;
    const connection = this.clients.get(clientId);

    if (!connection) {
      return;
    }

    try {
      // Create room if it doesn't exist
      if (!this.roomManager.hasRoom(roomId)) {
        this.roomManager.createRoom(roomId, sceneId);
        this.permissionManager.initializeRoom(roomId, userId);
        this.roomStates.set(roomId, new RoomStateManager(roomId, clientId));
      }

      // Check permissions
      if (!this.permissionManager.canRead(roomId, userId)) {
        this.sendError(connection.socket, 'PERMISSION_DENIED', 'No permission to join room');
        return;
      }

      // Add client to room
      this.roomManager.addClient(roomId, {
        clientId,
        userId,
        sessionId: uuidv4(),
        permissions: this.permissionManager.getPermissions(roomId, userId),
        joinedAt: Date.now(),
        lastActivity: Date.now(),
      });

      connection.roomId = roomId;

      // Update presence
      this.presenceTracker.updatePresence(clientId, {
        userId,
        clientId,
        sessionId: clientId,
        online: true,
        lastSeen: Date.now(),
      });

      // Get room state
      const roomState = this.roomStates.get(roomId)!;

      // Send room state to client
      this.sendMessage(connection.socket, {
        type: MessageType.ROOM_STATE,
        id: uuidv4(),
        clientId: 'server',
        roomId,
        timestamp: Date.now(),
        data: {
          state: roomState.getSceneState(),
          vectorClock: roomState.getVectorClock(),
          clients: this.roomManager.getClients(roomId),
          presences: this.presenceTracker.getOnlineUsers(),
        },
      });

      // Broadcast join to other clients
      this.broadcastToRoom(roomId, {
        type: MessageType.PRESENCE_UPDATE,
        id: uuidv4(),
        clientId: 'server',
        roomId,
        timestamp: Date.now(),
        data: {
          clientId,
          userId,
          online: true,
        },
      }, clientId);

      console.log(`Client ${clientId} joined room ${roomId}`);
    } catch (error: any) {
      this.sendError(connection.socket, 'JOIN_FAILED', error.message);
    }
  }

  /**
   * Handle leave room request
   */
  private handleLeaveRoom(clientId: ClientId, message: Message): void {
    const connection = this.clients.get(clientId);
    if (!connection || !connection.roomId) {
      return;
    }

    const roomId = connection.roomId;

    // Remove from room
    this.roomManager.removeClient(roomId, clientId);
    this.presenceTracker.setOffline(clientId);

    // Broadcast leave to other clients
    this.broadcastToRoom(roomId, {
      type: MessageType.PRESENCE_UPDATE,
      id: uuidv4(),
      clientId: 'server',
      roomId,
      timestamp: Date.now(),
      data: {
        clientId,
        online: false,
      },
    });

    connection.roomId = undefined;

    console.log(`Client ${clientId} left room ${roomId}`);
  }

  /**
   * Handle operation
   */
  private handleOperation(clientId: ClientId, message: Message): void {
    const connection = this.clients.get(clientId);
    if (!connection || !connection.roomId) {
      return;
    }

    const operation = message.data as Operation;
    const roomId = connection.roomId;
    const roomState = this.roomStates.get(roomId);

    if (!roomState) {
      return;
    }

    // Check write permission
    if (!this.permissionManager.canWrite(roomId, connection.userId)) {
      this.sendError(connection.socket, 'PERMISSION_DENIED', 'No write permission');
      return;
    }

    try {
      // Apply operation to room state
      roomState.applyOperation(operation);

      // Add to journal
      this.operationJournal.append(roomId, operation, true);

      // Add to history
      this.operationHistory.add(operation);

      // Broadcast to other clients
      this.broadcastToRoom(roomId, {
        type: MessageType.OPERATION,
        id: uuidv4(),
        clientId: 'server',
        roomId,
        timestamp: Date.now(),
        data: operation,
      }, clientId);

      // Send acknowledgment
      this.sendMessage(connection.socket, {
        type: MessageType.OPERATION_ACK,
        id: uuidv4(),
        clientId: 'server',
        roomId,
        timestamp: Date.now(),
        data: { operationId: operation.id },
      });

      // Check if snapshot needed
      if (this.snapshotManager.shouldCreateSnapshot(roomId)) {
        this.snapshotManager.createSnapshot(
          roomId,
          roomState.getSceneState(),
          roomState.getVectorClock()
        );
      }
    } catch (error: any) {
      this.sendError(connection.socket, 'OPERATION_FAILED', error.message);
    }
  }

  /**
   * Handle operation batch
   */
  private handleOperationBatch(clientId: ClientId, message: Message): void {
    const operations = message.data as Operation[];

    for (const operation of operations) {
      this.handleOperation(clientId, {
        ...message,
        type: MessageType.OPERATION,
        data: operation,
      });
    }
  }

  /**
   * Handle sync request
   */
  private handleSyncRequest(clientId: ClientId, message: Message): void {
    const connection = this.clients.get(clientId);
    if (!connection || !connection.roomId) {
      return;
    }

    const request = message.data as SyncRequest;
    const roomId = connection.roomId;
    const roomState = this.roomStates.get(roomId);

    if (!roomState) {
      return;
    }

    // Get operations since client's vector clock
    const operations = roomState.getOperationsSince(request.vectorClock);
    const snapshot = this.snapshotManager.getLatestSnapshot(roomId);

    // Send sync response
    this.sendMessage(connection.socket, {
      type: MessageType.SYNC_RESPONSE,
      id: uuidv4(),
      clientId: 'server',
      roomId,
      timestamp: Date.now(),
      data: {
        operations,
        vectorClock: roomState.getVectorClock(),
        snapshot,
      },
    });
  }

  /**
   * Handle presence update
   */
  private handlePresenceUpdate(clientId: ClientId, message: Message): void {
    const connection = this.clients.get(clientId);
    if (!connection || !connection.roomId) {
      return;
    }

    const presence = message.data as Partial<UserPresence>;
    this.presenceTracker.updatePresence(clientId, presence);

    // Update cursor if provided
    if (presence.cursor) {
      this.cursorManager.updateCursor(clientId, presence.cursor);
    }

    // Update selection if provided
    if (presence.selection) {
      this.selectionManager.updateSelection(clientId, presence.selection.objectIds);
    }

    // Broadcast presence update
    this.broadcastToRoom(connection.roomId, {
      type: MessageType.PRESENCE_UPDATE,
      id: uuidv4(),
      clientId: 'server',
      roomId: connection.roomId,
      timestamp: Date.now(),
      data: presence,
    }, clientId);
  }

  /**
   * Handle awareness update
   */
  private handleAwarenessUpdate(clientId: ClientId, message: Message): void {
    const connection = this.clients.get(clientId);
    if (!connection || !connection.roomId) {
      return;
    }

    const update = message.data as any;
    this.awarenessManager.setState(clientId, update);

    // Broadcast awareness update
    this.broadcastToRoom(connection.roomId, {
      type: MessageType.AWARENESS_UPDATE,
      id: uuidv4(),
      clientId: 'server',
      roomId: connection.roomId,
      timestamp: Date.now(),
      data: update,
    }, clientId);
  }

  /**
   * Handle ping
   */
  private handlePing(clientId: ClientId, message: Message): void {
    const connection = this.clients.get(clientId);
    if (!connection) {
      return;
    }

    this.sendMessage(connection.socket, {
      type: MessageType.PONG,
      id: uuidv4(),
      clientId: 'server',
      timestamp: Date.now(),
      data: {},
    });

    this.presenceTracker.heartbeat(clientId);
  }

  /**
   * Handle client disconnect
   */
  private handleDisconnect(clientId: ClientId): void {
    const connection = this.clients.get(clientId);
    if (!connection) {
      return;
    }

    console.log(`Client disconnected: ${clientId}`);

    // Leave room if in one
    if (connection.roomId) {
      this.handleLeaveRoom(clientId, {
        type: MessageType.LEAVE_ROOM,
        id: uuidv4(),
        clientId,
        timestamp: Date.now(),
        data: {},
      });
    }

    // Clean up
    this.clients.delete(clientId);
    this.presenceTracker.removePresence(clientId);
    this.cursorManager.removeCursor(clientId);
    this.selectionManager.clearSelection(clientId);
    this.awarenessManager.removeState(clientId);
  }

  /**
   * Broadcast message to all clients in a room
   */
  private broadcastToRoom(roomId: RoomId, message: Message, excludeClientId?: ClientId): void {
    for (const [clientId, connection] of this.clients.entries()) {
      if (connection.roomId === roomId && clientId !== excludeClientId) {
        this.sendMessage(connection.socket, message);
      }
    }
  }

  /**
   * Send message to a client
   */
  private sendMessage(socket: WebSocket, message: Message): void {
    if (socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(message));
    }
  }

  /**
   * Send error to a client
   */
  private sendError(socket: WebSocket, code: string, message: string): void {
    this.sendMessage(socket, {
      type: MessageType.ERROR,
      id: uuidv4(),
      clientId: 'server',
      timestamp: Date.now(),
      data: { code, message },
    });
  }

  /**
   * Extract user ID from request
   */
  private extractUserId(request: IncomingMessage): UserId {
    // In production, extract from JWT token or session
    const url = new URL(request.url || '', `http://${request.headers.host}`);
    return url.searchParams.get('userId') || 'anonymous';
  }

  /**
   * Start cleanup tasks
   */
  private startCleanupTasks(): void {
    // Clean up empty rooms every hour
    setInterval(() => {
      const removed = this.roomManager.cleanupEmptyRooms();
      if (removed.length > 0) {
        console.log(`Cleaned up ${removed.length} empty rooms`);
      }
    }, 3600000);

    // Prune old snapshots daily
    setInterval(() => {
      const pruned = this.snapshotManager.pruneSnapshots();
      console.log(`Pruned ${pruned} old snapshots`);
    }, 86400000);

    // Prune old journal entries daily
    setInterval(() => {
      const pruned = this.operationJournal.prune();
      console.log(`Pruned ${pruned} old journal entries`);
    }, 86400000);
  }

  /**
   * Close the server
   */
  close(): void {
    this.wss.close();
  }
}
