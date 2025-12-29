/**
 * AccuScene Enterprise v0.3.0
 *
 * Main export point for all v0.3.0 features and integrations.
 * This version adds advanced physics engineering, professional CAD/CAM GUI,
 * database algorithms, 3D rendering enhancements, and AI/ML capabilities.
 *
 * @module enterprise/v0.3.0
 * @version 0.3.0
 */

// ============================================================================
// Type Exports
// ============================================================================

export * from './types';

// ============================================================================
// Configuration Exports
// ============================================================================

export * from './config';
export { defaultV030Config as DEFAULT_V030_CONFIG } from './config';

// ============================================================================
// Version Information
// ============================================================================

export const VERSION = '0.3.0';
export const BUILD_DATE = new Date().toISOString();

/**
 * v0.3.0 features list
 */
export const FEATURES = [
  // v0.2.x features (inherited)
  'GraphQL Federation API',
  'Real-time Collaboration (CRDT)',
  'Advanced UI Components (3D, AR)',
  'Plugin Architecture',
  'Performance Monitoring',
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

  // v0.3.0 new features
  'Advanced Rigid Body Dynamics',
  'Deformable Body Collision Physics',
  'Tire Friction Modeling',
  'Energy Absorption Calculations',
  'Multi-body Constraint Solver',
  'Impact Force Distribution',
  'Momentum Transfer Analysis',
  'Vehicle Crush Zone Simulation',
  'Professional CAD Toolbar System',
  'Advanced Property Panels',
  'Contextual Command Palette',
  'Measurement Annotation Tools',
  'Layer Management System',
  'Snap-to-Grid with Magnetic Guides',
  'Multi-viewport Layout',
  'Custom Theme Engine',
  'LZ4 Real-time Compression',
  'Delta Encoding for Scene Diffs',
  'B-tree Spatial Indexing',
  'Bloom Filter Quick Lookups',
  'Write-ahead Logging (WAL)',
  'MVCC Transaction Support',
  'WebGPU Compute Shaders',
  'Instanced Mesh Rendering',
  'Level-of-Detail System',
  'Shadow Mapping',
  'Post-processing Effects',
  'VR Mode Support',
  'Crash Pattern Recognition',
  'Speed Estimation from Damage',
  'Trajectory Prediction',
  'Anomaly Detection',
] as const;

/**
 * Version metadata
 */
export const VERSION_INFO = {
  version: VERSION,
  buildDate: BUILD_DATE,
  features: FEATURES,
  rust: {
    version: '0.3.0',
    crates: 31, // 26 from v0.2.5 + 5 new in v0.3.0
    newCrates: [
      'accuscene-physics-v3',
      'accuscene-algorithms',
      'accuscene-ml-v3',
      'accuscene-security-v3',
      'accuscene-performance',
    ],
  },
  typescript: {
    version: '0.3.0',
    modules: 13, // 8 from v0.2.5 + 5 new categories in v0.3.0
  },
  categories: {
    physics: 'Advanced physics engine with deformable bodies and tire friction',
    gui: 'Professional CAD/CAM interface with multi-viewport and theming',
    algorithms: 'Database algorithms including compression and indexing',
    rendering: 'WebGPU-powered 3D rendering with shadows and post-processing',
    aiml: 'AI/ML for crash analysis and prediction',
    performance: 'Streaming optimization and performance monitoring',
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
 * Get features by category
 */
export function getFeaturesByCategory() {
  return {
    physics: [
      'Advanced Rigid Body Dynamics',
      'Deformable Body Collision Physics',
      'Tire Friction Modeling',
      'Energy Absorption Calculations',
      'Multi-body Constraint Solver',
      'Impact Force Distribution',
      'Momentum Transfer Analysis',
      'Vehicle Crush Zone Simulation',
    ],
    gui: [
      'Professional CAD Toolbar System',
      'Advanced Property Panels',
      'Contextual Command Palette',
      'Measurement Annotation Tools',
      'Layer Management System',
      'Snap-to-Grid with Magnetic Guides',
      'Multi-viewport Layout',
      'Custom Theme Engine',
    ],
    algorithms: [
      'LZ4 Real-time Compression',
      'Delta Encoding for Scene Diffs',
      'B-tree Spatial Indexing',
      'Bloom Filter Quick Lookups',
      'Write-ahead Logging (WAL)',
      'MVCC Transaction Support',
    ],
    rendering: [
      'WebGPU Compute Shaders',
      'Instanced Mesh Rendering',
      'Level-of-Detail System',
      'Shadow Mapping',
      'Post-processing Effects',
      'VR Mode Support',
    ],
    aiml: [
      'Crash Pattern Recognition',
      'Speed Estimation from Damage',
      'Trajectory Prediction',
      'Anomaly Detection',
    ],
  };
}

/**
 * Initialize v0.3.0 enterprise features
 *
 * @param config - Optional configuration override
 * @returns Promise that resolves when initialization is complete
 */
export async function initializeV030(
  config?: Partial<import('./types').V030EnterpriseConfig>
): Promise<void> {
  console.log(`🚀 Initializing AccuScene Enterprise v${VERSION}`);
  console.log(`📦 ${VERSION_INFO.rust.crates} Rust crates`);
  console.log(`📝 ${VERSION_INFO.typescript.modules} TypeScript modules`);

  // Log enabled feature categories
  if (config?.physics?.enabled) {
    console.log('  ⚛️  Advanced Physics Engine');
    if (config.physics.enableDeformables) {
      console.log('    └─ Deformable Bodies');
    }
    if (config.physics.tireFrictionModel) {
      console.log('    └─ Tire Friction Model');
    }
    if (config.physics.enableCrushZones) {
      console.log('    └─ Crush Zone Simulation');
    }
  }

  if (config?.gui?.cadToolbar) {
    console.log('  🎨 Professional CAD/CAM GUI');
    if (config.gui.propertyPanels) {
      console.log('    └─ Property Panels');
    }
    if (config.gui.commandPalette) {
      console.log('    └─ Command Palette');
    }
    if (config.gui.multiViewport) {
      console.log('    └─ Multi-viewport Layout');
    }
  }

  if (config?.algorithms?.lz4Compression) {
    console.log('  🔧 Database & Algorithms');
    if (config.algorithms.deltaEncoding) {
      console.log('    └─ Delta Encoding');
    }
    if (config.algorithms.btreeIndexing) {
      console.log('    └─ B-tree Indexing');
    }
    if (config.algorithms.mvcc) {
      console.log('    └─ MVCC Transactions');
    }
  }

  if (config?.rendering?.backend) {
    console.log(`  🎬 ${config.rendering.backend.toUpperCase()} Rendering`);
    if (config.rendering.computeShaders) {
      console.log('    └─ Compute Shaders');
    }
    if (config.rendering.shadows) {
      console.log('    └─ Shadow Mapping');
    }
    if (config.rendering.postProcessing) {
      console.log('    └─ Post-processing');
    }
  }

  if (config?.aiml?.enabled) {
    console.log('  🤖 AI/ML Capabilities');
    if (config.aiml.crashPattern) {
      console.log('    └─ Crash Pattern Recognition');
    }
    if (config.aiml.speedEstimation) {
      console.log('    └─ Speed Estimation');
    }
    if (config.aiml.trajectoryPrediction) {
      console.log('    └─ Trajectory Prediction');
    }
  }

  if (config?.performance?.streaming) {
    console.log('  ⚡ Performance Optimization');
    if (config.performance.metricsEnabled) {
      console.log('    └─ Performance Metrics');
    }
    if (config.performance.profiling) {
      console.log('    └─ Profiling');
    }
  }

  console.log(`✅ AccuScene Enterprise v${VERSION} initialized`);
}

/**
 * Get system requirements for v0.3.0
 */
export function getSystemRequirements() {
  return {
    minimum: {
      cpu: '4-core processor',
      memory: '8 GB RAM',
      gpu: 'WebGL 2.0 compatible',
      storage: '2 GB available space',
      os: 'Windows 10, macOS 10.15, or Linux',
    },
    recommended: {
      cpu: '8-core processor or better',
      memory: '16 GB RAM or more',
      gpu: 'WebGPU compatible (for advanced rendering)',
      storage: '5 GB available space',
      os: 'Windows 11, macOS 12, or Linux (latest)',
    },
    forPhysics: {
      cpu: '16-core processor or better',
      memory: '32 GB RAM or more',
      note: 'High-precision physics requires substantial computational resources',
    },
  };
}

/**
 * Get compatibility information
 */
export function getCompatibilityInfo() {
  return {
    browsers: {
      chrome: '113+',
      edge: '113+',
      firefox: '115+',
      safari: '16.4+',
    },
    webgpu: {
      chrome: '113+',
      edge: '113+',
      note: 'WebGPU support is required for advanced rendering features',
    },
    electron: '28.0.0+',
    node: '18.0.0+',
  };
}

// ============================================================================
// Re-export v0.2.5 features for convenience
// ============================================================================

export * from '../v0.2.5';
