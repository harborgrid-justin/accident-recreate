/**
 * Enterprise Provider Component
 *
 * Provides unified context for all v0.2.5 enterprise features.
 * This provider combines all service contexts and makes them available
 * to child components.
 *
 * @module enterprise/v0.2.5/EnterpriseProvider
 * @version 0.2.5
 */

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import type {
  EnterpriseConfig,
  EnterpriseContext,
  EnterpriseActions,
  ServiceDescriptor,
  OverallHealth,
  FeatureFlags,
  SystemInfo,
  EnterpriseError,
  EventHandler,
} from './types';
import { defaultEnterpriseConfig, validateConfig, getFeatureFlags } from './config';
import EnterpriseService from './services/EnterpriseService';

// ============================================================================
// Context Definition
// ============================================================================

interface EnterpriseContextValue extends EnterpriseContext, EnterpriseActions {}

const EnterpriseContextProvider = createContext<EnterpriseContextValue | undefined>(undefined);

// ============================================================================
// Provider Props
// ============================================================================

interface EnterpriseProviderProps {
  /** Child components */
  children: React.ReactNode;

  /** Optional configuration override */
  config?: Partial<EnterpriseConfig>;

  /** Auto-initialize on mount */
  autoInit?: boolean;

  /** Error handler */
  onError?: (error: Error | EnterpriseError) => void;

  /** Initialization complete handler */
  onInitialized?: () => void;
}

// ============================================================================
// Provider Component
// ============================================================================

/**
 * Enterprise Provider
 *
 * Wraps the application with enterprise context and services.
 *
 * @example
 * ```tsx
 * <EnterpriseProvider config={customConfig} autoInit>
 *   <App />
 * </EnterpriseProvider>
 * ```
 */
export default function EnterpriseProvider({
  children,
  config: configOverride,
  autoInit = true,
  onError,
  onInitialized,
}: EnterpriseProviderProps): JSX.Element {
  // State
  const [config, setConfig] = useState<EnterpriseConfig>(() => {
    const merged = { ...defaultEnterpriseConfig, ...configOverride };
    return merged;
  });

  const [services, setServices] = useState<ServiceDescriptor[]>([]);
  const [health, setHealth] = useState<OverallHealth>({
    status: 'unknown',
    totalServices: 0,
    healthyServices: 0,
    degradedServices: 0,
    unhealthyServices: 0,
    unknownServices: 0,
    checks: [],
  });
  const [features, setFeatures] = useState<FeatureFlags>(getFeatureFlags(config));
  const [systemInfo, setSystemInfo] = useState<SystemInfo>({
    version: '0.2.5',
    environment: config.app.environment,
    servicesCount: 0,
    coreReady: false,
    uxReady: false,
    productionReady: false,
  });
  const [initialized, setInitialized] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<EnterpriseError | undefined>(undefined);

  // Service instance
  const [service] = useState(() => new EnterpriseService(config));

  // Event handlers map
  const [eventHandlers] = useState<Map<string, Set<EventHandler>>>(new Map());

  // ========================================================================
  // Actions
  // ========================================================================

  /**
   * Initialize enterprise services
   */
  const initialize = useCallback(async () => {
    if (initialized || loading) {
      return;
    }

    setLoading(true);
    setError(undefined);

    try {
      // Validate configuration
      const validationErrors = validateConfig(config);
      if (validationErrors.length > 0) {
        throw new Error(`Configuration validation failed: ${validationErrors.join(', ')}`);
      }

      // Initialize service
      await service.initialize();

      // Update state
      const servicesData = await service.getServices();
      const healthData = await service.checkHealth();
      const systemData = await service.getSystemInfo();

      setServices(servicesData);
      setHealth(healthData);
      setSystemInfo(systemData);
      setFeatures(getFeatureFlags(config));
      setInitialized(true);

      onInitialized?.();
    } catch (err) {
      const enterpriseError: EnterpriseError = {
        code: 'INITIALIZATION_ERROR',
        message: err instanceof Error ? err.message : 'Unknown error',
        details: err,
        stack: err instanceof Error ? err.stack : undefined,
      };

      setError(enterpriseError);
      onError?.(enterpriseError);
    } finally {
      setLoading(false);
    }
  }, [config, initialized, loading, service, onInitialized, onError]);

  /**
   * Shutdown enterprise services
   */
  const shutdown = useCallback(async () => {
    if (!initialized) {
      return;
    }

    setLoading(true);

    try {
      await service.shutdown();
      setInitialized(false);
      setServices([]);
      setHealth({
        status: 'unknown',
        totalServices: 0,
        healthyServices: 0,
        degradedServices: 0,
        unhealthyServices: 0,
        unknownServices: 0,
        checks: [],
      });
    } catch (err) {
      const enterpriseError: EnterpriseError = {
        code: 'SHUTDOWN_ERROR',
        message: err instanceof Error ? err.message : 'Unknown error',
        details: err,
      };

      onError?.(enterpriseError);
    } finally {
      setLoading(false);
    }
  }, [initialized, service, onError]);

  /**
   * Get a specific service
   */
  const getService = useCallback(
    (name: string): ServiceDescriptor | undefined => {
      return services.find((s) => s.name === name);
    },
    [services]
  );

  /**
   * Update configuration
   */
  const updateConfig = useCallback(
    async (newConfig: Partial<EnterpriseConfig>) => {
      const merged = { ...config, ...newConfig };
      setConfig(merged);
      setFeatures(getFeatureFlags(merged));

      // Reinitialize if already initialized
      if (initialized) {
        await shutdown();
        await initialize();
      }
    },
    [config, initialized, shutdown, initialize]
  );

  /**
   * Check health
   */
  const checkHealth = useCallback(async (): Promise<OverallHealth> => {
    const healthData = await service.checkHealth();
    setHealth(healthData);
    return healthData;
  }, [service]);

  /**
   * Add event listener
   */
  const addEventListener = useCallback((type: string, handler: EventHandler) => {
    if (!eventHandlers.has(type)) {
      eventHandlers.set(type, new Set());
    }
    eventHandlers.get(type)!.add(handler);
  }, [eventHandlers]);

  /**
   * Remove event listener
   */
  const removeEventListener = useCallback((type: string, handler: EventHandler) => {
    const handlers = eventHandlers.get(type);
    if (handlers) {
      handlers.delete(handler);
    }
  }, [eventHandlers]);

  // ========================================================================
  // Effects
  // ========================================================================

  /**
   * Auto-initialize on mount
   */
  useEffect(() => {
    if (autoInit && !initialized && !loading) {
      initialize();
    }
  }, [autoInit, initialized, loading, initialize]);

  /**
   * Periodic health check
   */
  useEffect(() => {
    if (!initialized) {
      return;
    }

    const interval = setInterval(() => {
      checkHealth().catch((err) => {
        console.error('Health check failed:', err);
      });
    }, 60000); // Check every minute

    return () => clearInterval(interval);
  }, [initialized, checkHealth]);

  /**
   * Cleanup on unmount
   */
  useEffect(() => {
    return () => {
      if (initialized) {
        shutdown().catch((err) => {
          console.error('Shutdown failed:', err);
        });
      }
    };
  }, [initialized, shutdown]);

  // ========================================================================
  // Context Value
  // ========================================================================

  const contextValue: EnterpriseContextValue = {
    // State
    config,
    services,
    health,
    features,
    systemInfo,
    initialized,
    loading,
    error,

    // Actions
    initialize,
    shutdown,
    getService,
    updateConfig,
    checkHealth,
    addEventListener,
    removeEventListener,
  };

  // ========================================================================
  // Render
  // ========================================================================

  return (
    <EnterpriseContextProvider.Provider value={contextValue}>
      {children}
    </EnterpriseContextProvider.Provider>
  );
}

// ============================================================================
// Hook to use Enterprise Context
// ============================================================================

/**
 * Hook to access enterprise context
 *
 * @throws Error if used outside of EnterpriseProvider
 * @returns Enterprise context value
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const enterprise = useEnterpriseContext();
 *   return <div>{enterprise.systemInfo.version}</div>;
 * }
 * ```
 */
export function useEnterpriseContext(): EnterpriseContextValue {
  const context = useContext(EnterpriseContextProvider);

  if (!context) {
    throw new Error('useEnterpriseContext must be used within EnterpriseProvider');
  }

  return context;
}
