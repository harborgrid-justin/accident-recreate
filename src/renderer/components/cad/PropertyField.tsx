/**
 * AccuScene Enterprise v0.3.0 - Property Field Component
 */

import React, { useState, useCallback } from 'react';
import { Property, PropertyValue } from './types';

interface PropertyFieldProps {
  property: Property;
  onChange: (id: string, value: PropertyValue) => void;
  className?: string;
}

export const PropertyField: React.FC<PropertyFieldProps> = ({
  property,
  onChange,
  className = '',
}) => {
  const [isFocused, setIsFocused] = useState(false);

  const handleChange = useCallback(
    (value: PropertyValue) => {
      if (!property.readonly) {
        onChange(property.id, value);
      }
    },
    [property.id, property.readonly, onChange]
  );

  const renderInput = () => {
    switch (property.type) {
      case 'text':
        return (
          <input
            type="text"
            value={property.value as string}
            onChange={(e) => handleChange(e.target.value)}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            disabled={property.readonly}
            className="
              w-full px-2 py-1 text-sm
              bg-white dark:bg-gray-800
              border border-gray-300 dark:border-gray-600
              rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent
              disabled:opacity-50 disabled:cursor-not-allowed
            "
          />
        );

      case 'number':
      case 'angle':
        return (
          <div className="flex items-center gap-1">
            <input
              type="number"
              value={property.value as number}
              onChange={(e) => handleChange(parseFloat(e.target.value) || 0)}
              onFocus={() => setIsFocused(true)}
              onBlur={() => setIsFocused(false)}
              min={property.min}
              max={property.max}
              step={property.step || 1}
              disabled={property.readonly}
              className="
                flex-1 px-2 py-1 text-sm
                bg-white dark:bg-gray-800
                border border-gray-300 dark:border-gray-600
                rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent
                disabled:opacity-50 disabled:cursor-not-allowed
              "
            />
            {property.unit && (
              <span className="text-xs text-gray-500 dark:text-gray-400">
                {property.unit}
              </span>
            )}
          </div>
        );

      case 'checkbox':
        return (
          <label className="flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={property.value as boolean}
              onChange={(e) => handleChange(e.target.checked)}
              disabled={property.readonly}
              className="
                w-4 h-4 text-blue-600
                border-gray-300 dark:border-gray-600
                rounded focus:ring-2 focus:ring-blue-500
                disabled:opacity-50 disabled:cursor-not-allowed
              "
            />
            <span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
              {property.label}
            </span>
          </label>
        );

      case 'select':
        return (
          <select
            value={property.value as string}
            onChange={(e) => handleChange(e.target.value)}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            disabled={property.readonly}
            className="
              w-full px-2 py-1 text-sm
              bg-white dark:bg-gray-800
              border border-gray-300 dark:border-gray-600
              rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent
              disabled:opacity-50 disabled:cursor-not-allowed
            "
          >
            {property.options?.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        );

      case 'color':
        return (
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={property.value as string}
              onChange={(e) => handleChange(e.target.value)}
              disabled={property.readonly}
              className="
                w-12 h-8 rounded cursor-pointer
                border border-gray-300 dark:border-gray-600
                disabled:opacity-50 disabled:cursor-not-allowed
              "
            />
            <input
              type="text"
              value={property.value as string}
              onChange={(e) => handleChange(e.target.value)}
              disabled={property.readonly}
              className="
                flex-1 px-2 py-1 text-sm font-mono
                bg-white dark:bg-gray-800
                border border-gray-300 dark:border-gray-600
                rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent
                disabled:opacity-50 disabled:cursor-not-allowed
              "
            />
          </div>
        );

      case 'slider':
        return (
          <div className="space-y-1">
            <input
              type="range"
              value={property.value as number}
              onChange={(e) => handleChange(parseFloat(e.target.value))}
              min={property.min || 0}
              max={property.max || 100}
              step={property.step || 1}
              disabled={property.readonly}
              className="
                w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg
                appearance-none cursor-pointer
                disabled:opacity-50 disabled:cursor-not-allowed
              "
            />
            <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400">
              <span>{property.min || 0}</span>
              <span className="font-medium text-gray-700 dark:text-gray-300">
                {property.value}
                {property.unit}
              </span>
              <span>{property.max || 100}</span>
            </div>
          </div>
        );

      case 'point':
        const pointValue = (property.value as string).split(',');
        return (
          <div className="flex items-center gap-2">
            <input
              type="number"
              value={pointValue[0] || '0'}
              onChange={(e) => {
                const newValue = [e.target.value, pointValue[1] || '0'].join(',');
                handleChange(newValue);
              }}
              disabled={property.readonly}
              placeholder="X"
              className="
                flex-1 px-2 py-1 text-sm
                bg-white dark:bg-gray-800
                border border-gray-300 dark:border-gray-600
                rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent
                disabled:opacity-50 disabled:cursor-not-allowed
              "
            />
            <span className="text-gray-400">×</span>
            <input
              type="number"
              value={pointValue[1] || '0'}
              onChange={(e) => {
                const newValue = [pointValue[0] || '0', e.target.value].join(',');
                handleChange(newValue);
              }}
              disabled={property.readonly}
              placeholder="Y"
              className="
                flex-1 px-2 py-1 text-sm
                bg-white dark:bg-gray-800
                border border-gray-300 dark:border-gray-600
                rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent
                disabled:opacity-50 disabled:cursor-not-allowed
              "
            />
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className={`space-y-1 ${className}`}>
      {property.type !== 'checkbox' && (
        <label
          className="
            block text-xs font-medium
            text-gray-700 dark:text-gray-300
          "
        >
          {property.label}
        </label>
      )}
      {renderInput()}
    </div>
  );
};
