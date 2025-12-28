/**
 * Diagram Element Definitions
 * AccuScene Enterprise Accident Recreation Platform
 */

import {
  DiagramElement,
  ElementType,
  VehicleType,
  RoadElementType,
  EnvironmentalType,
  MarkerType,
  Point,
  Transform,
} from '../types/diagram';

export interface ElementDefinition {
  type: ElementType;
  subType: string;
  name: string;
  icon: string;
  defaultSize: { width: number; height: number };
  color: string;
  shape: 'rectangle' | 'circle' | 'polygon' | 'custom';
  points?: Point[]; // For polygons
  svgPath?: string; // For custom shapes
}

/**
 * Vehicle element definitions with real-world dimensions (in meters)
 */
export const VEHICLE_DEFINITIONS: Record<VehicleType, ElementDefinition> = {
  [VehicleType.CAR]: {
    type: ElementType.VEHICLE,
    subType: VehicleType.CAR,
    name: 'Passenger Car',
    icon: '🚗',
    defaultSize: { width: 1.8, height: 4.5 }, // meters
    color: '#3B82F6',
    shape: 'rectangle',
  },
  [VehicleType.TRUCK]: {
    type: ElementType.VEHICLE,
    subType: VehicleType.TRUCK,
    name: 'Truck',
    icon: '🚚',
    defaultSize: { width: 2.5, height: 6.5 },
    color: '#EF4444',
    shape: 'rectangle',
  },
  [VehicleType.MOTORCYCLE]: {
    type: ElementType.VEHICLE,
    subType: VehicleType.MOTORCYCLE,
    name: 'Motorcycle',
    icon: '🏍️',
    defaultSize: { width: 0.8, height: 2.2 },
    color: '#F59E0B',
    shape: 'rectangle',
  },
  [VehicleType.BICYCLE]: {
    type: ElementType.VEHICLE,
    subType: VehicleType.BICYCLE,
    name: 'Bicycle',
    icon: '🚲',
    defaultSize: { width: 0.6, height: 1.8 },
    color: '#10B981',
    shape: 'rectangle',
  },
  [VehicleType.PEDESTRIAN]: {
    type: ElementType.VEHICLE,
    subType: VehicleType.PEDESTRIAN,
    name: 'Pedestrian',
    icon: '🚶',
    defaultSize: { width: 0.5, height: 0.5 },
    color: '#8B5CF6',
    shape: 'circle',
  },
  [VehicleType.SUV]: {
    type: ElementType.VEHICLE,
    subType: VehicleType.SUV,
    name: 'SUV',
    icon: '🚙',
    defaultSize: { width: 2.0, height: 4.8 },
    color: '#06B6D4',
    shape: 'rectangle',
  },
  [VehicleType.VAN]: {
    type: ElementType.VEHICLE,
    subType: VehicleType.VAN,
    name: 'Van',
    icon: '🚐',
    defaultSize: { width: 2.0, height: 5.2 },
    color: '#84CC16',
    shape: 'rectangle',
  },
  [VehicleType.BUS]: {
    type: ElementType.VEHICLE,
    subType: VehicleType.BUS,
    name: 'Bus',
    icon: '🚌',
    defaultSize: { width: 2.5, height: 12.0 },
    color: '#F97316',
    shape: 'rectangle',
  },
};

/**
 * Road element definitions
 */
export const ROAD_DEFINITIONS: Record<RoadElementType, ElementDefinition> = {
  [RoadElementType.LANE]: {
    type: ElementType.ROAD,
    subType: RoadElementType.LANE,
    name: 'Lane',
    icon: '🛣️',
    defaultSize: { width: 3.6, height: 20.0 }, // Standard lane width
    color: '#6B7280',
    shape: 'rectangle',
  },
  [RoadElementType.INTERSECTION]: {
    type: ElementType.ROAD,
    subType: RoadElementType.INTERSECTION,
    name: 'Intersection',
    icon: '✖️',
    defaultSize: { width: 10.0, height: 10.0 },
    color: '#9CA3AF',
    shape: 'rectangle',
  },
  [RoadElementType.STOP_SIGN]: {
    type: ElementType.ROAD,
    subType: RoadElementType.STOP_SIGN,
    name: 'Stop Sign',
    icon: '🛑',
    defaultSize: { width: 0.9, height: 0.9 },
    color: '#DC2626',
    shape: 'polygon',
    points: [
      { x: 0.45, y: 0 },
      { x: 0.9, y: 0.18 },
      { x: 0.9, y: 0.72 },
      { x: 0.45, y: 0.9 },
      { x: 0, y: 0.72 },
      { x: 0, y: 0.18 },
    ],
  },
  [RoadElementType.TRAFFIC_LIGHT]: {
    type: ElementType.ROAD,
    subType: RoadElementType.TRAFFIC_LIGHT,
    name: 'Traffic Light',
    icon: '🚦',
    defaultSize: { width: 0.4, height: 1.2 },
    color: '#374151',
    shape: 'rectangle',
  },
  [RoadElementType.CROSSWALK]: {
    type: ElementType.ROAD,
    subType: RoadElementType.CROSSWALK,
    name: 'Crosswalk',
    icon: '🚸',
    defaultSize: { width: 4.0, height: 8.0 },
    color: '#FFFFFF',
    shape: 'rectangle',
  },
  [RoadElementType.PARKING_SPACE]: {
    type: ElementType.ROAD,
    subType: RoadElementType.PARKING_SPACE,
    name: 'Parking Space',
    icon: '🅿️',
    defaultSize: { width: 2.5, height: 5.0 },
    color: '#60A5FA',
    shape: 'rectangle',
  },
  [RoadElementType.CURB]: {
    type: ElementType.ROAD,
    subType: RoadElementType.CURB,
    name: 'Curb',
    icon: '📐',
    defaultSize: { width: 0.15, height: 10.0 },
    color: '#D1D5DB',
    shape: 'rectangle',
  },
};

/**
 * Environmental element definitions
 */
export const ENVIRONMENTAL_DEFINITIONS: Record<EnvironmentalType, ElementDefinition> = {
  [EnvironmentalType.TREE]: {
    type: ElementType.ENVIRONMENTAL,
    subType: EnvironmentalType.TREE,
    name: 'Tree',
    icon: '🌳',
    defaultSize: { width: 2.0, height: 2.0 },
    color: '#059669',
    shape: 'circle',
  },
  [EnvironmentalType.BUILDING]: {
    type: ElementType.ENVIRONMENTAL,
    subType: EnvironmentalType.BUILDING,
    name: 'Building',
    icon: '🏢',
    defaultSize: { width: 10.0, height: 15.0 },
    color: '#78716C',
    shape: 'rectangle',
  },
  [EnvironmentalType.BARRIER]: {
    type: ElementType.ENVIRONMENTAL,
    subType: EnvironmentalType.BARRIER,
    name: 'Barrier',
    icon: '🚧',
    defaultSize: { width: 0.4, height: 5.0 },
    color: '#F59E0B',
    shape: 'rectangle',
  },
  [EnvironmentalType.FENCE]: {
    type: ElementType.ENVIRONMENTAL,
    subType: EnvironmentalType.FENCE,
    name: 'Fence',
    icon: '🪧',
    defaultSize: { width: 0.1, height: 10.0 },
    color: '#92400E',
    shape: 'rectangle',
  },
  [EnvironmentalType.POLE]: {
    type: ElementType.ENVIRONMENTAL,
    subType: EnvironmentalType.POLE,
    name: 'Pole',
    icon: '🪔',
    defaultSize: { width: 0.3, height: 0.3 },
    color: '#52525B',
    shape: 'circle',
  },
  [EnvironmentalType.BOLLARD]: {
    type: ElementType.ENVIRONMENTAL,
    subType: EnvironmentalType.BOLLARD,
    name: 'Bollard',
    icon: '⚫',
    defaultSize: { width: 0.2, height: 0.2 },
    color: '#27272A',
    shape: 'circle',
  },
};

/**
 * Marker element definitions
 */
export const MARKER_DEFINITIONS: Record<MarkerType, ElementDefinition> = {
  [MarkerType.IMPACT_POINT]: {
    type: ElementType.MARKER,
    subType: MarkerType.IMPACT_POINT,
    name: 'Impact Point',
    icon: '💥',
    defaultSize: { width: 0.5, height: 0.5 },
    color: '#DC2626',
    shape: 'custom',
    svgPath: 'M 0.25 0 L 0.5 0.5 L 0.25 0.4 L 0 0.5 Z', // Star shape
  },
  [MarkerType.DEBRIS]: {
    type: ElementType.MARKER,
    subType: MarkerType.DEBRIS,
    name: 'Debris',
    icon: '🔸',
    defaultSize: { width: 0.3, height: 0.3 },
    color: '#F59E0B',
    shape: 'polygon',
    points: [
      { x: 0.15, y: 0 },
      { x: 0.3, y: 0.15 },
      { x: 0.15, y: 0.3 },
      { x: 0, y: 0.15 },
    ],
  },
  [MarkerType.SKID_MARK]: {
    type: ElementType.MARKER,
    subType: MarkerType.SKID_MARK,
    name: 'Skid Mark',
    icon: '〰️',
    defaultSize: { width: 0.3, height: 3.0 },
    color: '#1F2937',
    shape: 'rectangle',
  },
  [MarkerType.TIRE_MARK]: {
    type: ElementType.MARKER,
    subType: MarkerType.TIRE_MARK,
    name: 'Tire Mark',
    icon: '👣',
    defaultSize: { width: 0.2, height: 2.0 },
    color: '#374151',
    shape: 'rectangle',
  },
  [MarkerType.FINAL_REST]: {
    type: ElementType.MARKER,
    subType: MarkerType.FINAL_REST,
    name: 'Final Rest Position',
    icon: '📍',
    defaultSize: { width: 0.4, height: 0.4 },
    color: '#7C3AED',
    shape: 'circle',
  },
  [MarkerType.WITNESS_POSITION]: {
    type: ElementType.MARKER,
    subType: MarkerType.WITNESS_POSITION,
    name: 'Witness Position',
    icon: '👁️',
    defaultSize: { width: 0.4, height: 0.4 },
    color: '#0891B2',
    shape: 'circle',
  },
};

/**
 * Get all element definitions grouped by type
 */
export const getAllElementDefinitions = () => {
  return {
    vehicles: Object.values(VEHICLE_DEFINITIONS),
    roads: Object.values(ROAD_DEFINITIONS),
    environmental: Object.values(ENVIRONMENTAL_DEFINITIONS),
    markers: Object.values(MARKER_DEFINITIONS),
  };
};

/**
 * Get element definition by type and subtype
 */
export const getElementDefinition = (
  type: ElementType,
  subType: string
): ElementDefinition | undefined => {
  switch (type) {
    case ElementType.VEHICLE:
      return VEHICLE_DEFINITIONS[subType as VehicleType];
    case ElementType.ROAD:
      return ROAD_DEFINITIONS[subType as RoadElementType];
    case ElementType.ENVIRONMENTAL:
      return ENVIRONMENTAL_DEFINITIONS[subType as EnvironmentalType];
    case ElementType.MARKER:
      return MARKER_DEFINITIONS[subType as MarkerType];
    default:
      return undefined;
  }
};

/**
 * Create a new diagram element from definition
 */
export const createElement = (
  definition: ElementDefinition,
  position: Point = { x: 0, y: 0 }
): DiagramElement => {
  const id = `${definition.subType}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

  return {
    id,
    type: definition.type,
    subType: definition.subType,
    transform: {
      position,
      rotation: 0,
      scale: { x: 1, y: 1 },
    },
    properties: {
      color: definition.color,
      width: definition.defaultSize.width,
      height: definition.defaultSize.height,
      shape: definition.shape,
      points: definition.points,
      svgPath: definition.svgPath,
    },
    locked: false,
    visible: true,
    zIndex: 0,
    label: definition.name,
  };
};

/**
 * Get element bounds in world coordinates
 */
export const getElementBounds = (element: DiagramElement) => {
  const { position, scale } = element.transform;
  const width = (element.properties.width || 1) * scale.x;
  const height = (element.properties.height || 1) * scale.y;

  return {
    x: position.x - width / 2,
    y: position.y - height / 2,
    width,
    height,
  };
};

/**
 * Check if point is inside element
 */
export const isPointInElement = (point: Point, element: DiagramElement): boolean => {
  const bounds = getElementBounds(element);

  if (element.properties.shape === 'circle') {
    const radius = bounds.width / 2;
    const dx = point.x - element.transform.position.x;
    const dy = point.y - element.transform.position.y;
    return Math.sqrt(dx * dx + dy * dy) <= radius;
  }

  // Rectangle and other shapes - use bounding box
  return (
    point.x >= bounds.x &&
    point.x <= bounds.x + bounds.width &&
    point.y >= bounds.y &&
    point.y <= bounds.y + bounds.height
  );
};

/**
 * Rotate a point around a center point
 */
export const rotatePoint = (point: Point, center: Point, angle: number): Point => {
  const radians = (angle * Math.PI) / 180;
  const cos = Math.cos(radians);
  const sin = Math.sin(radians);

  const dx = point.x - center.x;
  const dy = point.y - center.y;

  return {
    x: center.x + dx * cos - dy * sin,
    y: center.y + dx * sin + dy * cos,
  };
};

/**
 * Calculate distance between two points
 */
export const distance = (p1: Point, p2: Point): number => {
  const dx = p2.x - p1.x;
  const dy = p2.y - p1.y;
  return Math.sqrt(dx * dx + dy * dy);
};

/**
 * Calculate angle between two points (in degrees)
 */
export const angle = (p1: Point, p2: Point): number => {
  return (Math.atan2(p2.y - p1.y, p2.x - p1.x) * 180) / Math.PI;
};
