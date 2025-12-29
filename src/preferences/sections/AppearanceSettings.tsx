/**
 * Appearance Settings Section
 */

import React from 'react';
import { usePreferences } from '../hooks/usePreferences';
import { Theme } from '../types';
import SettingSelect from '../components/SettingSelect';
import SettingToggle from '../components/SettingToggle';
import SettingSlider from '../components/SettingSlider';

const AppearanceSettings: React.FC = () => {
  const { appearance, updateAppearance, increaseFontSize, decreaseFontSize, resetFontSize } = usePreferences();

  const themeOptions = [
    { value: Theme.Light, label: 'Light' },
    { value: Theme.Dark, label: 'Dark' },
    { value: Theme.Auto, label: 'Auto (System)' },
  ];

  const fontFamilyOptions = [
    { value: 'system-ui', label: 'System Default' },
    { value: 'Inter', label: 'Inter' },
    { value: 'Roboto', label: 'Roboto' },
    { value: 'Arial', label: 'Arial' },
    { value: 'Georgia', label: 'Georgia' },
    { value: 'Monaco', label: 'Monaco (Monospace)' },
  ];

  return (
    <div className="appearance-settings">
      <h2>Appearance Settings</h2>
      <p className="section-description">
        Customize the look and feel of the application.
      </p>

      <div className="settings-group">
        <h3>Theme</h3>

        <SettingSelect
          label="Color Theme"
          description="Choose between light, dark, or automatic theme"
          value={appearance.theme}
          options={themeOptions}
          onChange={(value) => updateAppearance('theme', value)}
        />

        <SettingSelect
          label="Font Family"
          description="Choose your preferred font"
          value={appearance.fontFamily}
          options={fontFamilyOptions}
          onChange={(value) => updateAppearance('fontFamily', value)}
        />
      </div>

      <div className="settings-group">
        <h3>Text Size</h3>

        <SettingSlider
          label="Font Size"
          description="Adjust the base font size"
          value={appearance.fontSize}
          min={8}
          max={32}
          step={1}
          unit="px"
          onChange={(value) => updateAppearance('fontSize', value)}
        />

        <div className="font-size-controls">
          <button
            className="font-size-button"
            onClick={decreaseFontSize}
            disabled={appearance.fontSize <= 8}
          >
            Decrease
          </button>
          <button
            className="font-size-button"
            onClick={resetFontSize}
          >
            Reset to Default
          </button>
          <button
            className="font-size-button"
            onClick={increaseFontSize}
            disabled={appearance.fontSize >= 32}
          >
            Increase
          </button>
        </div>
      </div>

      <div className="settings-group">
        <h3>Display Options</h3>

        <SettingToggle
          label="Compact Mode"
          description="Reduce spacing for a more compact interface"
          checked={appearance.compactMode}
          onChange={(checked) => updateAppearance('compactMode', checked)}
        />

        <SettingToggle
          label="Show Animations"
          description="Enable smooth transitions and animations"
          checked={appearance.showAnimations}
          onChange={(checked) => updateAppearance('showAnimations', checked)}
        />
      </div>

      <style>{`
        .appearance-settings h2 {
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

        .font-size-controls {
          display: flex;
          gap: 0.5rem;
          margin-top: 1rem;
        }

        .font-size-button {
          padding: 0.5rem 1rem;
          border: 1px solid var(--border-color, #ccc);
          background: white;
          border-radius: 4px;
          cursor: pointer;
          font-size: 0.875rem;
          transition: all 0.2s;
        }

        .font-size-button:hover:not(:disabled) {
          background: var(--hover-color, #f0f0f0);
          border-color: var(--primary-color, #0066cc);
        }

        .font-size-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .dark .section-description {
          color: #999;
        }

        .dark .settings-group {
          border-color: #404040;
        }

        .dark .font-size-button {
          background: #2a2a2a;
          border-color: #404040;
          color: #e0e0e0;
        }

        .dark .font-size-button:hover:not(:disabled) {
          background: #353535;
        }
      `}</style>
    </div>
  );
};

export default AppearanceSettings;
