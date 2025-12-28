/**
 * Analysis Component
 * Visual analysis overlays for simulation
 */

import React, { useState } from 'react';
import { AnalysisProps } from '../types';
import './Analysis.css';

export const Analysis: React.FC<AnalysisProps> = ({
  showForces = false,
  showVelocities = false,
  showAccelerations = false,
  showCollisionPoints = false,
  showTrajectories = false,
  colorScheme = 'default',
}) => {
  const [expanded, setExpanded] = useState(true);

  return (
    <div className={`simulation-analysis ${expanded ? 'expanded' : 'collapsed'}`}>
      <div className="analysis-header" onClick={() => setExpanded(!expanded)}>
        <h4>Analysis Options</h4>
        <span className="toggle-icon">{expanded ? '−' : '+'}</span>
      </div>

      {expanded && (
        <div className="analysis-content">
          <div className="analysis-section">
            <h5>Visualizations</h5>
            <label className="analysis-option">
              <input type="checkbox" checked={showForces} readOnly />
              <span>Force Vectors</span>
            </label>
            <label className="analysis-option">
              <input type="checkbox" checked={showVelocities} readOnly />
              <span>Velocity Vectors</span>
            </label>
            <label className="analysis-option">
              <input type="checkbox" checked={showAccelerations} readOnly />
              <span>Acceleration Vectors</span>
            </label>
            <label className="analysis-option">
              <input type="checkbox" checked={showCollisionPoints} readOnly />
              <span>Collision Points</span>
            </label>
            <label className="analysis-option">
              <input type="checkbox" checked={showTrajectories} readOnly />
              <span>Vehicle Trajectories</span>
            </label>
          </div>

          <div className="analysis-section">
            <h5>Color Scheme</h5>
            <div className="color-scheme-options">
              <button
                className={`scheme-button ${colorScheme === 'default' ? 'active' : ''}`}
              >
                Default
              </button>
              <button
                className={`scheme-button ${colorScheme === 'heatmap' ? 'active' : ''}`}
              >
                Heatmap
              </button>
              <button
                className={`scheme-button ${colorScheme === 'physics' ? 'active' : ''}`}
              >
                Physics
              </button>
            </div>
          </div>

          <div className="analysis-legend">
            <h5>Legend</h5>
            <div className="legend-item">
              <span className="legend-color" style={{ background: '#ff0000' }} />
              <span>High Impact</span>
            </div>
            <div className="legend-item">
              <span className="legend-color" style={{ background: '#ffaa00' }} />
              <span>Medium Impact</span>
            </div>
            <div className="legend-item">
              <span className="legend-color" style={{ background: '#00ff00' }} />
              <span>Low Impact</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Analysis;
