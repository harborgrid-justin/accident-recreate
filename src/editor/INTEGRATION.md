# Diagram Editor - Integration Guide

## Overview

The Diagram Editor module provides a complete 2D canvas-based accident scene reconstruction system with React and Konva. This guide explains how to integrate the editor with the rest of the AccuScene Enterprise platform.

## Module Structure

```
src/editor/
├── index.ts                      # Main exports
├── DiagramCanvas.tsx             # Canvas component (583 lines)
├── DiagramToolbar.tsx            # Toolbar component (431 lines)
├── DiagramState.ts               # State manager (468 lines)
├── DiagramElements.ts            # Element definitions (446 lines)
├── DiagramExporter.ts            # Export utilities (379 lines)
├── VehiclePath.ts                # Path system (449 lines)
├── MeasurementTool.ts            # Measurements (341 lines)
├── DiagramEditorExample.tsx      # Usage example (308 lines)
├── __tests__/
│   └── DiagramEditor.test.ts     # Unit tests (312 lines)
└── README.md                     # Documentation

src/types/
└── diagram.ts                    # Type definitions (202 lines)
```

**Total**: 3,673 lines of TypeScript/React code

## Key Features

### 1. Canvas System
- **React-Konva Integration**: High-performance 2D rendering
- **Viewport Controls**: Zoom (0.1x - 5x), pan, grid overlay
- **Real-time Rendering**: 60 FPS animation support
- **Responsive**: Adapts to window size changes

### 2. Element Library
- **8 Vehicle Types**: Car, Truck, SUV, Van, Bus, Motorcycle, Bicycle, Pedestrian
- **7 Road Elements**: Lanes, Intersections, Signs, Lights, Crosswalks, Parking, Curbs
- **6 Environmental Objects**: Trees, Buildings, Barriers, Fences, Poles, Bollards
- **6 Marker Types**: Impact Points, Debris, Skid Marks, Tire Marks, Rest Positions, Witness Positions

### 3. Measurement Tools
- **Distance Measurements**: Accurate to centimeters
- **Angle Measurements**: 0.1° precision
- **Physics Calculations**:
  - Stopping distance
  - Impact speed from skid marks
  - Critical speed for curves
- **Unit Conversions**: m/s, km/h, mph

### 4. State Management
- **Undo/Redo**: 50-level history
- **Selection**: Single and multi-select
- **Real-time Updates**: Observer pattern
- **Serialization**: JSON import/export

### 5. Export Capabilities
- **PNG/JPG**: Raster image export
- **SVG**: Vector graphics export
- **JSON**: Complete state serialization
- **Report Data**: Structured data for reports

## Integration Points

### 1. Database Integration

The diagram editor needs to persist diagram data to the database through the Case entity.

```typescript
// In src/cases/CaseService.ts
import { diagramStateManager } from '../editor';

export class CaseService {
  async saveDiagram(caseId: string, diagramState: DiagramState): Promise<void> {
    const json = JSON.stringify(diagramState);

    await this.caseRepository.update(caseId, {
      diagramData: json,
      diagramLastModified: new Date(),
    });

    this.eventBus.publish(EventType.DIAGRAM_SAVED, {
      caseId,
      timestamp: new Date(),
    });
  }

  async loadDiagram(caseId: string): Promise<DiagramState | null> {
    const case_ = await this.caseRepository.findOne({ where: { id: caseId } });

    if (case_?.diagramData) {
      return JSON.parse(case_.diagramData);
    }

    return null;
  }
}
```

### 2. API Integration

Create REST endpoints for diagram operations.

```typescript
// In src/api/routes/diagrams.ts
import express from 'express';
import { DiagramExporter } from '../../editor';

const router = express.Router();
const exporter = new DiagramExporter();

// Save diagram
router.post('/cases/:caseId/diagram', async (req, res) => {
  const { caseId } = req.params;
  const diagramState = req.body;

  await caseService.saveDiagram(caseId, diagramState);
  res.json({ success: true });
});

// Load diagram
router.get('/cases/:caseId/diagram', async (req, res) => {
  const { caseId } = req.params;
  const diagram = await caseService.loadDiagram(caseId);
  res.json(diagram);
});

// Export diagram
router.post('/cases/:caseId/diagram/export', async (req, res) => {
  const { caseId } = req.params;
  const { format } = req.body; // png, jpg, svg, json

  const diagram = await caseService.loadDiagram(caseId);

  if (format === 'svg') {
    const svg = exporter.exportToSVG(diagram);
    res.type('image/svg+xml').send(svg);
  } else if (format === 'json') {
    const json = exporter.exportToJSON(diagram);
    res.type('application/json').send(json);
  }
  // PNG/JPG requires canvas on server side
});

export default router;
```

### 3. Event Bus Integration

Publish events for real-time collaboration and audit logging.

```typescript
// In DiagramCanvas.tsx or parent component
import { eventBus } from '../utils/eventBus';
import { EventType } from '../types/events';

// When element is added
eventBus.publish(EventType.DIAGRAM_ELEMENT_ADDED, {
  caseId,
  element,
  userId,
  timestamp: new Date(),
});

// When element is updated
eventBus.publish(EventType.DIAGRAM_ELEMENT_UPDATED, {
  caseId,
  element,
  changes,
  userId,
  timestamp: new Date(),
});

// When element is deleted
eventBus.publish(EventType.DIAGRAM_ELEMENT_DELETED, {
  caseId,
  elementId,
  userId,
  timestamp: new Date(),
});

// When diagram is exported
eventBus.publish(EventType.DIAGRAM_EXPORTED, {
  caseId,
  format,
  userId,
  timestamp: new Date(),
});
```

### 4. Report Integration

Include diagrams in generated reports.

```typescript
// In src/reports/ReportGenerator.ts
import { diagramExporter } from '../editor';

export class ReportGenerator {
  async generateReport(caseId: string): Promise<Report> {
    const diagram = await caseService.loadDiagram(caseId);

    if (diagram) {
      // Export as SVG for inclusion in PDF
      const svg = diagramExporter.exportToSVG(diagram, {
        format: 'svg',
        includeBackground: true,
        includeGrid: false,
      });

      // Get report data
      const reportData = diagramExporter.generateReportData(diagram);

      // Include in report sections
      sections.push({
        title: 'Accident Diagram',
        type: 'diagram',
        content: svg,
        metadata: reportData,
      });
    }

    return { sections, ... };
  }
}
```

### 5. Physics Integration

Connect diagram measurements to physics simulations.

```typescript
// In src/physics/PhysicsEngine.ts
import { measurementTool } from '../editor';

export class PhysicsEngine {
  calculateImpactFromDiagram(diagram: DiagramState): SimulationResult {
    // Extract vehicle positions
    const vehicles = diagram.elements.filter(e => e.type === ElementType.VEHICLE);

    // Find skid marks
    const skidMarks = diagram.elements.filter(
      e => e.type === ElementType.MARKER && e.subType === MarkerType.SKID_MARK
    );

    // Calculate impact speeds from skid marks
    const speeds = skidMarks.map(mark => {
      const length = measurementTool.calculateDistance(
        mark.transform.position,
        { x: mark.properties.width, y: mark.properties.height }
      );

      return measurementTool.calculateImpactSpeed(length, 0.7);
    });

    // Run simulation
    return this.runSimulation({
      vehicles,
      speeds,
      environment: diagram.elements.filter(e => e.type === ElementType.ENVIRONMENTAL),
    });
  }
}
```

## Frontend Integration

### Basic Usage

```typescript
import React, { useState, useEffect } from 'react';
import {
  DiagramCanvas,
  DiagramToolbar,
  DiagramStateManager,
  ToolType,
} from '../editor';

export const CaseDiagramPage: React.FC<{ caseId: string }> = ({ caseId }) => {
  const [stateManager] = useState(() => new DiagramStateManager());
  const [activeTool, setActiveTool] = useState(ToolType.SELECT);
  const [loading, setLoading] = useState(true);

  // Load diagram from API
  useEffect(() => {
    fetch(`/api/cases/${caseId}/diagram`)
      .then(res => res.json())
      .then(data => {
        if (data) {
          stateManager.deserialize(JSON.stringify(data));
        }
        setLoading(false);
      });
  }, [caseId]);

  // Auto-save on changes
  useEffect(() => {
    const unsubscribe = stateManager.subscribe((state) => {
      // Debounce auto-save
      clearTimeout(saveTimeout);
      saveTimeout = setTimeout(() => {
        fetch(`/api/cases/${caseId}/diagram`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: stateManager.serialize(),
        });
      }, 2000);
    });

    return unsubscribe;
  }, [stateManager, caseId]);

  if (loading) return <div>Loading...</div>;

  return (
    <div className="diagram-editor">
      <DiagramToolbar
        activeTool={activeTool}
        onToolChange={setActiveTool}
        onUndo={() => stateManager.undo()}
        onRedo={() => stateManager.redo()}
        canUndo={stateManager.canUndo()}
        canRedo={stateManager.canRedo()}
        onExport={handleExport}
        onImport={handleImport}
        onClear={handleClear}
        onToggleGrid={() => stateManager.toggleGrid()}
        gridVisible={stateManager.getState().gridVisible}
      />

      <DiagramCanvas
        stateManager={stateManager}
        activeTool={activeTool}
        width={window.innerWidth}
        height={window.innerHeight - 200}
      />
    </div>
  );
};
```

## Testing

Run the included tests:

```bash
npm test src/editor/__tests__/DiagramEditor.test.ts
```

Tests cover:
- State management (add/remove/select/undo/redo)
- Measurements (distance, angle, physics)
- Path creation and animation
- Export/import functionality
- Element definitions

## Performance Considerations

1. **Large Diagrams**: For diagrams with 100+ elements, enable viewport culling
2. **Auto-Save**: Debounce state changes (2-5 seconds recommended)
3. **Export**: SVG is faster than PNG/JPG for simple diagrams
4. **History**: Limit to 50 entries (configurable in DiagramStateManager)

## Security Considerations

1. **Input Validation**: Validate JSON imports before deserializing
2. **File Size Limits**: Limit diagram size to prevent DoS
3. **User Permissions**: Check edit permissions before allowing changes
4. **Audit Logging**: Log all diagram modifications with user ID and timestamp

## Next Steps

1. **Real-time Collaboration**: Add WebSocket support for multi-user editing
2. **Templates**: Create common accident scene templates
3. **3D View**: Add optional 3D perspective rendering
4. **Mobile Support**: Optimize for tablet use
5. **Video Integration**: Overlay diagrams on dashcam footage

## Support

For questions or issues with the diagram editor:
- See README.md for detailed API documentation
- Check DiagramEditorExample.tsx for usage examples
- Review unit tests for implementation details

## License

Proprietary - AccuScene Enterprise
