/**
 * AccuScene Enterprise v0.3.0 - Version Control
 *
 * Git-like version control system for scenes with branching and merging
 */

import { EventEmitter } from 'events';
import {
  Branch,
  BranchId,
  Commit,
  CommitId,
  SessionId,
  UserId,
  Operation,
  SceneSnapshot
} from './types';
import { BranchManager } from './BranchManager';
import { MergeEngine } from './MergeEngine';
import crypto from 'crypto';

export class VersionControl extends EventEmitter {
  private sessionId: SessionId | null = null;
  private branches: Map<BranchId, Branch> = new Map();
  private commits: Map<CommitId, Commit> = new Map();
  private currentBranchId: BranchId = 'main';
  private snapshots: Map<CommitId, SceneSnapshot> = new Map();

  private branchManager: BranchManager;
  private mergeEngine: MergeEngine;

  // Commit staging
  private stagedOperations: Operation[] = [];
  private commitCounter = 0;

  constructor() {
    super();
    this.branchManager = new BranchManager();
    this.mergeEngine = new MergeEngine();
  }

  // ============================================================================
  // Initialization
  // ============================================================================

  async initialize(sessionId: SessionId): Promise<void> {
    this.sessionId = sessionId;

    // Create main branch with initial commit
    const initialCommit = await this.createInitialCommit();
    const mainBranch = this.createBranch('main', initialCommit.id);

    this.branches.set('main', mainBranch);
    this.commits.set(initialCommit.id, initialCommit);
    this.currentBranchId = 'main';

    this.emit('initialized', { sessionId, branch: mainBranch });
  }

  // ============================================================================
  // Branch Operations
  // ============================================================================

  createBranch(name: string, baseCommitId?: CommitId, parentBranchId?: BranchId): Branch {
    const branchId = this.generateBranchId(name);
    const currentBranch = this.branches.get(this.currentBranchId);

    const branch: Branch = {
      id: branchId,
      name,
      parentBranchId: parentBranchId || this.currentBranchId,
      baseCommitId: baseCommitId || currentBranch?.headCommitId || this.generateCommitId(),
      headCommitId: baseCommitId || currentBranch?.headCommitId || this.generateCommitId(),
      createdBy: 'system', // TODO: Get current user
      createdAt: Date.now(),
      protected: name === 'main'
    };

    this.branches.set(branchId, branch);
    this.emit('branchCreated', branch);

    return branch;
  }

  deleteBranch(branchId: BranchId, force = false): boolean {
    const branch = this.branches.get(branchId);

    if (!branch) {
      throw new Error(`Branch not found: ${branchId}`);
    }

    if (branch.protected && !force) {
      throw new Error(`Cannot delete protected branch: ${branchId}`);
    }

    if (branchId === this.currentBranchId) {
      throw new Error('Cannot delete current branch');
    }

    this.branches.delete(branchId);
    this.emit('branchDeleted', { branchId, name: branch.name });

    return true;
  }

  switchBranch(branchId: BranchId): void {
    const branch = this.branches.get(branchId);

    if (!branch) {
      throw new Error(`Branch not found: ${branchId}`);
    }

    // Check for uncommitted changes
    if (this.stagedOperations.length > 0) {
      throw new Error('Cannot switch branch with uncommitted changes');
    }

    const previousBranch = this.currentBranchId;
    this.currentBranchId = branchId;

    this.emit('branchSwitched', {
      from: previousBranch,
      to: branchId,
      branch
    });
  }

  getCurrentBranch(): Branch | null {
    return this.branches.get(this.currentBranchId) || null;
  }

  getBranch(branchId: BranchId): Branch | null {
    return this.branches.get(branchId) || null;
  }

  getAllBranches(): Branch[] {
    return Array.from(this.branches.values());
  }

  getBranchHistory(branchId: BranchId): Commit[] {
    const branch = this.branches.get(branchId);
    if (!branch) return [];

    const history: Commit[] = [];
    let currentCommitId: CommitId | undefined = branch.headCommitId;

    while (currentCommitId) {
      const commit = this.commits.get(currentCommitId);
      if (!commit) break;

      history.push(commit);
      currentCommitId = commit.parentCommitId;
    }

    return history;
  }

  // ============================================================================
  // Commit Operations
  // ============================================================================

  stageOperation(operation: Operation): void {
    this.stagedOperations.push(operation);
    this.emit('operationStaged', operation);
  }

  unstageOperation(operationId: string): void {
    const index = this.stagedOperations.findIndex(op => op.id === operationId);
    if (index !== -1) {
      const [operation] = this.stagedOperations.splice(index, 1);
      this.emit('operationUnstaged', operation);
    }
  }

  getStagedOperations(): Operation[] {
    return [...this.stagedOperations];
  }

  clearStaging(): void {
    this.stagedOperations = [];
    this.emit('stagingCleared');
  }

  async commit(message: string, author: UserId, snapshot?: any): Promise<Commit> {
    const branch = this.getCurrentBranch();
    if (!branch) {
      throw new Error('No current branch');
    }

    if (this.stagedOperations.length === 0) {
      throw new Error('No operations staged for commit');
    }

    const commitId = this.generateCommitId();

    const commit: Commit = {
      id: commitId,
      branchId: branch.id,
      parentCommitId: branch.headCommitId,
      message,
      author,
      timestamp: Date.now(),
      operations: [...this.stagedOperations],
      tags: []
    };

    // Create snapshot if provided
    if (snapshot) {
      const sceneSnapshot = await this.createSnapshot(commitId, snapshot);
      commit.snapshot = sceneSnapshot;
      this.snapshots.set(commitId, sceneSnapshot);
    }

    this.commits.set(commitId, commit);

    // Update branch head
    branch.headCommitId = commitId;
    this.branches.set(branch.id, branch);

    // Clear staging
    this.clearStaging();

    this.emit('committed', commit);

    return commit;
  }

  getCommit(commitId: CommitId): Commit | null {
    return this.commits.get(commitId) || null;
  }

  getCommitHistory(limit = 100): Commit[] {
    const branch = this.getCurrentBranch();
    if (!branch) return [];

    return this.getBranchHistory(branch.id).slice(0, limit);
  }

  tagCommit(commitId: CommitId, tag: string): void {
    const commit = this.commits.get(commitId);
    if (!commit) {
      throw new Error(`Commit not found: ${commitId}`);
    }

    if (!commit.tags) {
      commit.tags = [];
    }

    if (!commit.tags.includes(tag)) {
      commit.tags.push(tag);
      this.commits.set(commitId, commit);
      this.emit('commitTagged', { commitId, tag });
    }
  }

  // ============================================================================
  // Merge Operations
  // ============================================================================

  async merge(sourceBranchId: BranchId, targetBranchId?: BranchId): Promise<any> {
    const source = this.branches.get(sourceBranchId);
    const target = this.branches.get(targetBranchId || this.currentBranchId);

    if (!source || !target) {
      throw new Error('Source or target branch not found');
    }

    // Get commits to merge
    const sourceCommits = this.getBranchHistory(source.id);
    const targetCommits = this.getBranchHistory(target.id);

    // Find common ancestor
    const commonAncestor = this.findCommonAncestor(sourceCommits, targetCommits);

    // Perform merge
    const mergeResult = await this.mergeEngine.merge(
      sourceCommits,
      targetCommits,
      commonAncestor
    );

    if (mergeResult.success && mergeResult.mergedCommit) {
      // Create merge commit
      const mergeCommit: Commit = {
        ...mergeResult.mergedCommit,
        branchId: target.id,
        parentCommitId: target.headCommitId,
        message: `Merge branch '${source.name}' into '${target.name}'`,
        timestamp: Date.now()
      };

      this.commits.set(mergeCommit.id, mergeCommit);

      // Update target branch
      target.headCommitId = mergeCommit.id;
      this.branches.set(target.id, target);

      this.emit('merged', {
        source: source.id,
        target: target.id,
        commit: mergeCommit,
        result: mergeResult
      });
    }

    return mergeResult;
  }

  // ============================================================================
  // Diff Operations
  // ============================================================================

  async diff(commitA: CommitId, commitB: CommitId): Promise<any> {
    const commitAObj = this.commits.get(commitA);
    const commitBObj = this.commits.get(commitB);

    if (!commitAObj || !commitBObj) {
      throw new Error('One or both commits not found');
    }

    return this.mergeEngine.diff(commitAObj, commitBObj);
  }

  // ============================================================================
  // Snapshot Operations
  // ============================================================================

  async createSnapshot(commitId: CommitId, sceneData: any): Promise<SceneSnapshot> {
    const serialized = JSON.stringify(sceneData);
    const checksum = this.calculateChecksum(serialized);

    const snapshot: SceneSnapshot = {
      sceneId: this.sessionId!,
      commitId,
      data: sceneData,
      checksum,
      compressed: false
    };

    return snapshot;
  }

  getSnapshot(commitId: CommitId): SceneSnapshot | null {
    return this.snapshots.get(commitId) || null;
  }

  async restoreSnapshot(commitId: CommitId): Promise<any> {
    const snapshot = this.snapshots.get(commitId);
    if (!snapshot) {
      throw new Error(`Snapshot not found for commit: ${commitId}`);
    }

    // Verify checksum
    const serialized = JSON.stringify(snapshot.data);
    const checksum = this.calculateChecksum(serialized);

    if (checksum !== snapshot.checksum) {
      throw new Error('Snapshot checksum mismatch - data may be corrupted');
    }

    this.emit('snapshotRestored', { commitId, snapshot });

    return snapshot.data;
  }

  // ============================================================================
  // Rebase Operations
  // ============================================================================

  async rebase(targetBranchId: BranchId): Promise<void> {
    const currentBranch = this.getCurrentBranch();
    const targetBranch = this.branches.get(targetBranchId);

    if (!currentBranch || !targetBranch) {
      throw new Error('Branch not found');
    }

    if (currentBranch.id === targetBranch.id) {
      throw new Error('Cannot rebase onto same branch');
    }

    // Get commits to rebase
    const currentCommits = this.getBranchHistory(currentBranch.id);
    const targetCommits = this.getBranchHistory(targetBranch.id);

    // Find common ancestor
    const commonAncestor = this.findCommonAncestor(currentCommits, targetCommits);

    // Get commits to replay (those after common ancestor)
    const commitsToReplay = currentCommits.filter(c =>
      !targetCommits.find(tc => tc.id === c.id) &&
      c.timestamp > (commonAncestor?.timestamp || 0)
    );

    // Replay commits on top of target
    let newHead = targetBranch.headCommitId;

    for (const commit of commitsToReplay.reverse()) {
      const rebasedCommit: Commit = {
        ...commit,
        id: this.generateCommitId(),
        parentCommitId: newHead,
        timestamp: Date.now()
      };

      this.commits.set(rebasedCommit.id, rebasedCommit);
      newHead = rebasedCommit.id;
    }

    // Update current branch head
    currentBranch.headCommitId = newHead;
    currentBranch.baseCommitId = targetBranch.headCommitId;
    this.branches.set(currentBranch.id, currentBranch);

    this.emit('rebased', {
      branch: currentBranch.id,
      onto: targetBranch.id,
      commitsReplayed: commitsToReplay.length
    });
  }

  // ============================================================================
  // Reset Operations
  // ============================================================================

  async reset(commitId: CommitId, hard = false): Promise<void> {
    const commit = this.commits.get(commitId);
    if (!commit) {
      throw new Error(`Commit not found: ${commitId}`);
    }

    const branch = this.getCurrentBranch();
    if (!branch) {
      throw new Error('No current branch');
    }

    if (hard) {
      // Hard reset: discard all changes
      this.clearStaging();
      branch.headCommitId = commitId;
      this.branches.set(branch.id, branch);
    } else {
      // Soft reset: keep changes in staging
      branch.headCommitId = commitId;
      this.branches.set(branch.id, branch);
    }

    this.emit('reset', { commitId, hard });
  }

  // ============================================================================
  // Private Helpers
  // ============================================================================

  private async createInitialCommit(): Promise<Commit> {
    const commitId = this.generateCommitId();

    return {
      id: commitId,
      branchId: 'main',
      parentCommitId: undefined,
      message: 'Initial commit',
      author: 'system',
      timestamp: Date.now(),
      operations: [],
      tags: ['initial']
    };
  }

  private findCommonAncestor(commitsA: Commit[], commitsB: Commit[]): Commit | null {
    const setB = new Set(commitsB.map(c => c.id));

    for (const commit of commitsA) {
      if (setB.has(commit.id)) {
        return commit;
      }
    }

    return null;
  }

  private generateBranchId(name: string): BranchId {
    return `branch-${name}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private generateCommitId(): CommitId {
    this.commitCounter++;
    const hash = crypto.createHash('sha1');
    hash.update(`${Date.now()}-${this.commitCounter}-${Math.random()}`);
    return hash.digest('hex').substring(0, 12);
  }

  private calculateChecksum(data: string): string {
    return crypto.createHash('sha256').update(data).digest('hex');
  }

  // ============================================================================
  // Statistics
  // ============================================================================

  getStatistics() {
    return {
      branches: this.branches.size,
      commits: this.commits.size,
      snapshots: this.snapshots.size,
      stagedOperations: this.stagedOperations.length,
      currentBranch: this.currentBranchId
    };
  }

  getBranchTree(): any {
    const tree: any = {};

    for (const branch of this.branches.values()) {
      tree[branch.id] = {
        name: branch.name,
        parent: branch.parentBranchId,
        commits: this.getBranchHistory(branch.id).length,
        head: branch.headCommitId
      };
    }

    return tree;
  }
}
