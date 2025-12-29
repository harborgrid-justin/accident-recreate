/**
 * AccuScene Enterprise v0.3.0 - Connection Manager
 *
 * WebSocket connection management with auto-reconnect and fallback
 */

import { EventEmitter } from 'events';
import {
  SessionId,
  User,
  Operation,
  SyncMessage,
  SyncMessageType,
  ConnectionStatus
} from './types';

export class ConnectionManager extends EventEmitter {
  private ws: WebSocket | null = null;
  private status: ConnectionStatus = ConnectionStatus.DISCONNECTED;
  private reconnectAttempts = 0;
  private reconnectTimeout: NodeJS.Timeout | null = null;

  private sessionId: SessionId | null = null;
  private user: User | null = null;

  // Ping/pong for heartbeat
  private pingInterval: NodeJS.Timeout | null = null;
  private lastPing = 0;
  private latencyHistory: number[] = [];

  private config: any;

  constructor(config: any) {
    super();
    this.config = config;
  }

  async connect(sessionId: SessionId, user: User): Promise<void> {
    this.sessionId = sessionId;
    this.user = user;

    return new Promise((resolve, reject) => {
      this.setStatus(ConnectionStatus.CONNECTING);

      this.ws = new WebSocket(this.config.wsUrl);

      this.ws.onopen = () => {
        this.setStatus(ConnectionStatus.CONNECTED);
        this.reconnectAttempts = 0;

        // Send join message
        this.send({
          type: SyncMessageType.JOIN,
          sessionId,
          senderId: user.id,
          timestamp: Date.now(),
          sequence: 0,
          payload: { user }
        });

        this.startHeartbeat();
        resolve();
      };

      this.ws.onerror = (error) => {
        this.emit('error', error);
        reject(error);
      };

      this.ws.onclose = () => {
        this.handleDisconnect();
      };

      this.ws.onmessage = (event) => {
        this.handleMessage(event.data);
      };
    });
  }

  async disconnect(): Promise<void> {
    this.stopHeartbeat();

    if (this.ws) {
      // Send leave message
      if (this.sessionId && this.user) {
        this.send({
          type: SyncMessageType.LEAVE,
          sessionId: this.sessionId,
          senderId: this.user.id,
          timestamp: Date.now(),
          sequence: 0,
          payload: { user: this.user }
        });
      }

      this.ws.close();
      this.ws = null;
    }

    this.setStatus(ConnectionStatus.DISCONNECTED);
  }

  async sendOperation(operation: Operation): Promise<void> {
    if (!this.sessionId || !this.user) {
      throw new Error('Not connected');
    }

    this.send({
      type: SyncMessageType.OPERATION,
      sessionId: this.sessionId,
      senderId: this.user.id,
      timestamp: Date.now(),
      sequence: 0,
      payload: operation
    });
  }

  private send(message: SyncMessage): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket not connected');
    }

    this.ws.send(JSON.stringify(message));
  }

  private handleMessage(data: string): void {
    try {
      const message: SyncMessage = JSON.parse(data);

      switch (message.type) {
        case SyncMessageType.OPERATION:
          this.emit('operation', message.payload);
          break;
        case SyncMessageType.PRESENCE_UPDATE:
          this.emit('presenceUpdate', message.payload);
          break;
        case SyncMessageType.PONG:
          this.handlePong(message.timestamp);
          break;
        default:
          this.emit('message', message);
      }
    } catch (error) {
      this.emit('error', error);
    }
  }

  private handleDisconnect(): void {
    this.stopHeartbeat();
    this.setStatus(ConnectionStatus.DISCONNECTED);

    if (this.reconnectAttempts < this.config.maxReconnectAttempts) {
      this.reconnect();
    } else {
      this.setStatus(ConnectionStatus.FAILED);
    }
  }

  private reconnect(): void {
    this.setStatus(ConnectionStatus.RECONNECTING);
    this.reconnectAttempts++;

    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);

    this.reconnectTimeout = setTimeout(() => {
      if (this.sessionId && this.user) {
        this.connect(this.sessionId, this.user).catch(() => {
          // Will retry in handleDisconnect
        });
      }
    }, delay);
  }

  private startHeartbeat(): void {
    this.pingInterval = setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN && this.sessionId && this.user) {
        this.lastPing = Date.now();
        this.send({
          type: SyncMessageType.PING,
          sessionId: this.sessionId,
          senderId: this.user.id,
          timestamp: this.lastPing,
          sequence: 0,
          payload: {}
        });
      }
    }, this.config.pingInterval || 30000);
  }

  private stopHeartbeat(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }

  private handlePong(timestamp: number): void {
    const latency = Date.now() - this.lastPing;
    this.latencyHistory.push(latency);

    if (this.latencyHistory.length > 10) {
      this.latencyHistory.shift();
    }
  }

  private setStatus(status: ConnectionStatus): void {
    this.status = status;
    this.emit('statusChange', status);
  }

  getStatus(): ConnectionStatus {
    return this.status;
  }

  getAverageLatency(): number {
    if (this.latencyHistory.length === 0) return 0;
    return this.latencyHistory.reduce((a, b) => a + b) / this.latencyHistory.length;
  }

  getPeakLatency(): number {
    if (this.latencyHistory.length === 0) return 0;
    return Math.max(...this.latencyHistory);
  }
}
