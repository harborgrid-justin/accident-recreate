/**
 * AccuScene Enterprise v0.3.0 - Sync Protocol
 *
 * Custom synchronization protocol with compression and ordering
 */

import { EventEmitter } from 'events';
import { SyncMessage, SyncMessageType, Operation } from './types';

export class SyncProtocol extends EventEmitter {
  private sequenceNumber = 0;
  private pendingAcks: Map<number, { message: SyncMessage; timestamp: number }> = new Map();
  private receivedSequences: Set<number> = new Set();

  async sendMessage(message: Omit<SyncMessage, 'sequence'>): Promise<void> {
    const sequenced: SyncMessage = {
      ...message,
      sequence: this.sequenceNumber++
    };

    // Compress if needed
    if (this.shouldCompress(sequenced)) {
      sequenced.payload = await this.compress(sequenced.payload);
      sequenced.compressed = true;
    }

    // Track for acknowledgement
    this.pendingAcks.set(sequenced.sequence, {
      message: sequenced,
      timestamp: Date.now()
    });

    this.emit('send', sequenced);
  }

  async receiveMessage(message: SyncMessage): Promise<void> {
    // Decompress if needed
    if (message.compressed) {
      message.payload = await this.decompress(message.payload);
    }

    // Check for duplicates
    if (this.receivedSequences.has(message.sequence)) {
      return; // Duplicate, ignore
    }

    this.receivedSequences.add(message.sequence);

    // Send acknowledgement
    this.emit('sendAck', message.sequence);

    // Process message
    this.emit('message', message);
  }

  acknowledgeMessage(sequence: number): void {
    this.pendingAcks.delete(sequence);
  }

  private shouldCompress(message: SyncMessage): boolean {
    const size = JSON.stringify(message.payload).length;
    return size > 1024; // Compress if > 1KB
  }

  private async compress(data: any): Promise<string> {
    // Simple JSON stringification - in production use actual compression
    return JSON.stringify(data);
  }

  private async decompress(data: string): Promise<any> {
    return JSON.parse(data);
  }
}
