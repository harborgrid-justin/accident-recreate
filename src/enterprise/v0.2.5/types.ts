/**
 * Unified types for AccuScene Enterprise v0.2.5
 *
 * This module provides comprehensive type definitions for all v0.2.5
 * enterprise features and integrations.
 *
 * @module enterprise/v0.2.5/types
 * @version 0.2.5
 */

// ============================================================================
// Core Types
// ============================================================================

/**
 * Enterprise version information
 */
export interface VersionInfo {
  version: string;
  buildDate: string;
  gitHash?: string;
  environment: Environment;
}

/**
 * Environment type
 */
export type Environment = 'development' | 'staging' | 'production';

/**
 * Log level
 */
export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

// ============================================================================
// Configuration Types
// ============================================================================

/**
 * Main enterprise configuration
 */
export interface EnterpriseConfig {
  app: AppConfig;
  database: DatabaseConfig;
  cache: CacheConfig;
  security: SecurityConfig;
  analytics: AnalyticsConfig;
  cluster: ClusterConfig;
  telemetry: TelemetryConfig;
  ux: UxConfig;
}

/**
 * Application configuration
 */
export interface AppConfig {
  name: string;
  version: string;
  environment: Environment;
  debug: boolean;
  logLevel: LogLevel;
  dataDir: string;
  tempDir: string;
}

/**
 * Database configuration
 */
export interface DatabaseConfig {
  url: string;
  maxConnections: number;
  minConnections: number;
  connectTimeout: number;
  autoMigrate: boolean;
}

/**
 * Cache configuration
 */
export interface CacheConfig {
  enabled: boolean;
  memorySizeMb: number;
  diskSizeMb: number;
  ttlSeconds: number;
  redisEnabled: boolean;
  redisUrl?: string;
}

/**
 * Security configuration
 */
export interface SecurityConfig {
  authEnabled: boolean;
  jwtSecret: string;
  jwtExpiration: number;
  ssoEnabled: boolean;
  ssoProvider?: string;
  encryptionEnabled: boolean;
  auditEnabled: boolean;
}

/**
 * Analytics configuration
 */
export interface AnalyticsConfig {
  enabled: boolean;
  bufferSize: number;
  batchSize: number;
  flushInterval: number;
  realtimeEnabled: boolean;
}

/**
 * Cluster configuration
 */
export interface ClusterConfig {
  enabled: boolean;
  nodeId: string;
  nodes: string[];
  port: number;
  autoDiscovery: boolean;
  heartbeatInterval: number;
}

/**
 * Telemetry configuration
 */
export interface TelemetryConfig {
  enabled: boolean;
  exportInterval: number;
  prometheusEnabled: boolean;
  prometheusPort: number;
  otelEnabled: boolean;
  otelEndpoint?: string;
}

/**
 * User experience configuration (v0.2.5)
 */
export interface UxConfig {
  accessibility: AccessibilityConfig;
  dashboard: DashboardConfig;
  gestures: GesturesConfig;
  notifications: NotificationsConfig;
  offline: OfflineConfig;
  preferences: PreferencesConfig;
  search: SearchConfig;
  visualization: VisualizationConfig;
}

/**
 * Accessibility configuration
 */
export interface AccessibilityConfig {
  enabled: boolean;
  screenReader: boolean;
  highContrast: boolean;
  keyboardNav: boolean;
}

/**
 * Dashboard configuration
 */
export interface DashboardConfig {
  enabled: boolean;
  refreshInterval: number;
  maxWidgets: number;
}

/**
 * Gestures configuration
 */
export interface GesturesConfig {
  enabled: boolean;
  sensitivity: number;
  multitouch: boolean;
}

/**
 * Notifications configuration
 */
export interface NotificationsConfig {
  enabled: boolean;
  maxNotifications: number;
  pushEnabled: boolean;
  pushUrl?: string;
}

/**
 * Offline configuration
 */
export interface OfflineConfig {
  enabled: boolean;
  storageSizeMb: number;
  autoSync: boolean;
  syncInterval: number;
}

/**
 * Preferences configuration
 */
export interface PreferencesConfig {
  enabled: boolean;
  backend: string;
  cloudSync: boolean;
}

/**
 * Search configuration
 */
export interface SearchConfig {
  enabled: boolean;
  indexSizeMb: number;
  fuzzyEnabled: boolean;
  maxResults: number;
}

/**
 * Visualization configuration
 */
export interface VisualizationConfig {
  enabled: boolean;
  backend: 'webgl' | 'webgpu' | 'canvas';
  hardwareAccel: boolean;
  maxChartPoints: number;
}

// ============================================================================
// Service Types
// ============================================================================

/**
 * Service status
 */
export type ServiceStatus = 'initializing' | 'running' | 'stopped' | 'error';

/**
 * Service descriptor
 */
export interface ServiceDescriptor {
  name: string;
  description: string;
  version: string;
  status: ServiceStatus;
  registeredAt: Date;
  updatedAt: Date;
  metadata: Record<string, any>;
}

/**
 * Service registry statistics
 */
export interface RegistryStats {
  total: number;
  running: number;
  stopped: number;
  error: number;
  initializing: number;
}

// ============================================================================
// Health Check Types
// ============================================================================

/**
 * Health status
 */
export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unknown';

/**
 * Health check
 */
export interface HealthCheck {
  service: string;
  status: HealthStatus;
  message?: string;
  checkedAt: Date;
  responseTimeMs: number;
  details: Record<string, any>;
}

/**
 * Overall health
 */
export interface OverallHealth {
  status: HealthStatus;
  totalServices: number;
  healthyServices: number;
  degradedServices: number;
  unhealthyServices: number;
  unknownServices: number;
  checks: HealthCheck[];
}

// ============================================================================
// Event Types
// ============================================================================

/**
 * Event metadata
 */
export interface EventMetadata {
  id: string;
  eventType: string;
  timestamp: Date;
  source?: string;
  correlationId?: string;
  custom: Record<string, any>;
}

/**
 * Base event interface
 */
export interface BaseEvent {
  metadata: EventMetadata;
}

/**
 * System event
 */
export interface SystemEvent extends BaseEvent {
  name: string;
  payload: any;
}

/**
 * Service action
 */
export type ServiceAction = 'started' | 'stopped' | 'error' | 'healthy' | 'unhealthy';

/**
 * Service event
 */
export interface ServiceEvent extends BaseEvent {
  service: string;
  action: ServiceAction;
  errorMessage?: string;
}

// ============================================================================
// UX Feature Types (v0.2.5)
// ============================================================================

/**
 * Notification
 */
export interface Notification {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  timestamp: Date;
  read: boolean;
  actions?: NotificationAction[];
}

/**
 * Notification action
 */
export interface NotificationAction {
  label: string;
  handler: () => void;
}

/**
 * User preference
 */
export interface UserPreference {
  key: string;
  value: any;
  type: 'string' | 'number' | 'boolean' | 'object';
  category: string;
  updatedAt: Date;
}

/**
 * Search result
 */
export interface SearchResult {
  id: string;
  type: string;
  title: string;
  description?: string;
  score: number;
  highlights?: string[];
  metadata: Record<string, any>;
}

/**
 * Dashboard widget
 */
export interface DashboardWidget {
  id: string;
  type: string;
  title: string;
  position: { x: number; y: number };
  size: { width: number; height: number };
  config: Record<string, any>;
  refreshInterval?: number;
}

/**
 * Gesture event
 */
export interface GestureEvent {
  type: 'tap' | 'swipe' | 'pinch' | 'rotate' | 'longpress';
  x: number;
  y: number;
  deltaX?: number;
  deltaY?: number;
  scale?: number;
  rotation?: number;
  timestamp: Date;
}

/**
 * Offline sync status
 */
export interface OfflineSyncStatus {
  online: boolean;
  lastSync: Date;
  pendingChanges: number;
  syncing: boolean;
  error?: string;
}

// ============================================================================
// System Information Types
// ============================================================================

/**
 * System information
 */
export interface SystemInfo {
  version: string;
  environment: string;
  servicesCount: number;
  coreReady: boolean;
  uxReady: boolean;
  productionReady: boolean;
}

/**
 * Feature flags
 */
export interface FeatureFlags {
  graphql: boolean;
  collaboration: boolean;
  plugins: boolean;
  monitoring: boolean;
  accessibility: boolean;
  dashboard: boolean;
  gestures: boolean;
  notifications: boolean;
  offline: boolean;
  preferences: boolean;
  search: boolean;
  visualization: boolean;
  sso: boolean;
}

// ============================================================================
// Error Types
// ============================================================================

/**
 * Enterprise error
 */
export interface EnterpriseError {
  code: string;
  message: string;
  details?: any;
  stack?: string;
}

/**
 * Configuration error
 */
export interface ConfigError extends EnterpriseError {
  field?: string;
  value?: any;
}

/**
 * Service error
 */
export interface ServiceError extends EnterpriseError {
  service: string;
  operation?: string;
}

// ============================================================================
// Callback Types
// ============================================================================

/**
 * Event handler callback
 */
export type EventHandler<T = any> = (event: T) => void | Promise<void>;

/**
 * Error handler callback
 */
export type ErrorHandler = (error: Error | EnterpriseError) => void;

/**
 * State change handler
 */
export type StateChangeHandler<T = any> = (newState: T, oldState: T) => void;

// ============================================================================
// Provider Context Types
// ============================================================================

/**
 * Enterprise context
 */
export interface EnterpriseContext {
  config: EnterpriseConfig;
  services: ServiceDescriptor[];
  health: OverallHealth;
  features: FeatureFlags;
  systemInfo: SystemInfo;
  initialized: boolean;
  loading: boolean;
  error?: EnterpriseError;
}

/**
 * Enterprise actions
 */
export interface EnterpriseActions {
  initialize: () => Promise<void>;
  shutdown: () => Promise<void>;
  getService: (name: string) => ServiceDescriptor | undefined;
  updateConfig: (config: Partial<EnterpriseConfig>) => Promise<void>;
  checkHealth: () => Promise<OverallHealth>;
  addEventListener: (type: string, handler: EventHandler) => void;
  removeEventListener: (type: string, handler: EventHandler) => void;
}
