/**
 * AccuScene Enterprise v0.3.0 - Merge Engine
 *
 * Three-way merge for scene data with intelligent conflict detection
 */

import { EventEmitter } from 'events';
import {
  Commit,
  CommitId,
  MergeResult,
  Conflict,
  ConflictType,
  Diff,
  DiffChange,
  DiffStatistics,
  Operation,
  ResolutionStrategy
} from './types';

interface MergeContext {
  base: Commit | null;
  ours: Commit;
  theirs: Commit;
  strategy: MergeStrategy;
}

enum MergeStrategy {
  RECURSIVE = 'recursive',
  OURS = 'ours',
  THEIRS = 'theirs',
  OCTOPUS = 'octopus'
}

export class MergeEngine extends EventEmitter {
  private mergeHistory: MergeResult[] = [];

  constructor() {
    super();
  }

  // ============================================================================
  // Three-Way Merge
  // ============================================================================

  async merge(
    sourceCommits: Commit[],
    targetCommits: Commit[],
    baseCommit: Commit | null
  ): Promise<MergeResult> {
    if (sourceCommits.length === 0 || targetCommits.length === 0) {
      throw new Error('Cannot merge empty commit lists');
    }

    const source = sourceCommits[0];
    const target = targetCommits[0];

    const context: MergeContext = {
      base: baseCommit,
      ours: target,
      theirs: source,
      strategy: MergeStrategy.RECURSIVE
    };

    // Perform three-way merge
    const result = await this.performThreeWayMerge(context);

    // Store in history
    this.mergeHistory.push(result);
    this.emit('mergeCompleted', result);

    return result;
  }

  private async performThreeWayMerge(context: MergeContext): Promise<MergeResult> {
    const conflicts: Conflict[] = [];
    const mergedOperations: Operation[] = [];

    // Get operations from each commit
    const baseOps = context.base?.operations || [];
    const ourOps = context.ours.operations;
    const theirOps = context.theirs.operations;

    // Track which operations we've processed
    const processedIds = new Set<string>();

    // Process our operations
    for (const ourOp of ourOps) {
      processedIds.add(ourOp.id);

      // Find corresponding operations in base and theirs
      const baseOp = baseOps.find(op => this.operationsMatch(op, ourOp));
      const theirOp = theirOps.find(op => this.operationsMatch(op, ourOp));

      if (!theirOp) {
        // Operation only in ours - take it
        mergedOperations.push(ourOp);
      } else if (!baseOp) {
        // New operation in both - check for conflict
        const conflict = this.detectMergeConflict(ourOp, theirOp, null);
        if (conflict) {
          conflicts.push(conflict);
        } else {
          // No conflict, can merge
          mergedOperations.push(this.mergeOperations(ourOp, theirOp));
        }
      } else {
        // Operation exists in all three - perform three-way merge
        const conflict = this.detectMergeConflict(ourOp, theirOp, baseOp);
        if (conflict) {
          conflicts.push(conflict);
        } else {
          mergedOperations.push(this.mergeOperations(ourOp, theirOp, baseOp));
        }
      }
    }

    // Process their operations that we haven't seen
    for (const theirOp of theirOps) {
      if (!processedIds.has(theirOp.id)) {
        const baseOp = baseOps.find(op => this.operationsMatch(op, theirOp));

        if (!baseOp) {
          // New operation only in theirs - take it
          mergedOperations.push(theirOp);
        }
      }
    }

    // Attempt auto-resolution
    const autoResolved = await this.autoResolveConflicts(conflicts);

    const success = conflicts.length === autoResolved;

    const result: MergeResult = {
      success,
      conflicts: conflicts.filter(c => !c.resolution),
      autoResolved,
      manualRequired: conflicts.length - autoResolved
    };

    if (success) {
      // Create merged commit
      result.mergedCommit = {
        id: this.generateCommitId(),
        branchId: context.ours.branchId,
        parentCommitId: context.ours.id,
        message: `Merge commit ${context.theirs.id} into ${context.ours.id}`,
        author: 'system',
        timestamp: Date.now(),
        operations: mergedOperations
      };
    }

    return result;
  }

  private operationsMatch(op1: Operation, op2: Operation): boolean {
    // Check if operations target the same object
    const id1 = (op1.data as any)?.objectId;
    const id2 = (op2.data as any)?.objectId;
    return id1 === id2 && id1 !== undefined;
  }

  private detectMergeConflict(
    ourOp: Operation,
    theirOp: Operation,
    baseOp: Operation | null
  ): Conflict | null {
    // If operations are identical, no conflict
    if (this.operationsEqual(ourOp, theirOp)) {
      return null;
    }

    // Check for conflicting changes
    if (baseOp) {
      // Three-way conflict detection
      const ourChanged = !this.operationsEqual(ourOp, baseOp);
      const theirChanged = !this.operationsEqual(theirOp, baseOp);

      if (ourChanged && theirChanged) {
        // Both changed the same thing differently - conflict!
        return this.createConflict(ourOp, theirOp, baseOp);
      }

      return null; // One or neither changed
    } else {
      // Two-way conflict (no base)
      return this.createConflict(ourOp, theirOp, null);
    }
  }

  private operationsEqual(op1: Operation, op2: Operation): boolean {
    // Deep equality check for operations
    return JSON.stringify(op1.data) === JSON.stringify(op2.data);
  }

  private createConflict(
    ourOp: Operation,
    theirOp: Operation,
    baseOp: Operation | null
  ): Conflict {
    return {
      id: this.generateConflictId(),
      type: this.getConflictType(ourOp, theirOp),
      path: (ourOp.data as any)?.path || 'unknown',
      objectId: (ourOp.data as any)?.objectId,
      ourValue: ourOp.data,
      theirValue: theirOp.data,
      baseValue: baseOp?.data
    };
  }

  private getConflictType(ourOp: Operation, theirOp: Operation): ConflictType {
    if (ourOp.type === 'delete' || theirOp.type === 'delete') {
      return ConflictType.DELETE_MODIFY;
    }
    if (ourOp.type === 'move' && theirOp.type === 'move') {
      return ConflictType.POSITION;
    }
    return ConflictType.CONTENT;
  }

  private mergeOperations(
    ourOp: Operation,
    theirOp: Operation,
    baseOp?: Operation
  ): Operation {
    // Merge operation data
    const mergedData = {
      ...(baseOp?.data || {}),
      ...ourOp.data,
      ...theirOp.data
    };

    return {
      ...ourOp,
      data: mergedData,
      timestamp: Math.max(ourOp.timestamp, theirOp.timestamp)
    };
  }

  private async autoResolveConflicts(conflicts: Conflict[]): Promise<number> {
    let resolved = 0;

    for (const conflict of conflicts) {
      if (await this.tryAutoResolve(conflict)) {
        resolved++;
      }
    }

    return resolved;
  }

  private async tryAutoResolve(conflict: Conflict): Promise<boolean> {
    // Try to automatically resolve conflict
    switch (conflict.type) {
      case ConflictType.POSITION:
        // Average positions
        conflict.resolution = {
          strategy: ResolutionStrategy.OPERATIONAL_TRANSFORM,
          value: this.averagePositions(conflict.ourValue, conflict.theirValue),
          resolvedBy: 'system',
          timestamp: Date.now()
        };
        return true;

      case ConflictType.CONTENT:
        // Try to merge object properties
        if (
          typeof conflict.ourValue === 'object' &&
          typeof conflict.theirValue === 'object'
        ) {
          conflict.resolution = {
            strategy: ResolutionStrategy.MERGE_BOTH,
            value: { ...conflict.ourValue, ...conflict.theirValue },
            resolvedBy: 'system',
            timestamp: Date.now()
          };
          return true;
        }
        return false;

      default:
        return false;
    }
  }

  private averagePositions(pos1: any, pos2: any): any {
    if (pos1?.x !== undefined && pos2?.x !== undefined) {
      return {
        x: (pos1.x + pos2.x) / 2,
        y: (pos1.y + pos2.y) / 2,
        z: pos1.z !== undefined && pos2.z !== undefined
          ? (pos1.z + pos2.z) / 2
          : pos1.z || pos2.z
      };
    }
    return pos2; // Fallback
  }

  // ============================================================================
  // Diff Operations
  // ============================================================================

  diff(commitA: Commit, commitB: Commit): Diff {
    const changes: DiffChange[] = [];

    // Create maps for quick lookup
    const opsA = new Map(commitA.operations.map(op => [(op.data as any)?.objectId, op]));
    const opsB = new Map(commitB.operations.map(op => [(op.data as any)?.objectId, op]));

    // Find added and modified
    for (const [objectId, opB] of opsB.entries()) {
      const opA = opsA.get(objectId);

      if (!opA) {
        // Added
        changes.push({
          type: 'added',
          path: (opB.data as any)?.path || objectId,
          objectId,
          afterValue: opB.data
        });
      } else if (!this.operationsEqual(opA, opB)) {
        // Modified
        changes.push({
          type: 'modified',
          path: (opB.data as any)?.path || objectId,
          objectId,
          beforeValue: opA.data,
          afterValue: opB.data
        });
      }
    }

    // Find deleted
    for (const [objectId, opA] of opsA.entries()) {
      if (!opsB.has(objectId)) {
        changes.push({
          type: 'deleted',
          path: (opA.data as any)?.path || objectId,
          objectId,
          beforeValue: opA.data
        });
      }
    }

    const statistics: DiffStatistics = {
      added: changes.filter(c => c.type === 'added').length,
      modified: changes.filter(c => c.type === 'modified').length,
      deleted: changes.filter(c => c.type === 'deleted').length,
      totalChanges: changes.length
    };

    return {
      commitA: commitA.id,
      commitB: commitB.id,
      changes,
      statistics
    };
  }

  // ============================================================================
  // Cherry-Pick
  // ============================================================================

  async cherryPick(commit: Commit, targetCommit: Commit): Promise<Commit> {
    // Apply specific commit to target
    const cherryPickedOps = [...targetCommit.operations, ...commit.operations];

    return {
      id: this.generateCommitId(),
      branchId: targetCommit.branchId,
      parentCommitId: targetCommit.id,
      message: `Cherry-pick ${commit.id}: ${commit.message}`,
      author: commit.author,
      timestamp: Date.now(),
      operations: cherryPickedOps
    };
  }

  // ============================================================================
  // Revert
  // ============================================================================

  async revert(commit: Commit): Promise<Commit> {
    // Create inverse operations
    const invertedOps = commit.operations.map(op => this.invertOperation(op));

    return {
      id: this.generateCommitId(),
      branchId: commit.branchId,
      parentCommitId: commit.id,
      message: `Revert "${commit.message}"`,
      author: 'system',
      timestamp: Date.now(),
      operations: invertedOps
    };
  }

  private invertOperation(op: Operation): Operation {
    // Create inverse operation
    const inverted: Operation = { ...op };

    switch (op.type) {
      case 'create':
        inverted.type = 'delete';
        break;
      case 'delete':
        inverted.type = 'create';
        break;
      case 'update':
        // Swap before/after values
        if (op.data && typeof op.data === 'object') {
          inverted.data = { ...op.data };
          // Would swap actual before/after values
        }
        break;
    }

    return inverted;
  }

  // ============================================================================
  // Squash
  // ============================================================================

  async squash(commits: Commit[]): Promise<Commit> {
    if (commits.length === 0) {
      throw new Error('No commits to squash');
    }

    // Combine all operations
    const allOperations = commits.flatMap(c => c.operations);

    // Deduplicate and merge operations
    const mergedOps = this.deduplicateOperations(allOperations);

    // Create combined commit message
    const message = commits.map(c => c.message).join('\n\n');

    return {
      id: this.generateCommitId(),
      branchId: commits[0].branchId,
      parentCommitId: commits[0].parentCommitId,
      message: `Squashed commits:\n\n${message}`,
      author: commits[0].author,
      timestamp: Date.now(),
      operations: mergedOps
    };
  }

  private deduplicateOperations(operations: Operation[]): Operation[] {
    const seen = new Map<string, Operation>();

    for (const op of operations) {
      const key = (op.data as any)?.objectId || op.id;
      const existing = seen.get(key);

      if (!existing || op.timestamp > existing.timestamp) {
        seen.set(key, op);
      }
    }

    return Array.from(seen.values());
  }

  // ============================================================================
  // Helpers
  // ============================================================================

  private generateCommitId(): CommitId {
    return `commit-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private generateConflictId(): string {
    return `conflict-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  // ============================================================================
  // History & Statistics
  // ============================================================================

  getMergeHistory(): MergeResult[] {
    return [...this.mergeHistory];
  }

  clearHistory(): void {
    this.mergeHistory = [];
    this.emit('historyCleared');
  }

  getStatistics() {
    const totalMerges = this.mergeHistory.length;
    const successfulMerges = this.mergeHistory.filter(m => m.success).length;
    const totalConflicts = this.mergeHistory.reduce((sum, m) => sum + m.conflicts.length, 0);
    const autoResolved = this.mergeHistory.reduce((sum, m) => sum + m.autoResolved, 0);

    return {
      totalMerges,
      successfulMerges,
      successRate: totalMerges > 0 ? successfulMerges / totalMerges : 0,
      totalConflicts,
      autoResolved,
      autoResolveRate: totalConflicts > 0 ? autoResolved / totalConflicts : 0
    };
  }
}
