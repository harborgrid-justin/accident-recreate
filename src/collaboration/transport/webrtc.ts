/**
 * AccuScene Enterprise - WebRTC Transport
 * v0.2.0
 *
 * WebRTC transport adapter for peer-to-peer communication
 */

import {
  TransportAdapter,
  ConnectionConfig,
  Message,
  TransportError,
  ClientId,
} from '../types';

/**
 * WebRTC Transport Adapter (Stub for future implementation)
 * Provides peer-to-peer data channel communication
 */
export class WebRTCTransport implements TransportAdapter {
  private peerConnection: RTCPeerConnection | null = null;
  private dataChannel: RTCDataChannel | null = null;
  private config: ConnectionConfig | null = null;
  private eventHandlers: Map<string, Set<(data: unknown) => void>>;
  private clientId: ClientId;
  private connected: boolean = false;

  constructor(clientId: ClientId) {
    this.clientId = clientId;
    this.eventHandlers = new Map();
  }

  /**
   * Connect via WebRTC
   */
  async connect(config: ConnectionConfig): Promise<void> {
    this.config = config;

    return new Promise((resolve, reject) => {
      try {
        // Create peer connection
        this.peerConnection = new RTCPeerConnection({
          iceServers: [
            { urls: 'stun:stun.l.google.com:19302' },
            { urls: 'stun:stun1.l.google.com:19302' },
          ],
        });

        // Create data channel
        this.dataChannel = this.peerConnection.createDataChannel('collaboration', {
          ordered: true,
          maxRetransmits: 3,
        });

        this.setupDataChannel();

        // Handle ICE candidates
        this.peerConnection.onicecandidate = (event) => {
          if (event.candidate) {
            this.emit('ice_candidate', event.candidate);
          }
        };

        // Handle connection state changes
        this.peerConnection.onconnectionstatechange = () => {
          const state = this.peerConnection!.connectionState;

          if (state === 'connected') {
            this.connected = true;
            this.emit('connected', { clientId: this.clientId });
            resolve();
          } else if (state === 'disconnected' || state === 'failed') {
            this.connected = false;
            this.emit('disconnected', { clientId: this.clientId });
          }
        };

        // Handle incoming data channels
        this.peerConnection.ondatachannel = (event) => {
          this.dataChannel = event.channel;
          this.setupDataChannel();
        };
      } catch (error) {
        reject(new TransportError('Failed to create WebRTC connection', error));
      }
    });
  }

  /**
   * Setup data channel event handlers
   */
  private setupDataChannel(): void {
    if (!this.dataChannel) {
      return;
    }

    this.dataChannel.onopen = () => {
      this.connected = true;
      this.emit('channel_open', { clientId: this.clientId });
    };

    this.dataChannel.onclose = () => {
      this.connected = false;
      this.emit('channel_close', { clientId: this.clientId });
    };

    this.dataChannel.onerror = (error) => {
      this.emit('error', new TransportError('Data channel error', error));
    };

    this.dataChannel.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as Message;
        this.emit('message', message);
        this.emit(message.type, message);
      } catch (error) {
        console.error('Failed to parse message:', error);
      }
    };
  }

  /**
   * Disconnect from WebRTC
   */
  async disconnect(): Promise<void> {
    if (this.dataChannel) {
      this.dataChannel.close();
      this.dataChannel = null;
    }

    if (this.peerConnection) {
      this.peerConnection.close();
      this.peerConnection = null;
    }

    this.connected = false;
  }

  /**
   * Send a message
   */
  async send(message: Message): Promise<void> {
    if (!this.dataChannel || this.dataChannel.readyState !== 'open') {
      throw new TransportError('Data channel not open');
    }

    try {
      this.dataChannel.send(JSON.stringify(message));
    } catch (error) {
      throw new TransportError('Failed to send message', error);
    }
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
    return this.connected &&
           this.dataChannel !== null &&
           this.dataChannel.readyState === 'open';
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
   * Create offer for connection
   */
  async createOffer(): Promise<RTCSessionDescriptionInit> {
    if (!this.peerConnection) {
      throw new TransportError('Peer connection not initialized');
    }

    const offer = await this.peerConnection.createOffer();
    await this.peerConnection.setLocalDescription(offer);
    return offer;
  }

  /**
   * Create answer for connection
   */
  async createAnswer(): Promise<RTCSessionDescriptionInit> {
    if (!this.peerConnection) {
      throw new TransportError('Peer connection not initialized');
    }

    const answer = await this.peerConnection.createAnswer();
    await this.peerConnection.setLocalDescription(answer);
    return answer;
  }

  /**
   * Set remote description
   */
  async setRemoteDescription(description: RTCSessionDescriptionInit): Promise<void> {
    if (!this.peerConnection) {
      throw new TransportError('Peer connection not initialized');
    }

    await this.peerConnection.setRemoteDescription(description);
  }

  /**
   * Add ICE candidate
   */
  async addIceCandidate(candidate: RTCIceCandidateInit): Promise<void> {
    if (!this.peerConnection) {
      throw new TransportError('Peer connection not initialized');
    }

    await this.peerConnection.addIceCandidate(candidate);
  }

  /**
   * Get connection state
   */
  getState(): string {
    if (!this.peerConnection) {
      return 'CLOSED';
    }
    return this.peerConnection.connectionState.toUpperCase();
  }
}
