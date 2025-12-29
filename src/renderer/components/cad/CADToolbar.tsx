/**
 * AccuScene Enterprise v0.3.0 - CAD Toolbar Component
 */

import React, { useState } from 'react';
import { CADToolbarButton } from './CADToolbarButton';
import { CADTool, ToolCategory } from './types';
import { useCADTool } from './hooks/useCADTool';

interface CADToolbarProps {
  position?: 'left' | 'right' | 'top';
  collapsible?: boolean;
  className?: string;
  onToolChange?: (tool: CADTool) => void;
}

export const CADToolbar: React.FC<CADToolbarProps> = ({
  position = 'left',
  collapsible = true,
  className = '',
  onToolChange,
}) => {
  const { activeTool, setActiveTool, tools, getToolsByCategory, isToolActive } =
    useCADTool();
  const [collapsed, setCollapsed] = useState(false);
  const [expandedCategories, setExpandedCategories] = useState<Set<ToolCategory>>(
    new Set([
      ToolCategory.SELECTION,
      ToolCategory.NAVIGATION,
      ToolCategory.DRAW,
    ])
  );

  const handleToolSelect = (tool: CADTool) => {
    setActiveTool(tool.type);
    onToolChange?.(tool);
  };

  const toggleCategory = (category: ToolCategory) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(category)) {
        next.delete(category);
      } else {
        next.add(category);
      }
      return next;
    });
  };

  const positionClasses = {
    left: 'left-0 top-0 h-full flex-col border-r',
    right: 'right-0 top-0 h-full flex-col border-l',
    top: 'top-0 left-0 w-full flex-row border-b',
  };

  const categories = [
    ToolCategory.SELECTION,
    ToolCategory.NAVIGATION,
    ToolCategory.DRAW,
    ToolCategory.MODIFY,
    ToolCategory.MEASURE,
    ToolCategory.ANNOTATION,
  ];

  return (
    <div
      className={`
        fixed ${positionClasses[position]}
        bg-white dark:bg-gray-900
        border-gray-200 dark:border-gray-700
        shadow-lg
        flex
        ${collapsed ? (position === 'top' ? 'h-12' : 'w-12') : position === 'top' ? 'h-auto' : 'w-64'}
        transition-all duration-300
        z-40
        ${className}
      `}
    >
      {/* Header */}
      <div
        className={`
          flex items-center justify-between
          p-2 border-b border-gray-200 dark:border-gray-700
          ${position === 'top' ? 'border-b-0' : ''}
        `}
      >
        {!collapsed && (
          <h2 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
            Tools
          </h2>
        )}
        {collapsible && (
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="
              p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-800
              transition-colors
            "
            aria-label={collapsed ? 'Expand toolbar' : 'Collapse toolbar'}
          >
            <span className="text-gray-600 dark:text-gray-400">
              {collapsed ? '▶' : '◀'}
            </span>
          </button>
        )}
      </div>

      {/* Tool Groups */}
      {!collapsed && (
        <div
          className={`
            flex-1 overflow-y-auto overflow-x-hidden
            ${position === 'top' ? 'flex flex-row gap-4 p-2' : 'p-2 space-y-2'}
          `}
        >
          {categories.map((category) => {
            const categoryTools = getToolsByCategory(category);
            if (categoryTools.length === 0) return null;

            const isExpanded = expandedCategories.has(category);

            return (
              <div
                key={category}
                className="
                  bg-gray-50 dark:bg-gray-800
                  rounded-lg overflow-hidden
                "
              >
                {/* Category Header */}
                <button
                  onClick={() => toggleCategory(category)}
                  className="
                    w-full px-3 py-2 flex items-center justify-between
                    text-xs font-medium text-gray-600 dark:text-gray-400
                    hover:bg-gray-100 dark:hover:bg-gray-700
                    transition-colors
                  "
                >
                  <span>{category}</span>
                  <span className="text-xs">{isExpanded ? '▼' : '▶'}</span>
                </button>

                {/* Tool Buttons */}
                {isExpanded && (
                  <div
                    className={`
                      grid gap-2 p-2
                      ${position === 'top' ? 'grid-cols-4' : 'grid-cols-2'}
                    `}
                  >
                    {categoryTools.map((tool) => (
                      <CADToolbarButton
                        key={tool.id}
                        tool={tool}
                        isActive={isToolActive(tool.type)}
                        onSelect={handleToolSelect}
                        showLabel={false}
                      />
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* Quick Tools (always visible when collapsed) */}
      {collapsed && (
        <div className="p-2 space-y-2">
          {getToolsByCategory(ToolCategory.SELECTION)
            .slice(0, 1)
            .map((tool) => (
              <CADToolbarButton
                key={tool.id}
                tool={tool}
                isActive={isToolActive(tool.type)}
                onSelect={handleToolSelect}
                showLabel={false}
                size="small"
              />
            ))}
          {getToolsByCategory(ToolCategory.NAVIGATION)
            .slice(0, 2)
            .map((tool) => (
              <CADToolbarButton
                key={tool.id}
                tool={tool}
                isActive={isToolActive(tool.type)}
                onSelect={handleToolSelect}
                showLabel={false}
                size="small"
              />
            ))}
        </div>
      )}
    </div>
  );
};
