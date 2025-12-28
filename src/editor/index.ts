/**
 * Diagram Editor Module - Main Export
 * AccuScene Enterprise Accident Recreation Platform
 */

// Core Components
export { DiagramCanvas } from './DiagramCanvas';
export type { DiagramCanvasProps } from './DiagramCanvas';

export { DiagramToolbar } from './DiagramToolbar';
export type { ToolbarProps } from './DiagramToolbar';

// State Management
export { DiagramStateManager, diagramStateManager } from './DiagramState';

// Element Definitions
export {
  VEHICLE_DEFINITIONS,
  ROAD_DEFINITIONS,
  ENVIRONMENTAL_DEFINITIONS,
  MARKER_DEFINITIONS,
  getAllElementDefinitions,
  getElementDefinition,
  createElement,
  getElementBounds,
  isPointInElement,
  rotatePoint,
  distance,
  angle,
} from './DiagramElements';
export type { ElementDefinition } from './DiagramElements';

// Vehicle Path
export { VehiclePath, vehiclePath } from './VehiclePath';
export type { PathPoint } from './VehiclePath';

// Measurement Tool
export { MeasurementTool, measurementTool } from './MeasurementTool';

// Exporter
export { DiagramExporter, diagramExporter } from './DiagramExporter';

// Re-export types
export type {
  Point,
  Size,
  Transform,
  ElementType,
  VehicleType,
  RoadElementType,
  EnvironmentalType,
  MarkerType,
  DiagramElement,
  VehicleElement,
  PathElement,
  AnnotationElement,
  Measurement,
  DiagramState,
  HistoryEntry,
  ToolType,
  EditorTool,
  ExportOptions,
  ViewportState,
  AnimationFrame,
  PathAnimation,
} from '../types/diagram';
