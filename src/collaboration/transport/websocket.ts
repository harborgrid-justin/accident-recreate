/**
 * AccuScene Enterprise - WebSocket Transport
 * v0.2.0
 *
 * WebSocket transport adapter for real-time communication
 */

import WebSocket from 'ws';
import {
  TransportAdapter,
  ConnectionConfig,
  Message,
  TransportError,
  ClientId,
} from '../types';

/**
 * WebSocket Transport Adapter
 */
export class WebSocketTransport implements TransportAdapter {
  private ws: WebSocket | null = null;
  private config: ConnectionConfig | null = null;
  private eventHandlers: Map<string, Set<(data: unknown) => void>>;
  private reconnectAttempts: number = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  private clientId: ClientId;

  constructor(clientId: ClientId) {
    this.clientId = clientId;
    this.eventHandlers = new Map();
  }

  /**
   * Connect to WebSocket server
   */
  async connect(config: ConnectionConfig): Promise<void> {
    this.config = config;

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(config.url);

        this.ws.on('open', () => {
          this.reconnectAttempts = 0;
          this.startHeartbeat();
          this.emit('connected', { clientId: this.clientId });
          resolve();
        });

        this.ws.on('message', (data: WebSocket.RawData) => {
          try {
            const message = JSON.parse(data.toString()) as Message;
            this.emit('message', message);
            this.emit(message.type, message);
          } catch (error) {
            console.error('Failed to parse message:', error);
          }
        });

        this.ws.on('close', () => {
          this.stopHeartbeat();
          this.emit('disconnected', { clientId: this.clientId });

          if (config.reconnect) {
            this.attemptReconnect();
          }
        });

        this.ws.on('error', (error) => {
          this.emit('error', new TransportError('WebSocket error', error));
          reject(new TransportError('Failed to connect', error));
        });
      } catch (error) {
        reject(new TransportError('Failed to create WebSocket', error));
      }
    });
  }

  /**
   * Disconnect from WebSocket server
   */
  async disconnect(): Promise<void> {
    this.stopHeartbeat();
    this.stopReconnect();

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Send a message
   */
  async send(message: Message): Promise<void> {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new TransportError('WebSocket not connected');
    }

    return new Promise((resolve, reject) => {
      this.ws!.send(JSON.stringify(message), (error) => {
        if (error) {
          reject(new TransportError('Failed to send message', error));
        } else {
          resolve();
        }
      });
    });
  }

  /**
   * Register event handler
   */
  on(event: string, handler: (data: unknown) => void): void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event)!.add(handler);
  }

  /**
   * Unregister event handler
   */
  off(event: string, handler: (data: unknown) => void): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      handlers.delete(handler);
    }
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }

  /**
   * Emit an event
   */
  private emit(event: string, data: unknown): void {
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
   * Attempt to reconnect
   */
  private attemptReconnect(): void {
    if (!this.config || !this.config.reconnect) {
      return;
    }

    const maxAttempts = this.config.maxReconnectAttempts || 5;
    if (this.reconnectAttempts >= maxAttempts) {
      this.emit('reconnect_failed', { attempts: this.reconnectAttempts });
      return;
    }

    this.reconnectAttempts++;
    const delay = this.config.reconnectDelay || 1000;

    this.emit('reconnecting', { attempt: this.reconnectAttempts });

    this.reconnectTimer = setTimeout(() => {
      this.connect(this.config!)
        .then(() => {
          this.emit('reconnected', { attempts: this.reconnectAttempts });
        })
        .catch((error) => {
          console.error('Reconnect failed:', error);
        });
    }, delay * this.reconnectAttempts);
  }

  /**
   * Stop reconnection attempts
   */
  private stopReconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  /**
   * Start heartbeat
   */
  private startHeartbeat(): void {
    if (!this.config) {
      return;
    }

    const interval = this.config.heartbeatInterval || 30000;

    this.heartbeatTimer = setInterval(() => {
      if (this.isConnected()) {
        this.send({
          type: 'ping' as any,
          id: `ping-${Date.now()}`,
          clientId: this.clientId,
          timestamp: Date.now(),
          data: {},
        }).catch((error) => {
          console.error('Heartbeat failed:', error);
        });
      }
    }, interval);
  }

  /**
   * Stop heartbeat
   */
  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  /**
   * Get connection state
   */
  getState(): string {
    if (!this.ws) {
      return 'CLOSED';
    }

    switch (this.ws.readyState) {
      case WebSocket.CONNECTING:
        return 'CONNECTING';
      case WebSocket.OPEN:
        return 'OPEN';
      case WebSocket.CLOSING:
        return 'CLOSING';
      case WebSocket.CLOSED:
        return 'CLOSED';
      default:
        return 'UNKNOWN';
    }
  }
}
