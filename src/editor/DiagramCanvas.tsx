/**
 * Diagram Canvas Component
 * AccuScene Enterprise Accident Recreation Platform
 */

import React, { useRef, useEffect, useState, useCallback } from 'react';
import Konva from 'konva';
import { Stage, Layer, Rect, Circle, Line, Text, Group, Arrow } from 'react-konva';
import {
  DiagramElement,
  DiagramState,
  Point,
  ToolType,
  ViewportState,
  Measurement,
} from '../types/diagram';
import { DiagramStateManager } from './DiagramState';
import { getElementBounds, isPointInElement } from './DiagramElements';

export interface DiagramCanvasProps {
  stateManager: DiagramStateManager;
  activeTool: ToolType;
  toolOptions?: any;
  width: number;
  height: number;
  onElementSelected?: (id: string) => void;
  onElementsSelected?: (ids: string[]) => void;
}

export const DiagramCanvas: React.FC<DiagramCanvasProps> = ({
  stateManager,
  activeTool,
  toolOptions,
  width,
  height,
  onElementSelected,
  onElementsSelected,
}) => {
  const [state, setState] = useState<DiagramState>(stateManager.getState());
  const [viewport, setViewport] = useState<ViewportState>({
    offset: { x: 0, y: 0 },
    zoom: 1,
    minZoom: 0.1,
    maxZoom: 5,
  });

  const stageRef = useRef<Konva.Stage>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState<Point>({ x: 0, y: 0 });
  const [drawingPoints, setDrawingPoints] = useState<Point[]>([]);
  const [measurementPoints, setMeasurementPoints] = useState<Point[]>([]);
  const [selectionRect, setSelectionRect] = useState<{
    x: number;
    y: number;
    width: number;
    height: number;
  } | null>(null);

  // Subscribe to state changes
  useEffect(() => {
    const unsubscribe = stateManager.subscribe((newState) => {
      setState(newState);
    });
    return unsubscribe;
  }, [stateManager]);

  // Convert screen coordinates to world coordinates
  const screenToWorld = useCallback(
    (screenPoint: Point): Point => {
      return {
        x: (screenPoint.x - viewport.offset.x) / viewport.zoom,
        y: (screenPoint.y - viewport.offset.y) / viewport.zoom,
      };
    },
    [viewport]
  );

  // Convert world coordinates to screen coordinates
  const worldToScreen = useCallback(
    (worldPoint: Point): Point => {
      return {
        x: worldPoint.x * viewport.zoom + viewport.offset.x,
        y: worldPoint.y * viewport.zoom + viewport.offset.y,
      };
    },
    [viewport]
  );

  // Handle mouse down
  const handleMouseDown = (e: Konva.KonvaEventObject<MouseEvent>) => {
    const stage = e.target.getStage();
    if (!stage) return;

    const pointerPos = stage.getPointerPosition();
    if (!pointerPos) return;

    const worldPos = screenToWorld(pointerPos);

    setDragStart(worldPos);
    setIsDragging(true);

    // Handle different tools
    switch (activeTool) {
      case ToolType.SELECT:
        handleSelectMouseDown(worldPos, e.evt.shiftKey);
        break;
      case ToolType.PAN:
        // Pan tool - handled in mouse move
        break;
      case ToolType.DRAW_PATH:
      case ToolType.DRAW_SKID:
        setDrawingPoints([...drawingPoints, worldPos]);
        break;
      case ToolType.MEASURE_DISTANCE:
      case ToolType.MEASURE_ANGLE:
        setMeasurementPoints([...measurementPoints, worldPos]);
        break;
      case ToolType.ADD_VEHICLE:
      case ToolType.ADD_ROAD:
      case ToolType.ADD_ENVIRONMENTAL:
      case ToolType.ADD_MARKER:
        handleAddElement(worldPos);
        break;
    }
  };

  // Handle mouse move
  const handleMouseMove = (e: Konva.KonvaEventObject<MouseEvent>) => {
    if (!isDragging) return;

    const stage = e.target.getStage();
    if (!stage) return;

    const pointerPos = stage.getPointerPosition();
    if (!pointerPos) return;

    const worldPos = screenToWorld(pointerPos);

    switch (activeTool) {
      case ToolType.MOVE:
        if (state.selectedIds.length > 0) {
          const delta = {
            x: worldPos.x - dragStart.x,
            y: worldPos.y - dragStart.y,
          };
          stateManager.moveSelectedElements(delta, false);
          setDragStart(worldPos);
        }
        break;
      case ToolType.PAN:
        const screenDelta = {
          x: pointerPos.x - worldToScreen(dragStart).x,
          y: pointerPos.y - worldToScreen(dragStart).y,
        };
        setViewport((prev) => ({
          ...prev,
          offset: {
            x: prev.offset.x + screenDelta.x,
            y: prev.offset.y + screenDelta.y,
          },
        }));
        break;
      case ToolType.SELECT:
        // Update selection rectangle
        setSelectionRect({
          x: Math.min(dragStart.x, worldPos.x),
          y: Math.min(dragStart.y, worldPos.y),
          width: Math.abs(worldPos.x - dragStart.x),
          height: Math.abs(worldPos.y - dragStart.y),
        });
        break;
    }
  };

  // Handle mouse up
  const handleMouseUp = (e: Konva.KonvaEventObject<MouseEvent>) => {
    if (activeTool === ToolType.MOVE && state.selectedIds.length > 0) {
      stateManager.moveSelectedElements({ x: 0, y: 0 }, true);
    }

    if (activeTool === ToolType.SELECT && selectionRect) {
      handleRectangleSelection(selectionRect);
      setSelectionRect(null);
    }

    setIsDragging(false);
  };

  // Handle wheel for zoom
  const handleWheel = (e: Konva.KonvaEventObject<WheelEvent>) => {
    e.evt.preventDefault();

    const stage = e.target.getStage();
    if (!stage) return;

    const pointerPos = stage.getPointerPosition();
    if (!pointerPos) return;

    const mousePointTo = screenToWorld(pointerPos);

    const zoomDelta = e.evt.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(
      viewport.minZoom,
      Math.min(viewport.maxZoom, viewport.zoom * zoomDelta)
    );

    const newOffset = {
      x: pointerPos.x - mousePointTo.x * newZoom,
      y: pointerPos.y - mousePointTo.y * newZoom,
    };

    setViewport((prev) => ({
      ...prev,
      zoom: newZoom,
      offset: newOffset,
    }));
  };

  // Handle select mouse down
  const handleSelectMouseDown = (worldPos: Point, addToSelection: boolean) => {
    // Find element at position
    let foundElement: DiagramElement | null = null;

    // Check in reverse order (top elements first)
    const sortedElements = [...state.elements].sort((a, b) => b.zIndex - a.zIndex);

    for (const element of sortedElements) {
      if (isPointInElement(worldPos, element)) {
        foundElement = element;
        break;
      }
    }

    if (foundElement) {
      if (addToSelection) {
        if (state.selectedIds.includes(foundElement.id)) {
          stateManager.deselectElement(foundElement.id);
        } else {
          stateManager.selectElement(foundElement.id, true);
        }
      } else {
        stateManager.selectElement(foundElement.id, false);
      }
      onElementSelected?.(foundElement.id);
    } else if (!addToSelection) {
      stateManager.clearSelection();
    }
  };

  // Handle rectangle selection
  const handleRectangleSelection = (rect: {
    x: number;
    y: number;
    width: number;
    height: number;
  }) => {
    const selectedIds: string[] = [];

    state.elements.forEach((element) => {
      const bounds = getElementBounds(element);
      const centerX = bounds.x + bounds.width / 2;
      const centerY = bounds.y + bounds.height / 2;

      if (
        centerX >= rect.x &&
        centerX <= rect.x + rect.width &&
        centerY >= rect.y &&
        centerY <= rect.y + rect.height
      ) {
        selectedIds.push(element.id);
      }
    });

    if (selectedIds.length > 0) {
      stateManager.selectElements(selectedIds, false);
      onElementsSelected?.(selectedIds);
    }
  };

  // Handle add element
  const handleAddElement = (worldPos: Point) => {
    // Element will be added by the parent component using toolOptions
    // This is just a placeholder for the interaction
  };

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Delete selected elements
      if (e.key === 'Delete' || e.key === 'Backspace') {
        stateManager.removeSelectedElements();
        e.preventDefault();
      }

      // Undo/Redo
      if (e.ctrlKey || e.metaKey) {
        if (e.key === 'z' && !e.shiftKey) {
          stateManager.undo();
          e.preventDefault();
        } else if (e.key === 'y' || (e.key === 'z' && e.shiftKey)) {
          stateManager.redo();
          e.preventDefault();
        }
      }

      // Select all
      if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
        stateManager.selectAll();
        e.preventDefault();
      }

      // Escape to clear selection or cancel drawing
      if (e.key === 'Escape') {
        stateManager.clearSelection();
        setDrawingPoints([]);
        setMeasurementPoints([]);
        e.preventDefault();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [stateManager]);

  // Render grid
  const renderGrid = () => {
    if (!state.gridVisible) return null;

    const gridSize = state.gridSize * state.scale * viewport.zoom;
    const lines: JSX.Element[] = [];

    const startX = Math.floor(-viewport.offset.x / gridSize) * gridSize;
    const startY = Math.floor(-viewport.offset.y / gridSize) * gridSize;

    // Vertical lines
    for (let x = startX; x < width + gridSize; x += gridSize) {
      lines.push(
        <Line
          key={`v-${x}`}
          points={[x, 0, x, height]}
          stroke="#E5E7EB"
          strokeWidth={1}
          listening={false}
        />
      );
    }

    // Horizontal lines
    for (let y = startY; y < height + gridSize; y += gridSize) {
      lines.push(
        <Line
          key={`h-${y}`}
          points={[0, y, width, y]}
          stroke="#E5E7EB"
          strokeWidth={1}
          listening={false}
        />
      );
    }

    return lines;
  };

  // Render element
  const renderElement = (element: DiagramElement) => {
    if (!element.visible) return null;

    const { position, rotation, scale } = element.transform;
    const screenPos = worldToScreen(position);
    const w = (element.properties.width || 1) * state.scale * viewport.zoom * scale.x;
    const h = (element.properties.height || 1) * state.scale * viewport.zoom * scale.y;
    const color = element.properties.color || element.color || '#3B82F6';
    const isSelected = state.selectedIds.includes(element.id);

    const commonProps = {
      key: element.id,
      x: screenPos.x,
      y: screenPos.y,
      rotation: rotation,
      opacity: element.locked ? 0.6 : 1,
    };

    let shapeElement: JSX.Element;

    if (element.properties.shape === 'circle') {
      shapeElement = (
        <Circle
          {...commonProps}
          radius={w / 2}
          fill={color}
          stroke={isSelected ? '#F59E0B' : '#000000'}
          strokeWidth={isSelected ? 3 : 1}
        />
      );
    } else if (element.properties.shape === 'polygon' && element.properties.points) {
      const points = element.properties.points.flatMap((p: Point) => [
        p.x * w,
        p.y * h,
      ]);
      shapeElement = (
        <Line
          {...commonProps}
          points={points}
          closed
          fill={color}
          stroke={isSelected ? '#F59E0B' : '#000000'}
          strokeWidth={isSelected ? 3 : 1}
        />
      );
    } else if (element.type === 'path') {
      const points = element.properties.points.flatMap((p: Point) => {
        const sp = worldToScreen(p);
        return [sp.x, sp.y];
      });
      shapeElement = (
        <Arrow
          key={element.id}
          points={points}
          stroke={color}
          strokeWidth={element.properties.strokeWidth * viewport.zoom}
          fill={color}
          pointerLength={10 * viewport.zoom}
          pointerWidth={10 * viewport.zoom}
          dash={element.properties.dashed ? [10, 5] : undefined}
          listening={false}
        />
      );
    } else {
      // Rectangle
      shapeElement = (
        <Rect
          {...commonProps}
          offsetX={w / 2}
          offsetY={h / 2}
          width={w}
          height={h}
          fill={color}
          stroke={isSelected ? '#F59E0B' : '#000000'}
          strokeWidth={isSelected ? 3 : 1}
        />
      );
    }

    return (
      <Group key={element.id}>
        {shapeElement}
        {element.label && (
          <Text
            x={screenPos.x}
            y={screenPos.y + h / 2 + 10}
            text={element.label}
            fontSize={12}
            fill="#000000"
            align="center"
            offsetX={element.label.length * 3}
          />
        )}
      </Group>
    );
  };

  // Render measurement
  const renderMeasurement = (measurement: Measurement) => {
    const screenPoints = measurement.points.map(worldToScreen);
    const points = screenPoints.flatMap((p) => [p.x, p.y]);

    return (
      <Group key={measurement.id}>
        <Line
          points={points}
          stroke={measurement.color}
          strokeWidth={2}
          dash={[5, 5]}
          listening={false}
        />
        {screenPoints.map((p, i) => (
          <Circle key={i} x={p.x} y={p.y} radius={4} fill={measurement.color} />
        ))}
        <Text
          x={screenPoints[0].x}
          y={screenPoints[0].y - 15}
          text={measurement.label}
          fontSize={12}
          fill={measurement.color}
          fontStyle="bold"
        />
      </Group>
    );
  };

  // Sort elements by zIndex
  const sortedElements = [...state.elements].sort((a, b) => a.zIndex - b.zIndex);

  return (
    <div style={{ width, height, backgroundColor: state.backgroundColor }}>
      <Stage
        ref={stageRef}
        width={width}
        height={height}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onWheel={handleWheel}
      >
        <Layer>
          {/* Grid */}
          {renderGrid()}

          {/* Elements */}
          {sortedElements.map(renderElement)}

          {/* Measurements */}
          {state.measurements.map(renderMeasurement)}

          {/* Drawing preview */}
          {drawingPoints.length > 0 && (
            <Line
              points={drawingPoints.flatMap((p) => {
                const sp = worldToScreen(p);
                return [sp.x, sp.y];
              })}
              stroke="#3B82F6"
              strokeWidth={2}
              dash={[5, 5]}
              listening={false}
            />
          )}

          {/* Measurement preview */}
          {measurementPoints.length > 0 && (
            <Group>
              <Line
                points={measurementPoints.flatMap((p) => {
                  const sp = worldToScreen(p);
                  return [sp.x, sp.y];
                })}
                stroke="#10B981"
                strokeWidth={2}
                dash={[5, 5]}
                listening={false}
              />
              {measurementPoints.map((p, i) => {
                const sp = worldToScreen(p);
                return <Circle key={i} x={sp.x} y={sp.y} radius={4} fill="#10B981" />;
              })}
            </Group>
          )}

          {/* Selection rectangle */}
          {selectionRect && (
            <Rect
              x={worldToScreen({ x: selectionRect.x, y: selectionRect.y }).x}
              y={worldToScreen({ x: selectionRect.x, y: selectionRect.y }).y}
              width={selectionRect.width * viewport.zoom}
              height={selectionRect.height * viewport.zoom}
              fill="rgba(59, 130, 246, 0.1)"
              stroke="#3B82F6"
              strokeWidth={2}
              dash={[5, 5]}
              listening={false}
            />
          )}
        </Layer>
      </Stage>

      {/* Zoom indicator */}
      <div
        style={{
          position: 'absolute',
          bottom: 10,
          right: 10,
          backgroundColor: 'rgba(255, 255, 255, 0.9)',
          padding: '5px 10px',
          borderRadius: '5px',
          fontSize: '12px',
          fontWeight: 'bold',
        }}
      >
        Zoom: {(viewport.zoom * 100).toFixed(0)}%
      </div>
    </div>
  );
};
