/**
 * AccuScene Enterprise v0.3.0 - Collaboration Engine
 *
 * Core collaboration engine with state machine and operation orchestration
 */

import { EventEmitter } from 'events';
import {
  CollaborationSession,
  SessionId,
  UserId,
  User,
  Operation,
  CollaborationConfig,
  ConnectionStatus,
  CollaborationEvent,
  CollaborationEventType,
  PerformanceMetrics,
  SessionSettings
} from './types';
import { ConnectionManager } from './ConnectionManager';
import { PresenceManager } from './PresenceManager';
import { VersionControl } from './VersionControl';
import { ConflictResolver } from './ConflictResolver';
import { PermissionManager } from './PermissionManager';
import { AuditLogger } from './AuditLogger';
import { OfflineQueue } from './OfflineQueue';

enum EngineState {
  IDLE = 'idle',
  INITIALIZING = 'initializing',
  CONNECTING = 'connecting',
  ACTIVE = 'active',
  SYNCING = 'syncing',
  OFFLINE = 'offline',
  RECONNECTING = 'reconnecting',
  ERROR = 'error',
  SHUTDOWN = 'shutdown'
}

interface StateTransition {
  from: EngineState;
  to: EngineState;
  timestamp: number;
  reason?: string;
}

export class CollaborationEngine extends EventEmitter {
  private state: EngineState = EngineState.IDLE;
  private session: CollaborationSession | null = null;
  private currentUser: User | null = null;
  private config: CollaborationConfig;

  // Component managers
  private connectionManager: ConnectionManager;
  private presenceManager: PresenceManager;
  private versionControl: VersionControl;
  private conflictResolver: ConflictResolver;
  private permissionManager: PermissionManager;
  private auditLogger: AuditLogger;
  private offlineQueue: OfflineQueue;

  // State tracking
  private stateHistory: StateTransition[] = [];
  private operationQueue: Operation[] = [];
  private pendingOperations: Map<string, Operation> = new Map();
  private acknowledgements: Map<string, number> = new Map();

  // Performance tracking
  private metrics: PerformanceMetrics = {
    operationsPerSecond: 0,
    averageLatency: 0,
    peakLatency: 0,
    syncLag: 0,
    memoryUsage: 0,
    activeConnections: 0
  };

  private metricsInterval: NodeJS.Timeout | null = null;
  private operationCounter = 0;
  private lastMetricsReset = Date.now();

  constructor(config: CollaborationConfig) {
    super();
    this.config = config;

    // Initialize component managers
    this.connectionManager = new ConnectionManager(config);
    this.presenceManager = new PresenceManager();
    this.versionControl = new VersionControl();
    this.conflictResolver = new ConflictResolver();
    this.permissionManager = new PermissionManager();
    this.auditLogger = new AuditLogger();
    this.offlineQueue = new OfflineQueue({
      maxOperations: config.maxOfflineOperations,
      persistToStorage: config.offlineSupport
    });

    this.setupEventHandlers();
    this.startMetricsTracking();
  }

  // ============================================================================
  // Public API
  // ============================================================================

  async initialize(user: User, sessionId: SessionId, settings?: Partial<SessionSettings>): Promise<void> {
    this.transitionState(EngineState.INITIALIZING);

    try {
      this.currentUser = user;

      // Initialize session
      this.session = {
        id: sessionId,
        sceneId: sessionId, // TODO: Get actual scene ID
        branchId: 'main',
        participants: new Map(),
        createdAt: Date.now(),
        expiresAt: Date.now() + 24 * 60 * 60 * 1000, // 24 hours
        settings: {
          maxParticipants: 50,
          allowAnonymous: false,
          requireApproval: false,
          encryption: {
            enabled: this.config.encryptionEnabled,
            algorithm: 'AES-256-GCM',
            keyRotationInterval: 3600000 // 1 hour
          },
          voiceEnabled: true,
          videoEnabled: true,
          recordingEnabled: false,
          ...settings
        }
      };

      // Initialize components
      await this.connectionManager.connect(sessionId, user);
      await this.presenceManager.initialize(user, sessionId);
      await this.versionControl.initialize(sessionId);

      this.auditLogger.log({
        id: this.generateId(),
        sessionId,
        userId: user.id,
        action: 'initialize',
        resource: 'session' as any,
        resourceId: sessionId,
        timestamp: Date.now(),
        severity: 'info' as any
      });

      this.transitionState(EngineState.CONNECTING);

      // Wait for connection
      await this.waitForConnection();

      this.transitionState(EngineState.ACTIVE);

      this.emitEvent({
        type: CollaborationEventType.USER_JOINED,
        timestamp: Date.now(),
        data: { user }
      });

    } catch (error) {
      this.transitionState(EngineState.ERROR, (error as Error).message);
      throw error;
    }
  }

  async shutdown(): Promise<void> {
    this.transitionState(EngineState.SHUTDOWN);

    try {
      // Process any pending operations
      await this.flushOperationQueue();

      // Cleanup components
      await this.connectionManager.disconnect();
      this.presenceManager.destroy();
      this.stopMetricsTracking();

      if (this.session && this.currentUser) {
        this.auditLogger.log({
          id: this.generateId(),
          sessionId: this.session.id,
          userId: this.currentUser.id,
          action: 'shutdown',
          resource: 'session' as any,
          resourceId: this.session.id,
          timestamp: Date.now(),
          severity: 'info' as any
        });

        this.emitEvent({
          type: CollaborationEventType.USER_LEFT,
          timestamp: Date.now(),
          data: { user: this.currentUser }
        });
      }

      this.removeAllListeners();

    } catch (error) {
      console.error('Error during shutdown:', error);
    }
  }

  async applyOperation(operation: Operation): Promise<void> {
    if (this.state !== EngineState.ACTIVE) {
      // Queue operation if offline
      if (this.state === EngineState.OFFLINE) {
        await this.offlineQueue.enqueue({
          id: operation.id,
          operation,
          queuedAt: Date.now(),
          retries: 0
        });
        return;
      }
      throw new Error(`Cannot apply operation in state: ${this.state}`);
    }

    if (!this.currentUser) {
      throw new Error('No current user');
    }

    // Check permissions
    const hasPermission = await this.permissionManager.checkPermission(
      this.currentUser.id,
      'object' as any,
      'edit' as any
    );

    if (!hasPermission) {
      throw new Error('Permission denied');
    }

    // Add to pending operations
    this.pendingOperations.set(operation.id, operation);

    // Send operation
    await this.connectionManager.sendOperation(operation);

    // Add to operation queue for processing
    this.operationQueue.push(operation);

    // Process queue
    await this.processOperationQueue();

    // Update metrics
    this.operationCounter++;

    this.auditLogger.log({
      id: this.generateId(),
      sessionId: this.session!.id,
      userId: this.currentUser.id,
      action: 'apply_operation',
      resource: 'object' as any,
      resourceId: operation.id,
      timestamp: Date.now(),
      details: { operationType: operation.type },
      severity: 'info' as any
    });
  }

  getState(): EngineState {
    return this.state;
  }

  getSession(): CollaborationSession | null {
    return this.session;
  }

  getCurrentUser(): User | null {
    return this.currentUser;
  }

  getMetrics(): PerformanceMetrics {
    return { ...this.metrics };
  }

  getConnectionStatus(): ConnectionStatus {
    return this.connectionManager.getStatus();
  }

  // Component accessors
  getPresenceManager(): PresenceManager {
    return this.presenceManager;
  }

  getVersionControl(): VersionControl {
    return this.versionControl;
  }

  getConflictResolver(): ConflictResolver {
    return this.conflictResolver;
  }

  getPermissionManager(): PermissionManager {
    return this.permissionManager;
  }

  getAuditLogger(): AuditLogger {
    return this.auditLogger;
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private transitionState(newState: EngineState, reason?: string): void {
    const transition: StateTransition = {
      from: this.state,
      to: newState,
      timestamp: Date.now(),
      reason
    };

    this.stateHistory.push(transition);
    this.state = newState;

    this.emit('stateChange', { from: transition.from, to: newState, reason });

    // Handle state-specific logic
    this.handleStateTransition(transition);
  }

  private handleStateTransition(transition: StateTransition): void {
    switch (transition.to) {
      case EngineState.OFFLINE:
        this.handleOfflineMode();
        break;
      case EngineState.RECONNECTING:
        this.handleReconnecting();
        break;
      case EngineState.ACTIVE:
        if (transition.from === EngineState.OFFLINE || transition.from === EngineState.RECONNECTING) {
          this.handleBackOnline();
        }
        break;
    }
  }

  private handleOfflineMode(): void {
    console.log('Entering offline mode');
    // Switch to offline queue for operations
  }

  private handleReconnecting(): void {
    console.log('Attempting to reconnect...');
  }

  private async handleBackOnline(): Promise<void> {
    console.log('Back online, syncing offline operations...');

    // Process offline queue
    const operations = await this.offlineQueue.getAll();

    for (const op of operations) {
      try {
        await this.connectionManager.sendOperation(op.operation);
        await this.offlineQueue.dequeue(op.id);
      } catch (error) {
        console.error('Failed to sync offline operation:', error);
      }
    }
  }

  private setupEventHandlers(): void {
    // Connection events
    this.connectionManager.on('statusChange', (status: ConnectionStatus) => {
      if (status === ConnectionStatus.CONNECTED && this.state === EngineState.CONNECTING) {
        this.transitionState(EngineState.ACTIVE);
      } else if (status === ConnectionStatus.DISCONNECTED && this.state === EngineState.ACTIVE) {
        this.transitionState(EngineState.OFFLINE);
      } else if (status === ConnectionStatus.RECONNECTING) {
        this.transitionState(EngineState.RECONNECTING);
      }

      this.emitEvent({
        type: CollaborationEventType.CONNECTION_STATE_CHANGED,
        timestamp: Date.now(),
        data: { status }
      });
    });

    // Operation events
    this.connectionManager.on('operation', async (operation: Operation) => {
      await this.handleRemoteOperation(operation);
    });

    // Presence events
    this.connectionManager.on('presenceUpdate', (presence: any) => {
      this.presenceManager.updatePresence(presence.userId, presence);
    });
  }

  private async handleRemoteOperation(operation: Operation): Promise<void> {
    try {
      // Check if we've seen this operation
      if (this.acknowledgements.has(operation.id)) {
        return;
      }

      // Check for conflicts
      const conflict = await this.conflictResolver.detectConflict(operation, this.operationQueue);

      if (conflict) {
        this.emitEvent({
          type: CollaborationEventType.CONFLICT_DETECTED,
          timestamp: Date.now(),
          data: { conflict, operation }
        });

        // Resolve conflict
        const resolution = await this.conflictResolver.resolveConflict(conflict);

        this.emitEvent({
          type: CollaborationEventType.CONFLICT_RESOLVED,
          timestamp: Date.now(),
          data: { conflict, resolution }
        });
      }

      // Apply operation
      this.operationQueue.push(operation);
      await this.processOperationQueue();

      // Acknowledge
      this.acknowledgements.set(operation.id, Date.now());

      this.emitEvent({
        type: CollaborationEventType.OPERATION_APPLIED,
        timestamp: Date.now(),
        data: { operation }
      });

    } catch (error) {
      console.error('Error handling remote operation:', error);
      this.emitEvent({
        type: CollaborationEventType.SYNC_ERROR,
        timestamp: Date.now(),
        data: { error, operation }
      });
    }
  }

  private async processOperationQueue(): Promise<void> {
    // Sort by timestamp and dependencies
    const sorted = this.sortOperationsByDependencies(this.operationQueue);

    // Process in order
    for (const operation of sorted) {
      await this.applyOperationToScene(operation);
    }

    // Clear processed operations
    this.operationQueue = [];
  }

  private sortOperationsByDependencies(operations: Operation[]): Operation[] {
    // Simple topological sort based on dependencies
    const sorted: Operation[] = [];
    const visited = new Set<string>();

    const visit = (op: Operation) => {
      if (visited.has(op.id)) return;

      // Visit dependencies first
      if (op.dependencies) {
        for (const depId of op.dependencies) {
          const dep = operations.find(o => o.id === depId);
          if (dep) visit(dep);
        }
      }

      visited.add(op.id);
      sorted.push(op);
    };

    for (const op of operations) {
      visit(op);
    }

    return sorted;
  }

  private async applyOperationToScene(operation: Operation): Promise<void> {
    // This would integrate with the actual scene graph
    // For now, just emit the operation
    this.emit('applyOperation', operation);
  }

  private async flushOperationQueue(): Promise<void> {
    if (this.operationQueue.length > 0) {
      await this.processOperationQueue();
    }
  }

  private async waitForConnection(): Promise<void> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Connection timeout'));
      }, 10000);

      const checkConnection = () => {
        if (this.connectionManager.getStatus() === ConnectionStatus.CONNECTED) {
          clearTimeout(timeout);
          resolve();
        } else {
          setTimeout(checkConnection, 100);
        }
      };

      checkConnection();
    });
  }

  private startMetricsTracking(): void {
    this.metricsInterval = setInterval(() => {
      this.updateMetrics();
    }, 1000);
  }

  private stopMetricsTracking(): void {
    if (this.metricsInterval) {
      clearInterval(this.metricsInterval);
      this.metricsInterval = null;
    }
  }

  private updateMetrics(): void {
    const now = Date.now();
    const elapsed = (now - this.lastMetricsReset) / 1000;

    this.metrics = {
      operationsPerSecond: this.operationCounter / elapsed,
      averageLatency: this.connectionManager.getAverageLatency(),
      peakLatency: this.connectionManager.getPeakLatency(),
      syncLag: this.calculateSyncLag(),
      memoryUsage: this.estimateMemoryUsage(),
      activeConnections: this.presenceManager.getActiveUserCount()
    };

    // Reset counters every minute
    if (elapsed > 60) {
      this.operationCounter = 0;
      this.lastMetricsReset = now;
    }

    this.emit('metricsUpdate', this.metrics);
  }

  private calculateSyncLag(): number {
    // Calculate lag between local and remote state
    const pendingOps = this.pendingOperations.size;
    const queuedOps = this.operationQueue.length;
    return pendingOps + queuedOps;
  }

  private estimateMemoryUsage(): number {
    // Rough estimation of memory usage in MB
    const pendingSize = this.pendingOperations.size * 1024; // ~1KB per operation
    const queueSize = this.operationQueue.length * 1024;
    const ackSize = this.acknowledgements.size * 100;
    return (pendingSize + queueSize + ackSize) / (1024 * 1024);
  }

  private emitEvent(event: CollaborationEvent): void {
    this.emit('collaborationEvent', event);
  }

  private generateId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}
