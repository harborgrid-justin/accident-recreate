# AccuScene Diagram Editor

Complete 2D diagram editor for accident scene reconstruction with React and Konva.

## Features

### Core Capabilities
- **Interactive Canvas**: Zoom, pan, and grid overlay
- **Element Library**: Vehicles, road elements, environmental objects, markers
- **Drawing Tools**: Paths, skid marks, annotations
- **Measurements**: Distance and angle measurements with real-world units
- **State Management**: Full undo/redo support with history
- **Export**: PNG, JPG, SVG, and JSON formats
- **Vehicle Paths**: Draw and animate vehicle movement paths
- **Physics-Based Calculations**: Stopping distance, impact speed, critical speed

### Element Types

#### Vehicles
- Car, Truck, SUV, Van, Bus
- Motorcycle, Bicycle
- Pedestrian

#### Road Elements
- Lanes, Intersections
- Stop Signs, Traffic Lights
- Crosswalks, Parking Spaces
- Curbs

#### Environmental Objects
- Trees, Buildings
- Barriers, Fences
- Poles, Bollards

#### Markers
- Impact Points
- Debris Fields
- Skid Marks, Tire Marks
- Final Rest Positions
- Witness Positions

## Quick Start

```typescript
import {
  DiagramCanvas,
  DiagramToolbar,
  DiagramStateManager,
  ToolType,
} from './editor';

// Create state manager
const stateManager = new DiagramStateManager();

// In your React component
function AccidentDiagram() {
  const [activeTool, setActiveTool] = useState(ToolType.SELECT);

  return (
    <>
      <DiagramToolbar
        activeTool={activeTool}
        onToolChange={setActiveTool}
        onUndo={() => stateManager.undo()}
        onRedo={() => stateManager.redo()}
        canUndo={stateManager.canUndo()}
        canRedo={stateManager.canRedo()}
      />

      <DiagramCanvas
        stateManager={stateManager}
        activeTool={activeTool}
        width={1920}
        height={1080}
      />
    </>
  );
}
```

## Architecture

### Components

#### DiagramCanvas.tsx
Main canvas component using React-Konva for rendering. Handles:
- Mouse/touch interactions
- Viewport transformations (zoom/pan)
- Element rendering
- Selection handling
- Keyboard shortcuts

#### DiagramToolbar.tsx
Toolbar component with:
- Tool selection (select, move, rotate, pan)
- Element type menus (vehicles, roads, etc.)
- History controls (undo/redo)
- File operations (import/export)
- View controls (grid toggle)

### Core Modules

#### DiagramState.ts
State management with:
- Element CRUD operations
- Selection handling
- History with undo/redo (50 levels)
- State serialization/deserialization
- Change notifications (Observer pattern)

#### DiagramElements.ts
Element definitions and utilities:
- Pre-defined element types with real-world dimensions
- Element creation and manipulation
- Collision detection
- Geometric calculations

#### MeasurementTool.ts
Measurement and physics calculations:
- Distance measurements (meters, cm, km)
- Angle measurements (degrees)
- Area and perimeter calculations
- Speed calculations (m/s, km/h, mph)
- Stopping distance formulas
- Impact speed from skid marks
- Critical speed for curves

#### VehiclePath.ts
Vehicle path system:
- Path creation and editing
- Path smoothing (Catmull-Rom spline)
- Path simplification (Ramer-Douglas-Peucker)
- Animation frame generation
- Path length calculations
- Curvature analysis

#### DiagramExporter.ts
Export functionality:
- JSON export/import (diagram data)
- SVG export (vector graphics)
- PNG/JPG export (raster images)
- Report data generation
- Clipboard support

## Usage Examples

### Creating Elements Programmatically

```typescript
import { createElement, getElementDefinition } from './editor';
import { ElementType, VehicleType } from '../types/diagram';

// Create a car
const carDef = getElementDefinition(ElementType.VEHICLE, VehicleType.CAR);
const car = createElement(carDef, { x: 100, y: 100 });
car.properties.color = '#3B82F6';
car.label = 'Vehicle 1';
stateManager.addElement(car);

// Create a road
const roadDef = getElementDefinition(ElementType.ROAD, RoadElementType.LANE);
const road = createElement(roadDef, { x: 200, y: 100 });
road.transform.rotation = 90;
stateManager.addElement(road);
```

### Adding Measurements

```typescript
import { measurementTool } from './editor';

// Distance measurement
const distance = measurementTool.createDistanceMeasurement(
  { x: 0, y: 0 },
  { x: 10, y: 10 }
);
stateManager.addMeasurement(distance);

// Angle measurement
const angle = measurementTool.createAngleMeasurement(
  { x: 0, y: 0 },
  { x: 10, y: 0 },
  { x: 10, y: 10 }
);
stateManager.addMeasurement(angle);
```

### Creating Vehicle Paths

```typescript
import { vehiclePath } from './editor';

// Create a path
const path = vehiclePath.createPath([
  { x: 0, y: 0 },
  { x: 10, y: 5 },
  { x: 20, y: 10 },
], {
  color: '#3B82F6',
  arrowEnd: true,
});
stateManager.addElement(path);

// Create animation
const animation = vehiclePath.createPathAnimation(
  vehicleId,
  path,
  5.0 // 5 seconds duration
);
```

### Exporting Diagrams

```typescript
import { diagramExporter } from './editor';

// Export to JSON
const json = diagramExporter.exportToJSON(state);
diagramExporter.downloadFile(json, 'diagram.json');

// Export to SVG
const svg = diagramExporter.exportToSVG(state, {
  format: 'svg',
  includeBackground: true,
  includeGrid: true,
});
diagramExporter.downloadFile(svg, 'diagram.svg');

// Export to PNG
const blob = await diagramExporter.exportToPNG(canvasElement, {
  format: 'png',
  quality: 1.0,
});
diagramExporter.downloadFile(blob, 'diagram.png');
```

### Physics Calculations

```typescript
import { measurementTool } from './editor';

// Calculate stopping distance
const stoppingDistance = measurementTool.calculateStoppingDistance(
  20, // 20 m/s (72 km/h)
  0.7  // friction coefficient
);

// Calculate impact speed from skid marks
const speed = measurementTool.calculateImpactSpeed(
  25, // 25 meters of skid marks
  0.7  // friction coefficient
);
console.log(`Impact speed: ${speed.kilometersPerHour.toFixed(1)} km/h`);

// Calculate critical speed for curve
const criticalSpeed = measurementTool.calculateCriticalSpeed(
  50, // 50 meter radius
  0.7  // friction coefficient
);
```

## Keyboard Shortcuts

- **V** - Select tool
- **M** - Move tool
- **R** - Rotate tool
- **H** - Pan tool
- **P** - Draw path
- **S** - Draw skid marks
- **T** - Annotate
- **D** - Measure distance
- **A** - Measure angle
- **G** - Toggle grid
- **Delete/Backspace** - Delete selected
- **Ctrl+Z** - Undo
- **Ctrl+Y / Ctrl+Shift+Z** - Redo
- **Ctrl+A** - Select all
- **Esc** - Clear selection
- **Mouse Wheel** - Zoom

## Scale and Units

- Default scale: 20 pixels per meter
- Measurements in meters (with cm/km conversion)
- Angles in degrees
- Vehicle dimensions based on real-world averages
- Grid size: 1 meter (configurable)

## Performance Notes

- Efficient rendering with Konva's layer caching
- Viewport culling for large diagrams
- Optimized state updates with change notifications
- History limited to 50 entries (configurable)

## Browser Support

- Chrome/Edge (recommended)
- Firefox
- Safari
- Requires HTML5 Canvas support

## Dependencies

- React 18+
- Konva 9+
- React-Konva 18+
- TypeScript 5+

## License

Proprietary - AccuScene Enterprise
