/**
 * Diagram Toolbar Component
 * AccuScene Enterprise Accident Recreation Platform
 */

import React, { useState } from 'react';
import { ToolType, VehicleType, RoadElementType, EnvironmentalType, MarkerType } from '../types/diagram';

export interface ToolbarProps {
  activeTool: ToolType;
  onToolChange: (tool: ToolType, options?: any) => void;
  onUndo: () => void;
  onRedo: () => void;
  canUndo: boolean;
  canRedo: boolean;
  onExport: (format: 'png' | 'jpg' | 'svg' | 'json') => void;
  onImport: () => void;
  onClear: () => void;
  onToggleGrid: () => void;
  gridVisible: boolean;
}

export const DiagramToolbar: React.FC<ToolbarProps> = ({
  activeTool,
  onToolChange,
  onUndo,
  onRedo,
  canUndo,
  canRedo,
  onExport,
  onImport,
  onClear,
  onToggleGrid,
  gridVisible,
}) => {
  const [showVehicleMenu, setShowVehicleMenu] = useState(false);
  const [showRoadMenu, setShowRoadMenu] = useState(false);
  const [showEnvironmentalMenu, setShowEnvironmentalMenu] = useState(false);
  const [showMarkerMenu, setShowMarkerMenu] = useState(false);
  const [showExportMenu, setShowExportMenu] = useState(false);

  const toolGroups = [
    {
      name: 'Selection',
      tools: [
        { type: ToolType.SELECT, name: 'Select', icon: '⬚', shortcut: 'V' },
        { type: ToolType.MOVE, name: 'Move', icon: '✋', shortcut: 'M' },
        { type: ToolType.ROTATE, name: 'Rotate', icon: '↻', shortcut: 'R' },
        { type: ToolType.PAN, name: 'Pan', icon: '👆', shortcut: 'H' },
      ],
    },
    {
      name: 'Drawing',
      tools: [
        { type: ToolType.DRAW_PATH, name: 'Draw Path', icon: '〰️', shortcut: 'P' },
        { type: ToolType.DRAW_SKID, name: 'Skid Marks', icon: '💨', shortcut: 'S' },
        { type: ToolType.ANNOTATE, name: 'Annotate', icon: '📝', shortcut: 'T' },
      ],
    },
    {
      name: 'Measurement',
      tools: [
        { type: ToolType.MEASURE_DISTANCE, name: 'Distance', icon: '📏', shortcut: 'D' },
        { type: ToolType.MEASURE_ANGLE, name: 'Angle', icon: '📐', shortcut: 'A' },
      ],
    },
  ];

  const handleToolClick = (tool: ToolType) => {
    onToolChange(tool);
  };

  const handleVehicleSelect = (vehicleType: VehicleType) => {
    onToolChange(ToolType.ADD_VEHICLE, { vehicleType });
    setShowVehicleMenu(false);
  };

  const handleRoadSelect = (roadType: RoadElementType) => {
    onToolChange(ToolType.ADD_ROAD, { roadType });
    setShowRoadMenu(false);
  };

  const handleEnvironmentalSelect = (envType: EnvironmentalType) => {
    onToolChange(ToolType.ADD_ENVIRONMENTAL, { envType });
    setShowEnvironmentalMenu(false);
  };

  const handleMarkerSelect = (markerType: MarkerType) => {
    onToolChange(ToolType.ADD_MARKER, { markerType });
    setShowMarkerMenu(false);
  };

  return (
    <div className="diagram-toolbar" style={styles.toolbar}>
      {/* Main Tools */}
      <div style={styles.toolSection}>
        {toolGroups.map((group) => (
          <div key={group.name} style={styles.toolGroup}>
            <div style={styles.groupLabel}>{group.name}</div>
            <div style={styles.toolButtons}>
              {group.tools.map((tool) => (
                <button
                  key={tool.type}
                  style={{
                    ...styles.toolButton,
                    ...(activeTool === tool.type ? styles.activeButton : {}),
                  }}
                  onClick={() => handleToolClick(tool.type)}
                  title={`${tool.name} (${tool.shortcut})`}
                >
                  <span style={styles.toolIcon}>{tool.icon}</span>
                  <span style={styles.toolName}>{tool.name}</span>
                </button>
              ))}
            </div>
          </div>
        ))}
      </div>

      {/* Element Tools */}
      <div style={styles.toolSection}>
        <div style={styles.groupLabel}>Add Elements</div>
        <div style={styles.toolButtons}>
          {/* Vehicle Menu */}
          <div style={styles.menuContainer}>
            <button
              style={{
                ...styles.toolButton,
                ...(activeTool === ToolType.ADD_VEHICLE ? styles.activeButton : {}),
              }}
              onClick={() => setShowVehicleMenu(!showVehicleMenu)}
              title="Add Vehicle"
            >
              <span style={styles.toolIcon}>🚗</span>
              <span style={styles.toolName}>Vehicle</span>
            </button>
            {showVehicleMenu && (
              <div style={styles.dropdown}>
                {Object.values(VehicleType).map((type) => (
                  <button
                    key={type}
                    style={styles.dropdownItem}
                    onClick={() => handleVehicleSelect(type)}
                  >
                    {type.charAt(0).toUpperCase() + type.slice(1)}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Road Menu */}
          <div style={styles.menuContainer}>
            <button
              style={{
                ...styles.toolButton,
                ...(activeTool === ToolType.ADD_ROAD ? styles.activeButton : {}),
              }}
              onClick={() => setShowRoadMenu(!showRoadMenu)}
              title="Add Road Element"
            >
              <span style={styles.toolIcon}>🛣️</span>
              <span style={styles.toolName}>Road</span>
            </button>
            {showRoadMenu && (
              <div style={styles.dropdown}>
                {Object.values(RoadElementType).map((type) => (
                  <button
                    key={type}
                    style={styles.dropdownItem}
                    onClick={() => handleRoadSelect(type)}
                  >
                    {type.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Environmental Menu */}
          <div style={styles.menuContainer}>
            <button
              style={{
                ...styles.toolButton,
                ...(activeTool === ToolType.ADD_ENVIRONMENTAL ? styles.activeButton : {}),
              }}
              onClick={() => setShowEnvironmentalMenu(!showEnvironmentalMenu)}
              title="Add Environmental Element"
            >
              <span style={styles.toolIcon}>🌳</span>
              <span style={styles.toolName}>Environment</span>
            </button>
            {showEnvironmentalMenu && (
              <div style={styles.dropdown}>
                {Object.values(EnvironmentalType).map((type) => (
                  <button
                    key={type}
                    style={styles.dropdownItem}
                    onClick={() => handleEnvironmentalSelect(type)}
                  >
                    {type.charAt(0).toUpperCase() + type.slice(1)}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Marker Menu */}
          <div style={styles.menuContainer}>
            <button
              style={{
                ...styles.toolButton,
                ...(activeTool === ToolType.ADD_MARKER ? styles.activeButton : {}),
              }}
              onClick={() => setShowMarkerMenu(!showMarkerMenu)}
              title="Add Marker"
            >
              <span style={styles.toolIcon}>💥</span>
              <span style={styles.toolName}>Marker</span>
            </button>
            {showMarkerMenu && (
              <div style={styles.dropdown}>
                {Object.values(MarkerType).map((type) => (
                  <button
                    key={type}
                    style={styles.dropdownItem}
                    onClick={() => handleMarkerSelect(type)}
                  >
                    {type.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* History Controls */}
      <div style={styles.toolSection}>
        <div style={styles.groupLabel}>History</div>
        <div style={styles.toolButtons}>
          <button
            style={{
              ...styles.toolButton,
              ...(canUndo ? {} : styles.disabledButton),
            }}
            onClick={onUndo}
            disabled={!canUndo}
            title="Undo (Ctrl+Z)"
          >
            <span style={styles.toolIcon}>↶</span>
            <span style={styles.toolName}>Undo</span>
          </button>
          <button
            style={{
              ...styles.toolButton,
              ...(canRedo ? {} : styles.disabledButton),
            }}
            onClick={onRedo}
            disabled={!canRedo}
            title="Redo (Ctrl+Y)"
          >
            <span style={styles.toolIcon}>↷</span>
            <span style={styles.toolName}>Redo</span>
          </button>
        </div>
      </div>

      {/* View Controls */}
      <div style={styles.toolSection}>
        <div style={styles.groupLabel}>View</div>
        <div style={styles.toolButtons}>
          <button
            style={{
              ...styles.toolButton,
              ...(gridVisible ? styles.activeButton : {}),
            }}
            onClick={onToggleGrid}
            title="Toggle Grid (G)"
          >
            <span style={styles.toolIcon}>⊞</span>
            <span style={styles.toolName}>Grid</span>
          </button>
        </div>
      </div>

      {/* File Operations */}
      <div style={styles.toolSection}>
        <div style={styles.groupLabel}>File</div>
        <div style={styles.toolButtons}>
          <div style={styles.menuContainer}>
            <button
              style={styles.toolButton}
              onClick={() => setShowExportMenu(!showExportMenu)}
              title="Export"
            >
              <span style={styles.toolIcon}>💾</span>
              <span style={styles.toolName}>Export</span>
            </button>
            {showExportMenu && (
              <div style={styles.dropdown}>
                <button style={styles.dropdownItem} onClick={() => onExport('png')}>
                  Export PNG
                </button>
                <button style={styles.dropdownItem} onClick={() => onExport('jpg')}>
                  Export JPG
                </button>
                <button style={styles.dropdownItem} onClick={() => onExport('svg')}>
                  Export SVG
                </button>
                <button style={styles.dropdownItem} onClick={() => onExport('json')}>
                  Export JSON
                </button>
              </div>
            )}
          </div>
          <button
            style={styles.toolButton}
            onClick={onImport}
            title="Import"
          >
            <span style={styles.toolIcon}>📂</span>
            <span style={styles.toolName}>Import</span>
          </button>
          <button
            style={styles.toolButton}
            onClick={onClear}
            title="Clear Diagram"
          >
            <span style={styles.toolIcon}>🗑️</span>
            <span style={styles.toolName}>Clear</span>
          </button>
        </div>
      </div>
    </div>
  );
};

const styles: { [key: string]: React.CSSProperties } = {
  toolbar: {
    display: 'flex',
    flexDirection: 'row',
    gap: '20px',
    padding: '10px',
    backgroundColor: '#FFFFFF',
    borderBottom: '2px solid #E5E7EB',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    overflowX: 'auto',
    flexWrap: 'wrap',
  },
  toolSection: {
    display: 'flex',
    flexDirection: 'column',
    gap: '5px',
  },
  groupLabel: {
    fontSize: '11px',
    fontWeight: 'bold',
    color: '#6B7280',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
    marginBottom: '2px',
  },
  toolGroup: {
    display: 'flex',
    flexDirection: 'column',
    gap: '5px',
  },
  toolButtons: {
    display: 'flex',
    flexDirection: 'row',
    gap: '5px',
  },
  menuContainer: {
    position: 'relative',
  },
  toolButton: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '8px 12px',
    border: '1px solid #D1D5DB',
    borderRadius: '6px',
    backgroundColor: '#FFFFFF',
    cursor: 'pointer',
    transition: 'all 0.2s',
    minWidth: '70px',
    fontSize: '12px',
  },
  activeButton: {
    backgroundColor: '#3B82F6',
    color: '#FFFFFF',
    borderColor: '#2563EB',
  },
  disabledButton: {
    opacity: 0.5,
    cursor: 'not-allowed',
  },
  toolIcon: {
    fontSize: '20px',
    marginBottom: '4px',
  },
  toolName: {
    fontSize: '11px',
    whiteSpace: 'nowrap',
  },
  dropdown: {
    position: 'absolute',
    top: '100%',
    left: 0,
    marginTop: '5px',
    backgroundColor: '#FFFFFF',
    border: '1px solid #D1D5DB',
    borderRadius: '6px',
    boxShadow: '0 4px 6px rgba(0,0,0,0.1)',
    zIndex: 1000,
    minWidth: '150px',
  },
  dropdownItem: {
    display: 'block',
    width: '100%',
    padding: '10px 15px',
    border: 'none',
    backgroundColor: 'transparent',
    cursor: 'pointer',
    textAlign: 'left',
    fontSize: '13px',
    transition: 'background-color 0.2s',
  },
};
