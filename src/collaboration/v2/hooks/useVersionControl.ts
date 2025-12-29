/**
 * AccuScene Enterprise v0.3.0 - useVersionControl Hook
 *
 * React hook for version control features
 */

import { useState, useEffect, useCallback } from 'react';
import { VersionControl } from '../VersionControl';
import {
  Branch,
  BranchId,
  Commit,
  CommitId,
  Operation,
  UserId
} from '../types';

export const useVersionControl = (versionControl: VersionControl | undefined) => {
  const [currentBranch, setCurrentBranch] = useState<Branch | null>(null);
  const [branches, setBranches] = useState<Branch[]>([]);
  const [commits, setCommits] = useState<Commit[]>([]);
  const [stagedOperations, setStagedOperations] = useState<Operation[]>([]);

  useEffect(() => {
    if (!versionControl) return;

    const updateState = () => {
      setCurrentBranch(versionControl.getCurrentBranch());
      setBranches(versionControl.getAllBranches());
      setCommits(versionControl.getCommitHistory(50));
      setStagedOperations(versionControl.getStagedOperations());
    };

    // Initial load
    updateState();

    // Listen for updates
    versionControl.on('branchCreated', updateState);
    versionControl.on('branchSwitched', updateState);
    versionControl.on('committed', updateState);
    versionControl.on('operationStaged', updateState);
    versionControl.on('operationUnstaged', updateState);

    return () => {
      versionControl.off('branchCreated', updateState);
      versionControl.off('branchSwitched', updateState);
      versionControl.off('committed', updateState);
      versionControl.off('operationStaged', updateState);
      versionControl.off('operationUnstaged', updateState);
    };
  }, [versionControl]);

  const createBranch = useCallback((name: string, baseCommitId?: CommitId) => {
    if (!versionControl) return null;
    return versionControl.createBranch(name, baseCommitId);
  }, [versionControl]);

  const switchBranch = useCallback((branchId: BranchId) => {
    if (!versionControl) return;
    versionControl.switchBranch(branchId);
  }, [versionControl]);

  const stageOperation = useCallback((operation: Operation) => {
    if (!versionControl) return;
    versionControl.stageOperation(operation);
  }, [versionControl]);

  const commit = useCallback(async (message: string, author: UserId, snapshot?: any) => {
    if (!versionControl) return null;
    return await versionControl.commit(message, author, snapshot);
  }, [versionControl]);

  const merge = useCallback(async (sourceBranchId: BranchId, targetBranchId?: BranchId) => {
    if (!versionControl) return null;
    return await versionControl.merge(sourceBranchId, targetBranchId);
  }, [versionControl]);

  return {
    currentBranch,
    branches,
    commits,
    stagedOperations,
    createBranch,
    switchBranch,
    stageOperation,
    commit,
    merge
  };
};
