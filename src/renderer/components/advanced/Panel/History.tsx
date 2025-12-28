/**
 * History Panel Component
 * Undo/redo history management
 */

import React from 'react';
import { HistoryProps } from '../types';
import './History.css';

export const History: React.FC<HistoryProps> = ({
  actions,
  currentIndex,
  onUndo,
  onRedo,
  onJumpTo,
  maxActions = 50,
}) => {
  const canUndo = currentIndex > 0;
  const canRedo = currentIndex < actions.length - 1;

  const handleJumpTo = (index: number) => {
    if (onJumpTo) {
      onJumpTo(index);
    }
  };

  return (
    <div className="history-panel">
      <div className="panel-header">
        <h3>History</h3>
        <div className="history-controls">
          <button
            className="history-btn"
            onClick={onUndo}
            disabled={!canUndo}
            title="Undo"
          >
            ↶
          </button>
          <button
            className="history-btn"
            onClick={onRedo}
            disabled={!canRedo}
            title="Redo"
          >
            ↷
          </button>
        </div>
      </div>

      <div className="history-list">
        {actions.length === 0 ? (
          <div className="history-empty">No actions recorded</div>
        ) : (
          actions.map((action, index) => {
            const isPast = index < currentIndex;
            const isCurrent = index === currentIndex;
            const isFuture = index > currentIndex;

            return (
              <div
                key={action.id}
                className={`history-item ${isCurrent ? 'current' : ''} ${isPast ? 'past' : ''} ${isFuture ? 'future' : ''}`}
                onClick={() => handleJumpTo(index)}
              >
                <div className="history-indicator">
                  {isCurrent && <span className="current-marker">●</span>}
                </div>

                <div className="history-info">
                  <div className="history-description">{action.description}</div>
                  <div className="history-meta">
                    <span className="history-type">{action.type}</span>
                    <span className="history-time">
                      {formatTimestamp(action.timestamp)}
                    </span>
                  </div>
                </div>

                {onJumpTo && (
                  <div className="history-actions">
                    {isFuture && action.canUndo && (
                      <button
                        className="jump-btn"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleJumpTo(index);
                        }}
                        title="Jump to this state"
                      >
                        ↑
                      </button>
                    )}
                    {isPast && action.canRedo && (
                      <button
                        className="jump-btn"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleJumpTo(index);
                        }}
                        title="Jump to this state"
                      >
                        ↓
                      </button>
                    )}
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>

      <div className="history-footer">
        <span className="history-count">
          {actions.length} / {maxActions} actions
        </span>
      </div>
    </div>
  );
};

function formatTimestamp(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;

  if (diff < 1000) return 'just now';
  if (diff < 60000) return `${Math.floor(diff / 1000)}s ago`;
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return new Date(timestamp).toLocaleDateString();
}

export default History;
