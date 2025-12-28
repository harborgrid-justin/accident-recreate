/**
 * Diagram Editor Tests
 * AccuScene Enterprise Accident Recreation Platform
 */

import { DiagramStateManager } from '../DiagramState';
import { createElement, getElementDefinition, distance, angle } from '../DiagramElements';
import { MeasurementTool } from '../MeasurementTool';
import { VehiclePath } from '../VehiclePath';
import { DiagramExporter } from '../DiagramExporter';
import {
  ElementType,
  VehicleType,
  RoadElementType,
  MarkerType,
  ToolType,
} from '../../types/diagram';

describe('DiagramStateManager', () => {
  let stateManager: DiagramStateManager;

  beforeEach(() => {
    stateManager = new DiagramStateManager();
  });

  test('should initialize with default state', () => {
    const state = stateManager.getState();
    expect(state.elements).toHaveLength(0);
    expect(state.measurements).toHaveLength(0);
    expect(state.selectedIds).toHaveLength(0);
    expect(state.scale).toBe(20);
  });

  test('should add element', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    expect(definition).toBeDefined();

    if (definition) {
      const element = createElement(definition, { x: 100, y: 100 });
      stateManager.addElement(element);

      const state = stateManager.getState();
      expect(state.elements).toHaveLength(1);
      expect(state.elements[0].id).toBe(element.id);
    }
  });

  test('should remove element', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    if (definition) {
      const element = createElement(definition, { x: 100, y: 100 });
      stateManager.addElement(element);
      stateManager.removeElement(element.id);

      const state = stateManager.getState();
      expect(state.elements).toHaveLength(0);
    }
  });

  test('should support undo/redo', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    if (definition) {
      const element = createElement(definition, { x: 100, y: 100 });
      stateManager.addElement(element);

      expect(stateManager.canUndo()).toBe(true);
      expect(stateManager.canRedo()).toBe(false);

      stateManager.undo();
      expect(stateManager.getState().elements).toHaveLength(0);
      expect(stateManager.canRedo()).toBe(true);

      stateManager.redo();
      expect(stateManager.getState().elements).toHaveLength(1);
    }
  });

  test('should select elements', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    if (definition) {
      const element1 = createElement(definition, { x: 100, y: 100 });
      const element2 = createElement(definition, { x: 200, y: 200 });

      stateManager.addElement(element1);
      stateManager.addElement(element2);

      stateManager.selectElement(element1.id);
      expect(stateManager.getState().selectedIds).toEqual([element1.id]);

      stateManager.selectElement(element2.id, true);
      expect(stateManager.getState().selectedIds).toHaveLength(2);

      stateManager.clearSelection();
      expect(stateManager.getState().selectedIds).toHaveLength(0);
    }
  });

  test('should serialize and deserialize', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    if (definition) {
      const element = createElement(definition, { x: 100, y: 100 });
      stateManager.addElement(element);

      const json = stateManager.serialize();
      expect(json).toContain(element.id);

      const newManager = new DiagramStateManager();
      const success = newManager.deserialize(json);
      expect(success).toBe(true);
      expect(newManager.getState().elements).toHaveLength(1);
    }
  });
});

describe('MeasurementTool', () => {
  let tool: MeasurementTool;

  beforeEach(() => {
    tool = new MeasurementTool(20); // 20 pixels per meter
  });

  test('should calculate distance', () => {
    const p1 = { x: 0, y: 0 };
    const p2 = { x: 20, y: 0 };
    const dist = tool.calculateDistance(p1, p2);
    expect(dist).toBe(1); // 1 meter
  });

  test('should calculate angle', () => {
    const p1 = { x: 0, y: 0 };
    const p2 = { x: 10, y: 0 };
    const p3 = { x: 10, y: 10 };
    const ang = tool.calculateAngle(p1, p2, p3);
    expect(ang).toBeCloseTo(90, 0);
  });

  test('should create distance measurement', () => {
    const p1 = { x: 0, y: 0 };
    const p2 = { x: 20, y: 0 };
    const measurement = tool.createDistanceMeasurement(p1, p2);

    expect(measurement.type).toBe('distance');
    expect(measurement.value).toBe(1);
    expect(measurement.unit).toBe('m');
  });

  test('should calculate stopping distance', () => {
    const speed = 20; // m/s (72 km/h)
    const friction = 0.7;
    const distance = tool.calculateStoppingDistance(speed, friction);
    expect(distance).toBeGreaterThan(0);
  });

  test('should calculate impact speed from skid marks', () => {
    const skidDistance = 25; // meters
    const friction = 0.7;
    const speed = tool.calculateImpactSpeed(skidDistance, friction);

    expect(speed.metersPerSecond).toBeGreaterThan(0);
    expect(speed.kilometersPerHour).toBeGreaterThan(0);
    expect(speed.milesPerHour).toBeGreaterThan(0);
  });
});

describe('VehiclePath', () => {
  let path: VehiclePath;

  beforeEach(() => {
    path = new VehiclePath(20);
  });

  test('should create path', () => {
    const points = [
      { x: 0, y: 0 },
      { x: 100, y: 100 },
    ];
    const pathElement = path.createPath(points);

    expect(pathElement.type).toBe(ElementType.PATH);
    expect(pathElement.properties.points).toEqual(points);
  });

  test('should calculate path length', () => {
    const points = [
      { x: 0, y: 0 },
      { x: 20, y: 0 },
      { x: 20, y: 20 },
    ];
    const length = path.calculatePathLength(points);
    expect(length).toBeCloseTo(2, 0); // 2 meters
  });

  test('should smooth path', () => {
    const points = [
      { x: 0, y: 0 },
      { x: 50, y: 50 },
      { x: 100, y: 0 },
    ];
    const smoothed = path.smoothPath(points);
    expect(smoothed.length).toBeGreaterThan(points.length);
  });

  test('should simplify path', () => {
    const points = [
      { x: 0, y: 0 },
      { x: 10, y: 1 },
      { x: 20, y: 0 },
      { x: 30, y: 1 },
      { x: 40, y: 0 },
    ];
    const simplified = path.simplifyPath(points, 0.5);
    expect(simplified.length).toBeLessThanOrEqual(points.length);
  });

  test('should create animation frames', () => {
    const points = [
      { x: 0, y: 0 },
      { x: 100, y: 100 },
    ];
    const pathElement = path.createPath(points);
    const animation = path.createPathAnimation('vehicle-1', pathElement, 5.0);

    expect(animation.frames.length).toBeGreaterThan(0);
    expect(animation.duration).toBe(5000);
  });
});

describe('DiagramExporter', () => {
  let exporter: DiagramExporter;
  let stateManager: DiagramStateManager;

  beforeEach(() => {
    exporter = new DiagramExporter();
    stateManager = new DiagramStateManager();
  });

  test('should export to JSON', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    if (definition) {
      const element = createElement(definition, { x: 100, y: 100 });
      stateManager.addElement(element);

      const json = exporter.exportToJSON(stateManager.getState());
      expect(json).toContain(element.id);

      const parsed = JSON.parse(json);
      expect(parsed.diagram).toBeDefined();
      expect(parsed.version).toBeDefined();
    }
  });

  test('should import from JSON', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    if (definition) {
      const element = createElement(definition, { x: 100, y: 100 });
      stateManager.addElement(element);

      const json = exporter.exportToJSON(stateManager.getState());
      const imported = exporter.importFromJSON(json);

      expect(imported).toBeDefined();
      expect(imported?.elements).toHaveLength(1);
    }
  });

  test('should export to SVG', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    if (definition) {
      const element = createElement(definition, { x: 100, y: 100 });
      stateManager.addElement(element);

      const svg = exporter.exportToSVG(stateManager.getState());
      expect(svg).toContain('<svg');
      expect(svg).toContain('</svg>');
    }
  });

  test('should generate report data', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    if (definition) {
      const element = createElement(definition, { x: 100, y: 100 });
      stateManager.addElement(element);

      const reportData = exporter.generateReportData(stateManager.getState());
      expect(reportData.summary.totalElements).toBe(1);
      expect(reportData.summary.vehicles).toBe(1);
      expect(reportData.metadata).toBeDefined();
    }
  });
});

describe('Element Definitions', () => {
  test('should get vehicle definition', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    expect(definition).toBeDefined();
    expect(definition?.type).toBe(ElementType.VEHICLE);
    expect(definition?.subType).toBe(VehicleType.CAR);
  });

  test('should get road definition', () => {
    const definition = getElementDefinition(ElementType.ROAD, RoadElementType.LANE);
    expect(definition).toBeDefined();
    expect(definition?.type).toBe(ElementType.ROAD);
  });

  test('should create element from definition', () => {
    const definition = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
    if (definition) {
      const element = createElement(definition, { x: 100, y: 100 });
      expect(element.id).toBeDefined();
      expect(element.transform.position).toEqual({ x: 100, y: 100 });
      expect(element.type).toBe(ElementType.VEHICLE);
    }
  });

  test('should calculate distance between points', () => {
    const p1 = { x: 0, y: 0 };
    const p2 = { x: 3, y: 4 };
    const dist = distance(p1, p2);
    expect(dist).toBe(5);
  });

  test('should calculate angle between points', () => {
    const p1 = { x: 0, y: 0 };
    const p2 = { x: 1, y: 0 };
    const ang = angle(p1, p2);
    expect(ang).toBe(0);
  });
});
