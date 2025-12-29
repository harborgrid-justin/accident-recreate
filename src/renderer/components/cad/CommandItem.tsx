/**
 * AccuScene Enterprise v0.3.0 - Command Item Component
 */

import React from 'react';
import { Command } from './types';

interface CommandItemProps {
  command: Command;
  isSelected: boolean;
  onClick: () => void;
  searchQuery?: string;
  className?: string;
}

export const CommandItem: React.FC<CommandItemProps> = ({
  command,
  isSelected,
  onClick,
  searchQuery = '',
  className = '',
}) => {
  // Highlight matching text
  const highlightText = (text: string, query: string): JSX.Element => {
    if (!query.trim()) {
      return <>{text}</>;
    }

    const parts = text.split(new RegExp(`(${query})`, 'gi'));
    return (
      <>
        {parts.map((part, index) =>
          part.toLowerCase() === query.toLowerCase() ? (
            <mark
              key={index}
              className="bg-yellow-200 dark:bg-yellow-700 text-gray-900 dark:text-gray-100"
            >
              {part}
            </mark>
          ) : (
            <span key={index}>{part}</span>
          )
        )}
      </>
    );
  };

  return (
    <div
      onClick={onClick}
      className={`
        flex items-center gap-3 px-3 py-2
        cursor-pointer rounded
        transition-colors duration-150
        ${
          isSelected
            ? 'bg-blue-100 dark:bg-blue-900 text-blue-900 dark:text-blue-100'
            : 'bg-transparent hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300'
        }
        ${className}
      `}
      role="option"
      aria-selected={isSelected}
    >
      {/* Icon */}
      {command.icon && (
        <div
          className="
            w-8 h-8 flex items-center justify-center
            text-lg
          "
        >
          {command.icon}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 min-w-0">
        {/* Name */}
        <div className="text-sm font-medium truncate">
          {highlightText(command.name, searchQuery)}
        </div>

        {/* Description */}
        {command.description && (
          <div className="text-xs text-gray-500 dark:text-gray-400 truncate">
            {command.description}
          </div>
        )}

        {/* Category & Tags */}
        <div className="flex items-center gap-2 mt-1">
          <span
            className="
              text-xs px-1.5 py-0.5 rounded
              bg-gray-200 dark:bg-gray-700
              text-gray-600 dark:text-gray-400
            "
          >
            {command.category}
          </span>
          {command.tags.slice(0, 2).map((tag) => (
            <span
              key={tag}
              className="
                text-xs px-1.5 py-0.5 rounded
                bg-blue-100 dark:bg-blue-900
                text-blue-600 dark:text-blue-300
              "
            >
              {tag}
            </span>
          ))}
        </div>
      </div>

      {/* Keyboard Shortcut */}
      {command.shortcut && (
        <div
          className="
            flex items-center gap-1
            text-xs font-mono
            text-gray-500 dark:text-gray-400
          "
        >
          {command.shortcut.split('+').map((key, index) => (
            <React.Fragment key={index}>
              {index > 0 && <span>+</span>}
              <kbd
                className="
                  px-1.5 py-0.5 rounded
                  bg-gray-200 dark:bg-gray-700
                  border border-gray-300 dark:border-gray-600
                "
              >
                {key}
              </kbd>
            </React.Fragment>
          ))}
        </div>
      )}
    </div>
  );
};
