/**
 * Enterprise configuration for AccuScene v0.2.5
 *
 * This module provides default configuration and configuration utilities
 * for the AccuScene Enterprise platform.
 *
 * @module enterprise/v0.2.5/config
 * @version 0.2.5
 */

import type {
  EnterpriseConfig,
  AppConfig,
  DatabaseConfig,
  CacheConfig,
  SecurityConfig,
  AnalyticsConfig,
  ClusterConfig,
  TelemetryConfig,
  UxConfig,
  AccessibilityConfig,
  DashboardConfig,
  GesturesConfig,
  NotificationsConfig,
  OfflineConfig,
  PreferencesConfig,
  SearchConfig,
  VisualizationConfig,
  Environment,
  LogLevel,
} from './types';

// ============================================================================
// Default Configurations
// ============================================================================

/**
 * Default application configuration
 */
export const defaultAppConfig: AppConfig = {
  name: 'AccuScene Enterprise',
  version: '0.2.5',
  environment: (process.env.NODE_ENV as Environment) || 'development',
  debug: process.env.NODE_ENV !== 'production',
  logLevel: (process.env.LOG_LEVEL as LogLevel) || 'info',
  dataDir: process.env.DATA_DIR || './data',
  tempDir: process.env.TEMP_DIR || './tmp',
};

/**
 * Default database configuration
 */
export const defaultDatabaseConfig: DatabaseConfig = {
  url: process.env.DATABASE_URL || 'sqlite://./data/accuscene.db',
  maxConnections: parseInt(process.env.DB_MAX_CONNECTIONS || '10', 10),
  minConnections: parseInt(process.env.DB_MIN_CONNECTIONS || '2', 10),
  connectTimeout: parseInt(process.env.DB_CONNECT_TIMEOUT || '30', 10),
  autoMigrate: process.env.DB_AUTO_MIGRATE !== 'false',
};

/**
 * Default cache configuration
 */
export const defaultCacheConfig: CacheConfig = {
  enabled: process.env.CACHE_ENABLED !== 'false',
  memorySizeMb: parseInt(process.env.CACHE_MEMORY_SIZE || '512', 10),
  diskSizeMb: parseInt(process.env.CACHE_DISK_SIZE || '2048', 10),
  ttlSeconds: parseInt(process.env.CACHE_TTL || '3600', 10),
  redisEnabled: process.env.REDIS_ENABLED === 'true',
  redisUrl: process.env.REDIS_URL,
};

/**
 * Default security configuration
 */
export const defaultSecurityConfig: SecurityConfig = {
  authEnabled: process.env.AUTH_ENABLED !== 'false',
  jwtSecret: process.env.JWT_SECRET || 'change-me-in-production',
  jwtExpiration: parseInt(process.env.JWT_EXPIRATION || '86400', 10),
  ssoEnabled: process.env.SSO_ENABLED === 'true',
  ssoProvider: process.env.SSO_PROVIDER,
  encryptionEnabled: process.env.ENCRYPTION_ENABLED !== 'false',
  auditEnabled: process.env.AUDIT_ENABLED !== 'false',
};

/**
 * Default analytics configuration
 */
export const defaultAnalyticsConfig: AnalyticsConfig = {
  enabled: process.env.ANALYTICS_ENABLED !== 'false',
  bufferSize: parseInt(process.env.ANALYTICS_BUFFER_SIZE || '10000', 10),
  batchSize: parseInt(process.env.ANALYTICS_BATCH_SIZE || '100', 10),
  flushInterval: parseInt(process.env.ANALYTICS_FLUSH_INTERVAL || '60', 10),
  realtimeEnabled: process.env.ANALYTICS_REALTIME !== 'false',
};

/**
 * Default cluster configuration
 */
export const defaultClusterConfig: ClusterConfig = {
  enabled: process.env.CLUSTER_ENABLED === 'true',
  nodeId: process.env.CLUSTER_NODE_ID || crypto.randomUUID(),
  nodes: process.env.CLUSTER_NODES?.split(',') || [],
  port: parseInt(process.env.CLUSTER_PORT || '7946', 10),
  autoDiscovery: process.env.CLUSTER_AUTO_DISCOVERY !== 'false',
  heartbeatInterval: parseInt(process.env.CLUSTER_HEARTBEAT || '30', 10),
};

/**
 * Default telemetry configuration
 */
export const defaultTelemetryConfig: TelemetryConfig = {
  enabled: process.env.TELEMETRY_ENABLED !== 'false',
  exportInterval: parseInt(process.env.TELEMETRY_EXPORT_INTERVAL || '60', 10),
  prometheusEnabled: process.env.PROMETHEUS_ENABLED !== 'false',
  prometheusPort: parseInt(process.env.PROMETHEUS_PORT || '9090', 10),
  otelEnabled: process.env.OTEL_ENABLED === 'true',
  otelEndpoint: process.env.OTEL_ENDPOINT,
};

/**
 * Default accessibility configuration
 */
export const defaultAccessibilityConfig: AccessibilityConfig = {
  enabled: process.env.A11Y_ENABLED !== 'false',
  screenReader: process.env.A11Y_SCREEN_READER !== 'false',
  highContrast: process.env.A11Y_HIGH_CONTRAST === 'true',
  keyboardNav: process.env.A11Y_KEYBOARD_NAV !== 'false',
};

/**
 * Default dashboard configuration
 */
export const defaultDashboardConfig: DashboardConfig = {
  enabled: process.env.DASHBOARD_ENABLED !== 'false',
  refreshInterval: parseInt(process.env.DASHBOARD_REFRESH || '30', 10),
  maxWidgets: parseInt(process.env.DASHBOARD_MAX_WIDGETS || '20', 10),
};

/**
 * Default gestures configuration
 */
export const defaultGesturesConfig: GesturesConfig = {
  enabled: process.env.GESTURES_ENABLED !== 'false',
  sensitivity: parseFloat(process.env.GESTURES_SENSITIVITY || '0.8'),
  multitouch: process.env.GESTURES_MULTITOUCH !== 'false',
};

/**
 * Default notifications configuration
 */
export const defaultNotificationsConfig: NotificationsConfig = {
  enabled: process.env.NOTIFICATIONS_ENABLED !== 'false',
  maxNotifications: parseInt(process.env.NOTIFICATIONS_MAX || '100', 10),
  pushEnabled: process.env.PUSH_ENABLED === 'true',
  pushUrl: process.env.PUSH_URL,
};

/**
 * Default offline configuration
 */
export const defaultOfflineConfig: OfflineConfig = {
  enabled: process.env.OFFLINE_ENABLED !== 'false',
  storageSizeMb: parseInt(process.env.OFFLINE_STORAGE_SIZE || '1024', 10),
  autoSync: process.env.OFFLINE_AUTO_SYNC !== 'false',
  syncInterval: parseInt(process.env.OFFLINE_SYNC_INTERVAL || '300', 10),
};

/**
 * Default preferences configuration
 */
export const defaultPreferencesConfig: PreferencesConfig = {
  enabled: process.env.PREFERENCES_ENABLED !== 'false',
  backend: process.env.PREFERENCES_BACKEND || 'sqlite',
  cloudSync: process.env.PREFERENCES_CLOUD_SYNC === 'true',
};

/**
 * Default search configuration
 */
export const defaultSearchConfig: SearchConfig = {
  enabled: process.env.SEARCH_ENABLED !== 'false',
  indexSizeMb: parseInt(process.env.SEARCH_INDEX_SIZE || '256', 10),
  fuzzyEnabled: process.env.SEARCH_FUZZY !== 'false',
  maxResults: parseInt(process.env.SEARCH_MAX_RESULTS || '100', 10),
};

/**
 * Default visualization configuration
 */
export const defaultVisualizationConfig: VisualizationConfig = {
  enabled: process.env.VISUALIZATION_ENABLED !== 'false',
  backend: (process.env.VISUALIZATION_BACKEND as any) || 'webgl',
  hardwareAccel: process.env.VISUALIZATION_HW_ACCEL !== 'false',
  maxChartPoints: parseInt(process.env.VISUALIZATION_MAX_POINTS || '10000', 10),
};

/**
 * Default UX configuration
 */
export const defaultUxConfig: UxConfig = {
  accessibility: defaultAccessibilityConfig,
  dashboard: defaultDashboardConfig,
  gestures: defaultGesturesConfig,
  notifications: defaultNotificationsConfig,
  offline: defaultOfflineConfig,
  preferences: defaultPreferencesConfig,
  search: defaultSearchConfig,
  visualization: defaultVisualizationConfig,
};

/**
 * Default enterprise configuration
 */
export const defaultEnterpriseConfig: EnterpriseConfig = {
  app: defaultAppConfig,
  database: defaultDatabaseConfig,
  cache: defaultCacheConfig,
  security: defaultSecurityConfig,
  analytics: defaultAnalyticsConfig,
  cluster: defaultClusterConfig,
  telemetry: defaultTelemetryConfig,
  ux: defaultUxConfig,
};

// ============================================================================
// Configuration Utilities
// ============================================================================

/**
 * Merge configurations with deep merge support
 */
export function mergeConfigs(
  base: Partial<EnterpriseConfig>,
  override: Partial<EnterpriseConfig>
): EnterpriseConfig {
  return {
    app: { ...defaultAppConfig, ...base.app, ...override.app },
    database: { ...defaultDatabaseConfig, ...base.database, ...override.database },
    cache: { ...defaultCacheConfig, ...base.cache, ...override.cache },
    security: { ...defaultSecurityConfig, ...base.security, ...override.security },
    analytics: { ...defaultAnalyticsConfig, ...base.analytics, ...override.analytics },
    cluster: { ...defaultClusterConfig, ...base.cluster, ...override.cluster },
    telemetry: { ...defaultTelemetryConfig, ...base.telemetry, ...override.telemetry },
    ux: {
      accessibility: {
        ...defaultAccessibilityConfig,
        ...base.ux?.accessibility,
        ...override.ux?.accessibility,
      },
      dashboard: {
        ...defaultDashboardConfig,
        ...base.ux?.dashboard,
        ...override.ux?.dashboard,
      },
      gestures: {
        ...defaultGesturesConfig,
        ...base.ux?.gestures,
        ...override.ux?.gestures,
      },
      notifications: {
        ...defaultNotificationsConfig,
        ...base.ux?.notifications,
        ...override.ux?.notifications,
      },
      offline: {
        ...defaultOfflineConfig,
        ...base.ux?.offline,
        ...override.ux?.offline,
      },
      preferences: {
        ...defaultPreferencesConfig,
        ...base.ux?.preferences,
        ...override.ux?.preferences,
      },
      search: {
        ...defaultSearchConfig,
        ...base.ux?.search,
        ...override.ux?.search,
      },
      visualization: {
        ...defaultVisualizationConfig,
        ...base.ux?.visualization,
        ...override.ux?.visualization,
      },
    },
  };
}

/**
 * Validate configuration
 */
export function validateConfig(config: EnterpriseConfig): string[] {
  const errors: string[] = [];

  // Validate security settings
  if (
    config.app.environment === 'production' &&
    config.security.authEnabled &&
    config.security.jwtSecret === 'change-me-in-production'
  ) {
    errors.push('JWT secret must be changed in production environment');
  }

  // Validate database settings
  if (config.database.maxConnections < config.database.minConnections) {
    errors.push('Database max connections must be >= min connections');
  }

  // Validate cache settings
  if (config.cache.redisEnabled && !config.cache.redisUrl) {
    errors.push('Redis URL is required when Redis is enabled');
  }

  // Validate cluster settings
  if (config.cluster.enabled && config.cluster.nodes.length === 0 && !config.cluster.autoDiscovery) {
    errors.push('Cluster nodes must be specified when auto-discovery is disabled');
  }

  // Validate SSO settings
  if (config.security.ssoEnabled && !config.security.ssoProvider) {
    errors.push('SSO provider is required when SSO is enabled');
  }

  // Validate telemetry settings
  if (config.telemetry.otelEnabled && !config.telemetry.otelEndpoint) {
    errors.push('OpenTelemetry endpoint is required when OTEL is enabled');
  }

  return errors;
}

/**
 * Load configuration from environment
 */
export function loadConfigFromEnv(): EnterpriseConfig {
  return defaultEnterpriseConfig;
}

/**
 * Check if running in production
 */
export function isProduction(config: EnterpriseConfig): boolean {
  return config.app.environment === 'production';
}

/**
 * Check if debug mode is enabled
 */
export function isDebug(config: EnterpriseConfig): boolean {
  return config.app.debug;
}

/**
 * Get feature flags from configuration
 */
export function getFeatureFlags(config: EnterpriseConfig) {
  return {
    graphql: true, // From v0.2.0
    collaboration: true, // From v0.2.0
    plugins: true, // From v0.2.0
    monitoring: true, // From v0.2.0
    accessibility: config.ux.accessibility.enabled,
    dashboard: config.ux.dashboard.enabled,
    gestures: config.ux.gestures.enabled,
    notifications: config.ux.notifications.enabled,
    offline: config.ux.offline.enabled,
    preferences: config.ux.preferences.enabled,
    search: config.ux.search.enabled,
    visualization: config.ux.visualization.enabled,
    sso: config.security.ssoEnabled,
  };
}
