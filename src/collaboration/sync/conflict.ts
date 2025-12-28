/**
 * AccuScene Enterprise - Conflict Resolution
 * v0.2.0
 *
 * Conflict detection and resolution for concurrent operations
 */

import {
  Operation,
  ConflictResolver,
  ConflictResult,
  ConflictResolutionStrategy,
  ClientId,
} from '../types';

/**
 * Default Last-Write-Wins conflict resolver
 */
export class LWWConflictResolver implements ConflictResolver {
  resolve(op1: Operation, op2: Operation): Operation {
    // Compare timestamps
    if (op1.timestamp.counter !== op2.timestamp.counter) {
      return op1.timestamp.counter > op2.timestamp.counter ? op1 : op2;
    }

    // Tie-break using client ID for deterministic ordering
    return op1.clientId > op2.clientId ? op1 : op2;
  }
}

/**
 * First-Write-Wins conflict resolver
 */
export class FWWConflictResolver implements ConflictResolver {
  resolve(op1: Operation, op2: Operation): Operation {
    // Compare timestamps
    if (op1.timestamp.counter !== op2.timestamp.counter) {
      return op1.timestamp.counter < op2.timestamp.counter ? op1 : op2;
    }

    // Tie-break using client ID for deterministic ordering
    return op1.clientId < op2.clientId ? op1 : op2;
  }
}

/**
 * Custom conflict resolver with user-defined logic
 */
export class CustomConflictResolver implements ConflictResolver {
  constructor(
    private resolveFn: (op1: Operation, op2: Operation) => Operation
  ) {}

  resolve(op1: Operation, op2: Operation): Operation {
    return this.resolveFn(op1, op2);
  }
}

/**
 * Merge conflict resolver that combines operations
 */
export class MergeConflictResolver implements ConflictResolver {
  resolve(op1: Operation, op2: Operation): Operation {
    // Create a merged operation
    const merged: Operation = {
      ...op1,
      id: `${op1.id}+${op2.id}`,
      data: this.mergeData(op1.data, op2.data),
      transformedAgainst: [
        ...(op1.transformedAgainst || []),
        ...(op2.transformedAgainst || []),
        op2.id,
      ],
    };

    return merged;
  }

  private mergeData(data1: unknown, data2: unknown): unknown {
    if (typeof data1 === 'object' && typeof data2 === 'object' && data1 && data2) {
      return { ...data1, ...data2 };
    }
    return data1;
  }
}

/**
 * Conflict Resolution Manager
 */
export class ConflictResolutionManager {
  private resolvers: Map<ConflictResolutionStrategy, ConflictResolver>;
  private defaultStrategy: ConflictResolutionStrategy;

  constructor(defaultStrategy: ConflictResolutionStrategy = ConflictResolutionStrategy.LAST_WRITE_WINS) {
    this.defaultStrategy = defaultStrategy;
    this.resolvers = new Map([
      [ConflictResolutionStrategy.LAST_WRITE_WINS, new LWWConflictResolver()],
      [ConflictResolutionStrategy.FIRST_WRITE_WINS, new FWWConflictResolver()],
      [ConflictResolutionStrategy.MERGE, new MergeConflictResolver()],
    ]);
  }

  /**
   * Set a custom resolver for a strategy
   */
  setResolver(strategy: ConflictResolutionStrategy, resolver: ConflictResolver): void {
    this.resolvers.set(strategy, resolver);
  }

  /**
   * Resolve a conflict between two operations
   */
  resolve(
    op1: Operation,
    op2: Operation,
    strategy?: ConflictResolutionStrategy
  ): ConflictResult {
    const resolveStrategy = strategy || this.defaultStrategy;
    const resolver = this.resolvers.get(resolveStrategy);

    if (!resolver) {
      throw new Error(`No resolver found for strategy: ${resolveStrategy}`);
    }

    const resolved = resolver.resolve(op1, op2);
    const discarded = resolved === op1 ? [op2] : [op1];

    return {
      resolved,
      discarded,
      strategy: resolveStrategy,
    };
  }

  /**
   * Detect conflicts in a set of operations
   */
  detectConflicts(operations: Operation[]): Array<[Operation, Operation]> {
    const conflicts: Array<[Operation, Operation]> = [];
    const byTarget = new Map<string, Operation[]>();

    // Group operations by target
    for (const op of operations) {
      const target = this.getOperationTarget(op);
      if (!byTarget.has(target)) {
        byTarget.set(target, []);
      }
      byTarget.get(target)!.push(op);
    }

    // Find concurrent operations on the same target
    for (const ops of byTarget.values()) {
      for (let i = 0; i < ops.length; i++) {
        for (let j = i + 1; j < ops.length; j++) {
          if (this.areConcurrent(ops[i], ops[j])) {
            conflicts.push([ops[i], ops[j]]);
          }
        }
      }
    }

    return conflicts;
  }

  /**
   * Resolve all conflicts in a set of operations
   */
  resolveAll(
    operations: Operation[],
    strategy?: ConflictResolutionStrategy
  ): Operation[] {
    const conflicts = this.detectConflicts(operations);
    const discardedIds = new Set<string>();

    // Resolve each conflict
    for (const [op1, op2] of conflicts) {
      if (!discardedIds.has(op1.id) && !discardedIds.has(op2.id)) {
        const result = this.resolve(op1, op2, strategy);
        for (const discarded of result.discarded) {
          discardedIds.add(discarded.id);
        }
      }
    }

    // Filter out discarded operations
    return operations.filter(op => !discardedIds.has(op.id));
  }

  /**
   * Get the target of an operation (what it modifies)
   */
  private getOperationTarget(op: Operation): string {
    const data = op.data as any;

    if (data.objectId) return `object:${data.objectId}`;
    if (data.annotationId) return `annotation:${data.annotationId}`;
    if (data.measurementId) return `measurement:${data.measurementId}`;
    if (data.sceneId) return `scene:${data.sceneId}`;

    return `unknown:${op.id}`;
  }

  /**
   * Check if two operations are concurrent
   */
  private areConcurrent(op1: Operation, op2: Operation): boolean {
    // Operations from the same client are never concurrent
    if (op1.clientId === op2.clientId) {
      return false;
    }

    // Check vector clocks
    const op1Counter = op1.vectorClock[op2.clientId] || 0;
    const op2Counter = op2.vectorClock[op1.clientId] || 0;

    // If op1 knows about op2's client, they're not concurrent
    if (op1Counter >= op2.timestamp.counter) {
      return false;
    }

    // If op2 knows about op1's client, they're not concurrent
    if (op2Counter >= op1.timestamp.counter) {
      return false;
    }

    // They are concurrent
    return true;
  }

  /**
   * Merge multiple conflicting operations
   */
  mergeConflicts(operations: Operation[]): Operation {
    if (operations.length === 0) {
      throw new Error('Cannot merge empty operations array');
    }

    if (operations.length === 1) {
      return operations[0];
    }

    // Start with the first operation
    let merged = operations[0];

    // Merge with each subsequent operation
    for (let i = 1; i < operations.length; i++) {
      const result = this.resolve(merged, operations[i], ConflictResolutionStrategy.MERGE);
      merged = result.resolved;
    }

    return merged;
  }

  /**
   * Check if an operation conflicts with any in a set
   */
  hasConflict(operation: Operation, operations: Operation[]): boolean {
    for (const other of operations) {
      if (
        this.areConcurrent(operation, other) &&
        this.getOperationTarget(operation) === this.getOperationTarget(other)
      ) {
        return true;
      }
    }
    return false;
  }

  /**
   * Find all operations that conflict with a given operation
   */
  findConflicts(operation: Operation, operations: Operation[]): Operation[] {
    const conflicts: Operation[] = [];
    const target = this.getOperationTarget(operation);

    for (const other of operations) {
      if (
        other.id !== operation.id &&
        this.areConcurrent(operation, other) &&
        this.getOperationTarget(other) === target
      ) {
        conflicts.push(other);
      }
    }

    return conflicts;
  }
}
