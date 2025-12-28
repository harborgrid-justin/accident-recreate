/**
 * Modes Component
 * Edit mode selection
 */

import React from 'react';
import { ModesProps } from '../types';
import './Modes.css';

export const Modes: React.FC<ModesProps> = ({
  modes,
  activeMode,
  onModeSelect,
}) => {
  return (
    <div className="edit-modes">
      {modes.map(mode => (
        <button
          key={mode.id}
          className={`mode-button ${activeMode === mode.id ? 'active' : ''}`}
          onClick={() => onModeSelect(mode.id)}
          title={mode.description}
        >
          <span className="mode-icon">{mode.icon}</span>
          <span className="mode-label">{mode.name}</span>
        </button>
      ))}
    </div>
  );
};

export default Modes;
