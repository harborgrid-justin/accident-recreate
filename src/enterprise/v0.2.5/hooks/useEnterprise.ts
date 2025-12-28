/**
 * useEnterprise Hook
 *
 * Unified hook for accessing all v0.2.5 enterprise features.
 * Provides a convenient interface to enterprise context and services.
 *
 * @module enterprise/v0.2.5/hooks/useEnterprise
 * @version 0.2.5
 */

import { useCallback, useEffect, useState } from 'react';
import { useEnterpriseContext } from '../EnterpriseProvider';
import type {
  EnterpriseConfig,
  ServiceDescriptor,
  OverallHealth,
  FeatureFlags,
  SystemInfo,
  EventHandler,
} from '../types';

/**
 * Enterprise hook return type
 */
export interface UseEnterpriseReturn {
  // State
  config: EnterpriseConfig;
  services: ServiceDescriptor[];
  health: OverallHealth;
  features: FeatureFlags;
  systemInfo: SystemInfo;
  initialized: boolean;
  loading: boolean;
  error?: Error;

  // Actions
  initialize: () => Promise<void>;
  shutdown: () => Promise<void>;
  getService: (name: string) => ServiceDescriptor | undefined;
  updateConfig: (config: Partial<EnterpriseConfig>) => Promise<void>;
  checkHealth: () => Promise<OverallHealth>;
  addEventListener: (type: string, handler: EventHandler) => void;
  removeEventListener: (type: string, handler: EventHandler) => void;

  // Feature checks
  isFeatureEnabled: (feature: keyof FeatureFlags) => boolean;
  isServiceAvailable: (service: string) => boolean;
  isHealthy: () => boolean;
  isProductionReady: () => boolean;

  // Utilities
  refreshHealth: () => Promise<void>;
  getServiceStatus: (service: string) => string | undefined;
}

/**
 * Hook to access enterprise features
 *
 * @returns Enterprise context and utilities
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const enterprise = useEnterprise();
 *
 *   if (!enterprise.initialized) {
 *     return <div>Initializing...</div>;
 *   }
 *
 *   return (
 *     <div>
 *       <h1>AccuScene Enterprise {enterprise.systemInfo.version}</h1>
 *       <p>Services: {enterprise.health.healthyServices}/{enterprise.health.totalServices}</p>
 *     </div>
 *   );
 * }
 * ```
 */
export default function useEnterprise(): UseEnterpriseReturn {
  const context = useEnterpriseContext();

  // ========================================================================
  // Feature Checks
  // ========================================================================

  /**
   * Check if a feature is enabled
   */
  const isFeatureEnabled = useCallback(
    (feature: keyof FeatureFlags): boolean => {
      return context.features[feature] === true;
    },
    [context.features]
  );

  /**
   * Check if a service is available
   */
  const isServiceAvailable = useCallback(
    (service: string): boolean => {
      return context.services.some((s) => s.name === service && s.status === 'running');
    },
    [context.services]
  );

  /**
   * Check if the system is healthy
   */
  const isHealthy = useCallback((): boolean => {
    return context.health.status === 'healthy';
  }, [context.health]);

  /**
   * Check if the system is production ready
   */
  const isProductionReady = useCallback((): boolean => {
    return context.systemInfo.productionReady;
  }, [context.systemInfo]);

  // ========================================================================
  // Utilities
  // ========================================================================

  /**
   * Refresh health status
   */
  const refreshHealth = useCallback(async (): Promise<void> => {
    await context.checkHealth();
  }, [context]);

  /**
   * Get service status
   */
  const getServiceStatus = useCallback(
    (service: string): string | undefined => {
      const serviceDescriptor = context.getService(service);
      return serviceDescriptor?.status;
    },
    [context]
  );

  // ========================================================================
  // Return
  // ========================================================================

  return {
    // State
    config: context.config,
    services: context.services,
    health: context.health,
    features: context.features,
    systemInfo: context.systemInfo,
    initialized: context.initialized,
    loading: context.loading,
    error: context.error ? new Error(context.error.message) : undefined,

    // Actions
    initialize: context.initialize,
    shutdown: context.shutdown,
    getService: context.getService,
    updateConfig: context.updateConfig,
    checkHealth: context.checkHealth,
    addEventListener: context.addEventListener,
    removeEventListener: context.removeEventListener,

    // Feature checks
    isFeatureEnabled,
    isServiceAvailable,
    isHealthy,
    isProductionReady,

    // Utilities
    refreshHealth,
    getServiceStatus,
  };
}

/**
 * Hook for specific feature availability
 */
export function useFeature(feature: keyof FeatureFlags): boolean {
  const { features } = useEnterprise();
  return features[feature] === true;
}

/**
 * Hook for service availability
 */
export function useService(serviceName: string): ServiceDescriptor | undefined {
  const { getService } = useEnterprise();
  return getService(serviceName);
}

/**
 * Hook for health monitoring
 */
export function useHealthMonitoring(interval = 60000): OverallHealth {
  const { health, checkHealth } = useEnterprise();
  const [localHealth, setLocalHealth] = useState(health);

  useEffect(() => {
    setLocalHealth(health);
  }, [health]);

  useEffect(() => {
    const timer = setInterval(() => {
      checkHealth().then((h) => setLocalHealth(h));
    }, interval);

    return () => clearInterval(timer);
  }, [checkHealth, interval]);

  return localHealth;
}

/**
 * Hook for event listening
 */
export function useEnterpriseEvent(eventType: string, handler: EventHandler): void {
  const { addEventListener, removeEventListener } = useEnterprise();

  useEffect(() => {
    addEventListener(eventType, handler);
    return () => removeEventListener(eventType, handler);
  }, [eventType, handler, addEventListener, removeEventListener]);
}
