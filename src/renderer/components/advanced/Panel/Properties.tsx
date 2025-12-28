/**
 * Properties Panel Component
 * Property editor with different input types
 */

import React, { useState } from 'react';
import { PropertiesProps } from '../types';
import './Properties.css';

export const Properties: React.FC<PropertiesProps> = ({
  properties,
  onChange,
  title = 'Properties',
  collapsible = true,
  groupByCategory = true,
}) => {
  const [collapsed, setCollapsed] = useState(false);
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());

  const groupedProperties = groupByCategory
    ? properties.reduce((acc, prop) => {
        const category = prop.category || 'General';
        if (!acc[category]) {
          acc[category] = [];
        }
        acc[category].push(prop);
        return acc;
      }, {} as Record<string, typeof properties>)
    : { All: properties };

  const toggleCategory = (category: string) => {
    const newExpanded = new Set(expandedCategories);
    if (newExpanded.has(category)) {
      newExpanded.delete(category);
    } else {
      newExpanded.add(category);
    }
    setExpandedCategories(newExpanded);
  };

  const renderPropertyInput = (prop: typeof properties[0]) => {
    switch (prop.type) {
      case 'number':
        return (
          <input
            type="number"
            value={prop.value}
            min={prop.min}
            max={prop.max}
            step={prop.step || 1}
            onChange={(e) => onChange(prop.id, parseFloat(e.target.value))}
          />
        );

      case 'boolean':
        return (
          <input
            type="checkbox"
            checked={prop.value}
            onChange={(e) => onChange(prop.id, e.target.checked)}
          />
        );

      case 'color':
        return (
          <input
            type="color"
            value={prop.value}
            onChange={(e) => onChange(prop.id, e.target.value)}
          />
        );

      case 'select':
        return (
          <select
            value={prop.value}
            onChange={(e) => onChange(prop.id, e.target.value)}
          >
            {prop.options?.map(option => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        );

      case 'vector3':
        const [x, y, z] = prop.value || [0, 0, 0];
        return (
          <div className="vector3-input">
            <input
              type="number"
              value={x}
              step={prop.step || 0.1}
              onChange={(e) => onChange(prop.id, [parseFloat(e.target.value), y, z])}
              placeholder="X"
            />
            <input
              type="number"
              value={y}
              step={prop.step || 0.1}
              onChange={(e) => onChange(prop.id, [x, parseFloat(e.target.value), z])}
              placeholder="Y"
            />
            <input
              type="number"
              value={z}
              step={prop.step || 0.1}
              onChange={(e) => onChange(prop.id, [x, y, parseFloat(e.target.value)])}
              placeholder="Z"
            />
          </div>
        );

      case 'string':
      default:
        return (
          <input
            type="text"
            value={prop.value}
            onChange={(e) => onChange(prop.id, e.target.value)}
          />
        );
    }
  };

  return (
    <div className="properties-panel">
      <div className="panel-header" onClick={() => collapsible && setCollapsed(!collapsed)}>
        <h3>{title}</h3>
        {collapsible && (
          <span className="collapse-icon">{collapsed ? '+' : '−'}</span>
        )}
      </div>

      {!collapsed && (
        <div className="panel-content">
          {Object.entries(groupedProperties).map(([category, props]) => (
            <div key={category} className="property-category">
              {groupByCategory && (
                <div
                  className="category-header"
                  onClick={() => toggleCategory(category)}
                >
                  <span>{category}</span>
                  <span className="expand-icon">
                    {expandedCategories.has(category) ? '−' : '+'}
                  </span>
                </div>
              )}

              {(!groupByCategory || expandedCategories.has(category)) && (
                <div className="property-list">
                  {props.map(prop => (
                    <div key={prop.id} className="property-item">
                      <label className="property-label">{prop.name}</label>
                      <div className="property-input">
                        {renderPropertyInput(prop)}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default Properties;
