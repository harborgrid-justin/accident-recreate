/**
 * EditorPage - Diagram editor for accident reconstruction
 */

import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Stage, Layer, Rect, Circle, Line, Text } from 'react-konva';
import { useEditorStore } from '../store/editorStore';
import { useUIStore } from '../store/uiStore';
import { api } from '../services/api';
import Button from '../components/Common/Button';

export const EditorPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const {
    elements,
    selectedElements,
    tool,
    zoom,
    gridEnabled,
    isDirty,
    addElement,
    selectElement,
    clearSelection,
    setTool,
    setZoom,
    toggleGrid,
    undo,
    redo,
    saveDiagram,
    loadDiagram,
    markClean,
  } = useEditorStore();
  const { showNotification, setLoading } = useUIStore();

  const [stageSize, setStageSize] = useState({ width: 800, height: 600 });

  useEffect(() => {
    if (id) {
      loadDiagramData(id);
    }

    const updateSize = () => {
      const container = document.getElementById('editor-canvas');
      if (container) {
        setStageSize({
          width: container.clientWidth,
          height: container.clientHeight,
        });
      }
    };

    updateSize();
    window.addEventListener('resize', updateSize);
    return () => window.removeEventListener('resize', updateSize);
  }, [id]);

  const loadDiagramData = async (caseId: string) => {
    setLoading(true);
    const response = await api.getAccident(caseId);
    setLoading(false);

    if (response.success && response.data?.diagram) {
      loadDiagram(response.data.diagram.elements || []);
    }
  };

  const handleSave = async () => {
    if (!id) return;

    setLoading(true);
    const elements = saveDiagram();
    const response = await api.updateAccident(id, {
      caseId: id,
      diagram: { elements },
      dateTime: new Date().toISOString(),
      location: 'Editor',
    });
    setLoading(false);

    if (response.success) {
      showNotification('success', 'Diagram saved successfully');
      markClean();
    } else {
      showNotification('error', response.error || 'Failed to save diagram');
    }
  };

  const handleStageClick = (e: any) => {
    if (e.target === e.target.getStage()) {
      clearSelection();
    }
  };

  const handleAddElement = (type: string) => {
    const newElement = {
      id: `element-${Date.now()}`,
      type: type as any,
      x: stageSize.width / 2,
      y: stageSize.height / 2,
      width: 100,
      height: 100,
      rotation: 0,
      color: '#3498db',
    };
    addElement(newElement);
  };

  return (
    <div className="page editor-page">
      <div className="editor-header">
        <div className="editor-header-left">
          <Button variant="outline" size="small" onClick={() => navigate(`/cases/${id}`)}>
            ← Back to Case
          </Button>
          <h1 className="editor-title">Accident Diagram Editor</h1>
          {isDirty && <span className="editor-unsaved">● Unsaved changes</span>}
        </div>
        <div className="editor-header-right">
          <Button variant="outline" size="small" onClick={undo}>
            ↶ Undo
          </Button>
          <Button variant="outline" size="small" onClick={redo}>
            ↷ Redo
          </Button>
          <Button variant="primary" size="small" onClick={handleSave} disabled={!isDirty}>
            Save Diagram
          </Button>
        </div>
      </div>

      <div className="editor-container">
        <div className="editor-toolbar">
          <div className="editor-toolbar-section">
            <h3>Tools</h3>
            <button
              className={`editor-tool-btn ${tool === 'select' ? 'active' : ''}`}
              onClick={() => setTool('select')}
            >
              Select
            </button>
            <button
              className={`editor-tool-btn ${tool === 'vehicle' ? 'active' : ''}`}
              onClick={() => {
                setTool('vehicle');
                handleAddElement('vehicle');
              }}
            >
              Vehicle
            </button>
            <button
              className={`editor-tool-btn ${tool === 'road' ? 'active' : ''}`}
              onClick={() => {
                setTool('road');
                handleAddElement('road');
              }}
            >
              Road
            </button>
            <button
              className={`editor-tool-btn ${tool === 'marker' ? 'active' : ''}`}
              onClick={() => {
                setTool('marker');
                handleAddElement('marker');
              }}
            >
              Marker
            </button>
            <button
              className={`editor-tool-btn ${tool === 'text' ? 'active' : ''}`}
              onClick={() => {
                setTool('text');
                handleAddElement('text');
              }}
            >
              Text
            </button>
            <button
              className={`editor-tool-btn ${tool === 'measurement' ? 'active' : ''}`}
              onClick={() => {
                setTool('measurement');
                handleAddElement('measurement');
              }}
            >
              Measure
            </button>
          </div>

          <div className="editor-toolbar-section">
            <h3>View</h3>
            <button className="editor-tool-btn" onClick={() => setZoom(zoom + 0.1)}>
              Zoom In
            </button>
            <button className="editor-tool-btn" onClick={() => setZoom(zoom - 0.1)}>
              Zoom Out
            </button>
            <button className="editor-tool-btn" onClick={() => setZoom(1)}>
              Reset Zoom
            </button>
            <button className="editor-tool-btn" onClick={toggleGrid}>
              {gridEnabled ? 'Hide Grid' : 'Show Grid'}
            </button>
          </div>

          <div className="editor-toolbar-section">
            <h3>Elements</h3>
            <div className="editor-elements-list">
              {elements.length === 0 ? (
                <p className="editor-no-elements">No elements added</p>
              ) : (
                elements.map((element) => (
                  <div
                    key={element.id}
                    className={`editor-element-item ${
                      selectedElements.includes(element.id) ? 'selected' : ''
                    }`}
                    onClick={() => selectElement(element.id)}
                  >
                    {element.type} - {element.id.substring(0, 8)}
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        <div className="editor-canvas" id="editor-canvas">
          <Stage
            width={stageSize.width}
            height={stageSize.height}
            scaleX={zoom}
            scaleY={zoom}
            onClick={handleStageClick}
          >
            <Layer>
              {gridEnabled && (
                <>
                  {Array.from({ length: Math.ceil(stageSize.width / 50) }).map((_, i) => (
                    <Line
                      key={`grid-v-${i}`}
                      points={[i * 50, 0, i * 50, stageSize.height]}
                      stroke="#e0e0e0"
                      strokeWidth={1}
                    />
                  ))}
                  {Array.from({ length: Math.ceil(stageSize.height / 50) }).map((_, i) => (
                    <Line
                      key={`grid-h-${i}`}
                      points={[0, i * 50, stageSize.width, i * 50]}
                      stroke="#e0e0e0"
                      strokeWidth={1}
                    />
                  ))}
                </>
              )}

              {elements.map((element) => {
                const isSelected = selectedElements.includes(element.id);

                switch (element.type) {
                  case 'vehicle':
                    return (
                      <Rect
                        key={element.id}
                        x={element.x}
                        y={element.y}
                        width={element.width || 100}
                        height={element.height || 50}
                        fill={element.color || '#3498db'}
                        stroke={isSelected ? '#e74c3c' : '#2c3e50'}
                        strokeWidth={isSelected ? 3 : 1}
                        rotation={element.rotation || 0}
                        draggable
                        onClick={() => selectElement(element.id)}
                      />
                    );

                  case 'marker':
                    return (
                      <Circle
                        key={element.id}
                        x={element.x}
                        y={element.y}
                        radius={20}
                        fill={element.color || '#e74c3c'}
                        stroke={isSelected ? '#c0392b' : '#2c3e50'}
                        strokeWidth={isSelected ? 3 : 1}
                        draggable
                        onClick={() => selectElement(element.id)}
                      />
                    );

                  case 'text':
                    return (
                      <Text
                        key={element.id}
                        x={element.x}
                        y={element.y}
                        text={element.label || 'Text'}
                        fontSize={16}
                        fill={element.color || '#2c3e50'}
                        draggable
                        onClick={() => selectElement(element.id)}
                      />
                    );

                  default:
                    return (
                      <Rect
                        key={element.id}
                        x={element.x}
                        y={element.y}
                        width={element.width || 100}
                        height={element.height || 100}
                        fill={element.color || '#95a5a6'}
                        stroke={isSelected ? '#e74c3c' : '#2c3e50'}
                        strokeWidth={isSelected ? 3 : 1}
                        draggable
                        onClick={() => selectElement(element.id)}
                      />
                    );
                }
              })}
            </Layer>
          </Stage>
        </div>

        <div className="editor-properties">
          <h3>Properties</h3>
          {selectedElements.length === 0 ? (
            <p className="editor-no-selection">No element selected</p>
          ) : (
            <div className="editor-properties-content">
              <p>Selected: {selectedElements.length} element(s)</p>
              <div className="editor-zoom-display">
                <label>Zoom: {Math.round(zoom * 100)}%</label>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default EditorPage;
