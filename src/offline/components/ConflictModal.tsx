/**
 * Conflict Modal Component
 * UI for resolving sync conflicts
 */

import React, { useState } from 'react';
import { Conflict, ResolutionResult } from '../types';

export interface ConflictModalProps {
  conflict: Conflict;
  onResolve: (resolution: ResolutionResult) => void;
  onCancel: () => void;
  isOpen: boolean;
}

export const ConflictModal: React.FC<ConflictModalProps> = ({
  conflict,
  onResolve,
  onCancel,
  isOpen,
}) => {
  const [selectedVersion, setSelectedVersion] = useState<'local' | 'remote' | 'merge'>(
    'remote'
  );
  const [mergedData, setMergedData] = useState<any>(conflict.remoteData);

  if (!isOpen) return null;

  const handleResolve = () => {
    let data: any;
    let version: any;

    switch (selectedVersion) {
      case 'local':
        data = conflict.localData;
        version = conflict.localVersion;
        break;
      case 'remote':
        data = conflict.remoteData;
        version = conflict.remoteVersion;
        break;
      case 'merge':
        data = mergedData;
        version = {
          ...conflict.remoteVersion,
          timestamp: new Date().toISOString(),
        };
        break;
    }

    const resolution: ResolutionResult = {
      data,
      version,
      strategy: 'manual',
      manual: true,
      metadata: {
        selectedVersion,
        resolvedAt: new Date().toISOString(),
      },
    };

    onResolve(resolution);
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  const renderDataDiff = () => {
    const localKeys = Object.keys(conflict.localData);
    const remoteKeys = Object.keys(conflict.remoteData);
    const allKeys = [...new Set([...localKeys, ...remoteKeys])];

    return (
      <div className="space-y-2">
        {allKeys.map((key) => {
          const localValue = conflict.localData[key];
          const remoteValue = conflict.remoteData[key];
          const isDifferent = JSON.stringify(localValue) !== JSON.stringify(remoteValue);

          return (
            <div
              key={key}
              className={`p-3 rounded ${
                isDifferent ? 'bg-yellow-50 border border-yellow-200' : 'bg-gray-50'
              }`}
            >
              <div className="font-medium text-sm text-gray-700 mb-1">{key}</div>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <div className="text-xs text-gray-500 mb-1">Local</div>
                  <div className="font-mono text-xs">
                    {JSON.stringify(localValue, null, 2)}
                  </div>
                </div>
                <div>
                  <div className="text-xs text-gray-500 mb-1">Remote</div>
                  <div className="font-mono text-xs">
                    {JSON.stringify(remoteValue, null, 2)}
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
        {/* Background overlay */}
        <div
          className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
          onClick={onCancel}
        />

        {/* Modal panel */}
        <div className="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-4xl sm:w-full">
          <div className="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
            {/* Header */}
            <div className="flex items-start justify-between mb-6">
              <div>
                <div className="flex items-center">
                  <div className="flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-yellow-100">
                    <svg
                      className="h-6 w-6 text-yellow-600"
                      fill="none"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="2"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                  </div>
                  <div className="ml-4">
                    <h3 className="text-lg font-medium text-gray-900">
                      Sync Conflict Detected
                    </h3>
                    <p className="mt-1 text-sm text-gray-500">
                      {conflict.entityType} #{conflict.entityId}
                    </p>
                  </div>
                </div>
              </div>
              <button
                onClick={onCancel}
                className="text-gray-400 hover:text-gray-500"
              >
                <svg
                  className="h-6 w-6"
                  fill="none"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Version selection */}
            <div className="mb-6">
              <label className="text-sm font-medium text-gray-700 mb-2 block">
                Select version to keep:
              </label>
              <div className="grid grid-cols-3 gap-4">
                {/* Local version */}
                <button
                  onClick={() => setSelectedVersion('local')}
                  className={`p-4 border rounded-lg text-left transition ${
                    selectedVersion === 'local'
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-300 hover:border-gray-400'
                  }`}
                >
                  <div className="flex items-center mb-2">
                    <div
                      className={`h-4 w-4 rounded-full border-2 mr-2 ${
                        selectedVersion === 'local'
                          ? 'border-blue-500 bg-blue-500'
                          : 'border-gray-300'
                      }`}
                    />
                    <span className="font-medium text-gray-900">Local Version</span>
                  </div>
                  <p className="text-xs text-gray-500">
                    Modified: {formatTimestamp(conflict.localVersion.timestamp)}
                  </p>
                </button>

                {/* Remote version */}
                <button
                  onClick={() => setSelectedVersion('remote')}
                  className={`p-4 border rounded-lg text-left transition ${
                    selectedVersion === 'remote'
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-300 hover:border-gray-400'
                  }`}
                >
                  <div className="flex items-center mb-2">
                    <div
                      className={`h-4 w-4 rounded-full border-2 mr-2 ${
                        selectedVersion === 'remote'
                          ? 'border-blue-500 bg-blue-500'
                          : 'border-gray-300'
                      }`}
                    />
                    <span className="font-medium text-gray-900">Server Version</span>
                  </div>
                  <p className="text-xs text-gray-500">
                    Modified: {formatTimestamp(conflict.remoteVersion.timestamp)}
                  </p>
                </button>

                {/* Merge */}
                <button
                  onClick={() => setSelectedVersion('merge')}
                  className={`p-4 border rounded-lg text-left transition ${
                    selectedVersion === 'merge'
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-300 hover:border-gray-400'
                  }`}
                >
                  <div className="flex items-center mb-2">
                    <div
                      className={`h-4 w-4 rounded-full border-2 mr-2 ${
                        selectedVersion === 'merge'
                          ? 'border-blue-500 bg-blue-500'
                          : 'border-gray-300'
                      }`}
                    />
                    <span className="font-medium text-gray-900">Merge Both</span>
                  </div>
                  <p className="text-xs text-gray-500">
                    Combine changes from both versions
                  </p>
                </button>
              </div>
            </div>

            {/* Data diff */}
            <div className="mb-6">
              <h4 className="text-sm font-medium text-gray-700 mb-3">
                Changes Comparison
              </h4>
              <div className="max-h-96 overflow-y-auto border border-gray-200 rounded-lg p-4">
                {renderDataDiff()}
              </div>
            </div>

            {/* Merge editor */}
            {selectedVersion === 'merge' && (
              <div className="mb-6">
                <h4 className="text-sm font-medium text-gray-700 mb-2">
                  Edit Merged Data
                </h4>
                <textarea
                  value={JSON.stringify(mergedData, null, 2)}
                  onChange={(e) => {
                    try {
                      setMergedData(JSON.parse(e.target.value));
                    } catch (err) {
                      // Invalid JSON, ignore
                    }
                  }}
                  className="w-full h-48 font-mono text-sm border border-gray-300 rounded-lg p-3 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
            <button
              type="button"
              onClick={handleResolve}
              className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm"
            >
              Resolve Conflict
            </button>
            <button
              type="button"
              onClick={onCancel}
              className="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
