/**
 * AccuScene Enterprise - Real-time Collaboration System
 * v0.2.0
 *
 * Complete collaboration system with CRDTs, OT, and distributed sync
 */

// Main exports
export { CollaborationServer } from './server';
export { CollaborationClient } from './client';
export type { CollaborationClientConfig } from './client';

// Types
export * from './types';

// CRDTs
export * from './crdt';

// Synchronization
export * from './sync';

// Presence & Awareness
export * from './presence';
export * from './awareness';

// Operations
export * from './operations';

// Room Management
export * from './room';

// Persistence
export * from './persistence';

// Transport
export * from './transport';
