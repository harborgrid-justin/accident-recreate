/**
 * Enterprise configuration for AccuScene v0.3.0
 *
 * This module provides default configuration and configuration utilities
 * for the AccuScene Enterprise v0.3.0 platform with advanced physics,
 * GUI, algorithms, rendering, and AI/ML capabilities.
 *
 * @module enterprise/v0.3.0/config
 * @version 0.3.0
 */

import type {
  V030EnterpriseConfig,
  PhysicsConfig,
  GUIConfig,
  AlgorithmsConfig,
  RenderingConfig,
  AIMLConfig,
  PerformanceConfig,
  PhysicsEngineMode,
  RenderingBackend,
} from './types';

// Re-export v0.2.5 config utilities for compatibility
export * from '../v0.2.5/config';

// ============================================================================
// v0.3.0 Default Configurations
// ============================================================================

/**
 * Default physics engine configuration
 */
export const defaultPhysicsConfig: PhysicsConfig = {
  enabled: process.env.PHYSICS_ENABLED !== 'false',
  mode: (process.env.PHYSICS_MODE as PhysicsEngineMode) || 'highprecision',
  timestep: parseFloat(process.env.PHYSICS_TIMESTEP || '0.016'),
  substeps: parseInt(process.env.PHYSICS_SUBSTEPS || '4', 10),
  solverIterations: parseInt(process.env.PHYSICS_SOLVER_ITERATIONS || '10', 10),
  enableDeformables: process.env.PHYSICS_DEFORMABLES !== 'false',
  tireFrictionModel: process.env.PHYSICS_TIRE_FRICTION !== 'false',
  enableCrushZones: process.env.PHYSICS_CRUSH_ZONES !== 'false',
};

/**
 * Default GUI configuration
 */
export const defaultGUIConfig: GUIConfig = {
  cadToolbar: process.env.GUI_CAD_TOOLBAR !== 'false',
  propertyPanels: process.env.GUI_PROPERTY_PANELS !== 'false',
  commandPalette: process.env.GUI_COMMAND_PALETTE !== 'false',
  measurements: process.env.GUI_MEASUREMENTS !== 'false',
  layers: process.env.GUI_LAYERS !== 'false',
  snapToGrid: process.env.GUI_SNAP_TO_GRID !== 'false',
  multiViewport: process.env.GUI_MULTI_VIEWPORT !== 'false',
  customThemes: process.env.GUI_CUSTOM_THEMES !== 'false',
};

/**
 * Default algorithms configuration
 */
export const defaultAlgorithmsConfig: AlgorithmsConfig = {
  lz4Compression: process.env.ALGO_LZ4 !== 'false',
  deltaEncoding: process.env.ALGO_DELTA !== 'false',
  btreeIndexing: process.env.ALGO_BTREE !== 'false',
  bloomFilter: process.env.ALGO_BLOOM !== 'false',
  wal: process.env.ALGO_WAL !== 'false',
  mvcc: process.env.ALGO_MVCC !== 'false',
};

/**
 * Default rendering configuration
 */
export const defaultRenderingConfig: RenderingConfig = {
  backend: (process.env.RENDERING_BACKEND as RenderingBackend) || 'webgpu',
  computeShaders: process.env.RENDERING_COMPUTE !== 'false',
  instancing: process.env.RENDERING_INSTANCING !== 'false',
  lod: process.env.RENDERING_LOD !== 'false',
  shadows: process.env.RENDERING_SHADOWS !== 'false',
  postProcessing: process.env.RENDERING_POST_PROCESSING !== 'false',
  vrMode: process.env.RENDERING_VR === 'true',
};

/**
 * Default AI/ML configuration
 */
export const defaultAIMLConfig: AIMLConfig = {
  enabled: process.env.AIML_ENABLED !== 'false',
  crashPattern: process.env.AIML_CRASH_PATTERN !== 'false',
  speedEstimation: process.env.AIML_SPEED_EST !== 'false',
  trajectoryPrediction: process.env.AIML_TRAJECTORY !== 'false',
  anomalyDetection: process.env.AIML_ANOMALY !== 'false',
  modelPath: process.env.AIML_MODEL_PATH || './models',
};

/**
 * Default performance configuration
 */
export const defaultPerformanceConfig: PerformanceConfig = {
  streaming: process.env.PERF_STREAMING !== 'false',
  metricsEnabled: process.env.PERF_METRICS !== 'false',
  profiling: process.env.PERF_PROFILING === 'true',
  optimization: (process.env.PERF_OPTIMIZATION as any) || 'balanced',
};

/**
 * Default v0.3.0 enterprise configuration
 */
export const defaultV030Config: V030EnterpriseConfig = {
  physics: defaultPhysicsConfig,
  gui: defaultGUIConfig,
  algorithms: defaultAlgorithmsConfig,
  rendering: defaultRenderingConfig,
  aiml: defaultAIMLConfig,
  performance: defaultPerformanceConfig,
};

// ============================================================================
// Configuration Utilities
// ============================================================================

/**
 * Merge v0.3.0 configurations
 */
export function mergeV030Configs(
  base: Partial<V030EnterpriseConfig>,
  override: Partial<V030EnterpriseConfig>
): V030EnterpriseConfig {
  return {
    physics: { ...defaultPhysicsConfig, ...base.physics, ...override.physics },
    gui: { ...defaultGUIConfig, ...base.gui, ...override.gui },
    algorithms: { ...defaultAlgorithmsConfig, ...base.algorithms, ...override.algorithms },
    rendering: { ...defaultRenderingConfig, ...base.rendering, ...override.rendering },
    aiml: { ...defaultAIMLConfig, ...base.aiml, ...override.aiml },
    performance: { ...defaultPerformanceConfig, ...base.performance, ...override.performance },
  };
}

/**
 * Validate v0.3.0 configuration
 */
export function validateV030Config(config: V030EnterpriseConfig): string[] {
  const errors: string[] = [];

  // Validate physics settings
  if (config.physics.enabled) {
    if (config.physics.timestep <= 0 || config.physics.timestep > 1) {
      errors.push('Physics timestep must be between 0 and 1 second');
    }
    if (config.physics.substeps < 1 || config.physics.substeps > 20) {
      errors.push('Physics substeps must be between 1 and 20');
    }
    if (config.physics.solverIterations < 1 || config.physics.solverIterations > 100) {
      errors.push('Physics solver iterations must be between 1 and 100');
    }
  }

  // Validate rendering settings
  if (config.rendering.backend === 'webgpu') {
    // Check WebGPU availability would happen at runtime
  }

  // Validate AI/ML settings
  if (config.aiml.enabled && !config.aiml.modelPath) {
    errors.push('AI/ML model path is required when AI/ML is enabled');
  }

  // Validate performance settings
  const validOptimizations = ['balanced', 'quality', 'performance'];
  if (!validOptimizations.includes(config.performance.optimization)) {
    errors.push(`Performance optimization must be one of: ${validOptimizations.join(', ')}`);
  }

  return errors;
}

/**
 * Get v0.3.0 feature flags from configuration
 */
export function getV030FeatureFlags(config: V030EnterpriseConfig) {
  return {
    // Physics features
    advancedPhysics: config.physics.enabled,
    deformableBodies: config.physics.enableDeformables,
    tireFriction: config.physics.tireFrictionModel,
    crushZones: config.physics.enableCrushZones,

    // GUI features
    cadToolbar: config.gui.cadToolbar,
    propertyPanels: config.gui.propertyPanels,
    commandPalette: config.gui.commandPalette,
    measurements: config.gui.measurements,
    layers: config.gui.layers,
    snapToGrid: config.gui.snapToGrid,
    multiViewport: config.gui.multiViewport,
    customThemes: config.gui.customThemes,

    // Algorithm features
    lz4Compression: config.algorithms.lz4Compression,
    deltaEncoding: config.algorithms.deltaEncoding,
    btreeIndexing: config.algorithms.btreeIndexing,
    bloomFilter: config.algorithms.bloomFilter,
    wal: config.algorithms.wal,
    mvcc: config.algorithms.mvcc,

    // Rendering features
    webgpu: config.rendering.backend === 'webgpu',
    computeShaders: config.rendering.computeShaders,
    instancing: config.rendering.instancing,
    lod: config.rendering.lod,
    shadows: config.rendering.shadows,
    postProcessing: config.rendering.postProcessing,
    vrMode: config.rendering.vrMode,

    // AI/ML features
    aiml: config.aiml.enabled,
    crashPattern: config.aiml.crashPattern,
    speedEstimation: config.aiml.speedEstimation,
    trajectoryPrediction: config.aiml.trajectoryPrediction,
    anomalyDetection: config.aiml.anomalyDetection,

    // Performance features
    streaming: config.performance.streaming,
    profiling: config.performance.profiling,
  };
}

/**
 * Load v0.3.0 configuration from environment
 */
export function loadV030ConfigFromEnv(): V030EnterpriseConfig {
  return defaultV030Config;
}

/**
 * Check if WebGPU is available
 */
export function isWebGPUAvailable(): boolean {
  if (typeof navigator !== 'undefined' && 'gpu' in navigator) {
    return true;
  }
  return false;
}

/**
 * Get recommended rendering backend
 */
export function getRecommendedRenderingBackend(): RenderingBackend {
  if (isWebGPUAvailable()) {
    return 'webgpu';
  }
  return 'webgl2';
}

/**
 * Check if high precision physics is enabled
 */
export function isHighPrecisionPhysics(config: V030EnterpriseConfig): boolean {
  return config.physics.mode === 'highprecision';
}

/**
 * Get physics engine performance settings
 */
export function getPhysicsPerformanceSettings(config: V030EnterpriseConfig) {
  const mode = config.physics.mode;

  switch (mode) {
    case 'realtime':
      return {
        timestep: 0.016,
        substeps: 2,
        solverIterations: 5,
      };
    case 'highprecision':
      return {
        timestep: 0.008,
        substeps: 8,
        solverIterations: 20,
      };
    case 'batch':
      return {
        timestep: 0.001,
        substeps: 10,
        solverIterations: 50,
      };
    default:
      return {
        timestep: config.physics.timestep,
        substeps: config.physics.substeps,
        solverIterations: config.physics.solverIterations,
      };
  }
}
