/**
 * Privacy Settings Section
 */

import React from 'react';
import { usePreferences } from '../hooks/usePreferences';
import SettingToggle from '../components/SettingToggle';

const PrivacySettings: React.FC = () => {
  const { privacy, updatePrivacy } = usePreferences();

  return (
    <div className="privacy-settings">
      <h2>Privacy Settings</h2>
      <p className="section-description">
        Control how your data is collected and used.
      </p>

      <div className="settings-group">
        <h3>Data Collection</h3>

        <SettingToggle
          label="Analytics & Usage Data"
          description="Help improve AccuScene by sharing anonymous usage statistics"
          checked={privacy.analytics}
          onChange={(checked) => updatePrivacy('analytics', checked)}
        />

        <SettingToggle
          label="Crash Reports"
          description="Automatically send crash reports to help us fix bugs"
          checked={privacy.crashReports}
          onChange={(checked) => updatePrivacy('crashReports', checked)}
        />

        <SettingToggle
          label="Diagnostics Data"
          description="Share diagnostic information for troubleshooting"
          checked={privacy.diagnostics}
          onChange={(checked) => updatePrivacy('diagnostics', checked)}
        />
      </div>

      <div className="settings-group">
        <h3>Third-Party Services</h3>

        <SettingToggle
          label="Third-Party Data Sharing"
          description="Allow sharing data with integrated third-party services"
          checked={privacy.thirdPartyData}
          onChange={(checked) => updatePrivacy('thirdPartyData', checked)}
        />

        <div className="privacy-note">
          <strong>Note:</strong> We never sell your personal data. Data shared with third parties
          is limited to what's necessary for the services you use, and is governed by our privacy policy.
        </div>
      </div>

      <div className="settings-group">
        <h3>Data Management</h3>

        <div className="data-actions">
          <button className="data-action-button">
            Download My Data
          </button>
          <button className="data-action-button danger">
            Delete My Data
          </button>
        </div>

        <div className="privacy-info">
          <p>
            <strong>Download My Data:</strong> Request a copy of all data associated with your account.
          </p>
          <p>
            <strong>Delete My Data:</strong> Permanently delete all data associated with your account.
            This action cannot be undone.
          </p>
        </div>
      </div>

      <div className="settings-group">
        <h3>Privacy Policy</h3>
        <p className="privacy-policy-text">
          For more information about how we collect, use, and protect your data,
          please read our <a href="/privacy-policy" target="_blank">Privacy Policy</a>.
        </p>
        <p className="privacy-policy-text">
          Last updated: December 28, 2025
        </p>
      </div>

      <style>{`
        .privacy-settings h2 {
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

        .privacy-note {
          margin-top: 1rem;
          padding: 1rem;
          background: var(--info-background, #e3f2fd);
          border-left: 4px solid var(--info-color, #2196f3);
          border-radius: 4px;
          font-size: 0.875rem;
          color: var(--text-color, #333);
        }

        .data-actions {
          display: flex;
          gap: 1rem;
          margin-bottom: 1.5rem;
        }

        .data-action-button {
          padding: 0.75rem 1.5rem;
          border: 1px solid var(--border-color, #ccc);
          background: white;
          border-radius: 4px;
          cursor: pointer;
          font-size: 0.875rem;
          font-weight: 500;
          transition: all 0.2s;
        }

        .data-action-button:hover {
          background: var(--hover-color, #f0f0f0);
          border-color: var(--primary-color, #0066cc);
        }

        .data-action-button.danger {
          color: var(--danger-color, #d32f2f);
          border-color: var(--danger-color, #d32f2f);
        }

        .data-action-button.danger:hover {
          background: var(--danger-background, #ffebee);
        }

        .privacy-info {
          font-size: 0.875rem;
          color: var(--text-secondary, #666);
        }

        .privacy-info p {
          margin: 0.5rem 0;
        }

        .privacy-policy-text {
          margin: 0.5rem 0;
          color: var(--text-secondary, #666);
          font-size: 0.875rem;
        }

        .privacy-policy-text a {
          color: var(--primary-color, #0066cc);
          text-decoration: none;
        }

        .privacy-policy-text a:hover {
          text-decoration: underline;
        }

        .dark .section-description,
        .dark .privacy-info,
        .dark .privacy-policy-text {
          color: #999;
        }

        .dark .settings-group {
          border-color: #404040;
        }

        .dark .privacy-note {
          background: #1a3a52;
          border-color: #2196f3;
          color: #e0e0e0;
        }

        .dark .data-action-button {
          background: #2a2a2a;
          border-color: #404040;
          color: #e0e0e0;
        }

        .dark .data-action-button:hover {
          background: #353535;
        }

        .dark .data-action-button.danger:hover {
          background: #3d1f1f;
        }
      `}</style>
    </div>
  );
};

export default PrivacySettings;
