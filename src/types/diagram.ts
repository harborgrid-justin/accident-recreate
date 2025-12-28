/**
 * Diagram Editor Type Definitions
 * AccuScene Enterprise Accident Recreation Platform
 */

export interface Point {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface Transform {
  position: Point;
  rotation: number;
  scale: Point;
}

export enum ElementType {
  VEHICLE = 'vehicle',
  ROAD = 'road',
  ENVIRONMENTAL = 'environmental',
  MARKER = 'marker',
  PATH = 'path',
  ANNOTATION = 'annotation',
}

export enum VehicleType {
  CAR = 'car',
  TRUCK = 'truck',
  MOTORCYCLE = 'motorcycle',
  BICYCLE = 'bicycle',
  PEDESTRIAN = 'pedestrian',
  SUV = 'suv',
  VAN = 'van',
  BUS = 'bus',
}

export enum RoadElementType {
  LANE = 'lane',
  INTERSECTION = 'intersection',
  STOP_SIGN = 'stop_sign',
  TRAFFIC_LIGHT = 'traffic_light',
  CROSSWALK = 'crosswalk',
  PARKING_SPACE = 'parking_space',
  CURB = 'curb',
}

export enum EnvironmentalType {
  TREE = 'tree',
  BUILDING = 'building',
  BARRIER = 'barrier',
  FENCE = 'fence',
  POLE = 'pole',
  BOLLARD = 'bollard',
}

export enum MarkerType {
  IMPACT_POINT = 'impact_point',
  DEBRIS = 'debris',
  SKID_MARK = 'skid_mark',
  TIRE_MARK = 'tire_mark',
  FINAL_REST = 'final_rest',
  WITNESS_POSITION = 'witness_position',
}

export interface DiagramElement {
  id: string;
  type: ElementType;
  subType: string;
  transform: Transform;
  properties: Record<string, any>;
  locked: boolean;
  visible: boolean;
  zIndex: number;
  color?: string;
  label?: string;
}

export interface VehicleElement extends DiagramElement {
  type: ElementType.VEHICLE;
  subType: VehicleType;
  properties: {
    make?: string;
    model?: string;
    color: string;
    length: number;
    width: number;
    heading: number;
  };
}

export interface PathElement extends DiagramElement {
  type: ElementType.PATH;
  properties: {
    points: Point[];
    color: string;
    strokeWidth: number;
    dashed: boolean;
    arrowStart: boolean;
    arrowEnd: boolean;
  };
}

export interface AnnotationElement extends DiagramElement {
  type: ElementType.ANNOTATION;
  properties: {
    text: string;
    fontSize: number;
    fontFamily: string;
    color: string;
    backgroundColor?: string;
    padding: number;
  };
}

export interface Measurement {
  id: string;
  type: 'distance' | 'angle';
  points: Point[];
  value: number;
  unit: string;
  label: string;
  color: string;
}

export interface DiagramState {
  elements: DiagramElement[];
  measurements: Measurement[];
  selectedIds: string[];
  scale: number; // pixels per meter
  gridVisible: boolean;
  gridSize: number;
  backgroundColor: string;
  canvasSize: Size;
}

export interface HistoryEntry {
  timestamp: number;
  state: DiagramState;
  description: string;
}

export enum ToolType {
  SELECT = 'select',
  MOVE = 'move',
  ROTATE = 'rotate',
  ADD_VEHICLE = 'add_vehicle',
  ADD_ROAD = 'add_road',
  ADD_ENVIRONMENTAL = 'add_environmental',
  ADD_MARKER = 'add_marker',
  DRAW_PATH = 'draw_path',
  DRAW_SKID = 'draw_skid',
  ANNOTATE = 'annotate',
  MEASURE_DISTANCE = 'measure_distance',
  MEASURE_ANGLE = 'measure_angle',
  PAN = 'pan',
  ZOOM = 'zoom',
}

export interface EditorTool {
  type: ToolType;
  name: string;
  icon: string;
  cursor: string;
  active: boolean;
  options?: Record<string, any>;
}

export interface ExportOptions {
  format: 'png' | 'jpg' | 'svg' | 'json';
  quality?: number;
  scale?: number;
  includeBackground?: boolean;
  includeGrid?: boolean;
  selectedOnly?: boolean;
}

export interface ViewportState {
  offset: Point;
  zoom: number;
  minZoom: number;
  maxZoom: number;
}

export interface AnimationFrame {
  elementId: string;
  transform: Transform;
  timestamp: number;
}

export interface PathAnimation {
  pathId: string;
  duration: number;
  frames: AnimationFrame[];
  currentFrame: number;
  playing: boolean;
  loop: boolean;
}
