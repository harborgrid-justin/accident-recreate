/**
 * AccuScene Enterprise v0.2.5
 *
 * Main export point for all v0.2.5 features and integrations.
 * This version adds comprehensive UX enhancements including accessibility,
 * dashboards, gestures, notifications, offline support, preferences,
 * search, and advanced visualizations.
 *
 * @module enterprise/v0.2.5
 * @version 0.2.5
 */

// ============================================================================
// Type Exports
// ============================================================================

export * from './types';

// ============================================================================
// Configuration Exports
// ============================================================================

export * from './config';
export { defaultEnterpriseConfig as DEFAULT_CONFIG } from './config';

// ============================================================================
// Component Exports
// ============================================================================

export { default as EnterpriseApp } from './EnterpriseApp';
export { default as EnterpriseProvider } from './EnterpriseProvider';

// ============================================================================
// Hook Exports
// ============================================================================

export { default as useEnterprise } from './hooks/useEnterprise';

// ============================================================================
// Service Exports
// ============================================================================

export { default as EnterpriseService } from './services/EnterpriseService';
export type { IEnterpriseService } from './services/EnterpriseService';

// ============================================================================
// Version Information
// ============================================================================

export const VERSION = '0.2.5';
export const BUILD_DATE = new Date().toISOString();

/**
 * v0.2.5 features list
 */
export const FEATURES = [
  // v0.2.0 features
  'GraphQL Federation API',
  'Real-time Collaboration (CRDT)',
  'Advanced UI Components (3D, AR)',
  'Plugin Architecture',
  'Performance Monitoring',
  // v0.2.5 new features
  'Accessibility (a11y) Support',
  'Interactive Dashboards',
  'Gesture Recognition',
  'Push Notifications',
  'Offline-first Capabilities',
  'User Preferences Management',
  'Full-text Search',
  'Advanced Data Visualization',
  'Single Sign-On (SSO)',
  'Data Transfer & Sync',
] as const;

/**
 * Version metadata
 */
export const VERSION_INFO = {
  version: VERSION,
  buildDate: BUILD_DATE,
  features: FEATURES,
  rust: {
    version: '0.2.5',
    crates: 26,
  },
  typescript: {
    version: '0.2.5',
    modules: 8,
  },
} as const;

/**
 * Check if a feature is available
 */
export function hasFeature(feature: string): boolean {
  return FEATURES.includes(feature as any);
}

/**
 * Get all available features
 */
export function getFeatures(): readonly string[] {
  return FEATURES;
}

/**
 * Get version information
 */
export function getVersionInfo() {
  return VERSION_INFO;
}

/**
 * Initialize v0.2.5 enterprise features
 *
 * @param config - Optional configuration override
 * @returns Promise that resolves when initialization is complete
 */
export async function initializeV025(config?: Partial<import('./types').EnterpriseConfig>): Promise<void> {
  console.log(`🚀 Initializing AccuScene Enterprise v${VERSION}`);

  // Log enabled features
  if (config?.ux?.accessibility?.enabled) {
    console.log('  ✓ Accessibility Support');
  }
  if (config?.ux?.dashboard?.enabled) {
    console.log('  ✓ Interactive Dashboards');
  }
  if (config?.ux?.gestures?.enabled) {
    console.log('  ✓ Gesture Recognition');
  }
  if (config?.ux?.notifications?.enabled) {
    console.log('  ✓ Push Notifications');
  }
  if (config?.ux?.offline?.enabled) {
    console.log('  ✓ Offline-first Capabilities');
  }
  if (config?.ux?.preferences?.enabled) {
    console.log('  ✓ User Preferences');
  }
  if (config?.ux?.search?.enabled) {
    console.log('  ✓ Full-text Search');
  }
  if (config?.ux?.visualization?.enabled) {
    console.log('  ✓ Advanced Visualization');
  }
  if (config?.security?.ssoEnabled) {
    console.log('  ✓ Single Sign-On');
  }

  console.log(`✅ AccuScene Enterprise v${VERSION} initialized`);
}

// ============================================================================
// Re-export v0.2.0 features for convenience
// ============================================================================

// GraphQL Federation
export * from '../../graphql';

// Real-time Collaboration
export * from '../../collaboration';

// Advanced UI Components
export * from '../../renderer/components/advanced';

// Plugin Architecture
export * from '../../plugins';

// Performance Monitoring
export * from '../../monitoring';
