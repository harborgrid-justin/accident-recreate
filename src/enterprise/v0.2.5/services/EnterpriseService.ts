/**
 * Enterprise Service
 *
 * Unified service class that manages all v0.2.5 enterprise features.
 * Provides a single interface to initialize, manage, and interact with
 * all enterprise services.
 *
 * @module enterprise/v0.2.5/services/EnterpriseService
 * @version 0.2.5
 */

import type {
  EnterpriseConfig,
  ServiceDescriptor,
  ServiceStatus,
  OverallHealth,
  HealthCheck,
  HealthStatus,
  SystemInfo,
  FeatureFlags,
} from '../types';
import { getFeatureFlags } from '../config';

/**
 * Enterprise service interface
 */
export interface IEnterpriseService {
  initialize(): Promise<void>;
  shutdown(): Promise<void>;
  getServices(): Promise<ServiceDescriptor[]>;
  getService(name: string): Promise<ServiceDescriptor | undefined>;
  checkHealth(): Promise<OverallHealth>;
  getSystemInfo(): Promise<SystemInfo>;
}

/**
 * Enterprise Service Implementation
 *
 * Central service class for managing all enterprise features.
 */
export default class EnterpriseService implements IEnterpriseService {
  private config: EnterpriseConfig;
  private services: Map<string, ServiceDescriptor> = new Map();
  private initialized = false;

  constructor(config: EnterpriseConfig) {
    this.config = config;
  }

  // ========================================================================
  // Lifecycle Management
  // ========================================================================

  /**
   * Initialize all enterprise services
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      console.warn('Enterprise services already initialized');
      return;
    }

    console.log('🚀 Initializing AccuScene Enterprise v0.2.5');

    // Initialize core services
    await this.initCoreServices();

    // Initialize security services
    await this.initSecurityServices();

    // Initialize processing services
    await this.initProcessingServices();

    // Initialize infrastructure services
    await this.initInfrastructureServices();

    // Initialize UX services (v0.2.5)
    await this.initUxServices();

    this.initialized = true;
    console.log('✅ AccuScene Enterprise v0.2.5 initialized');
  }

  /**
   * Shutdown all enterprise services
   */
  async shutdown(): Promise<void> {
    if (!this.initialized) {
      console.warn('Enterprise services not initialized');
      return;
    }

    console.log('🛑 Shutting down AccuScene Enterprise');

    // Shutdown all services in reverse order
    const serviceNames = Array.from(this.services.keys()).reverse();

    for (const name of serviceNames) {
      await this.stopService(name);
    }

    this.services.clear();
    this.initialized = false;

    console.log('✅ AccuScene Enterprise shutdown complete');
  }

  // ========================================================================
  // Service Initialization
  // ========================================================================

  private async initCoreServices(): Promise<void> {
    console.log('  Initializing core services...');

    await this.registerService('database', 'Database service', '0.2.0');
    await this.registerService('cache', 'Multi-tier cache service', '0.2.0');
    await this.registerService('eventsourcing', 'Event sourcing service', '0.2.0');

    await this.startService('database');
    if (this.config.cache.enabled) {
      await this.startService('cache');
    }
    await this.startService('eventsourcing');
  }

  private async initSecurityServices(): Promise<void> {
    console.log('  Initializing security services...');

    await this.registerService('security', 'Security and authentication service', '0.2.0');
    await this.registerService('crypto', 'Cryptographic primitives service', '0.2.0');

    await this.startService('security');
    await this.startService('crypto');

    if (this.config.security.ssoEnabled) {
      await this.registerService('sso', 'Single Sign-On service', '0.2.5');
      await this.startService('sso');
    }
  }

  private async initProcessingServices(): Promise<void> {
    console.log('  Initializing processing services...');

    await this.registerService('physics', 'Physics simulation engine', '0.2.0');
    await this.registerService('ml', 'Machine learning service', '0.2.0');
    await this.registerService('jobs', 'Job processing service', '0.2.0');

    await this.startService('physics');
    await this.startService('ml');
    await this.startService('jobs');

    if (this.config.analytics.enabled) {
      await this.registerService('analytics', 'Advanced analytics engine', '0.2.0');
      await this.startService('analytics');
    }
  }

  private async initInfrastructureServices(): Promise<void> {
    console.log('  Initializing infrastructure services...');

    await this.registerService('streaming', 'Real-time data streaming service', '0.2.0');
    await this.startService('streaming');

    if (this.config.cluster.enabled) {
      await this.registerService('cluster', 'Distributed clustering service', '0.2.0');
      await this.startService('cluster');
    }

    if (this.config.telemetry.enabled) {
      await this.registerService('telemetry', 'Telemetry and monitoring service', '0.2.0');
      await this.startService('telemetry');
    }
  }

  private async initUxServices(): Promise<void> {
    console.log('  Initializing UX services (v0.2.5)...');

    // Accessibility
    if (this.config.ux.accessibility.enabled) {
      await this.registerService('accessibility', 'Accessibility support service', '0.2.5');
      await this.startService('accessibility');
    }

    // Dashboard
    if (this.config.ux.dashboard.enabled) {
      await this.registerService('dashboard', 'Interactive dashboard service', '0.2.5');
      await this.startService('dashboard');
    }

    // Gestures
    if (this.config.ux.gestures.enabled) {
      await this.registerService('gestures', 'Gesture recognition service', '0.2.5');
      await this.startService('gestures');
    }

    // Notifications
    if (this.config.ux.notifications.enabled) {
      await this.registerService('notifications', 'Push notifications service', '0.2.5');
      await this.startService('notifications');
    }

    // Offline
    if (this.config.ux.offline.enabled) {
      await this.registerService('offline', 'Offline-first capabilities service', '0.2.5');
      await this.startService('offline');
    }

    // Preferences
    if (this.config.ux.preferences.enabled) {
      await this.registerService('preferences', 'User preferences service', '0.2.5');
      await this.startService('preferences');
    }

    // Search
    if (this.config.ux.search.enabled) {
      await this.registerService('search', 'Full-text search service', '0.2.5');
      await this.startService('search');
    }

    // Visualization
    if (this.config.ux.visualization.enabled) {
      await this.registerService('visualization', 'Advanced data visualization service', '0.2.5');
      await this.startService('visualization');
    }
  }

  // ========================================================================
  // Service Management
  // ========================================================================

  private async registerService(
    name: string,
    description: string,
    version: string
  ): Promise<void> {
    const service: ServiceDescriptor = {
      name,
      description,
      version,
      status: 'initializing',
      registeredAt: new Date(),
      updatedAt: new Date(),
      metadata: {},
    };

    this.services.set(name, service);
  }

  private async startService(name: string): Promise<void> {
    const service = this.services.get(name);
    if (!service) {
      throw new Error(`Service not found: ${name}`);
    }

    // Simulate service startup
    await new Promise((resolve) => setTimeout(resolve, 10));

    service.status = 'running';
    service.updatedAt = new Date();

    this.services.set(name, service);
  }

  private async stopService(name: string): Promise<void> {
    const service = this.services.get(name);
    if (!service) {
      return;
    }

    service.status = 'stopped';
    service.updatedAt = new Date();

    this.services.set(name, service);
  }

  // ========================================================================
  // Public API
  // ========================================================================

  /**
   * Get all services
   */
  async getServices(): Promise<ServiceDescriptor[]> {
    return Array.from(this.services.values());
  }

  /**
   * Get a specific service
   */
  async getService(name: string): Promise<ServiceDescriptor | undefined> {
    return this.services.get(name);
  }

  /**
   * Check overall health
   */
  async checkHealth(): Promise<OverallHealth> {
    const services = await this.getServices();
    const checks: HealthCheck[] = services.map((service) => ({
      service: service.name,
      status: this.mapServiceStatusToHealth(service.status),
      message: `Service is ${service.status}`,
      checkedAt: new Date(),
      responseTimeMs: Math.random() * 50, // Simulated response time
      details: {
        version: service.version,
        uptime: Date.now() - service.registeredAt.getTime(),
      },
    }));

    const total = checks.length;
    const healthy = checks.filter((c) => c.status === 'healthy').length;
    const degraded = checks.filter((c) => c.status === 'degraded').length;
    const unhealthy = checks.filter((c) => c.status === 'unhealthy').length;
    const unknown = checks.filter((c) => c.status === 'unknown').length;

    const status: HealthStatus = unhealthy > 0
      ? 'unhealthy'
      : degraded > 0
      ? 'degraded'
      : unknown > 0
      ? 'unknown'
      : 'healthy';

    return {
      status,
      totalServices: total,
      healthyServices: healthy,
      degradedServices: degraded,
      unhealthyServices: unhealthy,
      unknownServices: unknown,
      checks,
    };
  }

  /**
   * Get system information
   */
  async getSystemInfo(): Promise<SystemInfo> {
    const services = await this.getServices();
    const health = await this.checkHealth();
    const features = getFeatureFlags(this.config);

    const coreServices = ['database', 'security', 'physics'];
    const coreReady = coreServices.every((name) =>
      services.some((s) => s.name === name && s.status === 'running')
    );

    const uxServices = [
      this.config.ux.accessibility.enabled && 'accessibility',
      this.config.ux.dashboard.enabled && 'dashboard',
      this.config.ux.gestures.enabled && 'gestures',
      this.config.ux.notifications.enabled && 'notifications',
      this.config.ux.offline.enabled && 'offline',
      this.config.ux.preferences.enabled && 'preferences',
      this.config.ux.search.enabled && 'search',
      this.config.ux.visualization.enabled && 'visualization',
    ].filter(Boolean) as string[];

    const uxReady = uxServices.length === 0 || uxServices.every((name) =>
      services.some((s) => s.name === name && s.status === 'running')
    );

    const productionReady =
      coreReady &&
      this.config.app.environment === 'production' &&
      this.config.security.jwtSecret !== 'change-me-in-production' &&
      this.config.security.authEnabled &&
      this.config.security.encryptionEnabled;

    return {
      version: '0.2.5',
      environment: this.config.app.environment,
      servicesCount: services.length,
      coreReady,
      uxReady,
      productionReady,
    };
  }

  // ========================================================================
  // Helper Methods
  // ========================================================================

  private mapServiceStatusToHealth(status: ServiceStatus): HealthStatus {
    switch (status) {
      case 'running':
        return 'healthy';
      case 'initializing':
        return 'unknown';
      case 'stopped':
        return 'unhealthy';
      case 'error':
        return 'unhealthy';
      default:
        return 'unknown';
    }
  }

  /**
   * Get configuration
   */
  getConfig(): EnterpriseConfig {
    return this.config;
  }

  /**
   * Update configuration
   */
  updateConfig(newConfig: Partial<EnterpriseConfig>): void {
    this.config = { ...this.config, ...newConfig };
  }

  /**
   * Check if initialized
   */
  isInitialized(): boolean {
    return this.initialized;
  }
}
