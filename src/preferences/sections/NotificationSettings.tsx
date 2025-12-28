/**
 * Notification Settings Section
 */

import React from 'react';
import { usePreferences } from '../hooks/usePreferences';
import SettingToggle from '../components/SettingToggle';

const NotificationSettings: React.FC = () => {
  const { notifications, updateNotifications } = usePreferences();

  return (
    <div className="notification-settings">
      <h2>Notification Settings</h2>
      <p className="section-description">
        Control how and when you receive notifications.
      </p>

      <div className="settings-group">
        <h3>General</h3>

        <SettingToggle
          label="Enable Notifications"
          description="Allow the application to send notifications"
          checked={notifications.enabled}
          onChange={(checked) => updateNotifications('enabled', checked)}
        />

        <SettingToggle
          label="Notification Sounds"
          description="Play a sound when notifications appear"
          checked={notifications.sound}
          onChange={(checked) => updateNotifications('sound', checked)}
          disabled={!notifications.enabled}
        />
      </div>

      <div className="settings-group">
        <h3>Notification Channels</h3>

        <SettingToggle
          label="Desktop Notifications"
          description="Show system notifications on your desktop"
          checked={notifications.desktop}
          onChange={(checked) => updateNotifications('desktop', checked)}
          disabled={!notifications.enabled}
        />

        <SettingToggle
          label="Email Notifications"
          description="Receive notifications via email"
          checked={notifications.email}
          onChange={(checked) => updateNotifications('email', checked)}
          disabled={!notifications.enabled}
        />
      </div>

      <div className="settings-group">
        <h3>Notification Types</h3>
        <p className="setting-note">
          Configure which types of events trigger notifications:
        </p>

        <div className="notification-types">
          <div className="notification-type">
            <SettingToggle
              label="Analysis Complete"
              description="When an accident analysis finishes processing"
              checked={true}
              onChange={() => {}}
              disabled={!notifications.enabled}
            />
          </div>

          <div className="notification-type">
            <SettingToggle
              label="Collaboration Updates"
              description="When team members make changes"
              checked={true}
              onChange={() => {}}
              disabled={!notifications.enabled}
            />
          </div>

          <div className="notification-type">
            <SettingToggle
              label="System Updates"
              description="Important system announcements"
              checked={true}
              onChange={() => {}}
              disabled={!notifications.enabled}
            />
          </div>

          <div className="notification-type">
            <SettingToggle
              label="Report Generation"
              description="When reports are ready to download"
              checked={true}
              onChange={() => {}}
              disabled={!notifications.enabled}
            />
          </div>
        </div>
      </div>

      <style>{`
        .notification-settings h2 {
          margin: 0 0 0.5rem 0;
          font-size: 1.5rem;
          font-weight: 600;
        }

        .section-description {
          margin: 0 0 2rem 0;
          color: var(--text-secondary, #666);
        }

        .settings-group {
          margin-bottom: 2rem;
          padding-bottom: 2rem;
          border-bottom: 1px solid var(--border-color, #e0e0e0);
        }

        .settings-group:last-child {
          border-bottom: none;
        }

        .settings-group h3 {
          margin: 0 0 1.5rem 0;
          font-size: 1.125rem;
          font-weight: 600;
        }

        .setting-note {
          margin: 0 0 1rem 0;
          font-size: 0.875rem;
          color: var(--text-secondary, #666);
        }

        .notification-types {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }

        .notification-type {
          padding: 0.75rem;
          background: var(--background-secondary, #f9f9f9);
          border-radius: 4px;
        }

        .dark .section-description,
        .dark .setting-note {
          color: #999;
        }

        .dark .settings-group {
          border-color: #404040;
        }

        .dark .notification-type {
          background: #1f1f1f;
        }
      `}</style>
    </div>
  );
};

export default NotificationSettings;
