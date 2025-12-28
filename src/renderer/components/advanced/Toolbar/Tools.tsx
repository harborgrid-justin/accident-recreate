/**
 * Tools Component
 * Tool selection with labels and shortcuts
 */

import React from 'react';
import { ToolsProps } from '../types';
import './Tools.css';

export const Tools: React.FC<ToolsProps> = ({
  tools,
  activeTool,
  onToolSelect,
  showLabels = true,
  showShortcuts = true,
}) => {
  return (
    <div className="tools-panel">
      <div className="tools-grid">
        {tools.map(tool => (
          <div
            key={tool.id}
            className={`tool-item ${activeTool === tool.id ? 'active' : ''}`}
            onClick={() => onToolSelect(tool.id)}
          >
            <div className="tool-icon-wrapper">
              <span className="tool-icon">{tool.icon}</span>
            </div>

            {showLabels && (
              <div className="tool-info">
                <div className="tool-name">{tool.name}</div>
                {tool.description && (
                  <div className="tool-description">{tool.description}</div>
                )}
                {showShortcuts && tool.shortcut && (
                  <div className="tool-shortcut">{tool.shortcut}</div>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default Tools;
