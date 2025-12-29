/**
 * AccuScene Enterprise v0.3.0 - Conflict Resolver
 *
 * Advanced conflict resolution strategies for concurrent editing
 */

import { EventEmitter } from 'events';
import {
  Operation,
  Conflict,
  ConflictType,
  ConflictResolution,
  ResolutionStrategy,
  UserId,
  VectorClock
} from './types';

interface ConflictDetectionResult {
  hasConflict: boolean;
  conflict?: Conflict;
  reason?: string;
}

export class ConflictResolver extends EventEmitter {
  private activeConflicts: Map<string, Conflict> = new Map();
  private resolutionHistory: ConflictResolution[] = [];
  private vectorClocks: Map<UserId, VectorClock> = new Map();

  // Configuration
  private defaultStrategy: ResolutionStrategy = ResolutionStrategy.LAST_WRITE_WINS;
  private autoResolveEnabled = true;

  constructor() {
    super();
  }

  // ============================================================================
  // Conflict Detection
  // ============================================================================

  async detectConflict(
    incomingOp: Operation,
    localOps: Operation[]
  ): Promise<Conflict | null> {
    // Check for concurrent modifications
    const conflictingOps = this.findConflictingOperations(incomingOp, localOps);

    if (conflictingOps.length === 0) {
      return null;
    }

    // Determine conflict type
    const conflictType = this.determineConflictType(incomingOp, conflictingOps[0]);

    // Create conflict object
    const conflict: Conflict = {
      id: this.generateConflictId(),
      type: conflictType,
      path: this.getOperationPath(incomingOp),
      objectId: (incomingOp.data as any)?.objectId,
      ourValue: this.getOperationValue(conflictingOps[0]),
      theirValue: this.getOperationValue(incomingOp),
      baseValue: this.getBaseValue(incomingOp, conflictingOps[0])
    };

    this.activeConflicts.set(conflict.id, conflict);
    this.emit('conflictDetected', conflict);

    return conflict;
  }

  private findConflictingOperations(op: Operation, localOps: Operation[]): Operation[] {
    return localOps.filter(localOp => {
      // Same object/resource
      const sameTarget = this.operationsTargetSame(op, localOp);
      if (!sameTarget) return false;

      // Concurrent (not causally ordered)
      const concurrent = !this.isCausallyOrdered(op, localOp);
      if (!concurrent) return false;

      // Conflicting types
      return this.operationTypesConflict(op.type, localOp.type);
    });
  }

  private operationsTargetSame(op1: Operation, op2: Operation): boolean {
    // Check if operations target the same object
    const id1 = (op1.data as any)?.objectId;
    const id2 = (op2.data as any)?.objectId;
    return id1 === id2 && id1 !== undefined;
  }

  private isCausallyOrdered(op1: Operation, op2: Operation): boolean {
    // Check vector clocks for causal ordering
    if (!op1.vectorClock || !op2.vectorClock) {
      // Fallback to timestamp
      return Math.abs(op1.timestamp - op2.timestamp) > 1000; // 1 second threshold
    }

    return this.compareVectorClocks(op1.vectorClock, op2.vectorClock) !== 0;
  }

  private compareVectorClocks(vc1: VectorClock, vc2: VectorClock): number {
    // Returns: -1 if vc1 < vc2, 1 if vc1 > vc2, 0 if concurrent
    let vc1Greater = false;
    let vc2Greater = false;

    const allUsers = new Set([...vc1.keys(), ...vc2.keys()]);

    for (const user of allUsers) {
      const t1 = vc1.get(user) || 0;
      const t2 = vc2.get(user) || 0;

      if (t1 > t2) vc1Greater = true;
      if (t2 > t1) vc2Greater = true;
    }

    if (vc1Greater && !vc2Greater) return 1;
    if (vc2Greater && !vc1Greater) return -1;
    return 0; // Concurrent
  }

  private operationTypesConflict(type1: string, type2: string): boolean {
    // Define which operation types conflict
    const conflictMatrix: Record<string, string[]> = {
      update: ['update', 'delete'],
      delete: ['update', 'delete', 'move'],
      move: ['delete', 'move'],
      transform: ['transform', 'delete'],
      property_change: ['property_change', 'delete']
    };

    return conflictMatrix[type1]?.includes(type2) || false;
  }

  private determineConflictType(op1: Operation, op2: Operation): ConflictType {
    if (op1.type === 'delete' && op2.type === 'update') {
      return ConflictType.DELETE_MODIFY;
    }
    if (op1.type === 'update' && op2.type === 'delete') {
      return ConflictType.MODIFY_DELETE;
    }
    if (op1.type === 'move' && op2.type === 'move') {
      return ConflictType.POSITION;
    }
    return ConflictType.CONTENT;
  }

  // ============================================================================
  // Conflict Resolution
  // ============================================================================

  async resolveConflict(
    conflict: Conflict,
    strategy?: ResolutionStrategy,
    userId?: UserId
  ): Promise<ConflictResolution> {
    const resolveStrategy = strategy || this.defaultStrategy;

    let resolvedValue: any;

    switch (resolveStrategy) {
      case ResolutionStrategy.TAKE_OURS:
        resolvedValue = conflict.ourValue;
        break;

      case ResolutionStrategy.TAKE_THEIRS:
        resolvedValue = conflict.theirValue;
        break;

      case ResolutionStrategy.LAST_WRITE_WINS:
        resolvedValue = this.resolveLastWriteWins(conflict);
        break;

      case ResolutionStrategy.MERGE_BOTH:
        resolvedValue = this.resolveMergeBoth(conflict);
        break;

      case ResolutionStrategy.OPERATIONAL_TRANSFORM:
        resolvedValue = await this.resolveOperationalTransform(conflict);
        break;

      case ResolutionStrategy.MANUAL:
        // Wait for manual resolution
        return this.waitForManualResolution(conflict);

      default:
        throw new Error(`Unknown resolution strategy: ${resolveStrategy}`);
    }

    const resolution: ConflictResolution = {
      strategy: resolveStrategy,
      value: resolvedValue,
      resolvedBy: userId || 'system',
      timestamp: Date.now()
    };

    conflict.resolution = resolution;
    this.activeConflicts.delete(conflict.id);
    this.resolutionHistory.push(resolution);

    this.emit('conflictResolved', { conflict, resolution });

    return resolution;
  }

  private resolveLastWriteWins(conflict: Conflict): any {
    // Use timestamp metadata to determine latest write
    // In practice, this would come from operation metadata
    return conflict.theirValue; // Assume incoming is newer
  }

  private resolveMergeBoth(conflict: Conflict): any {
    // Attempt to merge both values
    if (typeof conflict.ourValue === 'object' && typeof conflict.theirValue === 'object') {
      return {
        ...conflict.ourValue,
        ...conflict.theirValue
      };
    }

    // For primitives, default to theirs
    return conflict.theirValue;
  }

  private async resolveOperationalTransform(conflict: Conflict): Promise<any> {
    // Apply operational transformation
    // This is a simplified version - real OT is more complex

    if (conflict.type === ConflictType.POSITION) {
      // For position conflicts, average the positions
      if (
        typeof conflict.ourValue === 'object' &&
        typeof conflict.theirValue === 'object' &&
        'x' in conflict.ourValue &&
        'x' in conflict.theirValue
      ) {
        return {
          x: (conflict.ourValue.x + conflict.theirValue.x) / 2,
          y: (conflict.ourValue.y + conflict.theirValue.y) / 2,
          z: conflict.ourValue.z !== undefined && conflict.theirValue.z !== undefined
            ? (conflict.ourValue.z + conflict.theirValue.z) / 2
            : undefined
        };
      }
    }

    // Default to their value
    return conflict.theirValue;
  }

  private waitForManualResolution(conflict: Conflict): Promise<ConflictResolution> {
    return new Promise((resolve) => {
      this.once(`manualResolution:${conflict.id}`, (resolution: ConflictResolution) => {
        resolve(resolution);
      });
    });
  }

  async resolveManually(conflictId: string, value: any, userId: UserId): Promise<void> {
    const conflict = this.activeConflicts.get(conflictId);
    if (!conflict) {
      throw new Error(`Conflict not found: ${conflictId}`);
    }

    const resolution: ConflictResolution = {
      strategy: ResolutionStrategy.MANUAL,
      value,
      resolvedBy: userId,
      timestamp: Date.now()
    };

    conflict.resolution = resolution;
    this.activeConflicts.delete(conflictId);
    this.resolutionHistory.push(resolution);

    this.emit(`manualResolution:${conflictId}`, resolution);
    this.emit('conflictResolved', { conflict, resolution });
  }

  // ============================================================================
  // Auto-Resolution
  // ============================================================================

  async autoResolveConflicts(conflicts: Conflict[]): Promise<Map<string, ConflictResolution>> {
    if (!this.autoResolveEnabled) {
      throw new Error('Auto-resolve is disabled');
    }

    const resolutions = new Map<string, ConflictResolution>();

    for (const conflict of conflicts) {
      if (this.canAutoResolve(conflict)) {
        const resolution = await this.resolveConflict(conflict);
        resolutions.set(conflict.id, resolution);
      }
    }

    return resolutions;
  }

  private canAutoResolve(conflict: Conflict): boolean {
    // Determine if conflict can be automatically resolved
    switch (conflict.type) {
      case ConflictType.POSITION:
        return true; // Can average positions
      case ConflictType.CONTENT:
        return typeof conflict.ourValue === 'object'; // Can merge objects
      case ConflictType.DELETE_MODIFY:
      case ConflictType.MODIFY_DELETE:
        return false; // Requires manual resolution
      default:
        return true;
    }
  }

  setAutoResolve(enabled: boolean): void {
    this.autoResolveEnabled = enabled;
    this.emit('autoResolveChanged', enabled);
  }

  setDefaultStrategy(strategy: ResolutionStrategy): void {
    this.defaultStrategy = strategy;
    this.emit('defaultStrategyChanged', strategy);
  }

  // ============================================================================
  // Conflict Management
  // ============================================================================

  getActiveConflicts(): Conflict[] {
    return Array.from(this.activeConflicts.values());
  }

  getConflict(conflictId: string): Conflict | null {
    return this.activeConflicts.get(conflictId) || null;
  }

  getResolutionHistory(): ConflictResolution[] {
    return [...this.resolutionHistory];
  }

  clearResolutionHistory(): void {
    this.resolutionHistory = [];
    this.emit('historyCleared');
  }

  // ============================================================================
  // Vector Clock Management
  // ============================================================================

  updateVectorClock(userId: UserId, operation: Operation): void {
    let clock = this.vectorClocks.get(userId);

    if (!clock) {
      clock = new Map();
      this.vectorClocks.set(userId, clock);
    }

    // Increment user's counter
    clock.set(userId, (clock.get(userId) || 0) + 1);

    // Update operation's vector clock
    operation.vectorClock = new Map(clock);
  }

  mergeVectorClock(userId: UserId, incomingClock: VectorClock): void {
    let localClock = this.vectorClocks.get(userId);

    if (!localClock) {
      localClock = new Map();
      this.vectorClocks.set(userId, localClock);
    }

    // Merge clocks (take max of each counter)
    for (const [user, count] of incomingClock.entries()) {
      const localCount = localClock.get(user) || 0;
      localClock.set(user, Math.max(localCount, count));
    }
  }

  // ============================================================================
  // Helpers
  // ============================================================================

  private getOperationPath(operation: Operation): string {
    return (operation.data as any)?.path || (operation.data as any)?.objectId || 'unknown';
  }

  private getOperationValue(operation: Operation): any {
    return (operation.data as any)?.value || operation.data;
  }

  private getBaseValue(op1: Operation, op2: Operation): any {
    // In a real implementation, this would fetch the base value from history
    return null;
  }

  private generateConflictId(): string {
    return `conflict-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  // ============================================================================
  // Statistics
  // ============================================================================

  getStatistics() {
    const resolutionsByStrategy = this.resolutionHistory.reduce((acc, res) => {
      acc[res.strategy] = (acc[res.strategy] || 0) + 1;
      return acc;
    }, {} as Record<ResolutionStrategy, number>);

    return {
      activeConflicts: this.activeConflicts.size,
      totalResolved: this.resolutionHistory.length,
      resolutionsByStrategy,
      autoResolveEnabled: this.autoResolveEnabled,
      defaultStrategy: this.defaultStrategy
    };
  }
}
