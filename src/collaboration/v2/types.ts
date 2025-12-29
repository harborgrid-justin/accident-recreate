/**
 * AccuScene Enterprise v0.3.0 - Enhanced Collaboration Types
 *
 * Comprehensive TypeScript types for real-time collaboration system
 */

// ============================================================================
// Core Types
// ============================================================================

export type UserId = string;
export type SessionId = string;
export type SceneId = string;
export type BranchId = string;
export type CommitId = string;
export type AnnotationId = string;
export type MessageId = string;

// ============================================================================
// User & Session Types
// ============================================================================

export interface User {
  id: UserId;
  name: string;
  email: string;
  avatar?: string;
  role: UserRole;
  color: string; // Hex color for cursor/selection
}

export enum UserRole {
  VIEWER = 'viewer',
  EDITOR = 'editor',
  ADMIN = 'admin',
  OWNER = 'owner'
}

export interface Presence {
  userId: UserId;
  user: User;
  cursor: CursorPosition | null;
  selection: Selection | null;
  viewport: Viewport | null;
  status: UserStatus;
  lastActivity: number;
  isTyping: boolean;
  currentTool?: string;
}

export interface CursorPosition {
  x: number;
  y: number;
  z?: number;
  timestamp: number;
}

export interface Selection {
  objectIds: string[];
  type: SelectionType;
  bounds?: BoundingBox;
}

export enum SelectionType {
  OBJECTS = 'objects',
  VERTICES = 'vertices',
  REGION = 'region'
}

export interface BoundingBox {
  min: { x: number; y: number; z?: number };
  max: { x: number; y: number; z?: number };
}

export interface Viewport {
  center: { x: number; y: number; z?: number };
  zoom: number;
  rotation?: number;
}

export enum UserStatus {
  ONLINE = 'online',
  AWAY = 'away',
  BUSY = 'busy',
  OFFLINE = 'offline'
}

// ============================================================================
// Collaboration Session Types
// ============================================================================

export interface CollaborationSession {
  id: SessionId;
  sceneId: SceneId;
  branchId: BranchId;
  participants: Map<UserId, Presence>;
  createdAt: number;
  expiresAt: number;
  settings: SessionSettings;
}

export interface SessionSettings {
  maxParticipants: number;
  allowAnonymous: boolean;
  requireApproval: boolean;
  encryption: EncryptionSettings;
  voiceEnabled: boolean;
  videoEnabled: boolean;
  recordingEnabled: boolean;
}

export interface EncryptionSettings {
  enabled: boolean;
  algorithm: 'AES-256-GCM' | 'ChaCha20-Poly1305';
  keyRotationInterval: number;
}

// ============================================================================
// Operation & Change Types
// ============================================================================

export interface Operation {
  id: string;
  type: OperationType;
  userId: UserId;
  timestamp: number;
  data: any;
  dependencies?: string[]; // For causal ordering
  vectorClock?: VectorClock;
}

export enum OperationType {
  CREATE = 'create',
  UPDATE = 'update',
  DELETE = 'delete',
  MOVE = 'move',
  TRANSFORM = 'transform',
  PROPERTY_CHANGE = 'property_change'
}

export type VectorClock = Map<UserId, number>;

export interface Change {
  id: string;
  operation: Operation;
  beforeState?: any;
  afterState?: any;
  applied: boolean;
  conflicted?: boolean;
}

// ============================================================================
// Version Control Types
// ============================================================================

export interface Branch {
  id: BranchId;
  name: string;
  parentBranchId?: BranchId;
  baseCommitId: CommitId;
  headCommitId: CommitId;
  createdBy: UserId;
  createdAt: number;
  protected: boolean;
}

export interface Commit {
  id: CommitId;
  branchId: BranchId;
  parentCommitId?: CommitId;
  message: string;
  author: UserId;
  timestamp: number;
  operations: Operation[];
  snapshot?: SceneSnapshot;
  tags?: string[];
}

export interface SceneSnapshot {
  sceneId: SceneId;
  commitId: CommitId;
  data: any;
  checksum: string;
  compressed: boolean;
}

export interface Diff {
  commitA: CommitId;
  commitB: CommitId;
  changes: DiffChange[];
  statistics: DiffStatistics;
}

export interface DiffChange {
  type: 'added' | 'modified' | 'deleted';
  path: string;
  objectId?: string;
  beforeValue?: any;
  afterValue?: any;
}

export interface DiffStatistics {
  added: number;
  modified: number;
  deleted: number;
  totalChanges: number;
}

export interface MergeResult {
  success: boolean;
  conflicts: Conflict[];
  mergedCommit?: Commit;
  autoResolved: number;
  manualRequired: number;
}

export interface Conflict {
  id: string;
  type: ConflictType;
  path: string;
  objectId?: string;
  ourValue: any;
  theirValue: any;
  baseValue?: any;
  resolution?: ConflictResolution;
}

export enum ConflictType {
  CONTENT = 'content',
  DELETE_MODIFY = 'delete_modify',
  MODIFY_DELETE = 'modify_delete',
  RENAME = 'rename',
  POSITION = 'position'
}

export interface ConflictResolution {
  strategy: ResolutionStrategy;
  value: any;
  resolvedBy: UserId;
  timestamp: number;
}

export enum ResolutionStrategy {
  TAKE_OURS = 'take_ours',
  TAKE_THEIRS = 'take_theirs',
  MANUAL = 'manual',
  MERGE_BOTH = 'merge_both',
  LAST_WRITE_WINS = 'last_write_wins',
  OPERATIONAL_TRANSFORM = 'operational_transform'
}

// ============================================================================
// Communication Types
// ============================================================================

export interface ChatMessage {
  id: MessageId;
  sessionId: SessionId;
  userId: UserId;
  content: string;
  timestamp: number;
  threadId?: MessageId;
  mentions?: UserId[];
  attachments?: Attachment[];
  reactions?: Map<string, UserId[]>; // emoji -> userIds
  edited?: boolean;
  editedAt?: number;
}

export interface Attachment {
  id: string;
  type: 'image' | 'file' | 'link' | 'scene_object';
  url?: string;
  name: string;
  size?: number;
  objectId?: string;
}

export interface Annotation {
  id: AnnotationId;
  sessionId: SessionId;
  sceneId: SceneId;
  userId: UserId;
  position: { x: number; y: number; z?: number };
  content: string;
  type: AnnotationType;
  attachedToObjectId?: string;
  resolved: boolean;
  createdAt: number;
  updatedAt: number;
  replies?: AnnotationReply[];
}

export enum AnnotationType {
  COMMENT = 'comment',
  QUESTION = 'question',
  ISSUE = 'issue',
  SUGGESTION = 'suggestion'
}

export interface AnnotationReply {
  id: string;
  userId: UserId;
  content: string;
  timestamp: number;
}

// ============================================================================
// WebRTC Types
// ============================================================================

export interface VoiceConnection {
  userId: UserId;
  peerId: string;
  stream: MediaStream;
  muted: boolean;
  volume: number;
  speaking: boolean;
}

export interface VideoConnection {
  userId: UserId;
  peerId: string;
  stream: MediaStream;
  videoEnabled: boolean;
  audioEnabled: boolean;
  screenSharing: boolean;
  quality: VideoQuality;
}

export enum VideoQuality {
  LOW = 'low',      // 320x240
  MEDIUM = 'medium', // 640x480
  HIGH = 'high',    // 1280x720
  HD = 'hd'         // 1920x1080
}

export interface RTCStats {
  userId: UserId;
  bitrate: number;
  packetLoss: number;
  latency: number;
  jitter: number;
}

// ============================================================================
// Permission Types
// ============================================================================

export interface Permission {
  resource: Resource;
  action: Action;
  granted: boolean;
  conditions?: PermissionCondition[];
}

export enum Resource {
  SCENE = 'scene',
  BRANCH = 'branch',
  OBJECT = 'object',
  ANNOTATION = 'annotation',
  CHAT = 'chat',
  VOICE = 'voice',
  VIDEO = 'video',
  SETTINGS = 'settings'
}

export enum Action {
  VIEW = 'view',
  CREATE = 'create',
  EDIT = 'edit',
  DELETE = 'delete',
  SHARE = 'share',
  EXPORT = 'export',
  MANAGE = 'manage'
}

export interface PermissionCondition {
  type: 'time' | 'ip' | 'device' | 'custom';
  value: any;
}

export interface RolePermissions {
  role: UserRole;
  permissions: Permission[];
}

// ============================================================================
// Audit Types
// ============================================================================

export interface AuditEvent {
  id: string;
  sessionId: SessionId;
  userId: UserId;
  action: string;
  resource: Resource;
  resourceId: string;
  timestamp: number;
  ip?: string;
  userAgent?: string;
  details?: any;
  severity: AuditSeverity;
}

export enum AuditSeverity {
  INFO = 'info',
  WARNING = 'warning',
  ERROR = 'error',
  CRITICAL = 'critical'
}

// ============================================================================
// Connection & Sync Types
// ============================================================================

export interface ConnectionState {
  status: ConnectionStatus;
  latency: number;
  lastPing: number;
  reconnectAttempts: number;
  offlineOperations: number;
}

export enum ConnectionStatus {
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  RECONNECTING = 'reconnecting',
  DISCONNECTED = 'disconnected',
  FAILED = 'failed'
}

export interface SyncMessage {
  type: SyncMessageType;
  sessionId: SessionId;
  senderId: UserId;
  timestamp: number;
  sequence: number;
  payload: any;
  compressed?: boolean;
  encrypted?: boolean;
}

export enum SyncMessageType {
  JOIN = 'join',
  LEAVE = 'leave',
  OPERATION = 'operation',
  PRESENCE_UPDATE = 'presence_update',
  CURSOR_MOVE = 'cursor_move',
  SELECTION_CHANGE = 'selection_change',
  CHAT_MESSAGE = 'chat_message',
  ANNOTATION = 'annotation',
  VOICE_SIGNAL = 'voice_signal',
  VIDEO_SIGNAL = 'video_signal',
  PING = 'ping',
  PONG = 'pong',
  ACK = 'ack'
}

export interface OfflineOperation {
  id: string;
  operation: Operation;
  queuedAt: number;
  retries: number;
  lastError?: string;
}

// ============================================================================
// Recording Types
// ============================================================================

export interface SessionRecording {
  id: string;
  sessionId: SessionId;
  startTime: number;
  endTime?: number;
  events: RecordedEvent[];
  participants: User[];
  duration: number;
  size: number;
}

export interface RecordedEvent {
  type: string;
  timestamp: number;
  userId: UserId;
  data: any;
}

export interface PlaybackState {
  recording: SessionRecording;
  currentTime: number;
  playing: boolean;
  speed: number;
  loop: boolean;
}

// ============================================================================
// UI State Types
// ============================================================================

export interface CollaborationUIState {
  panelOpen: boolean;
  activeTab: CollaborationTab;
  showCursors: boolean;
  showSelections: boolean;
  showAvatars: boolean;
  showAnnotations: boolean;
  voiceConnected: boolean;
  videoConnected: boolean;
}

export enum CollaborationTab {
  PARTICIPANTS = 'participants',
  CHAT = 'chat',
  HISTORY = 'history',
  BRANCHES = 'branches',
  ANNOTATIONS = 'annotations',
  SETTINGS = 'settings'
}

// ============================================================================
// Event Types
// ============================================================================

export interface CollaborationEvent {
  type: CollaborationEventType;
  timestamp: number;
  data: any;
}

export enum CollaborationEventType {
  USER_JOINED = 'user_joined',
  USER_LEFT = 'user_left',
  OPERATION_APPLIED = 'operation_applied',
  CONFLICT_DETECTED = 'conflict_detected',
  CONFLICT_RESOLVED = 'conflict_resolved',
  BRANCH_CREATED = 'branch_created',
  COMMIT_CREATED = 'commit_created',
  MERGE_COMPLETED = 'merge_completed',
  CONNECTION_STATE_CHANGED = 'connection_state_changed',
  SYNC_ERROR = 'sync_error'
}

// ============================================================================
// Configuration Types
// ============================================================================

export interface CollaborationConfig {
  serverUrl: string;
  wsUrl: string;
  stunServers: RTCIceServer[];
  turnServers: RTCIceServer[];
  maxReconnectAttempts: number;
  reconnectDelay: number;
  pingInterval: number;
  operationBatchSize: number;
  compressionThreshold: number;
  encryptionEnabled: boolean;
  offlineSupport: boolean;
  maxOfflineOperations: number;
  presenceUpdateInterval: number;
  cursorThrottleMs: number;
}

export interface PerformanceMetrics {
  operationsPerSecond: number;
  averageLatency: number;
  peakLatency: number;
  syncLag: number;
  memoryUsage: number;
  activeConnections: number;
}
