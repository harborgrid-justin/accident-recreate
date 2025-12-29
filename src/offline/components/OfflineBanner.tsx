/**
 * Offline Banner Component
 * Displays banner when offline with sync status
 */

import React from 'react';
import { NetworkState } from '../types';

export interface OfflineBannerProps {
  networkState: NetworkState;
  pendingOperations?: number;
  onRetry?: () => void;
  className?: string;
}

export const OfflineBanner: React.FC<OfflineBannerProps> = ({
  networkState,
  pendingOperations = 0,
  onRetry,
  className = '',
}) => {
  if (networkState === NetworkState.Online) {
    return null; // Don't show banner when online
  }

  const getBannerColor = () => {
    switch (networkState) {
      case NetworkState.Degraded:
        return 'bg-yellow-50 border-yellow-200';
      case NetworkState.Offline:
        return 'bg-red-50 border-red-200';
      default:
        return 'bg-gray-50 border-gray-200';
    }
  };

  const getIconColor = () => {
    switch (networkState) {
      case NetworkState.Degraded:
        return 'text-yellow-400';
      case NetworkState.Offline:
        return 'text-red-400';
      default:
        return 'text-gray-400';
    }
  };

  const getTextColor = () => {
    switch (networkState) {
      case NetworkState.Degraded:
        return 'text-yellow-800';
      case NetworkState.Offline:
        return 'text-red-800';
      default:
        return 'text-gray-800';
    }
  };

  const getMessage = () => {
    switch (networkState) {
      case NetworkState.Degraded:
        return 'Poor network connection. Some features may be slow.';
      case NetworkState.Offline:
        return 'You are currently offline. Changes will sync when connection is restored.';
      default:
        return 'Checking connection...';
    }
  };

  return (
    <div
      className={`${getBannerColor()} border-b ${className} transition-all duration-300`}
    >
      <div className="max-w-7xl mx-auto py-3 px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between flex-wrap">
          <div className="flex items-center flex-1">
            {/* Icon */}
            <span className={`flex p-2 rounded-lg ${getIconColor()}`}>
              {networkState === NetworkState.Offline ? (
                <svg
                  className="h-6 w-6"
                  fill="none"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path d="M18.364 5.636a9 9 0 010 12.728m0 0l-2.829-2.829m2.829 2.829L21 21M15.536 8.464a5 5 0 010 7.072m0 0l-2.829-2.829m-4.243 2.829a4.978 4.978 0 01-1.414-2.83m-1.414 5.658a9 9 0 01-2.167-9.238m7.824 2.167a1 1 0 111.414 1.414m-1.414-1.414L3 3m8.293 8.293l1.414 1.414" />
                </svg>
              ) : (
                <svg
                  className="h-6 w-6"
                  fill="none"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              )}
            </span>

            {/* Message */}
            <div className="ml-3 flex-1">
              <p className={`text-sm font-medium ${getTextColor()}`}>
                {getMessage()}
              </p>

              {/* Pending operations */}
              {pendingOperations > 0 && (
                <p className={`text-xs mt-1 ${getTextColor()} opacity-75`}>
                  {pendingOperations} change{pendingOperations !== 1 ? 's' : ''}{' '}
                  waiting to sync
                </p>
              )}
            </div>
          </div>

          {/* Retry button */}
          {onRetry && networkState === NetworkState.Offline && (
            <div className="flex-shrink-0 mt-2 sm:mt-0 sm:ml-3">
              <button
                type="button"
                onClick={onRetry}
                className={`inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white ${
                  networkState === NetworkState.Degraded
                    ? 'bg-yellow-600 hover:bg-yellow-700'
                    : 'bg-red-600 hover:bg-red-700'
                } focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500`}
              >
                <svg
                  className="-ml-1 mr-2 h-4 w-4"
                  fill="none"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
                Retry Connection
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
