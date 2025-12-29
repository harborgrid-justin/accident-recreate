/**
 * Conflict Resolver
 * Handles conflicts between local and remote data
 */

import {
  Conflict,
  ConflictResolution,
  ResolutionResult,
  Version,
} from './types';

export class ConflictResolver {
  private strategy: ConflictResolution;
  private customResolvers: Map<
    string,
    (conflict: Conflict) => Promise<ResolutionResult>
  > = new Map();

  constructor(strategy: ConflictResolution = ConflictResolution.LastWriteWins) {
    this.strategy = strategy;
  }

  /**
   * Register a custom resolver for entity type
   */
  registerCustomResolver(
    entityType: string,
    resolver: (conflict: Conflict) => Promise<ResolutionResult>
  ): void {
    this.customResolvers.set(entityType, resolver);
  }

  /**
   * Resolve a conflict
   */
  async resolve(conflict: Conflict): Promise<ResolutionResult> {
    // Check for custom resolver
    const customResolver = this.customResolvers.get(conflict.entityType);
    if (customResolver) {
      return customResolver(conflict);
    }

    // Use strategy
    return this.resolveWithStrategy(conflict, this.strategy);
  }

  /**
   * Resolve using specific strategy
   */
  async resolveWithStrategy(
    conflict: Conflict,
    strategy: ConflictResolution
  ): Promise<ResolutionResult> {
    switch (strategy) {
      case ConflictResolution.LastWriteWins:
        return this.lastWriteWins(conflict);

      case ConflictResolution.FirstWriteWins:
        return this.firstWriteWins(conflict);

      case ConflictResolution.ServerWins:
        return this.serverWins(conflict);

      case ConflictResolution.ClientWins:
        return this.clientWins(conflict);

      case ConflictResolution.OperationalTransform:
        return this.operationalTransform(conflict);

      case ConflictResolution.Manual:
        throw new Error('Manual resolution required');

      default:
        return this.lastWriteWins(conflict);
    }
  }

  /**
   * Last write wins - choose version with latest timestamp
   */
  private lastWriteWins(conflict: Conflict): ResolutionResult {
    const localTime = new Date(conflict.localVersion.timestamp).getTime();
    const remoteTime = new Date(conflict.remoteVersion.timestamp).getTime();

    const useLocal = localTime > remoteTime;

    return {
      data: useLocal ? conflict.localData : conflict.remoteData,
      version: useLocal ? conflict.localVersion : conflict.remoteVersion,
      strategy: ConflictResolution.LastWriteWins,
      manual: false,
      metadata: {
        localTimestamp: conflict.localVersion.timestamp,
        remoteTimestamp: conflict.remoteVersion.timestamp,
      },
    };
  }

  /**
   * First write wins - choose version with earliest timestamp
   */
  private firstWriteWins(conflict: Conflict): ResolutionResult {
    const localTime = new Date(conflict.localVersion.timestamp).getTime();
    const remoteTime = new Date(conflict.remoteVersion.timestamp).getTime();

    const useLocal = localTime < remoteTime;

    return {
      data: useLocal ? conflict.localData : conflict.remoteData,
      version: useLocal ? conflict.localVersion : conflict.remoteVersion,
      strategy: ConflictResolution.FirstWriteWins,
      manual: false,
      metadata: {},
    };
  }

  /**
   * Server wins - always choose remote version
   */
  private serverWins(conflict: Conflict): ResolutionResult {
    return {
      data: conflict.remoteData,
      version: conflict.remoteVersion,
      strategy: ConflictResolution.ServerWins,
      manual: false,
      metadata: {},
    };
  }

  /**
   * Client wins - always choose local version
   */
  private clientWins(conflict: Conflict): ResolutionResult {
    return {
      data: conflict.localData,
      version: conflict.localVersion,
      strategy: ConflictResolution.ClientWins,
      manual: false,
      metadata: {},
    };
  }

  /**
   * Operational transform - merge changes
   */
  private operationalTransform(conflict: Conflict): ResolutionResult {
    // Check if versions are concurrent
    if (this.isConcurrent(conflict.localVersion, conflict.remoteVersion)) {
      // Attempt to merge
      const merged = this.mergeObjects(conflict.localData, conflict.remoteData);

      // Create merged version
      const mergedVersion: Version = {
        clock: this.mergeVectorClocks(
          conflict.localVersion.clock,
          conflict.remoteVersion.clock
        ),
        nodeId: conflict.localVersion.nodeId,
        timestamp: new Date().toISOString(),
        contentHash: this.hashData(merged),
      };

      return {
        data: merged,
        version: mergedVersion,
        strategy: ConflictResolution.OperationalTransform,
        manual: false,
        metadata: {
          mergedFrom: [
            conflict.localVersion.contentHash,
            conflict.remoteVersion.contentHash,
          ],
        },
      };
    }

    // Not concurrent, use last write wins
    return this.lastWriteWins(conflict);
  }

  /**
   * Check if two versions are concurrent
   */
  private isConcurrent(v1: Version, v2: Version): boolean {
    // Simple concurrent check - can be improved with full vector clock comparison
    const v1Nodes = Object.keys(v1.clock.clocks);
    const v2Nodes = Object.keys(v2.clock.clocks);

    let v1Greater = false;
    let v2Greater = false;

    const allNodes = [...new Set([...v1Nodes, ...v2Nodes])];

    for (const node of allNodes) {
      const v1Clock = v1.clock.clocks[node] || 0;
      const v2Clock = v2.clock.clocks[node] || 0;

      if (v1Clock > v2Clock) v1Greater = true;
      if (v2Clock > v1Clock) v2Greater = true;
    }

    return v1Greater && v2Greater;
  }

  /**
   * Merge two objects
   */
  private mergeObjects(local: any, remote: any): any {
    if (typeof local !== 'object' || typeof remote !== 'object') {
      return remote; // Primitive value, use remote
    }

    if (Array.isArray(local) && Array.isArray(remote)) {
      // For arrays, combine and deduplicate
      return [...new Set([...local, ...remote])];
    }

    // Merge objects
    const merged: any = { ...local };

    for (const key in remote) {
      if (!(key in local)) {
        // Added in remote only
        merged[key] = remote[key];
      } else if (local[key] !== remote[key]) {
        // Both have different values, recursively merge
        merged[key] = this.mergeObjects(local[key], remote[key]);
      }
    }

    return merged;
  }

  /**
   * Merge two vector clocks
   */
  private mergeVectorClocks(c1: any, c2: any): any {
    const merged: any = { clocks: { ...c1.clocks } };

    for (const node in c2.clocks) {
      const v1 = merged.clocks[node] || 0;
      const v2 = c2.clocks[node];
      merged.clocks[node] = Math.max(v1, v2);
    }

    return merged;
  }

  /**
   * Hash data for content addressing
   */
  private hashData(data: any): string {
    const str = JSON.stringify(data);
    // Simple hash (in production, use crypto.subtle)
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }
    return Math.abs(hash).toString(16);
  }

  /**
   * Set default strategy
   */
  setStrategy(strategy: ConflictResolution): void {
    this.strategy = strategy;
  }

  /**
   * Get current strategy
   */
  getStrategy(): ConflictResolution {
    return this.strategy;
  }
}
