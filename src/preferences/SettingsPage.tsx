/**
 * Settings Page
 * Main settings page with tabbed sections
 */

import React, { useState, useEffect } from 'react';
import { usePreferences } from './hooks/usePreferences';
import GeneralSettings from './sections/GeneralSettings';
import AppearanceSettings from './sections/AppearanceSettings';
import NotificationSettings from './sections/NotificationSettings';
import PrivacySettings from './sections/PrivacySettings';
import KeyboardShortcuts from './sections/KeyboardShortcuts';
import AccessibilitySettings from './sections/AccessibilitySettings';

/**
 * Settings section type
 */
interface SettingsSection {
  id: string;
  label: string;
  icon: string;
  component: React.ComponentType;
}

/**
 * Settings sections configuration
 */
const SETTINGS_SECTIONS: SettingsSection[] = [
  {
    id: 'general',
    label: 'General',
    icon: '⚙️',
    component: GeneralSettings,
  },
  {
    id: 'appearance',
    label: 'Appearance',
    icon: '🎨',
    component: AppearanceSettings,
  },
  {
    id: 'notifications',
    label: 'Notifications',
    icon: '🔔',
    component: NotificationSettings,
  },
  {
    id: 'privacy',
    label: 'Privacy',
    icon: '🔒',
    component: PrivacySettings,
  },
  {
    id: 'keyboard',
    label: 'Keyboard',
    icon: '⌨️',
    component: KeyboardShortcuts,
  },
  {
    id: 'accessibility',
    label: 'Accessibility',
    icon: '♿',
    component: AccessibilitySettings,
  },
];

/**
 * Settings Page Component
 */
const SettingsPage: React.FC = () => {
  const [activeSection, setActiveSection] = useState('general');
  const { syncStatus, syncNow, exportPreferences, loading, error, applyTheme } = usePreferences();

  // Apply theme on mount and when preferences change
  useEffect(() => {
    applyTheme();
  }, [applyTheme]);

  /**
   * Handle export
   */
  const handleExport = async () => {
    try {
      const exported = await exportPreferences();
      const blob = new Blob([JSON.stringify(exported, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `accuscene-preferences-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Failed to export preferences:', err);
    }
  };

  /**
   * Handle sync
   */
  const handleSync = async () => {
    try {
      await syncNow();
    } catch (err) {
      console.error('Failed to sync preferences:', err);
    }
  };

  // Get active section component
  const activeSectionConfig = SETTINGS_SECTIONS.find(s => s.id === activeSection);
  const ActiveComponent = activeSectionConfig?.component || GeneralSettings;

  if (loading) {
    return (
      <div className="settings-page loading">
        <div className="loading-spinner">Loading preferences...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="settings-page error">
        <div className="error-message">
          <h2>Error Loading Preferences</h2>
          <p>{error.message}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="settings-page">
      <div className="settings-header">
        <h1>Settings</h1>
        <div className="settings-actions">
          {syncStatus.lastSync && (
            <span className="last-sync">
              Last sync: {syncStatus.lastSync.toLocaleString()}
            </span>
          )}
          {syncStatus.pendingCount > 0 && (
            <span className="pending-changes">
              {syncStatus.pendingCount} pending changes
            </span>
          )}
          <button
            className="sync-button"
            onClick={handleSync}
            disabled={syncStatus.isSyncing}
          >
            {syncStatus.isSyncing ? 'Syncing...' : 'Sync Now'}
          </button>
          <button className="export-button" onClick={handleExport}>
            Export
          </button>
        </div>
      </div>

      <div className="settings-content">
        <aside className="settings-sidebar">
          <nav className="settings-nav">
            {SETTINGS_SECTIONS.map(section => (
              <button
                key={section.id}
                className={`settings-nav-item ${activeSection === section.id ? 'active' : ''}`}
                onClick={() => setActiveSection(section.id)}
              >
                <span className="nav-icon">{section.icon}</span>
                <span className="nav-label">{section.label}</span>
              </button>
            ))}
          </nav>
        </aside>

        <main className="settings-main">
          <div className="settings-section">
            <ActiveComponent />
          </div>
        </main>
      </div>

      <style>{`
        .settings-page {
          min-height: 100vh;
          background: var(--background-color, #f5f5f5);
          color: var(--text-color, #333);
        }

        .settings-page.loading,
        .settings-page.error {
          display: flex;
          align-items: center;
          justify-content: center;
        }

        .loading-spinner {
          font-size: 1.2rem;
          color: var(--primary-color, #0066cc);
        }

        .error-message {
          text-align: center;
          padding: 2rem;
          background: white;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .settings-header {
          background: white;
          padding: 1.5rem 2rem;
          border-bottom: 1px solid var(--border-color, #e0e0e0);
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .settings-header h1 {
          margin: 0;
          font-size: 1.75rem;
          font-weight: 600;
        }

        .settings-actions {
          display: flex;
          gap: 1rem;
          align-items: center;
        }

        .last-sync,
        .pending-changes {
          font-size: 0.875rem;
          color: var(--text-secondary, #666);
        }

        .pending-changes {
          color: var(--warning-color, #ff9800);
          font-weight: 500;
        }

        .sync-button,
        .export-button {
          padding: 0.5rem 1rem;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 0.875rem;
          font-weight: 500;
          transition: all 0.2s;
        }

        .sync-button {
          background: var(--primary-color, #0066cc);
          color: white;
        }

        .sync-button:hover:not(:disabled) {
          background: var(--primary-color-dark, #0052a3);
        }

        .sync-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .export-button {
          background: var(--secondary-color, #6c757d);
          color: white;
        }

        .export-button:hover {
          background: var(--secondary-color-dark, #545b62);
        }

        .settings-content {
          display: flex;
          min-height: calc(100vh - 80px);
        }

        .settings-sidebar {
          width: 240px;
          background: white;
          border-right: 1px solid var(--border-color, #e0e0e0);
        }

        .settings-nav {
          padding: 1rem 0;
        }

        .settings-nav-item {
          width: 100%;
          padding: 0.75rem 1.5rem;
          border: none;
          background: none;
          display: flex;
          align-items: center;
          gap: 0.75rem;
          cursor: pointer;
          font-size: 0.9375rem;
          color: var(--text-color, #333);
          transition: all 0.2s;
          text-align: left;
        }

        .settings-nav-item:hover {
          background: var(--hover-color, #f0f0f0);
        }

        .settings-nav-item.active {
          background: var(--primary-color-light, #e3f2fd);
          color: var(--primary-color, #0066cc);
          font-weight: 500;
          border-left: 3px solid var(--primary-color, #0066cc);
        }

        .nav-icon {
          font-size: 1.25rem;
        }

        .settings-main {
          flex: 1;
          padding: 2rem;
          overflow-y: auto;
        }

        .settings-section {
          max-width: 800px;
          background: white;
          border-radius: 8px;
          padding: 2rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }

        /* Dark mode */
        .dark .settings-page {
          background: #1a1a1a;
          color: #e0e0e0;
        }

        .dark .settings-header,
        .dark .settings-sidebar,
        .dark .settings-section {
          background: #2a2a2a;
          border-color: #404040;
        }

        .dark .settings-nav-item:hover {
          background: #353535;
        }

        .dark .error-message {
          background: #2a2a2a;
        }

        /* High contrast mode */
        .high-contrast .settings-page {
          background: #000;
          color: #fff;
        }

        .high-contrast .settings-header,
        .high-contrast .settings-sidebar,
        .high-contrast .settings-section {
          background: #1a1a1a;
          border-color: #fff;
          border-width: 2px;
        }

        .high-contrast .settings-nav-item.active {
          background: #fff;
          color: #000;
          border-left-width: 5px;
        }
      `}</style>
    </div>
  );
};

export default SettingsPage;
