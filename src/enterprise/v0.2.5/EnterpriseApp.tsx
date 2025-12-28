/**
 * Enterprise App Shell Component
 *
 * Main application shell that integrates all v0.2.5 enterprise features.
 * Provides a unified interface for the entire AccuScene Enterprise application.
 *
 * @module enterprise/v0.2.5/EnterpriseApp
 * @version 0.2.5
 */

import React, { useEffect, useState } from 'react';
import { useEnterpriseContext } from './EnterpriseProvider';
import type { HealthStatus } from './types';

// ============================================================================
// Enterprise App Props
// ============================================================================

interface EnterpriseAppProps {
  /** Child components (main application content) */
  children: React.ReactNode;

  /** Show loading spinner during initialization */
  showLoader?: boolean;

  /** Custom loading component */
  loadingComponent?: React.ReactNode;

  /** Show error UI on initialization failure */
  showErrors?: boolean;

  /** Custom error component */
  errorComponent?: (error: Error) => React.ReactNode;

  /** Show system status bar */
  showStatusBar?: boolean;

  /** Enable development tools */
  devTools?: boolean;
}

// ============================================================================
// Enterprise App Component
// ============================================================================

/**
 * Enterprise Application Shell
 *
 * Wraps the main application content with enterprise features and provides
 * initialization, error handling, and status monitoring.
 *
 * @example
 * ```tsx
 * <EnterpriseProvider>
 *   <EnterpriseApp showStatusBar devTools>
 *     <YourApp />
 *   </EnterpriseApp>
 * </EnterpriseProvider>
 * ```
 */
export default function EnterpriseApp({
  children,
  showLoader = true,
  loadingComponent,
  showErrors = true,
  errorComponent,
  showStatusBar = false,
  devTools = false,
}: EnterpriseAppProps): JSX.Element {
  const {
    initialized,
    loading,
    error,
    health,
    systemInfo,
    features,
    checkHealth,
  } = useEnterpriseContext();

  const [healthStatus, setHealthStatus] = useState<HealthStatus>(health.status);

  // ========================================================================
  // Effects
  // ========================================================================

  /**
   * Update health status when health changes
   */
  useEffect(() => {
    setHealthStatus(health.status);
  }, [health]);

  /**
   * Periodic health monitoring
   */
  useEffect(() => {
    if (!initialized) {
      return;
    }

    const interval = setInterval(() => {
      checkHealth().catch((err) => {
        console.error('Health check failed:', err);
      });
    }, 30000); // Check every 30 seconds

    return () => clearInterval(interval);
  }, [initialized, checkHealth]);

  // ========================================================================
  // Render States
  // ========================================================================

  /**
   * Loading state
   */
  if (loading && showLoader) {
    if (loadingComponent) {
      return <>{loadingComponent}</>;
    }

    return (
      <div style={styles.container}>
        <div style={styles.loader}>
          <div style={styles.spinner} />
          <h2 style={styles.title}>Initializing AccuScene Enterprise</h2>
          <p style={styles.subtitle}>v{systemInfo.version}</p>
          <p style={styles.text}>Loading enterprise features...</p>
        </div>
      </div>
    );
  }

  /**
   * Error state
   */
  if (error && showErrors) {
    if (errorComponent) {
      return <>{errorComponent(new Error(error.message))}</>;
    }

    return (
      <div style={styles.container}>
        <div style={styles.error}>
          <h2 style={styles.errorTitle}>❌ Initialization Error</h2>
          <p style={styles.errorMessage}>{error.message}</p>
          {error.details && (
            <details style={styles.errorDetails}>
              <summary>Details</summary>
              <pre style={styles.errorCode}>
                {JSON.stringify(error.details, null, 2)}
              </pre>
            </details>
          )}
          {error.stack && devTools && (
            <details style={styles.errorDetails}>
              <summary>Stack Trace</summary>
              <pre style={styles.errorCode}>{error.stack}</pre>
            </details>
          )}
        </div>
      </div>
    );
  }

  // ========================================================================
  // Main Application
  // ========================================================================

  return (
    <div style={styles.app}>
      {/* Status Bar */}
      {showStatusBar && initialized && (
        <div style={styles.statusBar}>
          <div style={styles.statusLeft}>
            <span style={styles.statusBadge}>
              AccuScene Enterprise v{systemInfo.version}
            </span>
            <span style={styles.statusSeparator}>|</span>
            <span style={styles.statusBadge}>
              {systemInfo.environment}
            </span>
            <span style={styles.statusSeparator}>|</span>
            <span style={{
              ...styles.statusBadge,
              ...getHealthStatusStyle(healthStatus),
            }}>
              {healthStatus.toUpperCase()}
            </span>
          </div>
          <div style={styles.statusRight}>
            <span style={styles.statusText}>
              Services: {health.healthyServices}/{health.totalServices}
            </span>
            {devTools && (
              <>
                <span style={styles.statusSeparator}>|</span>
                <span style={styles.statusText}>
                  Features: {Object.values(features).filter(Boolean).length}/
                  {Object.keys(features).length}
                </span>
              </>
            )}
          </div>
        </div>
      )}

      {/* Main Content */}
      <div style={styles.content}>
        {children}
      </div>

      {/* Dev Tools Overlay */}
      {devTools && initialized && (
        <DevToolsOverlay />
      )}
    </div>
  );
}

// ============================================================================
// Dev Tools Overlay
// ============================================================================

function DevToolsOverlay(): JSX.Element {
  const { systemInfo, health, features, services } = useEnterpriseContext();
  const [isOpen, setIsOpen] = useState(false);

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        style={styles.devToolsButton}
        title="Open Developer Tools"
      >
        🛠️
      </button>
    );
  }

  return (
    <div style={styles.devToolsOverlay}>
      <div style={styles.devToolsHeader}>
        <h3 style={styles.devToolsTitle}>Developer Tools</h3>
        <button onClick={() => setIsOpen(false)} style={styles.devToolsClose}>
          ✕
        </button>
      </div>
      <div style={styles.devToolsContent}>
        {/* System Info */}
        <section style={styles.devToolsSection}>
          <h4 style={styles.devToolsSectionTitle}>System Information</h4>
          <pre style={styles.devToolsCode}>
            {JSON.stringify(systemInfo, null, 2)}
          </pre>
        </section>

        {/* Health Status */}
        <section style={styles.devToolsSection}>
          <h4 style={styles.devToolsSectionTitle}>Health Status</h4>
          <pre style={styles.devToolsCode}>
            {JSON.stringify(health, null, 2)}
          </pre>
        </section>

        {/* Features */}
        <section style={styles.devToolsSection}>
          <h4 style={styles.devToolsSectionTitle}>Features</h4>
          <ul style={styles.featureList}>
            {Object.entries(features).map(([key, enabled]) => (
              <li key={key} style={styles.featureItem}>
                <span style={enabled ? styles.featureEnabled : styles.featureDisabled}>
                  {enabled ? '✓' : '✗'}
                </span>{' '}
                {key}
              </li>
            ))}
          </ul>
        </section>

        {/* Services */}
        <section style={styles.devToolsSection}>
          <h4 style={styles.devToolsSectionTitle}>Services ({services.length})</h4>
          <ul style={styles.serviceList}>
            {services.map((service) => (
              <li key={service.name} style={styles.serviceItem}>
                <strong>{service.name}</strong> - {service.status} (v{service.version})
              </li>
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}

// ============================================================================
// Helper Functions
// ============================================================================

function getHealthStatusStyle(status: HealthStatus): React.CSSProperties {
  switch (status) {
    case 'healthy':
      return { backgroundColor: '#10b981', color: '#fff' };
    case 'degraded':
      return { backgroundColor: '#f59e0b', color: '#fff' };
    case 'unhealthy':
      return { backgroundColor: '#ef4444', color: '#fff' };
    default:
      return { backgroundColor: '#6b7280', color: '#fff' };
  }
}

// ============================================================================
// Styles
// ============================================================================

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '100vh',
    backgroundColor: '#f3f4f6',
  },
  loader: {
    textAlign: 'center',
    padding: '2rem',
  },
  spinner: {
    width: '64px',
    height: '64px',
    border: '4px solid #e5e7eb',
    borderTopColor: '#3b82f6',
    borderRadius: '50%',
    margin: '0 auto 1rem',
    animation: 'spin 1s linear infinite',
  },
  title: {
    fontSize: '1.5rem',
    fontWeight: 'bold',
    color: '#111827',
    margin: '0 0 0.5rem',
  },
  subtitle: {
    fontSize: '1rem',
    color: '#6b7280',
    margin: '0 0 1rem',
  },
  text: {
    fontSize: '0.875rem',
    color: '#9ca3af',
    margin: 0,
  },
  error: {
    maxWidth: '600px',
    padding: '2rem',
    backgroundColor: '#fff',
    borderRadius: '8px',
    boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
  },
  errorTitle: {
    fontSize: '1.5rem',
    fontWeight: 'bold',
    color: '#ef4444',
    margin: '0 0 1rem',
  },
  errorMessage: {
    fontSize: '1rem',
    color: '#374151',
    margin: '0 0 1rem',
  },
  errorDetails: {
    marginTop: '1rem',
  },
  errorCode: {
    fontSize: '0.75rem',
    backgroundColor: '#f3f4f6',
    padding: '1rem',
    borderRadius: '4px',
    overflow: 'auto',
  },
  app: {
    display: 'flex',
    flexDirection: 'column',
    minHeight: '100vh',
  },
  statusBar: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '0.5rem 1rem',
    backgroundColor: '#1f2937',
    color: '#fff',
    fontSize: '0.875rem',
  },
  statusLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
  },
  statusRight: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
  },
  statusBadge: {
    padding: '0.25rem 0.5rem',
    borderRadius: '4px',
    backgroundColor: '#374151',
  },
  statusText: {
    opacity: 0.8,
  },
  statusSeparator: {
    opacity: 0.5,
  },
  content: {
    flex: 1,
  },
  devToolsButton: {
    position: 'fixed',
    bottom: '1rem',
    right: '1rem',
    width: '48px',
    height: '48px',
    borderRadius: '50%',
    backgroundColor: '#3b82f6',
    color: '#fff',
    border: 'none',
    fontSize: '1.5rem',
    cursor: 'pointer',
    boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
    zIndex: 9999,
  },
  devToolsOverlay: {
    position: 'fixed',
    top: '0',
    right: '0',
    width: '400px',
    height: '100vh',
    backgroundColor: '#fff',
    boxShadow: '-4px 0 6px rgba(0, 0, 0, 0.1)',
    zIndex: 10000,
    display: 'flex',
    flexDirection: 'column',
  },
  devToolsHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '1rem',
    backgroundColor: '#1f2937',
    color: '#fff',
  },
  devToolsTitle: {
    margin: 0,
    fontSize: '1.125rem',
    fontWeight: 'bold',
  },
  devToolsClose: {
    backgroundColor: 'transparent',
    border: 'none',
    color: '#fff',
    fontSize: '1.5rem',
    cursor: 'pointer',
    padding: 0,
    width: '32px',
    height: '32px',
  },
  devToolsContent: {
    flex: 1,
    overflow: 'auto',
    padding: '1rem',
  },
  devToolsSection: {
    marginBottom: '1.5rem',
  },
  devToolsSectionTitle: {
    fontSize: '0.875rem',
    fontWeight: 'bold',
    color: '#374151',
    marginBottom: '0.5rem',
  },
  devToolsCode: {
    fontSize: '0.75rem',
    backgroundColor: '#f3f4f6',
    padding: '0.75rem',
    borderRadius: '4px',
    overflow: 'auto',
    maxHeight: '200px',
  },
  featureList: {
    listStyle: 'none',
    padding: 0,
    margin: 0,
  },
  featureItem: {
    padding: '0.25rem 0',
    fontSize: '0.875rem',
  },
  featureEnabled: {
    color: '#10b981',
    fontWeight: 'bold',
  },
  featureDisabled: {
    color: '#ef4444',
    fontWeight: 'bold',
  },
  serviceList: {
    listStyle: 'none',
    padding: 0,
    margin: 0,
  },
  serviceItem: {
    padding: '0.5rem',
    marginBottom: '0.25rem',
    backgroundColor: '#f3f4f6',
    borderRadius: '4px',
    fontSize: '0.875rem',
  },
};
