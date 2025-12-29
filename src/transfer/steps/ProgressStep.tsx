import React, { useEffect, useState } from 'react';
import { TransferProgress, ProgressStatus } from '../types';

interface ProgressStepProps {
  state: any;
  mode: 'import' | 'export';
}

export const ProgressStep: React.FC<ProgressStepProps> = ({ state, mode }) => {
  const [elapsed, setElapsed] = useState(0);
  const progress: TransferProgress | undefined = state.progress;

  // Update elapsed time
  useEffect(() => {
    if (!progress || progress.status !== 'running') return;

    const interval = setInterval(() => {
      const start = new Date(progress.startedAt).getTime();
      const now = Date.now();
      setElapsed(Math.floor((now - start) / 1000));
    }, 1000);

    return () => clearInterval(interval);
  }, [progress]);

  if (!progress) {
    return (
      <div className="text-center py-12">
        <div className="mx-auto w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4 animate-pulse">
          <svg
            className="w-8 h-8 text-gray-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>
        <p className="text-gray-600">Preparing {mode}...</p>
      </div>
    );
  }

  const percentage = progress.percentage || 0;
  const isComplete = progress.status === 'completed';
  const isFailed = progress.status === 'failed';
  const isCancelled = progress.status === 'cancelled';
  const isRunning = progress.status === 'running';

  return (
    <div className="progress-step">
      {/* Status Icon */}
      <div className="text-center mb-6">
        <div
          className={`mx-auto w-20 h-20 rounded-full flex items-center justify-center ${
            isComplete
              ? 'bg-green-100'
              : isFailed || isCancelled
              ? 'bg-red-100'
              : 'bg-blue-100 animate-pulse'
          }`}
        >
          {isComplete ? (
            <svg
              className="w-10 h-10 text-green-600"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                clipRule="evenodd"
              />
            </svg>
          ) : isFailed || isCancelled ? (
            <svg
              className="w-10 h-10 text-red-600"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                clipRule="evenodd"
              />
            </svg>
          ) : (
            <svg
              className="w-10 h-10 text-blue-600 animate-spin"
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
          )}
        </div>
        <h3
          className={`mt-4 text-xl font-semibold ${
            isComplete
              ? 'text-green-900'
              : isFailed || isCancelled
              ? 'text-red-900'
              : 'text-blue-900'
          }`}
        >
          {getStatusText(progress.status, mode)}
        </h3>
        <p className="text-sm text-gray-600 mt-1">{progress.stage}</p>
      </div>

      {/* Progress Bar */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm font-medium text-gray-700">
            {progress.processed.toLocaleString()} / {progress.total.toLocaleString()}{' '}
            records
          </span>
          <span className="text-sm font-medium text-gray-700">
            {percentage.toFixed(1)}%
          </span>
        </div>
        <div className="w-full bg-gray-200 rounded-full h-4 overflow-hidden">
          <div
            className={`h-full rounded-full transition-all duration-300 ${
              isComplete
                ? 'bg-green-600'
                : isFailed || isCancelled
                ? 'bg-red-600'
                : 'bg-blue-600'
            }`}
            style={{ width: `${percentage}%` }}
          />
        </div>
      </div>

      {/* Statistics Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <div className="p-4 bg-gray-50 rounded-lg">
          <p className="text-xs text-gray-600">Elapsed Time</p>
          <p className="text-lg font-semibold text-gray-900">
            {formatDuration(elapsed)}
          </p>
        </div>
        {progress.etaSeconds !== undefined && isRunning && (
          <div className="p-4 bg-gray-50 rounded-lg">
            <p className="text-xs text-gray-600">Time Remaining</p>
            <p className="text-lg font-semibold text-gray-900">
              {formatDuration(progress.etaSeconds)}
            </p>
          </div>
        )}
        {progress.totalBytes && (
          <div className="p-4 bg-gray-50 rounded-lg">
            <p className="text-xs text-gray-600">Data Processed</p>
            <p className="text-lg font-semibold text-gray-900">
              {formatBytes(progress.bytesProcessed)} /{' '}
              {formatBytes(progress.totalBytes)}
            </p>
          </div>
        )}
        <div className="p-4 bg-gray-50 rounded-lg">
          <p className="text-xs text-gray-600">Status</p>
          <p className="text-lg font-semibold text-gray-900 capitalize">
            {progress.status}
          </p>
        </div>
      </div>

      {/* Error Message */}
      {progress.error && (
        <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
          <h4 className="text-sm font-medium text-red-900 mb-1">Error</h4>
          <p className="text-sm text-red-700">{progress.error}</p>
        </div>
      )}

      {/* Success Message */}
      {isComplete && (
        <div className="p-6 bg-green-50 border border-green-200 rounded-lg text-center">
          <svg
            className="mx-auto w-12 h-12 text-green-600 mb-3"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fillRule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clipRule="evenodd"
            />
          </svg>
          <h4 className="text-lg font-semibold text-green-900 mb-2">
            {mode === 'export' ? 'Export' : 'Import'} Completed Successfully!
          </h4>
          <p className="text-sm text-green-700">
            {progress.total.toLocaleString()} records processed in{' '}
            {formatDuration(elapsed)}
          </p>
        </div>
      )}

      {/* Activity Log */}
      <div className="mt-6">
        <h4 className="text-sm font-medium text-gray-900 mb-3">Activity Log</h4>
        <div className="bg-gray-50 rounded-lg p-4 max-h-48 overflow-y-auto">
          <div className="space-y-2 text-sm font-mono text-gray-700">
            <div className="flex items-start">
              <span className="text-gray-500 mr-2">[{formatTime(progress.startedAt)}]</span>
              <span>Started {mode}</span>
            </div>
            {progress.stage && (
              <div className="flex items-start">
                <span className="text-gray-500 mr-2">[{formatTime(progress.updatedAt)}]</span>
                <span>{progress.stage}</span>
              </div>
            )}
            {isComplete && (
              <div className="flex items-start text-green-700">
                <span className="text-gray-500 mr-2">[{formatTime(progress.updatedAt)}]</span>
                <span>Completed successfully</span>
              </div>
            )}
            {isFailed && progress.error && (
              <div className="flex items-start text-red-700">
                <span className="text-gray-500 mr-2">[{formatTime(progress.updatedAt)}]</span>
                <span>Failed: {progress.error}</span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

function getStatusText(status: ProgressStatus, mode: string): string {
  switch (status) {
    case 'pending':
      return `${mode === 'export' ? 'Export' : 'Import'} Pending`;
    case 'running':
      return `${mode === 'export' ? 'Exporting' : 'Importing'} Data...`;
    case 'paused':
      return 'Paused';
    case 'completed':
      return `${mode === 'export' ? 'Export' : 'Import'} Complete`;
    case 'failed':
      return `${mode === 'export' ? 'Export' : 'Import'} Failed`;
    case 'cancelled':
      return 'Cancelled';
    default:
      return 'Unknown Status';
  }
}

function formatDuration(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  } else if (seconds < 3600) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}m ${secs}s`;
  } else {
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${mins}m`;
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

function formatTime(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString();
}
