/**
 * AccuScene Enterprise v0.3.0 - Enhanced Collaboration System
 *
 * Complete real-time collaboration module with advanced features
 */

// Core Engine
export { CollaborationEngine } from './CollaborationEngine';

// Managers
export { PresenceManager } from './PresenceManager';
export { VersionControl } from './VersionControl';
export { ConflictResolver } from './ConflictResolver';
export { BranchManager } from './BranchManager';
export { MergeEngine } from './MergeEngine';
export { HistoryTimeline } from './HistoryTimeline';
export { SessionRecorder } from './SessionRecorder';
export { PermissionManager } from './PermissionManager';
export { RoleManager } from './RoleManager';
export { AuditLogger } from './AuditLogger';

// Communication
export { VoiceChat } from './VoiceChat';
export { VideoCall } from './VideoCall';
export { ChatSystem } from './ChatSystem';
export { AnnotationSync } from './AnnotationSync';

// Network & Sync
export { ConnectionManager } from './ConnectionManager';
export { SyncProtocol } from './SyncProtocol';
export { OfflineQueue } from './OfflineQueue';

// React Components
export {
  ConnectionIndicator,
  SyncIndicator,
  PerformanceIndicator,
  TypingIndicator
} from './RealtimeIndicators';

export {
  UserAvatar,
  UserAvatarStack,
  UserListItem
} from './UserAvatars';

export { CursorOverlay, Cursor } from './CursorOverlay';
export { SelectionOverlay, SelectionHighlight } from './SelectionHighlight';
export { CollaborationPanel } from './CollaborationPanel';

// React Hooks
export { useCollaboration } from './hooks/useCollaboration';
export { usePresence } from './hooks/usePresence';
export { useVersionControl } from './hooks/useVersionControl';

// Types
export * from './types';

/**
 * Usage Example:
 *
 * ```typescript
 * import { useCollaboration, CollaborationPanel } from '@/collaboration/v2';
 *
 * function MyCollaborativeEditor() {
 *   const collaboration = useCollaboration({
 *     config: {
 *       serverUrl: 'https://api.example.com',
 *       wsUrl: 'wss://ws.example.com',
 *       stunServers: [{ urls: 'stun:stun.l.google.com:19302' }],
 *       maxReconnectAttempts: 5,
 *       encryptionEnabled: true
 *     },
 *     user: currentUser,
 *     sessionId: sceneId
 *   });
 *
 *   const { presences } = usePresence(collaboration.presenceManager);
 *
 *   return (
 *     <div>
 *       <CursorOverlay presences={presences} />
 *       <SelectionOverlay presences={presences} />
 *       <CollaborationPanel presences={presences} />
 *     </div>
 *   );
 * }
 * ```
 */
