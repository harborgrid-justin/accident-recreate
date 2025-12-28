/**
 * Diagram Editor Example Usage
 * AccuScene Enterprise Accident Recreation Platform
 *
 * This file demonstrates how to use the complete diagram editor system
 */

import React, { useState, useRef, useCallback } from 'react';
import { DiagramCanvas } from './DiagramCanvas';
import { DiagramToolbar } from './DiagramToolbar';
import { DiagramStateManager } from './DiagramState';
import { createElement, getElementDefinition } from './DiagramElements';
import { measurementTool } from './MeasurementTool';
import { vehiclePath } from './VehiclePath';
import { diagramExporter } from './DiagramExporter';
import {
  ToolType,
  VehicleType,
  RoadElementType,
  EnvironmentalType,
  MarkerType,
  ElementType,
  Point,
} from '../types/diagram';

export const DiagramEditorExample: React.FC = () => {
  const [stateManager] = useState(() => new DiagramStateManager());
  const [activeTool, setActiveTool] = useState<ToolType>(ToolType.SELECT);
  const [toolOptions, setToolOptions] = useState<any>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Handle tool change
  const handleToolChange = useCallback((tool: ToolType, options?: any) => {
    setActiveTool(tool);
    setToolOptions(options);

    // If adding an element, create it at canvas center
    if (
      tool === ToolType.ADD_VEHICLE ||
      tool === ToolType.ADD_ROAD ||
      tool === ToolType.ADD_ENVIRONMENTAL ||
      tool === ToolType.ADD_MARKER
    ) {
      const state = stateManager.getState();
      const centerX = state.canvasSize.width / 2 / state.scale;
      const centerY = state.canvasSize.height / 2 / state.scale;

      let elementType: ElementType;
      let subType: string;

      if (tool === ToolType.ADD_VEHICLE && options?.vehicleType) {
        elementType = ElementType.VEHICLE;
        subType = options.vehicleType;
      } else if (tool === ToolType.ADD_ROAD && options?.roadType) {
        elementType = ElementType.ROAD;
        subType = options.roadType;
      } else if (tool === ToolType.ADD_ENVIRONMENTAL && options?.envType) {
        elementType = ElementType.ENVIRONMENTAL;
        subType = options.envType;
      } else if (tool === ToolType.ADD_MARKER && options?.markerType) {
        elementType = ElementType.MARKER;
        subType = options.markerType;
      } else {
        return;
      }

      const definition = getElementDefinition(elementType, subType);
      if (definition) {
        const element = createElement(definition, { x: centerX, y: centerY });
        stateManager.addElement(element);
        setActiveTool(ToolType.SELECT);
      }
    }
  }, [stateManager]);

  // Handle undo
  const handleUndo = useCallback(() => {
    stateManager.undo();
  }, [stateManager]);

  // Handle redo
  const handleRedo = useCallback(() => {
    stateManager.redo();
  }, [stateManager]);

  // Handle export
  const handleExport = useCallback(async (format: 'png' | 'jpg' | 'svg' | 'json') => {
    const state = stateManager.getState();

    if (format === 'json') {
      const json = diagramExporter.exportToJSON(state);
      diagramExporter.downloadFile(json, 'accident-diagram.json');
    } else if (format === 'svg') {
      const svg = diagramExporter.exportToSVG(state, {
        format: 'svg',
        includeBackground: true,
        includeGrid: true,
      });
      diagramExporter.downloadFile(svg, 'accident-diagram.svg');
    } else if (canvasRef.current) {
      await diagramExporter.exportAndDownload(
        state,
        canvasRef.current,
        { format, quality: format === 'png' ? 1.0 : 0.9 },
        'accident-diagram'
      );
    }
  }, [stateManager]);

  // Handle import
  const handleImport = useCallback(() => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = (event) => {
          const json = event.target?.result as string;
          stateManager.deserialize(json);
        };
        reader.readAsText(file);
      }
    };
    input.click();
  }, [stateManager]);

  // Handle clear
  const handleClear = useCallback(() => {
    if (confirm('Are you sure you want to clear the diagram? This cannot be undone.')) {
      stateManager.clear();
    }
  }, [stateManager]);

  // Handle toggle grid
  const handleToggleGrid = useCallback(() => {
    stateManager.toggleGrid();
  }, [stateManager]);

  const state = stateManager.getState();

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h1 style={styles.title}>AccuScene Diagram Editor</h1>
        <div style={styles.info}>
          <span>Elements: {state.elements.length}</span>
          <span>Selected: {state.selectedIds.length}</span>
          <span>Scale: 1:{state.scale}px/m</span>
        </div>
      </div>

      <DiagramToolbar
        activeTool={activeTool}
        onToolChange={handleToolChange}
        onUndo={handleUndo}
        onRedo={handleRedo}
        canUndo={stateManager.canUndo()}
        canRedo={stateManager.canRedo()}
        onExport={handleExport}
        onImport={handleImport}
        onClear={handleClear}
        onToggleGrid={handleToggleGrid}
        gridVisible={state.gridVisible}
      />

      <div style={styles.canvasContainer}>
        <DiagramCanvas
          stateManager={stateManager}
          activeTool={activeTool}
          toolOptions={toolOptions}
          width={state.canvasSize.width}
          height={state.canvasSize.height}
          onElementSelected={(id) => console.log('Selected:', id)}
          onElementsSelected={(ids) => console.log('Selected multiple:', ids)}
        />
      </div>

      <div style={styles.footer}>
        <div style={styles.shortcuts}>
          <strong>Shortcuts:</strong>
          <span>V - Select</span>
          <span>M - Move</span>
          <span>R - Rotate</span>
          <span>H - Pan</span>
          <span>P - Draw Path</span>
          <span>D - Measure Distance</span>
          <span>Delete - Remove Selected</span>
          <span>Ctrl+Z - Undo</span>
          <span>Ctrl+Y - Redo</span>
          <span>Ctrl+A - Select All</span>
          <span>Esc - Clear Selection</span>
          <span>Mouse Wheel - Zoom</span>
        </div>
      </div>
    </div>
  );
};

const styles: { [key: string]: React.CSSProperties } = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100vh',
    backgroundColor: '#F3F4F6',
    fontFamily: 'Arial, sans-serif',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '10px 20px',
    backgroundColor: '#1F2937',
    color: '#FFFFFF',
  },
  title: {
    margin: 0,
    fontSize: '24px',
    fontWeight: 'bold',
  },
  info: {
    display: 'flex',
    gap: '20px',
    fontSize: '14px',
  },
  canvasContainer: {
    flex: 1,
    overflow: 'hidden',
    position: 'relative',
  },
  footer: {
    padding: '10px 20px',
    backgroundColor: '#FFFFFF',
    borderTop: '1px solid #E5E7EB',
  },
  shortcuts: {
    display: 'flex',
    gap: '15px',
    fontSize: '12px',
    flexWrap: 'wrap',
    color: '#6B7280',
  },
};

/**
 * Example: Creating a simple accident scene programmatically
 */
export const createExampleScene = (stateManager: DiagramStateManager) => {
  // Add a road
  const roadDef = getElementDefinition(ElementType.ROAD, RoadElementType.LANE);
  if (roadDef) {
    const road = createElement(roadDef, { x: 100, y: 100 });
    road.transform.rotation = 90;
    stateManager.addElement(road);
  }

  // Add Vehicle 1 (Car)
  const car1Def = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
  if (car1Def) {
    const car1 = createElement(car1Def, { x: 80, y: 90 });
    car1.properties.color = '#3B82F6';
    car1.label = 'Vehicle 1';
    stateManager.addElement(car1);
  }

  // Add Vehicle 2 (SUV)
  const car2Def = getElementDefinition(ElementType.VEHICLE, VehicleType.SUV);
  if (car2Def) {
    const car2 = createElement(car2Def, { x: 120, y: 110 });
    car2.properties.color = '#EF4444';
    car2.label = 'Vehicle 2';
    car2.transform.rotation = 45;
    stateManager.addElement(car2);
  }

  // Add impact point
  const impactDef = getElementDefinition(ElementType.MARKER, MarkerType.IMPACT_POINT);
  if (impactDef) {
    const impact = createElement(impactDef, { x: 100, y: 100 });
    stateManager.addElement(impact);
  }

  // Add traffic light
  const lightDef = getElementDefinition(ElementType.ROAD, RoadElementType.TRAFFIC_LIGHT);
  if (lightDef) {
    const light = createElement(lightDef, { x: 85, y: 80 });
    stateManager.addElement(light);
  }

  // Add a tree
  const treeDef = getElementDefinition(ElementType.ENVIRONMENTAL, EnvironmentalType.TREE);
  if (treeDef) {
    const tree = createElement(treeDef, { x: 130, y: 130 });
    stateManager.addElement(tree);
  }

  // Add measurement
  const measurement = measurementTool.createDistanceMeasurement(
    { x: 80, y: 90 },
    { x: 120, y: 110 }
  );
  stateManager.addMeasurement(measurement);

  console.log('Example scene created!');
};

export default DiagramEditorExample;
