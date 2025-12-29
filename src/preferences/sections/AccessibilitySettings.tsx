/**
 * Accessibility Settings Section
 */

import React from 'react';
import { usePreferences } from '../hooks/usePreferences';
import SettingToggle from '../components/SettingToggle';
import SettingSlider from '../components/SettingSlider';

const AccessibilitySettings: React.FC = () => {
  const { accessibility, updateAccessibility } = usePreferences();

  return (
    <div className="accessibility-settings">
      <h2>Accessibility Settings</h2>
      <p className="section-description">
        Configure options to improve accessibility and usability.
      </p>

      <div className="settings-group">
        <h3>Visual Accessibility</h3>

        <SettingToggle
          label="High Contrast Mode"
          description="Increase contrast for better visibility"
          checked={accessibility.highContrast}
          onChange={(checked) => updateAccessibility('highContrast', checked)}
        />

        <SettingToggle
          label="Reduce Motion"
          description="Minimize animations and transitions"
          checked={accessibility.reduceMotion}
          onChange={(checked) => updateAccessibility('reduceMotion', checked)}
        />

        <SettingSlider
          label="Accessibility Font Size"
          description="Override font size for better readability"
          value={accessibility.fontSize}
          min={12}
          max={24}
          step={1}
          unit="px"
          onChange={(value) => updateAccessibility('fontSize', value)}
        />
      </div>

      <div className="settings-group">
        <h3>Screen Reader</h3>

        <SettingToggle
          label="Screen Reader Support"
          description="Enable enhanced support for screen readers"
          checked={accessibility.screenReader}
          onChange={(checked) => updateAccessibility('screenReader', checked)}
        />

        <div className="accessibility-note">
          <strong>Note:</strong> Enabling screen reader support may require restarting
          the application for all features to work correctly.
        </div>
      </div>

      <div className="settings-group">
        <h3>Keyboard Navigation</h3>

        <SettingToggle
          label="Enhanced Keyboard Navigation"
          description="Improve keyboard accessibility with visible focus indicators"
          checked={accessibility.keyboardNavigation}
          onChange={(checked) => updateAccessibility('keyboardNavigation', checked)}
        />

        <div className="keyboard-tips">
          <h4>Keyboard Navigation Tips:</h4>
          <ul>
            <li><kbd>Tab</kbd> - Move to next interactive element</li>
            <li><kbd>Shift + Tab</kbd> - Move to previous interactive element</li>
            <li><kbd>Enter</kbd> or <kbd>Space</kbd> - Activate buttons and controls</li>
            <li><kbd>Esc</kbd> - Close dialogs and menus</li>
            <li><kbd>Arrow keys</kbd> - Navigate within components</li>
          </ul>
        </div>
      </div>

      <div className="settings-group">
        <h3>Additional Resources</h3>

        <div className="resource-links">
          <a href="/accessibility-guide" className="resource-link">
            <div className="resource-icon">📖</div>
            <div className="resource-info">
              <div className="resource-title">Accessibility Guide</div>
              <div className="resource-description">
                Learn about all accessibility features
              </div>
            </div>
          </a>

          <a href="/keyboard-shortcuts" className="resource-link">
            <div className="resource-icon">⌨️</div>
            <div className="resource-info">
              <div className="resource-title">Keyboard Shortcuts</div>
              <div className="resource-description">
                View and customize keyboard shortcuts
              </div>
            </div>
          </a>

          <a href="/support" className="resource-link">
            <div className="resource-icon">💬</div>
            <div className="resource-info">
              <div className="resource-title">Accessibility Support</div>
              <div className="resource-description">
                Get help with accessibility features
              </div>
            </div>
          </a>
        </div>
      </div>

      <style>{`
        .accessibility-settings h2 {
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

        .accessibility-note {
          margin-top: 1rem;
          padding: 1rem;
          background: var(--info-background, #fff3cd);
          border-left: 4px solid var(--warning-color, #ff9800);
          border-radius: 4px;
          font-size: 0.875rem;
          color: var(--text-color, #333);
        }

        .keyboard-tips {
          margin-top: 1rem;
          padding: 1rem;
          background: var(--background-secondary, #f9f9f9);
          border-radius: 4px;
        }

        .keyboard-tips h4 {
          margin: 0 0 0.75rem 0;
          font-size: 0.9375rem;
          font-weight: 600;
        }

        .keyboard-tips ul {
          margin: 0;
          padding-left: 1.5rem;
          list-style: disc;
        }

        .keyboard-tips li {
          margin: 0.5rem 0;
          font-size: 0.875rem;
          color: var(--text-secondary, #666);
        }

        .keyboard-tips kbd {
          padding: 0.125rem 0.375rem;
          background: white;
          border: 1px solid var(--border-color, #ccc);
          border-radius: 3px;
          font-family: Monaco, monospace;
          font-size: 0.8125rem;
          box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
        }

        .resource-links {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .resource-link {
          display: flex;
          align-items: center;
          gap: 1rem;
          padding: 1rem;
          background: var(--background-secondary, #f9f9f9);
          border: 1px solid var(--border-color, #e0e0e0);
          border-radius: 4px;
          text-decoration: none;
          color: inherit;
          transition: all 0.2s;
        }

        .resource-link:hover {
          background: var(--hover-color, #f0f0f0);
          border-color: var(--primary-color, #0066cc);
          transform: translateX(4px);
        }

        .resource-icon {
          font-size: 2rem;
        }

        .resource-info {
          flex: 1;
        }

        .resource-title {
          font-weight: 500;
          margin-bottom: 0.25rem;
        }

        .resource-description {
          font-size: 0.875rem;
          color: var(--text-secondary, #666);
        }

        .dark .section-description,
        .dark .keyboard-tips li,
        .dark .resource-description {
          color: #999;
        }

        .dark .settings-group {
          border-color: #404040;
        }

        .dark .accessibility-note {
          background: #3d2f1f;
          color: #e0e0e0;
        }

        .dark .keyboard-tips,
        .dark .resource-link {
          background: #1f1f1f;
          border-color: #404040;
        }

        .dark .resource-link:hover {
          background: #2a2a2a;
        }

        .dark .keyboard-tips kbd {
          background: #2a2a2a;
          border-color: #404040;
        }

        .high-contrast .resource-link {
          border-width: 2px;
        }

        .high-contrast .keyboard-tips kbd {
          border-width: 2px;
          font-weight: bold;
        }
      `}</style>
    </div>
  );
};

export default AccessibilitySettings;
