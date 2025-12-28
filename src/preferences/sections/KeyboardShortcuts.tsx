/**
 * Keyboard Shortcuts Section
 */

import React, { useState } from 'react';
import { KeyboardShortcut } from '../types';

const KEYBOARD_SHORTCUTS: KeyboardShortcut[] = [
  // Navigation
  {
    id: 'nav.dashboard',
    name: 'Go to Dashboard',
    description: 'Navigate to the dashboard',
    defaultKeys: ['Ctrl', 'D'],
    currentKeys: ['Ctrl', 'D'],
    category: 'Navigation',
    editable: true,
  },
  {
    id: 'nav.cases',
    name: 'Go to Cases',
    description: 'Navigate to the cases list',
    defaultKeys: ['Ctrl', 'C'],
    currentKeys: ['Ctrl', 'C'],
    category: 'Navigation',
    editable: true,
  },
  {
    id: 'nav.settings',
    name: 'Open Settings',
    description: 'Open the settings page',
    defaultKeys: ['Ctrl', ','],
    currentKeys: ['Ctrl', ','],
    category: 'Navigation',
    editable: true,
  },

  // Actions
  {
    id: 'action.new',
    name: 'New Case',
    description: 'Create a new accident case',
    defaultKeys: ['Ctrl', 'N'],
    currentKeys: ['Ctrl', 'N'],
    category: 'Actions',
    editable: true,
  },
  {
    id: 'action.save',
    name: 'Save',
    description: 'Save current changes',
    defaultKeys: ['Ctrl', 'S'],
    currentKeys: ['Ctrl', 'S'],
    category: 'Actions',
    editable: false,
  },
  {
    id: 'action.search',
    name: 'Search',
    description: 'Open search',
    defaultKeys: ['Ctrl', 'K'],
    currentKeys: ['Ctrl', 'K'],
    category: 'Actions',
    editable: true,
  },

  // View
  {
    id: 'view.toggle_sidebar',
    name: 'Toggle Sidebar',
    description: 'Show or hide the sidebar',
    defaultKeys: ['Ctrl', 'B'],
    currentKeys: ['Ctrl', 'B'],
    category: 'View',
    editable: true,
  },
  {
    id: 'view.fullscreen',
    name: 'Toggle Fullscreen',
    description: 'Enter or exit fullscreen mode',
    defaultKeys: ['F11'],
    currentKeys: ['F11'],
    category: 'View',
    editable: false,
  },
  {
    id: 'view.zoom_in',
    name: 'Zoom In',
    description: 'Increase zoom level',
    defaultKeys: ['Ctrl', '+'],
    currentKeys: ['Ctrl', '+'],
    category: 'View',
    editable: false,
  },
  {
    id: 'view.zoom_out',
    name: 'Zoom Out',
    description: 'Decrease zoom level',
    defaultKeys: ['Ctrl', '-'],
    currentKeys: ['Ctrl', '-'],
    category: 'View',
    editable: false,
  },

  // Analysis
  {
    id: 'analysis.run',
    name: 'Run Analysis',
    description: 'Start accident reconstruction analysis',
    defaultKeys: ['Ctrl', 'R'],
    currentKeys: ['Ctrl', 'R'],
    category: 'Analysis',
    editable: true,
  },
  {
    id: 'analysis.export',
    name: 'Export Report',
    description: 'Export analysis report',
    defaultKeys: ['Ctrl', 'E'],
    currentKeys: ['Ctrl', 'E'],
    category: 'Analysis',
    editable: true,
  },
];

const KeyboardShortcuts: React.FC = () => {
  const [shortcuts, setShortcuts] = useState(KEYBOARD_SHORTCUTS);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [recordingKeys, setRecordingKeys] = useState<string[]>([]);

  const categories = Array.from(new Set(shortcuts.map(s => s.category)));

  /**
   * Start recording a new shortcut
   */
  const startRecording = (id: string) => {
    setEditingId(id);
    setRecordingKeys([]);
  };

  /**
   * Cancel recording
   */
  const cancelRecording = () => {
    setEditingId(null);
    setRecordingKeys([]);
  };

  /**
   * Save the recorded shortcut
   */
  const saveShortcut = (id: string) => {
    if (recordingKeys.length > 0) {
      setShortcuts(prev =>
        prev.map(s =>
          s.id === id ? { ...s, currentKeys: recordingKeys } : s
        )
      );
    }
    setEditingId(null);
    setRecordingKeys([]);
  };

  /**
   * Reset shortcut to default
   */
  const resetShortcut = (id: string) => {
    setShortcuts(prev =>
      prev.map(s =>
        s.id === id ? { ...s, currentKeys: s.defaultKeys } : s
      )
    );
  };

  /**
   * Reset all shortcuts to defaults
   */
  const resetAll = () => {
    if (confirm('Reset all keyboard shortcuts to defaults?')) {
      setShortcuts(prev =>
        prev.map(s => ({ ...s, currentKeys: s.defaultKeys }))
      );
    }
  };

  /**
   * Format keys for display
   */
  const formatKeys = (keys: string[]): string => {
    return keys.join(' + ');
  };

  return (
    <div className="keyboard-shortcuts">
      <div className="shortcuts-header">
        <div>
          <h2>Keyboard Shortcuts</h2>
          <p className="section-description">
            Customize keyboard shortcuts to match your workflow.
          </p>
        </div>
        <button className="reset-all-button" onClick={resetAll}>
          Reset All to Defaults
        </button>
      </div>

      {categories.map(category => (
        <div key={category} className="shortcut-category">
          <h3>{category}</h3>

          <div className="shortcut-list">
            {shortcuts
              .filter(s => s.category === category)
              .map(shortcut => (
                <div key={shortcut.id} className="shortcut-item">
                  <div className="shortcut-info">
                    <div className="shortcut-name">{shortcut.name}</div>
                    <div className="shortcut-description">{shortcut.description}</div>
                  </div>

                  <div className="shortcut-controls">
                    {editingId === shortcut.id ? (
                      <>
                        <div className="shortcut-recording">
                          {recordingKeys.length > 0
                            ? formatKeys(recordingKeys)
                            : 'Press keys...'}
                        </div>
                        <button
                          className="shortcut-button save"
                          onClick={() => saveShortcut(shortcut.id)}
                          disabled={recordingKeys.length === 0}
                        >
                          Save
                        </button>
                        <button
                          className="shortcut-button cancel"
                          onClick={cancelRecording}
                        >
                          Cancel
                        </button>
                      </>
                    ) : (
                      <>
                        <div className="shortcut-keys">
                          {formatKeys(shortcut.currentKeys)}
                        </div>
                        {shortcut.editable && (
                          <>
                            <button
                              className="shortcut-button edit"
                              onClick={() => startRecording(shortcut.id)}
                            >
                              Edit
                            </button>
                            {shortcut.currentKeys.join(',') !== shortcut.defaultKeys.join(',') && (
                              <button
                                className="shortcut-button reset"
                                onClick={() => resetShortcut(shortcut.id)}
                              >
                                Reset
                              </button>
                            )}
                          </>
                        )}
                      </>
                    )}
                  </div>
                </div>
              ))}
          </div>
        </div>
      ))}

      <style>{`
        .keyboard-shortcuts {
          max-width: 900px;
        }

        .shortcuts-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 2rem;
        }

        .keyboard-shortcuts h2 {
          margin: 0 0 0.5rem 0;
          font-size: 1.5rem;
          font-weight: 600;
        }

        .section-description {
          margin: 0;
          color: var(--text-secondary, #666);
        }

        .reset-all-button {
          padding: 0.5rem 1rem;
          border: 1px solid var(--border-color, #ccc);
          background: white;
          border-radius: 4px;
          cursor: pointer;
          font-size: 0.875rem;
          transition: all 0.2s;
        }

        .reset-all-button:hover {
          background: var(--hover-color, #f0f0f0);
          border-color: var(--primary-color, #0066cc);
        }

        .shortcut-category {
          margin-bottom: 2rem;
          padding-bottom: 2rem;
          border-bottom: 1px solid var(--border-color, #e0e0e0);
        }

        .shortcut-category:last-child {
          border-bottom: none;
        }

        .shortcut-category h3 {
          margin: 0 0 1rem 0;
          font-size: 1.125rem;
          font-weight: 600;
        }

        .shortcut-list {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }

        .shortcut-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem;
          background: var(--background-secondary, #f9f9f9);
          border-radius: 4px;
          transition: all 0.2s;
        }

        .shortcut-item:hover {
          background: var(--hover-color, #f0f0f0);
        }

        .shortcut-info {
          flex: 1;
        }

        .shortcut-name {
          font-weight: 500;
          margin-bottom: 0.25rem;
        }

        .shortcut-description {
          font-size: 0.875rem;
          color: var(--text-secondary, #666);
        }

        .shortcut-controls {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .shortcut-keys,
        .shortcut-recording {
          padding: 0.5rem 0.75rem;
          background: white;
          border: 1px solid var(--border-color, #ccc);
          border-radius: 4px;
          font-family: Monaco, monospace;
          font-size: 0.875rem;
          min-width: 120px;
          text-align: center;
        }

        .shortcut-recording {
          background: var(--primary-color-light, #e3f2fd);
          border-color: var(--primary-color, #0066cc);
          animation: pulse 1.5s ease-in-out infinite;
        }

        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.7; }
        }

        .shortcut-button {
          padding: 0.375rem 0.75rem;
          border: 1px solid var(--border-color, #ccc);
          background: white;
          border-radius: 4px;
          cursor: pointer;
          font-size: 0.8125rem;
          transition: all 0.2s;
        }

        .shortcut-button:hover:not(:disabled) {
          background: var(--hover-color, #f0f0f0);
        }

        .shortcut-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .shortcut-button.edit {
          border-color: var(--primary-color, #0066cc);
          color: var(--primary-color, #0066cc);
        }

        .shortcut-button.save {
          background: var(--primary-color, #0066cc);
          color: white;
          border-color: var(--primary-color, #0066cc);
        }

        .shortcut-button.save:hover:not(:disabled) {
          background: var(--primary-color-dark, #0052a3);
        }

        .shortcut-button.reset,
        .shortcut-button.cancel {
          color: var(--text-secondary, #666);
        }

        .dark .section-description,
        .dark .shortcut-description {
          color: #999;
        }

        .dark .shortcut-category {
          border-color: #404040;
        }

        .dark .shortcut-item {
          background: #1f1f1f;
        }

        .dark .shortcut-item:hover {
          background: #2a2a2a;
        }

        .dark .shortcut-keys,
        .dark .shortcut-button,
        .dark .reset-all-button {
          background: #2a2a2a;
          border-color: #404040;
          color: #e0e0e0;
        }

        .dark .shortcut-button:hover:not(:disabled),
        .dark .reset-all-button:hover {
          background: #353535;
        }

        .dark .shortcut-recording {
          background: #1a3a52;
        }
      `}</style>
    </div>
  );
};

export default KeyboardShortcuts;
