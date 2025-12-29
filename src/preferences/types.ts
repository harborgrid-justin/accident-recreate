/**
 * AccuScene Enterprise Preferences System
 * TypeScript type definitions
 */

/**
 * Preference value types
 */
export type PreferenceValue =
  | { type: 'string'; value: string }
  | { type: 'integer'; value: number }
  | { type: 'float'; value: number }
  | { type: 'boolean'; value: boolean }
  | { type: 'array'; value: PreferenceValue[] }
  | { type: 'object'; value: Record<string, PreferenceValue> }
  | { type: 'null'; value: null };

/**
 * Simplified preference value for common use
 */
export type SimplePreferenceValue = string | number | boolean | null | SimplePreferenceValue[] | { [key: string]: SimplePreferenceValue };

/**
 * Preference type enum
 */
export enum PreferenceType {
  String = 'string',
  Integer = 'integer',
  Float = 'float',
  Boolean = 'boolean',
  Array = 'array',
  Object = 'object',
  Null = 'null',
}

/**
 * Preference schema definition
 */
export interface PreferenceSchema {
  key: string;
  prefType: PreferenceType;
  defaultValue?: PreferenceValue;
  allowedValues?: PreferenceValue[];
  description: string;
  category: string;
  requiresRestart: boolean;
  sensitive: boolean;
}

/**
 * Preferences configuration
 */
export interface PreferencesConfig {
  enableSync: boolean;
  syncEndpoint?: string;
  syncApiKey?: string;
  storageBackend: StorageBackend;
  storagePath: string;
  enableEncryption: boolean;
  encryptionKey?: string;
  syncIntervalSeconds: number;
  autoSync: boolean;
  maxCacheSize: number;
  enableInheritance: boolean;
  userId?: string;
  deviceId: string;
  appVersion: string;
}

/**
 * Storage backend type
 */
export enum StorageBackend {
  Local = 'local',
  Cloud = 'cloud',
  Hybrid = 'hybrid',
}

/**
 * Preference category
 */
export enum PreferenceCategory {
  General = 'general',
  Appearance = 'appearance',
  Notifications = 'notifications',
  Privacy = 'privacy',
  Accessibility = 'accessibility',
  Keyboard = 'keyboard',
}

/**
 * Theme options
 */
export enum Theme {
  Light = 'light',
  Dark = 'dark',
  Auto = 'auto',
}

/**
 * Language options
 */
export enum Language {
  English = 'en',
  Spanish = 'es',
  French = 'fr',
  German = 'de',
  Japanese = 'ja',
  Chinese = 'zh',
}

/**
 * Sync status
 */
export interface SyncStatus {
  isSyncing: boolean;
  lastSync?: Date;
  pendingCount: number;
  conflictCount: number;
}

/**
 * Sync conflict
 */
export interface SyncConflict {
  key: string;
  localValue: PreferenceValue;
  remoteValue: PreferenceValue;
  localVersion: number;
  remoteVersion: number;
  resolution: ConflictResolution;
}

/**
 * Conflict resolution strategy
 */
export enum ConflictResolution {
  UseLocal = 'use_local',
  UseRemote = 'use_remote',
  UseMostRecent = 'use_most_recent',
  Manual = 'manual',
}

/**
 * Export format
 */
export enum ExportFormat {
  Json = 'json',
  Toml = 'toml',
  Binary = 'binary',
}

/**
 * Preference export
 */
export interface PreferenceExport {
  version: string;
  appVersion: string;
  exportedAt: Date;
  deviceId: string;
  userId?: string;
  format: ExportFormat;
  preferences: Record<string, PreferenceValue>;
  metadata: ExportMetadata;
}

/**
 * Export metadata
 */
export interface ExportMetadata {
  totalCount: number;
  categories: string[];
  includesSensitive: boolean;
  notes?: string;
}

/**
 * Preference context value
 */
export interface PreferencesContextValue {
  // Preferences data
  preferences: Record<string, SimplePreferenceValue>;

  // Getters
  get: <T = SimplePreferenceValue>(key: string) => T | undefined;
  getOrDefault: <T = SimplePreferenceValue>(key: string) => T;

  // Setters
  set: (key: string, value: SimplePreferenceValue) => Promise<void>;
  setMany: (prefs: Record<string, SimplePreferenceValue>) => Promise<void>;

  // Delete
  delete: (key: string) => Promise<void>;
  reset: (key: string) => Promise<void>;
  resetAll: () => Promise<void>;

  // Export/Import
  exportPreferences: () => Promise<PreferenceExport>;
  importPreferences: (exported: PreferenceExport) => Promise<void>;

  // Sync
  syncNow: () => Promise<void>;
  syncStatus: SyncStatus;

  // Loading state
  loading: boolean;
  error?: Error;
}

/**
 * Keyboard shortcut
 */
export interface KeyboardShortcut {
  id: string;
  name: string;
  description: string;
  defaultKeys: string[];
  currentKeys: string[];
  category: string;
  editable: boolean;
}

/**
 * Notification preference
 */
export interface NotificationPreference {
  type: string;
  enabled: boolean;
  sound: boolean;
  desktop: boolean;
  email: boolean;
}

/**
 * Privacy preference
 */
export interface PrivacyPreference {
  analytics: boolean;
  crashReports: boolean;
  diagnostics: boolean;
  thirdPartyData: boolean;
}

/**
 * Accessibility preference
 */
export interface AccessibilityPreference {
  highContrast: boolean;
  reduceMotion: boolean;
  screenReader: boolean;
  fontSize: number;
  keyboardNavigation: boolean;
}

/**
 * Appearance preference
 */
export interface AppearancePreference {
  theme: Theme;
  fontSize: number;
  fontFamily: string;
  compactMode: boolean;
  showAnimations: boolean;
  colorScheme?: string;
}

/**
 * General preference
 */
export interface GeneralPreference {
  language: Language;
  timezone: string;
  dateFormat: string;
  timeFormat: string;
  startupBehavior: string;
}
