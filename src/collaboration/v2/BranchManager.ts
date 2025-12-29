/**
 * AccuScene Enterprise v0.3.0 - Branch Manager
 *
 * Scene branching with visual diff and branch management
 */

import { EventEmitter } from 'events';
import {
  Branch,
  BranchId,
  Commit,
  CommitId,
  Diff,
  DiffChange,
  DiffStatistics
} from './types';

interface BranchNode {
  branch: Branch;
  children: BranchNode[];
  depth: number;
}

export class BranchManager extends EventEmitter {
  private branches: Map<BranchId, Branch> = new Map();
  private branchTree: BranchNode | null = null;

  constructor() {
    super();
  }

  // ============================================================================
  // Branch Visualization
  // ============================================================================

  buildBranchTree(rootBranchId: BranchId): BranchNode | null {
    const rootBranch = this.branches.get(rootBranchId);
    if (!rootBranch) return null;

    const buildNode = (branch: Branch, depth = 0): BranchNode => {
      const children = Array.from(this.branches.values())
        .filter(b => b.parentBranchId === branch.id)
        .map(b => buildNode(b, depth + 1));

      return {
        branch,
        children,
        depth
      };
    };

    this.branchTree = buildNode(rootBranch);
    return this.branchTree;
  }

  getBranchTree(): BranchNode | null {
    return this.branchTree;
  }

  getBranchPath(branchId: BranchId): Branch[] {
    const path: Branch[] = [];
    let currentBranchId: BranchId | undefined = branchId;

    while (currentBranchId) {
      const branch = this.branches.get(currentBranchId);
      if (!branch) break;

      path.unshift(branch);
      currentBranchId = branch.parentBranchId;
    }

    return path;
  }

  getChildBranches(branchId: BranchId): Branch[] {
    return Array.from(this.branches.values())
      .filter(b => b.parentBranchId === branchId);
  }

  getSiblingBranches(branchId: BranchId): Branch[] {
    const branch = this.branches.get(branchId);
    if (!branch || !branch.parentBranchId) return [];

    return Array.from(this.branches.values())
      .filter(b => b.parentBranchId === branch.parentBranchId && b.id !== branchId);
  }

  // ============================================================================
  // Branch Comparison
  // ============================================================================

  compareBranches(branchIdA: BranchId, branchIdB: BranchId): {
    divergedAt: CommitId | null;
    commitsAhead: number;
    commitsBehind: number;
    canFastForward: boolean;
  } {
    const branchA = this.branches.get(branchIdA);
    const branchB = this.branches.get(branchIdB);

    if (!branchA || !branchB) {
      throw new Error('One or both branches not found');
    }

    // Find divergence point
    const pathA = new Set([branchA.headCommitId, branchA.baseCommitId]);
    const pathB = new Set([branchB.headCommitId, branchB.baseCommitId]);

    let divergedAt: CommitId | null = null;
    for (const commitId of pathA) {
      if (pathB.has(commitId)) {
        divergedAt = commitId;
        break;
      }
    }

    // Calculate commits ahead/behind
    // This is simplified - real implementation would traverse commit history
    const commitsAhead = pathA.size;
    const commitsBehind = pathB.size;

    // Check if can fast-forward
    const canFastForward = pathA.has(branchB.headCommitId);

    return {
      divergedAt,
      commitsAhead,
      commitsBehind,
      canFastForward
    };
  }

  // ============================================================================
  // Visual Diff
  // ============================================================================

  createVisualDiff(diff: Diff): {
    summary: string;
    changesByType: Map<string, DiffChange[]>;
    affectedObjects: Set<string>;
    visualization: any;
  } {
    const changesByType = new Map<string, DiffChange[]>();
    const affectedObjects = new Set<string>();

    // Group changes by type
    for (const change of diff.changes) {
      const type = change.type;
      if (!changesByType.has(type)) {
        changesByType.set(type, []);
      }
      changesByType.get(type)!.push(change);

      if (change.objectId) {
        affectedObjects.add(change.objectId);
      }
    }

    // Create summary
    const summary = this.createDiffSummary(diff.statistics);

    // Create visualization data
    const visualization = this.createDiffVisualization(diff);

    return {
      summary,
      changesByType,
      affectedObjects,
      visualization
    };
  }

  private createDiffSummary(stats: DiffStatistics): string {
    const parts: string[] = [];

    if (stats.added > 0) {
      parts.push(`${stats.added} added`);
    }
    if (stats.modified > 0) {
      parts.push(`${stats.modified} modified`);
    }
    if (stats.deleted > 0) {
      parts.push(`${stats.deleted} deleted`);
    }

    return parts.join(', ') || 'No changes';
  }

  private createDiffVisualization(diff: Diff): any {
    // Create visualization data for UI
    return {
      nodes: diff.changes.map((change, index) => ({
        id: `change-${index}`,
        type: change.type,
        label: this.getChangeLabel(change),
        data: change
      })),
      edges: [], // Could show relationships between changes
      statistics: diff.statistics
    };
  }

  private getChangeLabel(change: DiffChange): string {
    const objectId = change.objectId || 'unknown';
    return `${change.type}: ${objectId}`;
  }

  // ============================================================================
  // Branch Statistics
  // ============================================================================

  getBranchStatistics(branchId: BranchId): {
    commitCount: number;
    age: number;
    lastActivity: number;
    contributors: Set<string>;
    isStale: boolean;
  } | null {
    const branch = this.branches.get(branchId);
    if (!branch) return null;

    const now = Date.now();
    const age = now - branch.createdAt;
    const lastActivity = branch.createdAt; // Would be updated with actual activity

    // Calculate staleness (no activity in 7 days)
    const STALE_THRESHOLD = 7 * 24 * 60 * 60 * 1000;
    const isStale = (now - lastActivity) > STALE_THRESHOLD;

    return {
      commitCount: 0, // Would count actual commits
      age,
      lastActivity,
      contributors: new Set([branch.createdBy]),
      isStale
    };
  }

  getStaleBranches(thresholdDays = 7): Branch[] {
    const threshold = thresholdDays * 24 * 60 * 60 * 1000;
    const now = Date.now();

    return Array.from(this.branches.values()).filter(branch => {
      const stats = this.getBranchStatistics(branch.id);
      return stats?.isStale || (now - branch.createdAt) > threshold;
    });
  }

  // ============================================================================
  // Branch Cleanup
  // ============================================================================

  async cleanupMergedBranches(): Promise<BranchId[]> {
    const deleted: BranchId[] = [];

    // Find branches that have been merged
    for (const branch of this.branches.values()) {
      if (this.isBranchMerged(branch)) {
        if (!branch.protected) {
          this.branches.delete(branch.id);
          deleted.push(branch.id);
          this.emit('branchDeleted', branch);
        }
      }
    }

    return deleted;
  }

  private isBranchMerged(branch: Branch): boolean {
    // Check if branch has been merged into parent
    // This is simplified - real implementation would check commit history
    return false;
  }

  async pruneStaleBranches(thresholdDays = 30): Promise<BranchId[]> {
    const staleBranches = this.getStaleBranches(thresholdDays);
    const deleted: BranchId[] = [];

    for (const branch of staleBranches) {
      if (!branch.protected) {
        this.branches.delete(branch.id);
        deleted.push(branch.id);
        this.emit('branchPruned', branch);
      }
    }

    return deleted;
  }

  // ============================================================================
  // Branch Protection
  // ============================================================================

  protectBranch(branchId: BranchId): void {
    const branch = this.branches.get(branchId);
    if (branch) {
      branch.protected = true;
      this.branches.set(branchId, branch);
      this.emit('branchProtected', branch);
    }
  }

  unprotectBranch(branchId: BranchId): void {
    const branch = this.branches.get(branchId);
    if (branch) {
      branch.protected = false;
      this.branches.set(branchId, branch);
      this.emit('branchUnprotected', branch);
    }
  }

  isProtected(branchId: BranchId): boolean {
    const branch = this.branches.get(branchId);
    return branch?.protected || false;
  }

  // ============================================================================
  // Branch Naming
  // ============================================================================

  suggestBranchName(purpose: string, userId: string): string {
    const timestamp = new Date().toISOString().split('T')[0];
    const baseName = `${purpose}-${userId}-${timestamp}`;

    // Ensure unique name
    let counter = 1;
    let name = baseName;

    while (this.branchNameExists(name)) {
      name = `${baseName}-${counter}`;
      counter++;
    }

    return name;
  }

  private branchNameExists(name: string): boolean {
    return Array.from(this.branches.values()).some(b => b.name === name);
  }

  renameBranch(branchId: BranchId, newName: string): void {
    const branch = this.branches.get(branchId);
    if (!branch) {
      throw new Error(`Branch not found: ${branchId}`);
    }

    if (branch.protected) {
      throw new Error('Cannot rename protected branch');
    }

    if (this.branchNameExists(newName)) {
      throw new Error(`Branch name already exists: ${newName}`);
    }

    const oldName = branch.name;
    branch.name = newName;
    this.branches.set(branchId, branch);

    this.emit('branchRenamed', {
      branchId,
      oldName,
      newName
    });
  }

  // ============================================================================
  // Branch Search
  // ============================================================================

  searchBranches(query: string): Branch[] {
    const lowerQuery = query.toLowerCase();

    return Array.from(this.branches.values()).filter(branch =>
      branch.name.toLowerCase().includes(lowerQuery) ||
      branch.id.toLowerCase().includes(lowerQuery) ||
      branch.createdBy.toLowerCase().includes(lowerQuery)
    );
  }

  filterBranches(filter: {
    protected?: boolean;
    stale?: boolean;
    createdBy?: string;
    createdAfter?: number;
    createdBefore?: number;
  }): Branch[] {
    return Array.from(this.branches.values()).filter(branch => {
      if (filter.protected !== undefined && branch.protected !== filter.protected) {
        return false;
      }

      if (filter.createdBy && branch.createdBy !== filter.createdBy) {
        return false;
      }

      if (filter.createdAfter && branch.createdAt < filter.createdAfter) {
        return false;
      }

      if (filter.createdBefore && branch.createdAt > filter.createdBefore) {
        return false;
      }

      if (filter.stale !== undefined) {
        const stats = this.getBranchStatistics(branch.id);
        if (stats && stats.isStale !== filter.stale) {
          return false;
        }
      }

      return true;
    });
  }

  // ============================================================================
  // Branch Registry
  // ============================================================================

  registerBranch(branch: Branch): void {
    this.branches.set(branch.id, branch);
    this.emit('branchRegistered', branch);
  }

  unregisterBranch(branchId: BranchId): void {
    const branch = this.branches.get(branchId);
    if (branch) {
      this.branches.delete(branchId);
      this.emit('branchUnregistered', branch);
    }
  }

  getAllBranches(): Branch[] {
    return Array.from(this.branches.values());
  }

  getBranchCount(): number {
    return this.branches.size;
  }
}
