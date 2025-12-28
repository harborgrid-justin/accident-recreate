/**
 * Advanced UI Components
 * AccuScene Enterprise v0.2.0
 *
 * Export all advanced 3D visualization and UI components
 */

// Types
export * from './types';

// Hooks
export { useScene3D, type UseScene3DOptions, type UseScene3DReturn } from './hooks/useScene3D';
export { useAnimation, useSpring, type UseAnimationOptions, type UseAnimationReturn } from './hooks/useAnimation';
export { useAR, type UseAROptions, type UseARReturn } from './hooks/useAR';

// Scene3D Components
export { default as Scene3D } from './Scene3D';
export { default as Camera } from './Scene3D/Camera';
export { default as Lighting } from './Scene3D/Lighting';
export { default as Ground } from './Scene3D/Ground';
export { default as Environment } from './Scene3D/Environment';

// Vehicle3D Components
export { default as Vehicle3D } from './Vehicle3D';
export { default as Damage } from './Vehicle3D/Damage';
export { default as PhysicsOverlay } from './Vehicle3D/Physics';
export { default as Trajectory } from './Vehicle3D/Trajectory';

// Simulation3D Components
export { default as Simulation3D } from './Simulation3D';
export { default as SimulationTimeline } from './Simulation3D/Timeline';
export { default as SimulationControls } from './Simulation3D/Controls';
export { default as SimulationAnalysis } from './Simulation3D/Analysis';

// AR Components
export { default as AR } from './AR';
export { default as AROverlay } from './AR/Overlay';
export { default as ARMarkers } from './AR/Markers';

// Chart Components
export { ForceChart, VelocityChart, EnergyChart, ImpactChart } from './Charts';

// Heatmap Components
export { default as Heatmap } from './Heatmap';
export { default as DamageHeatmap } from './Heatmap/Damage';
export { default as ForceHeatmap } from './Heatmap/Force';

// Timeline Components
export { default as Timeline } from './Timeline';
export { default as TimelineEvents } from './Timeline/Events';
export { default as Keyframes } from './Timeline/Keyframes';

// Toolbar Components
export { default as Toolbar } from './Toolbar';
export { default as Tools } from './Toolbar/Tools';
export { default as Modes } from './Toolbar/Modes';

// Panel Components
export { default as Properties } from './Panel/Properties';
export { default as Layers } from './Panel/Layers';
export { default as History } from './Panel/History';
