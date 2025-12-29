/**
 * Sync Status Indicator Component
 * Displays current sync status with visual feedback
 */

import React from 'react';
import { SyncStatus, SyncStats } from '../types';

export interface SyncStatusIndicatorProps {
  status: SyncStatus;
  stats: SyncStats;
  className?: string;
  showDetails?: boolean;
}

export const SyncStatusIndicator: React.FC<SyncStatusIndicatorProps> = ({
  status,
  stats,
  className = '',
  showDetails = false,
}) => {
  const getStatusColor = () => {
    switch (status) {
      case SyncStatus.Syncing:
        return 'bg-blue-500';
      case SyncStatus.Completed:
        return 'bg-green-500';
      case SyncStatus.Failed:
        return 'bg-red-500';
      case SyncStatus.Paused:
        return 'bg-yellow-500';
      case SyncStatus.WaitingForNetwork:
        return 'bg-orange-500';
      default:
        return 'bg-gray-400';
    }
  };

  const getStatusText = () => {
    switch (status) {
      case SyncStatus.Syncing:
        return 'Syncing...';
      case SyncStatus.Completed:
        return 'Synced';
      case SyncStatus.Failed:
        return 'Sync Failed';
      case SyncStatus.Paused:
        return 'Paused';
      case SyncStatus.WaitingForNetwork:
        return 'Waiting for Network';
      default:
        return 'Idle';
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case SyncStatus.Syncing:
        return (
          <svg
            className="animate-spin h-4 w-4 text-white"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
        );
      case SyncStatus.Completed:
        return (
          <svg
            className="h-4 w-4 text-white"
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="2"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path d="M5 13l4 4L19 7" />
          </svg>
        );
      case SyncStatus.Failed:
        return (
          <svg
            className="h-4 w-4 text-white"
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="2"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path d="M6 18L18 6M6 6l12 12" />
          </svg>
        );
      default:
        return (
          <svg
            className="h-4 w-4 text-white"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <circle cx="10" cy="10" r="3" />
          </svg>
        );
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  };

  return (
    <div className={`flex items-center space-x-3 ${className}`}>
      {/* Status indicator */}
      <div className="flex items-center space-x-2">
        <div
          className={`${getStatusColor()} rounded-full p-1 flex items-center justify-center`}
        >
          {getStatusIcon()}
        </div>
        <span className="text-sm font-medium text-gray-700">
          {getStatusText()}
        </span>
      </div>

      {/* Pending operations badge */}
      {stats.pending > 0 && (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
          {stats.pending} pending
        </span>
      )}

      {/* Details */}
      {showDetails && (
        <div className="flex items-center space-x-4 text-xs text-gray-500">
          <div className="flex items-center space-x-1">
            <svg
              className="h-3 w-3"
              fill="none"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path d="M5 13l4 4L19 7" />
            </svg>
            <span>{stats.totalSynced} synced</span>
          </div>

          {stats.failed > 0 && (
            <div className="flex items-center space-x-1 text-red-500">
              <svg
                className="h-3 w-3"
                fill="none"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path d="M6 18L18 6M6 6l12 12" />
              </svg>
              <span>{stats.failed} failed</span>
            </div>
          )}

          {stats.conflicts > 0 && (
            <div className="flex items-center space-x-1 text-yellow-500">
              <svg
                className="h-3 w-3"
                fill="none"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <span>{stats.conflicts} conflicts</span>
            </div>
          )}

          {stats.lastSync && (
            <div>
              Last sync:{' '}
              {stats.lastSyncDurationMs
                ? formatDuration(stats.lastSyncDurationMs)
                : 'N/A'}
            </div>
          )}

          {(stats.bytesUploaded > 0 || stats.bytesDownloaded > 0) && (
            <div>
              ↑ {formatBytes(stats.bytesUploaded)} / ↓{' '}
              {formatBytes(stats.bytesDownloaded)}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
