/**
 * AccuScene Enterprise v0.3.0 - CAD UI Type Definitions
 * Professional CAD/CAM type system for TypeScript
 */

export type Point2D = {
  x: number;
  y: number;
};

export type Point3D = Point2D & {
  z: number;
};

export type BoundingBox = {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
};

export enum CADToolType {
  SELECT = 'select',
  PAN = 'pan',
  ZOOM = 'zoom',
  LINE = 'line',
  RECTANGLE = 'rectangle',
  CIRCLE = 'circle',
  ARC = 'arc',
  POLYLINE = 'polyline',
  POLYGON = 'polygon',
  SPLINE = 'spline',
  TEXT = 'text',
  DIMENSION = 'dimension',
  MEASURE_DISTANCE = 'measure_distance',
  MEASURE_ANGLE = 'measure_angle',
  MEASURE_AREA = 'measure_area',
  TRIM = 'trim',
  EXTEND = 'extend',
  OFFSET = 'offset',
  MIRROR = 'mirror',
  ROTATE = 'rotate',
  SCALE = 'scale',
  MOVE = 'move',
  COPY = 'copy',
  DELETE = 'delete',
}

export type CADTool = {
  id: string;
  type: CADToolType;
  name: string;
  icon: string;
  shortcut?: string;
  description: string;
  category: ToolCategory;
  enabled: boolean;
};

export enum ToolCategory {
  SELECTION = 'Selection',
  NAVIGATION = 'Navigation',
  DRAW = 'Draw',
  MODIFY = 'Modify',
  MEASURE = 'Measure',
  ANNOTATION = 'Annotation',
}

export type ToolGroup = {
  id: string;
  name: string;
  category: ToolCategory;
  tools: CADTool[];
  expanded?: boolean;
};

export type Layer = {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  color: string;
  opacity: number;
  order: number;
  objectCount: number;
  selected?: boolean;
  parent?: string;
};

export type PropertyValue = string | number | boolean | string[];

export type PropertyType =
  | 'text'
  | 'number'
  | 'angle'
  | 'color'
  | 'select'
  | 'checkbox'
  | 'slider'
  | 'point';

export type Property = {
  id: string;
  label: string;
  type: PropertyType;
  value: PropertyValue;
  min?: number;
  max?: number;
  step?: number;
  options?: Array<{ value: string; label: string }>;
  unit?: string;
  readonly?: boolean;
  section?: string;
};

export type PropertySection = {
  id: string;
  title: string;
  properties: Property[];
  collapsed?: boolean;
};

export type Command = {
  id: string;
  name: string;
  description: string;
  icon?: string;
  shortcut?: string;
  category: string;
  tags: string[];
  action: () => void | Promise<void>;
};

export enum SnapType {
  GRID = 'grid',
  ENDPOINT = 'endpoint',
  MIDPOINT = 'midpoint',
  CENTER = 'center',
  INTERSECTION = 'intersection',
  PERPENDICULAR = 'perpendicular',
  TANGENT = 'tangent',
  NEAREST = 'nearest',
  QUADRANT = 'quadrant',
}

export type SnapPoint = {
  point: Point2D;
  type: SnapType;
  objectId?: string;
  angle?: number;
};

export type SnapSettings = {
  enabled: boolean;
  gridSnap: boolean;
  objectSnap: boolean;
  snapTypes: SnapType[];
  snapDistance: number;
  gridSize: number;
};

export type Measurement = {
  id: string;
  type: 'distance' | 'angle' | 'area' | 'radius';
  points: Point2D[];
  value: number;
  unit: string;
  label?: string;
  color?: string;
};

export type GridSettings = {
  enabled: boolean;
  type: 'line' | 'dot';
  majorSpacing: number;
  minorSpacing: number;
  majorColor: string;
  minorColor: string;
  opacity: number;
};

export type RulerSettings = {
  enabled: boolean;
  size: number;
  backgroundColor: string;
  textColor: string;
  lineColor: string;
  unit: 'mm' | 'cm' | 'in' | 'ft' | 'px';
};

export type ViewportState = {
  zoom: number;
  panX: number;
  panY: number;
  rotation: number;
  width: number;
  height: number;
};

export type ContextMenuItem = {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  disabled?: boolean;
  separator?: boolean;
  submenu?: ContextMenuItem[];
  action?: () => void;
};

export type FloatingPanelConfig = {
  id: string;
  title: string;
  defaultPosition: Point2D;
  defaultSize: { width: number; height: number };
  minSize?: { width: number; height: number };
  resizable?: boolean;
  collapsible?: boolean;
  closable?: boolean;
};

export type Theme = 'light' | 'dark' | 'auto';

export type CADWorkspaceConfig = {
  theme: Theme;
  showGrid: boolean;
  showRulers: boolean;
  showStatusBar: boolean;
  showToolbar: boolean;
  showLayerPanel: boolean;
  showPropertyPanel: boolean;
  snapSettings: SnapSettings;
  gridSettings: GridSettings;
  rulerSettings: RulerSettings;
};

export type CADState = {
  activeTool: CADToolType;
  selectedObjects: string[];
  activeLayer: string;
  viewport: ViewportState;
  config: CADWorkspaceConfig;
  measurements: Measurement[];
  undoStack: any[];
  redoStack: any[];
};

export type KeyboardShortcut = {
  key: string;
  modifiers?: ('ctrl' | 'shift' | 'alt' | 'meta')[];
  description: string;
  action: () => void;
};
