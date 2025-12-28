/**
 * Toolbar Component
 * Main toolbar with tools and modes
 */

import React from 'react';
import { ToolbarProps } from '../types';
import './Toolbar.css';

export const Toolbar: React.FC<ToolbarProps> = ({
  tools,
  activeTool,
  onToolSelect,
  orientation = 'horizontal',
  compact = false,
}) => {
  // Group tools by category
  const groupedTools = tools.reduce((acc, tool) => {
    const category = tool.category || 'General';
    if (!acc[category]) {
      acc[category] = [];
    }
    acc[category].push(tool);
    return acc;
  }, {} as Record<string, typeof tools>);

  return (
    <div className={`toolbar toolbar-${orientation} ${compact ? 'compact' : ''}`}>
      {Object.entries(groupedTools).map(([category, categoryTools]) => (
        <div key={category} className="toolbar-group">
          {!compact && <div className="toolbar-group-label">{category}</div>}
          <div className="toolbar-tools">
            {categoryTools.map(tool => (
              <button
                key={tool.id}
                className={`toolbar-tool ${activeTool === tool.id ? 'active' : ''}`}
                onClick={() => onToolSelect(tool.id)}
                title={`${tool.name}${tool.shortcut ? ` (${tool.shortcut})` : ''}`}
              >
                <span className="tool-icon">{tool.icon}</span>
                {!compact && <span className="tool-label">{tool.name}</span>}
              </button>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
};

export default Toolbar;
