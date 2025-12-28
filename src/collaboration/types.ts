/**
 * AccuScene Enterprise - Real-time Collaboration Types
 * v0.2.0
 *
 * Core type definitions for the distributed collaboration system
 */

// ============================================================================
// Core Identifiers
// ============================================================================

export type ClientId = string;
export type RoomId = string;
export type UserId = string;
export type SessionId = string;
export type OperationId = string;
export type ObjectId = string;

// ============================================================================
// Vector Clock
// ============================================================================

export interface VectorClock {
  [clientId: string]: number;
}

export interface Timestamp {
  clientId: ClientId;
  counter: number;
}

// ============================================================================
// CRDT Base Types
// ============================================================================

export interface CRDTOperation {
  id: OperationId;
  type: string;
  clientId: ClientId;
  timestamp: Timestamp;
  vectorClock: VectorClock;
  data: unknown;
}

export interface CRDTState {
  vectorClock: VectorClock;
  data: unknown;
}

export interface CRDT<T, O extends CRDTOperation = CRDTOperation> {
  state: CRDTState;
  apply(operation: O): void;
  merge(other: CRDT<T, O>): void;
  getValue(): T;
  clone(): CRDT<T, O>;
}

// ============================================================================
// Operation Types
// ============================================================================

export enum OperationType {
  // Scene operations
  SCENE_CREATE = 'scene:create',
  SCENE_UPDATE = 'scene:update',
  SCENE_DELETE = 'scene:delete',

  // Object operations
  OBJECT_CREATE = 'object:create',
  OBJECT_UPDATE = 'object:update',
  OBJECT_DELETE = 'object:delete',
  OBJECT_MOVE = 'object:move',
  OBJECT_TRANSFORM = 'object:transform',

  // Annotation operations
  ANNOTATION_CREATE = 'annotation:create',
  ANNOTATION_UPDATE = 'annotation:update',
  ANNOTATION_DELETE = 'annotation:delete',

  // Measurement operations
  MEASUREMENT_CREATE = 'measurement:create',
  MEASUREMENT_UPDATE = 'measurement:update',
  MEASUREMENT_DELETE = 'measurement:delete',

  // Custom operations
  CUSTOM = 'custom',
}

export interface Operation {
  id: OperationId;
  type: OperationType;
  clientId: ClientId;
  timestamp: Timestamp;
  vectorClock: VectorClock;
  data: OperationData;
  transformedAgainst?: OperationId[];
}

export type OperationData =
  | SceneOperationData
  | ObjectOperationData
  | AnnotationOperationData
  | MeasurementOperationData
  | CustomOperationData;

export interface SceneOperationData {
  sceneId: string;
  properties?: Record<string, unknown>;
}

export interface ObjectOperationData {
  objectId: ObjectId;
  objectType: string;
  position?: { x: number; y: number; z?: number };
  rotation?: number;
  scale?: { x: number; y: number };
  properties?: Record<string, unknown>;
}

export interface AnnotationOperationData {
  annotationId: string;
  text?: string;
  position?: { x: number; y: number };
  style?: Record<string, unknown>;
}

export interface MeasurementOperationData {
  measurementId: string;
  points: Array<{ x: number; y: number }>;
  value?: number;
  unit?: string;
}

export interface CustomOperationData {
  [key: string]: unknown;
}

// ============================================================================
// Presence Types
// ============================================================================

export interface UserPresence {
  userId: UserId;
  clientId: ClientId;
  sessionId: SessionId;
  online: boolean;
  lastSeen: number;
  cursor?: CursorPosition;
  selection?: SelectionState;
  viewport?: ViewportState;
  metadata?: UserMetadata;
}

export interface CursorPosition {
  x: number;
  y: number;
  timestamp: number;
}

export interface SelectionState {
  objectIds: ObjectId[];
  timestamp: number;
}

export interface ViewportState {
  x: number;
  y: number;
  zoom: number;
  width: number;
  height: number;
}

export interface UserMetadata {
  name: string;
  color: string;
  avatar?: string;
  role?: string;
}

// ============================================================================
// Awareness Types
// ============================================================================

export interface AwarenessState {
  clientId: ClientId;
  user: UserMetadata;
  presence: UserPresence;
  lastUpdate: number;
}

export interface AwarenessUpdate {
  added: ClientId[];
  updated: ClientId[];
  removed: ClientId[];
}

// ============================================================================
// Room Types
// ============================================================================

export enum RoomPermission {
  READ = 'read',
  WRITE = 'write',
  ADMIN = 'admin',
  OWNER = 'owner',
}

export interface RoomState {
  id: RoomId;
  name: string;
  sceneId: string;
  clients: Map<ClientId, ClientInfo>;
  operations: Operation[];
  vectorClock: VectorClock;
  snapshot?: StateSnapshot;
  createdAt: number;
  updatedAt: number;
}

export interface ClientInfo {
  clientId: ClientId;
  userId: UserId;
  sessionId: SessionId;
  permissions: RoomPermission[];
  joinedAt: number;
  lastActivity: number;
}

export interface RoomConfig {
  maxClients?: number;
  autoSnapshot?: boolean;
  snapshotInterval?: number;
  operationHistoryLimit?: number;
  persistenceEnabled?: boolean;
}

// ============================================================================
// Synchronization Types
// ============================================================================

export interface SyncRequest {
  clientId: ClientId;
  vectorClock: VectorClock;
  merkleRoot?: string;
}

export interface SyncResponse {
  operations: Operation[];
  vectorClock: VectorClock;
  snapshot?: StateSnapshot;
  merkleRoot?: string;
}

export interface MerkleNode {
  hash: string;
  left?: MerkleNode;
  right?: MerkleNode;
  data?: Operation[];
}

export interface DiffResult {
  missing: Operation[];
  conflicting: Operation[];
}

// ============================================================================
// Persistence Types
// ============================================================================

export interface StateSnapshot {
  id: string;
  roomId: RoomId;
  vectorClock: VectorClock;
  state: SceneState;
  timestamp: number;
  checksum: string;
}

export interface SceneState {
  objects: Map<ObjectId, SceneObject>;
  annotations: Map<string, Annotation>;
  measurements: Map<string, Measurement>;
  properties: Record<string, unknown>;
}

export interface SceneObject {
  id: ObjectId;
  type: string;
  position: { x: number; y: number; z?: number };
  rotation: number;
  scale: { x: number; y: number };
  properties: Record<string, unknown>;
  createdAt: number;
  updatedAt: number;
  createdBy: ClientId;
}

export interface Annotation {
  id: string;
  text: string;
  position: { x: number; y: number };
  style: Record<string, unknown>;
  createdAt: number;
  updatedAt: number;
  createdBy: ClientId;
}

export interface Measurement {
  id: string;
  points: Array<{ x: number; y: number }>;
  value: number;
  unit: string;
  createdAt: number;
  updatedAt: number;
  createdBy: ClientId;
}

export interface JournalEntry {
  id: string;
  roomId: RoomId;
  operation: Operation;
  timestamp: number;
  applied: boolean;
}

// ============================================================================
// Transport Types
// ============================================================================

export enum MessageType {
  // Connection
  CONNECT = 'connect',
  DISCONNECT = 'disconnect',
  PING = 'ping',
  PONG = 'pong',

  // Room
  JOIN_ROOM = 'join_room',
  LEAVE_ROOM = 'leave_room',
  ROOM_STATE = 'room_state',

  // Operations
  OPERATION = 'operation',
  OPERATION_ACK = 'operation_ack',
  OPERATION_BATCH = 'operation_batch',

  // Sync
  SYNC_REQUEST = 'sync_request',
  SYNC_RESPONSE = 'sync_response',

  // Presence
  PRESENCE_UPDATE = 'presence_update',
  PRESENCE_STATE = 'presence_state',

  // Awareness
  AWARENESS_UPDATE = 'awareness_update',
  AWARENESS_STATE = 'awareness_state',

  // Error
  ERROR = 'error',
}

export interface Message<T = unknown> {
  type: MessageType;
  id: string;
  clientId: ClientId;
  roomId?: RoomId;
  timestamp: number;
  data: T;
}

export interface ConnectionConfig {
  url: string;
  reconnect?: boolean;
  reconnectDelay?: number;
  maxReconnectAttempts?: number;
  heartbeatInterval?: number;
  timeout?: number;
}

export interface TransportAdapter {
  connect(config: ConnectionConfig): Promise<void>;
  disconnect(): Promise<void>;
  send(message: Message): Promise<void>;
  on(event: string, handler: (data: unknown) => void): void;
  off(event: string, handler: (data: unknown) => void): void;
  isConnected(): boolean;
}

// ============================================================================
// Conflict Resolution Types
// ============================================================================

export enum ConflictResolutionStrategy {
  LAST_WRITE_WINS = 'lww',
  FIRST_WRITE_WINS = 'fww',
  CUSTOM = 'custom',
  MERGE = 'merge',
}

export interface ConflictResolver {
  resolve(op1: Operation, op2: Operation): Operation;
}

export interface ConflictResult {
  resolved: Operation;
  discarded: Operation[];
  strategy: ConflictResolutionStrategy;
}

// ============================================================================
// Event Types
// ============================================================================

export interface CollaborationEvent {
  type: string;
  timestamp: number;
  data: unknown;
}

export interface OperationEvent extends CollaborationEvent {
  type: 'operation';
  data: Operation;
}

export interface PresenceEvent extends CollaborationEvent {
  type: 'presence';
  data: UserPresence;
}

export interface AwarenessEvent extends CollaborationEvent {
  type: 'awareness';
  data: AwarenessUpdate;
}

export interface SyncEvent extends CollaborationEvent {
  type: 'sync';
  data: SyncResponse;
}

export interface ErrorEvent extends CollaborationEvent {
  type: 'error';
  data: {
    code: string;
    message: string;
    details?: unknown;
  };
}

// ============================================================================
// Utility Types
// ============================================================================

export type EventHandler<T = unknown> = (event: T) => void;
export type UnsubscribeFn = () => void;

export interface EventEmitter<T = CollaborationEvent> {
  on(event: string, handler: EventHandler<T>): UnsubscribeFn;
  off(event: string, handler: EventHandler<T>): void;
  emit(event: string, data: T): void;
  once(event: string, handler: EventHandler<T>): UnsubscribeFn;
}

// ============================================================================
// Error Types
// ============================================================================

export class CollaborationError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: unknown
  ) {
    super(message);
    this.name = 'CollaborationError';
  }
}

export class SyncError extends CollaborationError {
  constructor(message: string, details?: unknown) {
    super(message, 'SYNC_ERROR', details);
    this.name = 'SyncError';
  }
}

export class TransportError extends CollaborationError {
  constructor(message: string, details?: unknown) {
    super(message, 'TRANSPORT_ERROR', details);
    this.name = 'TransportError';
  }
}

export class RoomError extends CollaborationError {
  constructor(message: string, details?: unknown) {
    super(message, 'ROOM_ERROR', details);
    this.name = 'RoomError';
  }
}

export class PermissionError extends CollaborationError {
  constructor(message: string, details?: unknown) {
    super(message, 'PERMISSION_ERROR', details);
    this.name = 'PermissionError';
  }
}
