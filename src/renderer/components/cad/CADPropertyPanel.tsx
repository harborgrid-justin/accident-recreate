/**
 * AccuScene Enterprise v0.3.0 - CAD Property Panel Component
 */

import React, { useState } from 'react';
import { PropertyField } from './PropertyField';
import { PropertySection, PropertyValue } from './types';

interface CADPropertyPanelProps {
  sections: PropertySection[];
  onPropertyChange: (id: string, value: PropertyValue) => void;
  position?: 'left' | 'right';
  collapsible?: boolean;
  className?: string;
}

export const CADPropertyPanel: React.FC<CADPropertyPanelProps> = ({
  sections,
  onPropertyChange,
  position = 'right',
  collapsible = true,
  className = '',
}) => {
  const [collapsed, setCollapsed] = useState(false);
  const [collapsedSections, setCollapsedSections] = useState<Set<string>>(
    new Set()
  );

  const toggleSection = (sectionId: string) => {
    setCollapsedSections((prev) => {
      const next = new Set(prev);
      if (next.has(sectionId)) {
        next.delete(sectionId);
      } else {
        next.add(sectionId);
      }
      return next;
    });
  };

  const positionClasses = {
    left: 'left-0',
    right: 'right-0',
  };

  return (
    <div
      className={`
        fixed ${positionClasses[position]} top-0 h-full
        ${collapsed ? 'w-12' : 'w-80'}
        bg-white dark:bg-gray-900
        border-${position === 'left' ? 'r' : 'l'}
        border-gray-200 dark:border-gray-700
        shadow-lg
        transition-all duration-300
        z-40
        flex flex-col
        ${className}
      `}
    >
      {/* Header */}
      <div
        className="
          flex items-center justify-between
          p-3 border-b border-gray-200 dark:border-gray-700
        "
      >
        {!collapsed && (
          <h2 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
            Properties
          </h2>
        )}
        {collapsible && (
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="
              p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-800
              transition-colors
            "
            aria-label={collapsed ? 'Expand panel' : 'Collapse panel'}
          >
            <span className="text-gray-600 dark:text-gray-400">
              {collapsed ? (position === 'left' ? '▶' : '◀') : position === 'left' ? '◀' : '▶'}
            </span>
          </button>
        )}
      </div>

      {/* Content */}
      {!collapsed && (
        <div className="flex-1 overflow-y-auto p-3 space-y-3">
          {sections.length === 0 ? (
            <div className="text-center py-8">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                No properties available
              </p>
              <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
                Select an object to view its properties
              </p>
            </div>
          ) : (
            sections.map((section) => {
              const isSectionCollapsed = collapsedSections.has(section.id);

              return (
                <div
                  key={section.id}
                  className="
                    bg-gray-50 dark:bg-gray-800
                    rounded-lg overflow-hidden
                  "
                >
                  {/* Section Header */}
                  <button
                    onClick={() => toggleSection(section.id)}
                    className="
                      w-full px-3 py-2 flex items-center justify-between
                      text-xs font-semibold text-gray-700 dark:text-gray-300
                      hover:bg-gray-100 dark:hover:bg-gray-700
                      transition-colors
                    "
                  >
                    <span>{section.title}</span>
                    <span className="text-xs">
                      {isSectionCollapsed ? '▶' : '▼'}
                    </span>
                  </button>

                  {/* Section Properties */}
                  {!isSectionCollapsed && (
                    <div className="p-3 space-y-3">
                      {section.properties.map((property) => (
                        <PropertyField
                          key={property.id}
                          property={property}
                          onChange={onPropertyChange}
                        />
                      ))}
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>
      )}

      {/* Footer Info */}
      {!collapsed && sections.length > 0 && (
        <div
          className="
            p-2 border-t border-gray-200 dark:border-gray-700
            text-xs text-gray-500 dark:text-gray-400
            text-center
          "
        >
          {sections.reduce((sum, s) => sum + s.properties.length, 0)} properties
        </div>
      )}
    </div>
  );
};
