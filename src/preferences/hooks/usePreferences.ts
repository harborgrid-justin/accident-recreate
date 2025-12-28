/**
 * usePreferences Hook
 * Custom hook for accessing and managing user preferences
 */

import { useCallback, useMemo } from 'react';
import { usePreferencesContext } from '../context/PreferencesContext';
import {
  Theme,
  Language,
  AppearancePreference,
  GeneralPreference,
  NotificationPreference,
  PrivacyPreference,
  AccessibilityPreference,
  SimplePreferenceValue,
} from '../types';

/**
 * Preferences hook
 */
export const usePreferences = () => {
  const context = usePreferencesContext();

  /**
   * Get general preferences
   */
  const general = useMemo((): GeneralPreference => ({
    language: (context.get('general.language') as Language) || Language.English,
    timezone: (context.get('general.timezone') as string) || 'UTC',
    dateFormat: (context.get('general.dateFormat') as string) || 'YYYY-MM-DD',
    timeFormat: (context.get('general.timeFormat') as string) || '24h',
    startupBehavior: (context.get('general.startupBehavior') as string) || 'last_session',
  }), [context]);

  /**
   * Get appearance preferences
   */
  const appearance = useMemo((): AppearancePreference => ({
    theme: (context.get('appearance.theme') as Theme) || Theme.Light,
    fontSize: (context.get('appearance.fontSize') as number) || 14,
    fontFamily: (context.get('appearance.fontFamily') as string) || 'system-ui',
    compactMode: (context.get('appearance.compactMode') as boolean) || false,
    showAnimations: (context.get('appearance.showAnimations') as boolean) ?? true,
    colorScheme: context.get('appearance.colorScheme') as string | undefined,
  }), [context]);

  /**
   * Get notification preferences
   */
  const notifications = useMemo((): NotificationPreference => ({
    type: 'all',
    enabled: (context.get('notifications.enabled') as boolean) ?? true,
    sound: (context.get('notifications.sound') as boolean) ?? true,
    desktop: (context.get('notifications.desktop') as boolean) ?? true,
    email: (context.get('notifications.email') as boolean) ?? false,
  }), [context]);

  /**
   * Get privacy preferences
   */
  const privacy = useMemo((): PrivacyPreference => ({
    analytics: (context.get('privacy.analytics') as boolean) || false,
    crashReports: (context.get('privacy.crashReports') as boolean) ?? true,
    diagnostics: (context.get('privacy.diagnostics') as boolean) ?? true,
    thirdPartyData: (context.get('privacy.thirdPartyData') as boolean) || false,
  }), [context]);

  /**
   * Get accessibility preferences
   */
  const accessibility = useMemo((): AccessibilityPreference => ({
    highContrast: (context.get('accessibility.highContrast') as boolean) || false,
    reduceMotion: (context.get('accessibility.reduceMotion') as boolean) || false,
    screenReader: (context.get('accessibility.screenReader') as boolean) || false,
    fontSize: (context.get('accessibility.fontSize') as number) || 14,
    keyboardNavigation: (context.get('accessibility.keyboardNavigation') as boolean) ?? true,
  }), [context]);

  /**
   * Update general preference
   */
  const updateGeneral = useCallback(async (key: keyof GeneralPreference, value: SimplePreferenceValue) => {
    await context.set(`general.${key}`, value);
  }, [context]);

  /**
   * Update appearance preference
   */
  const updateAppearance = useCallback(async (key: keyof AppearancePreference, value: SimplePreferenceValue) => {
    await context.set(`appearance.${key}`, value);
  }, [context]);

  /**
   * Update notification preference
   */
  const updateNotifications = useCallback(async (key: keyof NotificationPreference, value: SimplePreferenceValue) => {
    await context.set(`notifications.${key}`, value);
  }, [context]);

  /**
   * Update privacy preference
   */
  const updatePrivacy = useCallback(async (key: keyof PrivacyPreference, value: SimplePreferenceValue) => {
    await context.set(`privacy.${key}`, value);
  }, [context]);

  /**
   * Update accessibility preference
   */
  const updateAccessibility = useCallback(async (key: keyof AccessibilityPreference, value: SimplePreferenceValue) => {
    await context.set(`accessibility.${key}`, value);
  }, [context]);

  /**
   * Toggle theme
   */
  const toggleTheme = useCallback(async () => {
    const currentTheme = appearance.theme;
    const newTheme = currentTheme === Theme.Light ? Theme.Dark : Theme.Light;
    await updateAppearance('theme', newTheme);
  }, [appearance.theme, updateAppearance]);

  /**
   * Increase font size
   */
  const increaseFontSize = useCallback(async () => {
    const current = appearance.fontSize;
    const newSize = Math.min(current + 2, 32);
    await updateAppearance('fontSize', newSize);
  }, [appearance.fontSize, updateAppearance]);

  /**
   * Decrease font size
   */
  const decreaseFontSize = useCallback(async () => {
    const current = appearance.fontSize;
    const newSize = Math.max(current - 2, 8);
    await updateAppearance('fontSize', newSize);
  }, [appearance.fontSize, updateAppearance]);

  /**
   * Reset font size to default
   */
  const resetFontSize = useCallback(async () => {
    await updateAppearance('fontSize', 14);
  }, [updateAppearance]);

  /**
   * Apply theme to document
   */
  const applyTheme = useCallback(() => {
    const theme = appearance.theme;
    const root = document.documentElement;

    if (theme === Theme.Dark) {
      root.classList.add('dark');
      root.classList.remove('light');
    } else if (theme === Theme.Light) {
      root.classList.add('light');
      root.classList.remove('dark');
    } else if (theme === Theme.Auto) {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      if (prefersDark) {
        root.classList.add('dark');
        root.classList.remove('light');
      } else {
        root.classList.add('light');
        root.classList.remove('dark');
      }
    }

    // Apply font size
    root.style.fontSize = `${appearance.fontSize}px`;

    // Apply reduce motion
    if (accessibility.reduceMotion) {
      root.style.setProperty('--animation-duration', '0.001ms');
    } else {
      root.style.removeProperty('--animation-duration');
    }

    // Apply high contrast
    if (accessibility.highContrast) {
      root.classList.add('high-contrast');
    } else {
      root.classList.remove('high-contrast');
    }
  }, [appearance.theme, appearance.fontSize, accessibility.reduceMotion, accessibility.highContrast]);

  return {
    // Raw context
    ...context,

    // Categorized preferences
    general,
    appearance,
    notifications,
    privacy,
    accessibility,

    // Update methods
    updateGeneral,
    updateAppearance,
    updateNotifications,
    updatePrivacy,
    updateAccessibility,

    // Convenience methods
    toggleTheme,
    increaseFontSize,
    decreaseFontSize,
    resetFontSize,
    applyTheme,
  };
};
