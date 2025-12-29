/**
 * AccuScene Enterprise v0.3.0 - CAD Toolbar Button Component
 */

import React, { useState } from 'react';
import { CADTool } from './types';

interface CADToolbarButtonProps {
  tool: CADTool;
  isActive: boolean;
  onSelect: (tool: CADTool) => void;
  size?: 'small' | 'medium' | 'large';
  showLabel?: boolean;
  className?: string;
}

export const CADToolbarButton: React.FC<CADToolbarButtonProps> = ({
  tool,
  isActive,
  onSelect,
  size = 'medium',
  showLabel = true,
  className = '',
}) => {
  const [isHovered, setIsHovered] = useState(false);

  const sizeClasses = {
    small: 'w-8 h-8 text-xs',
    medium: 'w-10 h-10 text-sm',
    large: 'w-12 h-12 text-base',
  };

  const handleClick = () => {
    if (tool.enabled) {
      onSelect(tool);
    }
  };

  return (
    <div className="relative">
      <button
        onClick={handleClick}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
        disabled={!tool.enabled}
        aria-label={tool.name}
        aria-pressed={isActive}
        title={`${tool.name}${tool.shortcut ? ` (${tool.shortcut})` : ''}`}
        className={`
          ${sizeClasses[size]}
          flex items-center justify-center
          rounded-md
          transition-all duration-150
          ${
            isActive
              ? 'bg-blue-600 text-white shadow-md'
              : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
          }
          ${
            !tool.enabled
              ? 'opacity-40 cursor-not-allowed'
              : 'cursor-pointer hover:shadow-sm'
          }
          focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2
          ${className}
        `}
        style={{
          fontFamily: 'system-ui, -apple-system, sans-serif',
        }}
      >
        <span className="text-lg">{tool.icon}</span>
      </button>

      {showLabel && (
        <span
          className="
            mt-1 text-xs text-center text-gray-600 dark:text-gray-400
            block truncate max-w-full
          "
        >
          {tool.name}
        </span>
      )}

      {/* Tooltip on hover */}
      {isHovered && !showLabel && (
        <div
          className="
            absolute z-50 px-2 py-1 text-xs
            bg-gray-900 text-white rounded shadow-lg
            whitespace-nowrap
            bottom-full left-1/2 transform -translate-x-1/2 mb-2
            pointer-events-none
          "
          style={{
            animation: 'fadeIn 0.2s ease-in',
          }}
        >
          {tool.name}
          {tool.shortcut && (
            <span className="ml-2 text-gray-400">({tool.shortcut})</span>
          )}
          <div
            className="
              absolute top-full left-1/2 transform -translate-x-1/2
              border-4 border-transparent border-t-gray-900
            "
          />
        </div>
      )}

      <style jsx>{`
        @keyframes fadeIn {
          from {
            opacity: 0;
            transform: translateX(-50%) translateY(-4px);
          }
          to {
            opacity: 1;
            transform: translateX(-50%) translateY(0);
          }
        }
      `}</style>
    </div>
  );
};
