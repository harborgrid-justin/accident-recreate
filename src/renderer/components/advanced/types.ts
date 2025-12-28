/**
 * Advanced UI Component Types
 * AccuScene Enterprise v0.2.0
 */

import { ReactNode } from 'react';
import * as THREE from 'three';

// ============================================================================
// Scene3D Types
// ============================================================================

export interface Scene3DProps {
  children?: ReactNode;
  backgroundColor?: string;
  fog?: {
    color: string;
    near: number;
    far: number;
  };
  onSceneReady?: (scene: THREE.Scene) => void;
}

export interface CameraProps {
  position?: [number, number, number];
  fov?: number;
  near?: number;
  far?: number;
  lookAt?: [number, number, number];
  enableControls?: boolean;
  controlsType?: 'orbit' | 'fly' | 'first-person';
  autoRotate?: boolean;
  autoRotateSpeed?: number;
}

export interface LightingProps {
  ambient?: {
    color: string;
    intensity: number;
  };
  directional?: Array<{
    color: string;
    intensity: number;
    position: [number, number, number];
    castShadow?: boolean;
  }>;
  point?: Array<{
    color: string;
    intensity: number;
    position: [number, number, number];
    distance?: number;
    decay?: number;
  }>;
  spot?: Array<{
    color: string;
    intensity: number;
    position: [number, number, number];
    angle?: number;
    penumbra?: number;
    castShadow?: boolean;
  }>;
}

export interface GroundProps {
  size?: number;
  gridSize?: number;
  color?: string;
  gridColor?: string;
  showGrid?: boolean;
  receiveShadow?: boolean;
}

export interface EnvironmentProps {
  preset?: 'sunset' | 'dawn' | 'night' | 'warehouse' | 'forest' | 'apartment' | 'studio' | 'city' | 'park' | 'lobby';
  background?: boolean;
  blur?: number;
}

// ============================================================================
// Vehicle3D Types
// ============================================================================

export interface Vehicle3DProps {
  vehicleId: string;
  modelUrl?: string;
  position: [number, number, number];
  rotation: [number, number, number];
  scale?: [number, number, number];
  color?: string;
  showDamage?: boolean;
  showPhysics?: boolean;
  showTrajectory?: boolean;
  interactive?: boolean;
  onClick?: (vehicleId: string) => void;
  onHover?: (vehicleId: string, hovered: boolean) => void;
}

export interface DamagePoint {
  position: [number, number, number];
  severity: number; // 0-1
  type: 'impact' | 'scratch' | 'dent' | 'deformation';
  radius: number;
}

export interface DamageProps {
  damages: DamagePoint[];
  showLabels?: boolean;
  colorScale?: string[];
}

export interface PhysicsOverlayProps {
  centerOfMass?: [number, number, number];
  forceVectors?: Array<{
    origin: [number, number, number];
    direction: [number, number, number];
    magnitude: number;
    color?: string;
  }>;
  velocityVector?: {
    direction: [number, number, number];
    magnitude: number;
  };
  showLabels?: boolean;
}

export interface TrajectoryPoint {
  position: [number, number, number];
  rotation: [number, number, number];
  timestamp: number;
  velocity: number;
}

export interface TrajectoryProps {
  points: TrajectoryPoint[];
  color?: string;
  opacity?: number;
  showVelocityGradient?: boolean;
  showKeyframes?: boolean;
}

// ============================================================================
// Simulation3D Types
// ============================================================================

export interface Simulation3DProps {
  simulationData: SimulationData;
  currentTime: number;
  playing?: boolean;
  playbackSpeed?: number;
  onTimeChange?: (time: number) => void;
  onPlayingChange?: (playing: boolean) => void;
}

export interface SimulationData {
  duration: number;
  frameRate: number;
  vehicles: VehicleSimulation[];
  events: SimulationEvent[];
  environment: {
    weather?: 'clear' | 'rain' | 'snow' | 'fog';
    timeOfDay?: 'day' | 'night' | 'dawn' | 'dusk';
    temperature?: number;
  };
}

export interface VehicleSimulation {
  vehicleId: string;
  keyframes: TrajectoryPoint[];
  model: string;
  color: string;
}

export interface SimulationEvent {
  timestamp: number;
  type: 'collision' | 'brake' | 'acceleration' | 'swerve';
  vehicleIds: string[];
  severity?: number;
  description?: string;
}

export interface TimelineProps {
  duration: number;
  currentTime: number;
  events: SimulationEvent[];
  playing: boolean;
  playbackSpeed: number;
  onTimeChange: (time: number) => void;
  onPlayingChange: (playing: boolean) => void;
  onSpeedChange: (speed: number) => void;
}

export interface ControlsProps {
  playing: boolean;
  playbackSpeed: number;
  onPlay: () => void;
  onPause: () => void;
  onStop: () => void;
  onSpeedChange: (speed: number) => void;
  showFrameControls?: boolean;
  onNextFrame?: () => void;
  onPrevFrame?: () => void;
}

export interface AnalysisProps {
  showForces?: boolean;
  showVelocities?: boolean;
  showAccelerations?: boolean;
  showCollisionPoints?: boolean;
  showTrajectories?: boolean;
  colorScheme?: 'default' | 'heatmap' | 'physics';
}

// ============================================================================
// AR Types
// ============================================================================

export interface ARProps {
  enabled: boolean;
  scene: THREE.Scene;
  camera: THREE.Camera;
  markerDetection?: boolean;
  trackingMode?: 'world' | 'image' | 'face';
  onTrackingStatusChange?: (tracking: boolean) => void;
}

export interface AROverlayProps {
  visible: boolean;
  opacity?: number;
  elements?: ARElement[];
}

export interface ARElement {
  id: string;
  type: 'label' | 'measurement' | 'arrow' | 'annotation';
  position: [number, number, number];
  content: string | ReactNode;
  color?: string;
}

export interface ARMarkersProps {
  markers: ARMarker[];
  showLabels?: boolean;
  onMarkerDetected?: (markerId: string) => void;
}

export interface ARMarker {
  id: string;
  type: 'qr' | 'image' | 'nft';
  position: [number, number, number];
  scale: number;
  imageUrl?: string;
}

// ============================================================================
// Charts Types
// ============================================================================

export interface ChartData {
  labels: string[];
  datasets: ChartDataset[];
}

export interface ChartDataset {
  label: string;
  data: number[];
  color?: string;
  borderColor?: string;
  backgroundColor?: string;
  fill?: boolean;
}

export interface ForceChartProps {
  data: ChartData;
  title?: string;
  showLegend?: boolean;
  height?: number;
  unit?: string;
}

export interface VelocityChartProps {
  data: ChartData;
  title?: string;
  showLegend?: boolean;
  height?: number;
  showAcceleration?: boolean;
}

export interface EnergyChartProps {
  data: ChartData;
  title?: string;
  showLegend?: boolean;
  height?: number;
  showKinetic?: boolean;
  showPotential?: boolean;
}

export interface ImpactChartProps {
  data: ChartData;
  title?: string;
  showLegend?: boolean;
  height?: number;
  highlightPeak?: boolean;
}

// ============================================================================
// Heatmap Types
// ============================================================================

export interface HeatmapProps {
  width: number;
  height: number;
  data: HeatmapData[];
  colorScale?: string[];
  opacity?: number;
  blur?: number;
}

export interface HeatmapData {
  x: number;
  y: number;
  value: number;
}

export interface DamageHeatmapProps extends HeatmapProps {
  vehicleOutline?: string;
  showSeverityLabels?: boolean;
}

export interface ForceHeatmapProps extends HeatmapProps {
  showVectors?: boolean;
  vectorScale?: number;
}

// ============================================================================
// Timeline Types
// ============================================================================

export interface TimelineEvent {
  id: string;
  timestamp: number;
  type: string;
  title: string;
  description?: string;
  color?: string;
  icon?: string;
}

export interface TimelineComponentProps {
  events: TimelineEvent[];
  currentTime: number;
  duration: number;
  onTimeChange: (time: number) => void;
  onEventClick?: (event: TimelineEvent) => void;
  showKeyframes?: boolean;
  editable?: boolean;
}

export interface EventsProps {
  events: TimelineEvent[];
  currentTime: number;
  onEventClick?: (event: TimelineEvent) => void;
  editable?: boolean;
  onEventAdd?: (timestamp: number) => void;
  onEventRemove?: (eventId: string) => void;
  onEventUpdate?: (event: TimelineEvent) => void;
}

export interface Keyframe {
  id: string;
  timestamp: number;
  properties: Record<string, any>;
  interpolation?: 'linear' | 'ease' | 'ease-in' | 'ease-out' | 'ease-in-out';
}

export interface KeyframesProps {
  keyframes: Keyframe[];
  currentTime: number;
  duration: number;
  onKeyframeAdd?: (keyframe: Keyframe) => void;
  onKeyframeRemove?: (keyframeId: string) => void;
  onKeyframeUpdate?: (keyframe: Keyframe) => void;
  selectedProperty?: string;
}

// ============================================================================
// Toolbar Types
// ============================================================================

export interface Tool {
  id: string;
  name: string;
  icon: string;
  description?: string;
  shortcut?: string;
  category?: string;
}

export interface ToolbarProps {
  tools: Tool[];
  activeTool?: string;
  onToolSelect: (toolId: string) => void;
  orientation?: 'horizontal' | 'vertical';
  compact?: boolean;
}

export interface ToolsProps {
  tools: Tool[];
  activeTool?: string;
  onToolSelect: (toolId: string) => void;
  showLabels?: boolean;
  showShortcuts?: boolean;
}

export interface EditMode {
  id: string;
  name: string;
  icon: string;
  description?: string;
}

export interface ModesProps {
  modes: EditMode[];
  activeMode?: string;
  onModeSelect: (modeId: string) => void;
}

// ============================================================================
// Panel Types
// ============================================================================

export interface Property {
  id: string;
  name: string;
  value: any;
  type: 'string' | 'number' | 'boolean' | 'color' | 'select' | 'vector3';
  min?: number;
  max?: number;
  step?: number;
  options?: Array<{ label: string; value: any }>;
  category?: string;
}

export interface PropertiesProps {
  properties: Property[];
  onChange: (propertyId: string, value: any) => void;
  title?: string;
  collapsible?: boolean;
  groupByCategory?: boolean;
}

export interface Layer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  type: 'vehicle' | 'environment' | 'annotation' | 'measurement';
  children?: Layer[];
}

export interface LayersProps {
  layers: Layer[];
  selectedLayer?: string;
  onLayerSelect: (layerId: string) => void;
  onLayerToggle: (layerId: string, visible: boolean) => void;
  onLayerLock: (layerId: string, locked: boolean) => void;
  onLayerAdd?: (parentId?: string) => void;
  onLayerRemove?: (layerId: string) => void;
  onLayerRename?: (layerId: string, name: string) => void;
}

export interface HistoryAction {
  id: string;
  type: string;
  description: string;
  timestamp: number;
  canUndo: boolean;
  canRedo: boolean;
}

export interface HistoryProps {
  actions: HistoryAction[];
  currentIndex: number;
  onUndo: () => void;
  onRedo: () => void;
  onJumpTo?: (index: number) => void;
  maxActions?: number;
}

// ============================================================================
// Animation Types
// ============================================================================

export interface AnimationConfig {
  duration: number;
  easing?: 'linear' | 'ease' | 'ease-in' | 'ease-out' | 'ease-in-out';
  delay?: number;
  loop?: boolean;
  yoyo?: boolean;
}

export interface SpringConfig {
  mass?: number;
  tension?: number;
  friction?: number;
  clamp?: boolean;
  precision?: number;
  velocity?: number;
}
