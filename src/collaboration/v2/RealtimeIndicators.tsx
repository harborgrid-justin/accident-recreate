/**
 * AccuScene Enterprise v0.3.0 - Realtime Indicators
 *
 * React components for real-time collaboration UI indicators
 */

import React from 'react';
import { ConnectionStatus, PerformanceMetrics } from './types';

interface ConnectionIndicatorProps {
  status: ConnectionStatus;
  latency?: number;
}

export const ConnectionIndicator: React.FC<ConnectionIndicatorProps> = ({ status, latency }) => {
  const getStatusColor = () => {
    switch (status) {
      case ConnectionStatus.CONNECTED: return '#10b981';
      case ConnectionStatus.CONNECTING: return '#f59e0b';
      case ConnectionStatus.RECONNECTING: return '#f59e0b';
      case ConnectionStatus.DISCONNECTED: return '#ef4444';
      case ConnectionStatus.FAILED: return '#dc2626';
      default: return '#6b7280';
    }
  };

  const getStatusText = () => {
    switch (status) {
      case ConnectionStatus.CONNECTED: return `Connected${latency ? ` (${latency}ms)` : ''}`;
      case ConnectionStatus.CONNECTING: return 'Connecting...';
      case ConnectionStatus.RECONNECTING: return 'Reconnecting...';
      case ConnectionStatus.DISCONNECTED: return 'Disconnected';
      case ConnectionStatus.FAILED: return 'Connection Failed';
      default: return 'Unknown';
    }
  };

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '14px' }}>
      <div
        style={{
          width: '8px',
          height: '8px',
          borderRadius: '50%',
          backgroundColor: getStatusColor(),
          animation: status === ConnectionStatus.CONNECTING ? 'pulse 2s infinite' : 'none'
        }}
      />
      <span>{getStatusText()}</span>
    </div>
  );
};

interface SyncIndicatorProps {
  syncing: boolean;
  pendingOperations?: number;
}

export const SyncIndicator: React.FC<SyncIndicatorProps> = ({ syncing, pendingOperations = 0 }) => {
  if (!syncing && pendingOperations === 0) return null;

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '14px', color: '#6b7280' }}>
      {syncing && (
        <>
          <div className="spinner" style={{ width: '14px', height: '14px' }} />
          <span>Syncing...</span>
        </>
      )}
      {pendingOperations > 0 && (
        <span>{pendingOperations} pending</span>
      )}
    </div>
  );
};

interface PerformanceIndicatorProps {
  metrics: PerformanceMetrics;
}

export const PerformanceIndicator: React.FC<PerformanceIndicatorProps> = ({ metrics }) => {
  const getLatencyColor = (latency: number) => {
    if (latency < 50) return '#10b981';
    if (latency < 150) return '#f59e0b';
    return '#ef4444';
  };

  return (
    <div style={{ fontSize: '12px', color: '#6b7280', display: 'flex', gap: '16px' }}>
      <div>
        <span style={{ color: getLatencyColor(metrics.averageLatency) }}>
          {metrics.averageLatency.toFixed(0)}ms
        </span>
      </div>
      <div>{metrics.operationsPerSecond.toFixed(1)} ops/s</div>
      <div>{metrics.activeConnections} users</div>
    </div>
  );
};

interface TypingIndicatorProps {
  userNames: string[];
}

export const TypingIndicator: React.FC<TypingIndicatorProps> = ({ userNames }) => {
  if (userNames.length === 0) return null;

  const text = userNames.length === 1
    ? `${userNames[0]} is typing...`
    : userNames.length === 2
    ? `${userNames[0]} and ${userNames[1]} are typing...`
    : `${userNames.length} people are typing...`;

  return (
    <div style={{ fontSize: '14px', color: '#6b7280', fontStyle: 'italic' }}>
      {text}
    </div>
  );
};
