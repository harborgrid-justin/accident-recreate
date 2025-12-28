/**
 * Preferences Context
 * React context for managing user preferences
 */

import React, { createContext, useContext, useEffect, useState, useCallback, useRef } from 'react';
import {
  PreferencesContextValue,
  SimplePreferenceValue,
  PreferenceExport,
  SyncStatus,
  ConflictResolution,
} from '../types';

// Create context
const PreferencesContext = createContext<PreferencesContextValue | undefined>(undefined);

/**
 * Preferences Provider Props
 */
interface PreferencesProviderProps {
  children: React.ReactNode;
  apiEndpoint?: string;
  apiKey?: string;
  userId?: string;
  deviceId?: string;
  enableSync?: boolean;
  autoSync?: boolean;
  syncIntervalSeconds?: number;
}

/**
 * Default preferences
 */
const DEFAULT_PREFERENCES: Record<string, SimplePreferenceValue> = {
  'general.language': 'en',
  'general.timezone': 'UTC',
  'appearance.theme': 'light',
  'appearance.fontSize': 14,
  'appearance.compactMode': false,
  'notifications.enabled': true,
  'notifications.sound': true,
  'privacy.analytics': false,
  'privacy.crashReports': true,
  'accessibility.highContrast': false,
  'accessibility.reduceMotion': false,
  'accessibility.screenReader': false,
};

/**
 * Preferences Provider Component
 */
export const PreferencesProvider: React.FC<PreferencesProviderProps> = ({
  children,
  apiEndpoint,
  apiKey,
  userId,
  deviceId = generateDeviceId(),
  enableSync = false,
  autoSync = false,
  syncIntervalSeconds = 300,
}) => {
  const [preferences, setPreferences] = useState<Record<string, SimplePreferenceValue>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | undefined>();
  const [syncStatus, setSyncStatus] = useState<SyncStatus>({
    isSyncing: false,
    pendingCount: 0,
    conflictCount: 0,
  });

  const syncIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const pendingChangesRef = useRef<Record<string, SimplePreferenceValue>>({});

  /**
   * Load preferences from local storage
   */
  const loadPreferences = useCallback(() => {
    try {
      const stored = localStorage.getItem('accuscene_preferences');
      if (stored) {
        const parsed = JSON.parse(stored);
        setPreferences({ ...DEFAULT_PREFERENCES, ...parsed });
      } else {
        setPreferences(DEFAULT_PREFERENCES);
      }
      setLoading(false);
    } catch (err) {
      console.error('Failed to load preferences:', err);
      setError(err as Error);
      setPreferences(DEFAULT_PREFERENCES);
      setLoading(false);
    }
  }, []);

  /**
   * Save preferences to local storage
   */
  const savePreferences = useCallback((prefs: Record<string, SimplePreferenceValue>) => {
    try {
      localStorage.setItem('accuscene_preferences', JSON.stringify(prefs));
    } catch (err) {
      console.error('Failed to save preferences:', err);
      setError(err as Error);
    }
  }, []);

  /**
   * Sync preferences to cloud
   */
  const syncToCloud = useCallback(async (prefs: Record<string, SimplePreferenceValue>) => {
    if (!enableSync || !apiEndpoint || !apiKey) {
      return;
    }

    try {
      setSyncStatus(prev => ({ ...prev, isSyncing: true }));

      const response = await fetch(`${apiEndpoint}/sync`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${apiKey}`,
          'X-Device-ID': deviceId,
          ...(userId && { 'X-User-ID': userId }),
        },
        body: JSON.stringify(prefs),
      });

      if (!response.ok) {
        throw new Error(`Sync failed: ${response.statusText}`);
      }

      setSyncStatus({
        isSyncing: false,
        lastSync: new Date(),
        pendingCount: 0,
        conflictCount: 0,
      });

      pendingChangesRef.current = {};
    } catch (err) {
      console.error('Sync to cloud failed:', err);
      setSyncStatus(prev => ({ ...prev, isSyncing: false }));
    }
  }, [enableSync, apiEndpoint, apiKey, deviceId, userId]);

  /**
   * Pull preferences from cloud
   */
  const pullFromCloud = useCallback(async () => {
    if (!enableSync || !apiEndpoint || !apiKey) {
      return;
    }

    try {
      const response = await fetch(`${apiEndpoint}/preferences`, {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${apiKey}`,
          'X-Device-ID': deviceId,
          ...(userId && { 'X-User-ID': userId }),
        },
      });

      if (!response.ok) {
        throw new Error(`Pull failed: ${response.statusText}`);
      }

      const cloudPrefs = await response.json();

      // Merge with local preferences (cloud takes precedence)
      const merged = { ...preferences, ...cloudPrefs };
      setPreferences(merged);
      savePreferences(merged);

      setSyncStatus(prev => ({ ...prev, lastSync: new Date() }));
    } catch (err) {
      console.error('Pull from cloud failed:', err);
    }
  }, [enableSync, apiEndpoint, apiKey, deviceId, userId, preferences, savePreferences]);

  /**
   * Get a preference value
   */
  const get = useCallback(<T = SimplePreferenceValue>(key: string): T | undefined => {
    return preferences[key] as T | undefined;
  }, [preferences]);

  /**
   * Get a preference value with default
   */
  const getOrDefault = useCallback(<T = SimplePreferenceValue>(key: string): T => {
    return (preferences[key] ?? DEFAULT_PREFERENCES[key]) as T;
  }, [preferences]);

  /**
   * Set a preference value
   */
  const set = useCallback(async (key: string, value: SimplePreferenceValue) => {
    const newPrefs = { ...preferences, [key]: value };
    setPreferences(newPrefs);
    savePreferences(newPrefs);

    // Track pending changes for sync
    if (enableSync) {
      pendingChangesRef.current[key] = value;
      setSyncStatus(prev => ({ ...prev, pendingCount: Object.keys(pendingChangesRef.current).length }));
    }
  }, [preferences, savePreferences, enableSync]);

  /**
   * Set multiple preferences at once
   */
  const setMany = useCallback(async (prefs: Record<string, SimplePreferenceValue>) => {
    const newPrefs = { ...preferences, ...prefs };
    setPreferences(newPrefs);
    savePreferences(newPrefs);

    // Track pending changes for sync
    if (enableSync) {
      Object.assign(pendingChangesRef.current, prefs);
      setSyncStatus(prev => ({ ...prev, pendingCount: Object.keys(pendingChangesRef.current).length }));
    }
  }, [preferences, savePreferences, enableSync]);

  /**
   * Delete a preference
   */
  const deletePreference = useCallback(async (key: string) => {
    const newPrefs = { ...preferences };
    delete newPrefs[key];
    setPreferences(newPrefs);
    savePreferences(newPrefs);
  }, [preferences, savePreferences]);

  /**
   * Reset a preference to default
   */
  const reset = useCallback(async (key: string) => {
    if (DEFAULT_PREFERENCES[key] !== undefined) {
      await set(key, DEFAULT_PREFERENCES[key]);
    } else {
      await deletePreference(key);
    }
  }, [set, deletePreference]);

  /**
   * Reset all preferences to defaults
   */
  const resetAll = useCallback(async () => {
    setPreferences(DEFAULT_PREFERENCES);
    savePreferences(DEFAULT_PREFERENCES);
  }, [savePreferences]);

  /**
   * Export preferences
   */
  const exportPreferences = useCallback(async (): Promise<PreferenceExport> => {
    const categories = Array.from(
      new Set(
        Object.keys(preferences).map(key => {
          const parts = key.split('.');
          return parts.length >= 2 ? parts[0] : 'general';
        })
      )
    );

    return {
      version: '1.0',
      appVersion: '0.2.5',
      exportedAt: new Date(),
      deviceId,
      userId,
      format: 'json' as any,
      preferences: preferences as any,
      metadata: {
        totalCount: Object.keys(preferences).length,
        categories,
        includesSensitive: false,
      },
    };
  }, [preferences, deviceId, userId]);

  /**
   * Import preferences
   */
  const importPreferences = useCallback(async (exported: PreferenceExport) => {
    const imported = exported.preferences as any;
    await setMany(imported);
  }, [setMany]);

  /**
   * Sync now
   */
  const syncNow = useCallback(async () => {
    if (Object.keys(pendingChangesRef.current).length > 0) {
      await syncToCloud(pendingChangesRef.current);
    }
    await pullFromCloud();
  }, [syncToCloud, pullFromCloud]);

  // Load preferences on mount
  useEffect(() => {
    loadPreferences();
  }, [loadPreferences]);

  // Pull from cloud on mount if sync is enabled
  useEffect(() => {
    if (enableSync && !loading) {
      pullFromCloud();
    }
  }, [enableSync, loading, pullFromCloud]);

  // Set up auto-sync interval
  useEffect(() => {
    if (autoSync && enableSync && !loading) {
      syncIntervalRef.current = setInterval(() => {
        if (Object.keys(pendingChangesRef.current).length > 0) {
          syncToCloud(pendingChangesRef.current);
        }
      }, syncIntervalSeconds * 1000);

      return () => {
        if (syncIntervalRef.current) {
          clearInterval(syncIntervalRef.current);
        }
      };
    }
  }, [autoSync, enableSync, loading, syncIntervalSeconds, syncToCloud]);

  const value: PreferencesContextValue = {
    preferences,
    get,
    getOrDefault,
    set,
    setMany,
    delete: deletePreference,
    reset,
    resetAll,
    exportPreferences,
    importPreferences,
    syncNow,
    syncStatus,
    loading,
    error,
  };

  return (
    <PreferencesContext.Provider value={value}>
      {children}
    </PreferencesContext.Provider>
  );
};

/**
 * Hook to use preferences context
 */
export const usePreferencesContext = (): PreferencesContextValue => {
  const context = useContext(PreferencesContext);
  if (!context) {
    throw new Error('usePreferencesContext must be used within a PreferencesProvider');
  }
  return context;
};

/**
 * Generate a unique device ID
 */
function generateDeviceId(): string {
  const stored = localStorage.getItem('accuscene_device_id');
  if (stored) {
    return stored;
  }

  const id = `device_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  localStorage.setItem('accuscene_device_id', id);
  return id;
}
