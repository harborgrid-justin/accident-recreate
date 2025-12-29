/**
 * Unified types for AccuScene Enterprise v0.3.0
 *
 * This module provides comprehensive type definitions for all v0.3.0
 * enterprise features including advanced physics, CAD/CAM GUI, algorithms,
 * 3D rendering, and AI/ML capabilities.
 *
 * @module enterprise/v0.3.0/types
 * @version 0.3.0
 */

// ============================================================================
// Re-export v0.2.5 types for compatibility
// ============================================================================

export * from '../v0.2.5/types';

// ============================================================================
// v0.3.0 Core Types
// ============================================================================

/**
 * v0.3.0 Feature categories
 */
export type V030FeatureCategory =
  | 'physics'
  | 'gui'
  | 'algorithms'
  | 'rendering'
  | 'ai-ml'
  | 'performance';

/**
 * Physics engine mode
 */
export type PhysicsEngineMode = 'realtime' | 'highprecision' | 'batch';

/**
 * Rendering backend
 */
export type RenderingBackend = 'webgl2' | 'webgpu' | 'hybrid';

// ============================================================================
// Physics Engineering Types (v0.3.0)
// ============================================================================

/**
 * Advanced rigid body configuration
 */
export interface RigidBodyConfig {
  mass: number;
  centerOfMass: Vector3;
  momentOfInertia: Matrix3x3;
  restitution: number;
  friction: number;
  angularDamping: number;
  linearDamping: number;
}

/**
 * Deformable body properties
 */
export interface DeformableBodyConfig {
  stiffness: number;
  plasticityThreshold: number;
  fractureThreshold: number;
  damping: number;
  meshResolution: number;
}

/**
 * Tire friction model parameters
 */
export interface TireFrictionModel {
  staticFriction: number;
  kineticFriction: number;
  rollingResistance: number;
  lateralStiffness: number;
  longitudinalStiffness: number;
  slipAngle: number;
  slipRatio: number;
}

/**
 * Energy absorption calculation
 */
export interface EnergyAbsorption {
  totalEnergy: number;
  kineticEnergy: number;
  potentialEnergy: number;
  deformationEnergy: number;
  dissipatedEnergy: number;
  timestamp: number;
}

/**
 * Multi-body constraint
 */
export interface MultiBodyConstraint {
  id: string;
  type: 'fixed' | 'hinge' | 'slider' | 'ball-socket' | 'universal';
  bodyA: string;
  bodyB: string;
  anchorA: Vector3;
  anchorB: Vector3;
  axis?: Vector3;
  limits?: { min: number; max: number };
  breakForce?: number;
}

/**
 * Impact force distribution
 */
export interface ImpactForceDistribution {
  contactPoints: ContactPoint[];
  totalForce: Vector3;
  totalMoment: Vector3;
  maxPressure: number;
  averagePressure: number;
  contactArea: number;
}

/**
 * Contact point data
 */
export interface ContactPoint {
  position: Vector3;
  normal: Vector3;
  force: number;
  penetration: number;
  impulse: number;
}

/**
 * Momentum transfer analysis
 */
export interface MomentumTransfer {
  linearMomentumBefore: Vector3;
  linearMomentumAfter: Vector3;
  angularMomentumBefore: Vector3;
  angularMomentumAfter: Vector3;
  impulse: Vector3;
  conservationError: number;
}

/**
 * Vehicle crush zone data
 */
export interface VehicleCrushZone {
  zoneId: string;
  depth: number;
  area: number;
  volume: number;
  energyAbsorbed: number;
  peakForce: number;
  crushDuration: number;
  deformationProfile: Vector3[];
}

// ============================================================================
// Enterprise GUI/CAD Types (v0.3.0)
// ============================================================================

/**
 * CAD toolbar configuration
 */
export interface CADToolbarConfig {
  position: 'top' | 'left' | 'right' | 'bottom';
  orientation: 'horizontal' | 'vertical';
  collapsible: boolean;
  tools: CADTool[];
}

/**
 * CAD tool definition
 */
export interface CADTool {
  id: string;
  name: string;
  icon: string;
  tooltip: string;
  shortcut?: string;
  category: string;
  enabled: boolean;
  action: () => void;
}

/**
 * Property panel configuration
 */
export interface PropertyPanel {
  title: string;
  properties: PropertyDefinition[];
  collapsible: boolean;
  defaultCollapsed: boolean;
}

/**
 * Property definition
 */
export interface PropertyDefinition {
  key: string;
  label: string;
  type: 'string' | 'number' | 'boolean' | 'color' | 'select' | 'vector' | 'matrix';
  value: any;
  options?: any[];
  min?: number;
  max?: number;
  step?: number;
  readonly?: boolean;
  onChange?: (value: any) => void;
}

/**
 * Command palette item
 */
export interface CommandPaletteItem {
  id: string;
  name: string;
  description?: string;
  category: string;
  keywords: string[];
  shortcut?: string;
  icon?: string;
  execute: () => void | Promise<void>;
}

/**
 * Measurement annotation
 */
export interface MeasurementAnnotation {
  id: string;
  type: 'distance' | 'angle' | 'area' | 'volume' | 'radius' | 'arc';
  points: Vector3[];
  value: number;
  unit: string;
  label?: string;
  visible: boolean;
  style: AnnotationStyle;
}

/**
 * Annotation style
 */
export interface AnnotationStyle {
  color: string;
  lineWidth: number;
  fontSize: number;
  arrowSize: number;
  precision: number;
}

/**
 * Layer definition
 */
export interface Layer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  opacity: number;
  color?: string;
  objects: string[];
  parent?: string;
  children: string[];
}

/**
 * Snap settings
 */
export interface SnapSettings {
  enabled: boolean;
  gridSnap: boolean;
  vertexSnap: boolean;
  edgeSnap: boolean;
  centerSnap: boolean;
  gridSize: number;
  snapDistance: number;
  magneticGuides: boolean;
  angleSnap: boolean;
  angleIncrement: number;
}

/**
 * Viewport layout
 */
export interface ViewportLayout {
  type: 'single' | 'dual-horizontal' | 'dual-vertical' | 'quad' | 'triple' | 'custom';
  viewports: ViewportConfig[];
}

/**
 * Viewport configuration
 */
export interface ViewportConfig {
  id: string;
  position: { x: number; y: number };
  size: { width: number; height: number };
  camera: CameraConfig;
  renderMode: 'wireframe' | 'shaded' | 'textured' | 'realistic';
  showGrid: boolean;
  showAxes: boolean;
}

/**
 * Camera configuration
 */
export interface CameraConfig {
  type: 'perspective' | 'orthographic';
  position: Vector3;
  target: Vector3;
  up: Vector3;
  fov?: number;
  near: number;
  far: number;
  zoom: number;
}

/**
 * Theme configuration
 */
export interface ThemeConfig {
  id: string;
  name: string;
  colors: ThemeColors;
  fonts: ThemeFonts;
  spacing: ThemeSpacing;
  shadows: ThemeShadows;
}

/**
 * Theme colors
 */
export interface ThemeColors {
  primary: string;
  secondary: string;
  background: string;
  surface: string;
  text: string;
  textSecondary: string;
  border: string;
  success: string;
  warning: string;
  error: string;
  info: string;
}

/**
 * Theme fonts
 */
export interface ThemeFonts {
  family: string;
  monoFamily: string;
  sizeBase: number;
  sizeSmall: number;
  sizeLarge: number;
  weightNormal: number;
  weightBold: number;
}

/**
 * Theme spacing
 */
export interface ThemeSpacing {
  xs: number;
  sm: number;
  md: number;
  lg: number;
  xl: number;
}

/**
 * Theme shadows
 */
export interface ThemeShadows {
  small: string;
  medium: string;
  large: string;
}

// ============================================================================
// Database & Algorithm Types (v0.3.0)
// ============================================================================

/**
 * LZ4 compression settings
 */
export interface LZ4CompressionConfig {
  enabled: boolean;
  compressionLevel: number;
  blockSize: number;
  checksum: boolean;
  dictionary?: Uint8Array;
}

/**
 * Delta encoding configuration
 */
export interface DeltaEncodingConfig {
  enabled: boolean;
  baselineInterval: number;
  compressionThreshold: number;
  maxDeltaChain: number;
}

/**
 * B-tree index configuration
 */
export interface BTreeIndexConfig {
  order: number;
  unique: boolean;
  sparse: boolean;
  dimensions: number;
}

/**
 * Bloom filter settings
 */
export interface BloomFilterConfig {
  expectedElements: number;
  falsePositiveRate: number;
  hashFunctions: number;
  bitsPerElement: number;
}

/**
 * Write-ahead log configuration
 */
export interface WALConfig {
  enabled: boolean;
  maxLogSize: number;
  syncMode: 'none' | 'normal' | 'full';
  checkpointInterval: number;
  compressionEnabled: boolean;
}

/**
 * MVCC transaction settings
 */
export interface MVCCConfig {
  isolationLevel: 'read-uncommitted' | 'read-committed' | 'repeatable-read' | 'serializable';
  snapshotRetention: number;
  gcInterval: number;
  maxConcurrentTxns: number;
}

// ============================================================================
// 3D/Rendering Types (v0.3.0)
// ============================================================================

/**
 * WebGPU compute shader configuration
 */
export interface ComputeShaderConfig {
  code: string;
  entryPoint: string;
  workgroupSize: Vector3;
  buffers: GPUBufferBinding[];
}

/**
 * GPU buffer binding
 */
export interface GPUBufferBinding {
  binding: number;
  buffer: GPUBuffer;
  type: 'storage' | 'uniform' | 'read-only-storage';
}

/**
 * Instanced rendering configuration
 */
export interface InstancedRenderConfig {
  maxInstances: number;
  instanceData: Float32Array;
  attributes: InstanceAttribute[];
  frustumCulling: boolean;
}

/**
 * Instance attribute
 */
export interface InstanceAttribute {
  name: string;
  size: number;
  offset: number;
  stride: number;
}

/**
 * Level of detail system
 */
export interface LODConfig {
  levels: LODLevel[];
  distanceMetric: 'euclidean' | 'screen-space' | 'pixel-error';
  hysteresis: number;
}

/**
 * LOD level definition
 */
export interface LODLevel {
  distance: number;
  mesh: string;
  materialOverride?: string;
}

/**
 * Shadow mapping configuration
 */
export interface ShadowMapConfig {
  enabled: boolean;
  resolution: number;
  cascades: number;
  bias: number;
  normalBias: number;
  softness: number;
  fadeDistance: number;
}

/**
 * Post-processing effect
 */
export interface PostProcessEffect {
  id: string;
  type: 'bloom' | 'ssao' | 'fxaa' | 'tonemap' | 'color-grading' | 'motion-blur' | 'dof';
  enabled: boolean;
  params: Record<string, any>;
  order: number;
}

/**
 * VR mode configuration
 */
export interface VRModeConfig {
  enabled: boolean;
  stereoMode: 'side-by-side' | 'top-bottom' | 'anaglyph';
  ipd: number;
  fov: number;
  distortionCorrection: boolean;
}

// ============================================================================
// AI/ML Types (v0.3.0)
// ============================================================================

/**
 * Crash pattern recognition result
 */
export interface CrashPatternResult {
  patternId: string;
  patternName: string;
  confidence: number;
  features: Record<string, number>;
  similar: SimilarCrash[];
}

/**
 * Similar crash reference
 */
export interface SimilarCrash {
  id: string;
  similarity: number;
  metadata: Record<string, any>;
}

/**
 * Speed estimation from damage
 */
export interface SpeedEstimation {
  estimatedSpeed: number;
  confidence: number;
  range: { min: number; max: number };
  method: string;
  factors: SpeedEstimationFactor[];
}

/**
 * Speed estimation factor
 */
export interface SpeedEstimationFactor {
  name: string;
  value: number;
  weight: number;
  confidence: number;
}

/**
 * Trajectory prediction
 */
export interface TrajectoryPrediction {
  points: TrajectoryPoint[];
  confidence: number;
  alternativePaths: TrajectoryPoint[][];
}

/**
 * Trajectory point
 */
export interface TrajectoryPoint {
  time: number;
  position: Vector3;
  velocity: Vector3;
  acceleration: Vector3;
  probability: number;
}

/**
 * Anomaly detection result
 */
export interface AnomalyDetection {
  isAnomaly: boolean;
  score: number;
  threshold: number;
  features: string[];
  explanation?: string;
}

// ============================================================================
// Performance Optimization Types (v0.3.0)
// ============================================================================

/**
 * Streaming optimization configuration
 */
export interface StreamingOptimizationConfig {
  enabled: boolean;
  chunkSize: number;
  prefetchDistance: number;
  cacheSize: number;
  priorityLevels: number;
}

/**
 * Performance metrics
 */
export interface PerformanceMetrics {
  fps: number;
  frameTime: number;
  drawCalls: number;
  triangles: number;
  memoryUsage: MemoryUsage;
  gpuUtilization: number;
  cpuUtilization: number;
}

/**
 * Memory usage breakdown
 */
export interface MemoryUsage {
  total: number;
  used: number;
  free: number;
  buffers: number;
  textures: number;
  geometry: number;
  other: number;
}

// ============================================================================
// Utility Types
// ============================================================================

/**
 * 3D vector
 */
export interface Vector3 {
  x: number;
  y: number;
  z: number;
}

/**
 * 3x3 matrix
 */
export type Matrix3x3 = [
  [number, number, number],
  [number, number, number],
  [number, number, number]
];

/**
 * GPU buffer (placeholder)
 */
export type GPUBuffer = any;

// ============================================================================
// v0.3.0 Configuration Types
// ============================================================================

/**
 * v0.3.0 extended enterprise configuration
 */
export interface V030EnterpriseConfig {
  physics: PhysicsConfig;
  gui: GUIConfig;
  algorithms: AlgorithmsConfig;
  rendering: RenderingConfig;
  aiml: AIMLConfig;
  performance: PerformanceConfig;
}

/**
 * Physics engine configuration
 */
export interface PhysicsConfig {
  enabled: boolean;
  mode: PhysicsEngineMode;
  timestep: number;
  substeps: number;
  solverIterations: number;
  enableDeformables: boolean;
  tireFrictionModel: boolean;
  enableCrushZones: boolean;
}

/**
 * GUI configuration
 */
export interface GUIConfig {
  cadToolbar: boolean;
  propertyPanels: boolean;
  commandPalette: boolean;
  measurements: boolean;
  layers: boolean;
  snapToGrid: boolean;
  multiViewport: boolean;
  customThemes: boolean;
}

/**
 * Algorithms configuration
 */
export interface AlgorithmsConfig {
  lz4Compression: boolean;
  deltaEncoding: boolean;
  btreeIndexing: boolean;
  bloomFilter: boolean;
  wal: boolean;
  mvcc: boolean;
}

/**
 * Rendering configuration
 */
export interface RenderingConfig {
  backend: RenderingBackend;
  computeShaders: boolean;
  instancing: boolean;
  lod: boolean;
  shadows: boolean;
  postProcessing: boolean;
  vrMode: boolean;
}

/**
 * AI/ML configuration
 */
export interface AIMLConfig {
  enabled: boolean;
  crashPattern: boolean;
  speedEstimation: boolean;
  trajectoryPrediction: boolean;
  anomalyDetection: boolean;
  modelPath?: string;
}

/**
 * Performance configuration
 */
export interface PerformanceConfig {
  streaming: boolean;
  metricsEnabled: boolean;
  profiling: boolean;
  optimization: 'balanced' | 'quality' | 'performance';
}
