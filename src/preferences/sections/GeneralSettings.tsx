/**
 * General Settings Section
 */

import React from 'react';
import { usePreferences } from '../hooks/usePreferences';
import { Language } from '../types';
import SettingSelect from '../components/SettingSelect';

const GeneralSettings: React.FC = () => {
  const { general, updateGeneral } = usePreferences();

  const languageOptions = [
    { value: Language.English, label: 'English' },
    { value: Language.Spanish, label: 'Español' },
    { value: Language.French, label: 'Français' },
    { value: Language.German, label: 'Deutsch' },
    { value: Language.Japanese, label: '日本語' },
    { value: Language.Chinese, label: '中文' },
  ];

  const timezoneOptions = [
    { value: 'UTC', label: 'UTC' },
    { value: 'America/New_York', label: 'Eastern Time (US)' },
    { value: 'America/Chicago', label: 'Central Time (US)' },
    { value: 'America/Denver', label: 'Mountain Time (US)' },
    { value: 'America/Los_Angeles', label: 'Pacific Time (US)' },
    { value: 'Europe/London', label: 'London' },
    { value: 'Europe/Paris', label: 'Paris' },
    { value: 'Asia/Tokyo', label: 'Tokyo' },
    { value: 'Asia/Shanghai', label: 'Shanghai' },
    { value: 'Australia/Sydney', label: 'Sydney' },
  ];

  const dateFormatOptions = [
    { value: 'YYYY-MM-DD', label: 'YYYY-MM-DD (2025-12-28)' },
    { value: 'DD/MM/YYYY', label: 'DD/MM/YYYY (28/12/2025)' },
    { value: 'MM/DD/YYYY', label: 'MM/DD/YYYY (12/28/2025)' },
    { value: 'DD MMM YYYY', label: 'DD MMM YYYY (28 Dec 2025)' },
  ];

  const timeFormatOptions = [
    { value: '24h', label: '24-hour (13:00)' },
    { value: '12h', label: '12-hour (1:00 PM)' },
  ];

  const startupOptions = [
    { value: 'last_session', label: 'Restore last session' },
    { value: 'blank', label: 'Start with blank page' },
    { value: 'dashboard', label: 'Open dashboard' },
  ];

  return (
    <div className="general-settings">
      <h2>General Settings</h2>
      <p className="section-description">
        Configure language, timezone, and other general preferences.
      </p>

      <div className="settings-group">
        <h3>Language & Region</h3>

        <SettingSelect
          label="Language"
          description="Choose your preferred language"
          value={general.language}
          options={languageOptions}
          onChange={(value) => updateGeneral('language', value)}
        />

        <SettingSelect
          label="Timezone"
          description="Your local timezone"
          value={general.timezone}
          options={timezoneOptions}
          onChange={(value) => updateGeneral('timezone', value)}
        />

        <SettingSelect
          label="Date Format"
          description="How dates are displayed"
          value={general.dateFormat}
          options={dateFormatOptions}
          onChange={(value) => updateGeneral('dateFormat', value)}
        />

        <SettingSelect
          label="Time Format"
          description="12-hour or 24-hour time"
          value={general.timeFormat}
          options={timeFormatOptions}
          onChange={(value) => updateGeneral('timeFormat', value)}
        />
      </div>

      <div className="settings-group">
        <h3>Startup</h3>

        <SettingSelect
          label="On Startup"
          description="What to show when the application starts"
          value={general.startupBehavior}
          options={startupOptions}
          onChange={(value) => updateGeneral('startupBehavior', value)}
        />
      </div>

      <style>{`
        .general-settings h2 {
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

        .dark .section-description {
          color: #999;
        }

        .dark .settings-group {
          border-color: #404040;
        }
      `}</style>
    </div>
  );
};

export default GeneralSettings;
