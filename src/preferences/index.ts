/**
 * AccuScene Enterprise Preferences System
 * Main exports
 */

// Types
export * from './types';

// Context
export { PreferencesProvider, usePreferencesContext } from './context/PreferencesContext';

// Hooks
export { usePreferences } from './hooks/usePreferences';

// Components
export { default as SettingsPage } from './SettingsPage';
export { default as SettingToggle } from './components/SettingToggle';
export { default as SettingSelect } from './components/SettingSelect';
export { default as SettingSlider } from './components/SettingSlider';

// Sections
export { default as GeneralSettings } from './sections/GeneralSettings';
export { default as AppearanceSettings } from './sections/AppearanceSettings';
export { default as NotificationSettings } from './sections/NotificationSettings';
export { default as PrivacySettings } from './sections/PrivacySettings';
export { default as KeyboardShortcuts } from './sections/KeyboardShortcuts';
export { default as AccessibilitySettings } from './sections/AccessibilitySettings';
