/**
 * SettingsPage - User settings and preferences
 */

import React, { useState } from 'react';
import { useAuth } from '../hooks/useAuth';
import { useUIStore } from '../store/uiStore';
import Input from '../components/Common/Input';
import Select from '../components/Common/Select';
import Button from '../components/Common/Button';

export const SettingsPage: React.FC = () => {
  const { user } = useAuth();
  const { theme, setTheme, showNotification } = useUIStore();

  const [formData, setFormData] = useState({
    firstName: user?.firstName || '',
    lastName: user?.lastName || '',
    email: user?.email || '',
    department: '',
    phoneNumber: '',
  });

  const [passwordData, setPasswordData] = useState({
    currentPassword: '',
    newPassword: '',
    confirmPassword: '',
  });

  const [isSaving, setIsSaving] = useState(false);
  const [isChangingPassword, setIsChangingPassword] = useState(false);

  const handleProfileUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true);

    // Simulate API call
    setTimeout(() => {
      setIsSaving(false);
      showNotification('success', 'Profile updated successfully');
    }, 1000);
  };

  const handlePasswordChange = async (e: React.FormEvent) => {
    e.preventDefault();

    if (passwordData.newPassword !== passwordData.confirmPassword) {
      showNotification('error', 'New passwords do not match');
      return;
    }

    if (passwordData.newPassword.length < 8) {
      showNotification('error', 'Password must be at least 8 characters');
      return;
    }

    setIsChangingPassword(true);

    // Simulate API call
    setTimeout(() => {
      setIsChangingPassword(false);
      setPasswordData({ currentPassword: '', newPassword: '', confirmPassword: '' });
      showNotification('success', 'Password changed successfully');
    }, 1000);
  };

  return (
    <div className="page settings-page">
      <div className="page-header">
        <div className="page-title-section">
          <h1 className="page-title">Settings</h1>
          <p className="page-subtitle">Manage your account and preferences</p>
        </div>
      </div>

      <div className="settings-content">
        <div className="settings-section">
          <h2>Profile Information</h2>
          <form onSubmit={handleProfileUpdate}>
            <div className="settings-form-row">
              <Input
                label="First Name"
                value={formData.firstName}
                onChange={(e) => setFormData({ ...formData, firstName: e.target.value })}
                fullWidth
              />
              <Input
                label="Last Name"
                value={formData.lastName}
                onChange={(e) => setFormData({ ...formData, lastName: e.target.value })}
                fullWidth
              />
            </div>

            <div className="settings-form-row">
              <Input
                label="Email"
                type="email"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                fullWidth
                disabled
              />
            </div>

            <div className="settings-form-row">
              <Input
                label="Department"
                value={formData.department}
                onChange={(e) => setFormData({ ...formData, department: e.target.value })}
                fullWidth
              />
              <Input
                label="Phone Number"
                type="tel"
                value={formData.phoneNumber}
                onChange={(e) => setFormData({ ...formData, phoneNumber: e.target.value })}
                fullWidth
              />
            </div>

            <div className="settings-form-row">
              <div className="settings-form-info">
                <p>
                  <strong>Role:</strong> {user?.role}
                </p>
                <p>
                  <strong>User ID:</strong> {user?.id}
                </p>
              </div>
            </div>

            <div className="settings-form-actions">
              <Button type="submit" variant="primary" loading={isSaving}>
                Save Changes
              </Button>
            </div>
          </form>
        </div>

        <div className="settings-section">
          <h2>Change Password</h2>
          <form onSubmit={handlePasswordChange}>
            <div className="settings-form-row">
              <Input
                label="Current Password"
                type="password"
                value={passwordData.currentPassword}
                onChange={(e) =>
                  setPasswordData({ ...passwordData, currentPassword: e.target.value })
                }
                fullWidth
                required
              />
            </div>

            <div className="settings-form-row">
              <Input
                label="New Password"
                type="password"
                value={passwordData.newPassword}
                onChange={(e) => setPasswordData({ ...passwordData, newPassword: e.target.value })}
                helperText="Must be at least 8 characters"
                fullWidth
                required
              />
            </div>

            <div className="settings-form-row">
              <Input
                label="Confirm New Password"
                type="password"
                value={passwordData.confirmPassword}
                onChange={(e) =>
                  setPasswordData({ ...passwordData, confirmPassword: e.target.value })
                }
                fullWidth
                required
              />
            </div>

            <div className="settings-form-actions">
              <Button type="submit" variant="primary" loading={isChangingPassword}>
                Change Password
              </Button>
            </div>
          </form>
        </div>

        <div className="settings-section">
          <h2>Appearance</h2>
          <div className="settings-appearance">
            <Select
              label="Theme"
              value={theme}
              onChange={(e) => setTheme(e.target.value as 'light' | 'dark')}
              options={[
                { value: 'light', label: 'Light Mode' },
                { value: 'dark', label: 'Dark Mode' },
              ]}
            />
          </div>
        </div>

        <div className="settings-section">
          <h2>Application Information</h2>
          <div className="settings-info">
            <div className="settings-info-item">
              <label>Application:</label>
              <span>AccuScene Enterprise</span>
            </div>
            <div className="settings-info-item">
              <label>Version:</label>
              <span>1.0.0</span>
            </div>
            <div className="settings-info-item">
              <label>Environment:</label>
              <span>Production</span>
            </div>
            <div className="settings-info-item">
              <label>API Status:</label>
              <span className="status-badge status-active">Connected</span>
            </div>
          </div>
        </div>

        <div className="settings-section settings-danger">
          <h2>Danger Zone</h2>
          <div className="settings-danger-content">
            <div className="settings-danger-item">
              <div>
                <h4>Clear Local Cache</h4>
                <p>Remove all cached data from your local storage</p>
              </div>
              <Button variant="outline" size="small">
                Clear Cache
              </Button>
            </div>
            <div className="settings-danger-item">
              <div>
                <h4>Export Data</h4>
                <p>Download all your case data in JSON format</p>
              </div>
              <Button variant="outline" size="small">
                Export Data
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsPage;
