/**
 * AccuScene Enterprise v0.3.0 - CAD Command Palette Component
 */

import React, { useEffect, useRef } from 'react';
import { CommandItem } from './CommandItem';
import { Command } from './types';
import { useCommandPalette } from './hooks/useCommandPalette';

interface CADCommandPaletteProps {
  commands: Command[];
  onCommandExecute?: (command: Command) => void;
  className?: string;
}

export const CADCommandPalette: React.FC<CADCommandPaletteProps> = ({
  commands,
  onCommandExecute,
  className = '',
}) => {
  const {
    isOpen,
    close,
    searchQuery,
    setSearchQuery,
    filteredCommands,
    selectedIndex,
    executeCommand,
    executeSelected,
    navigateUp,
    navigateDown,
  } = useCommandPalette(commands);

  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Focus input when opened
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current) {
      const selectedElement = listRef.current.children[selectedIndex] as HTMLElement;
      if (selectedElement) {
        selectedElement.scrollIntoView({
          block: 'nearest',
          behavior: 'smooth',
        });
      }
    }
  }, [selectedIndex]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        navigateDown();
        break;
      case 'ArrowUp':
        e.preventDefault();
        navigateUp();
        break;
      case 'Enter':
        e.preventDefault();
        executeSelected();
        onCommandExecute?.(filteredCommands[selectedIndex]);
        break;
      case 'Escape':
        e.preventDefault();
        close();
        break;
    }
  };

  const handleCommandClick = (command: Command) => {
    executeCommand(command);
    onCommandExecute?.(command);
  };

  if (!isOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        className="
          fixed inset-0 bg-black bg-opacity-50
          backdrop-blur-sm
          z-50
          transition-opacity duration-200
        "
        onClick={close}
      />

      {/* Command Palette */}
      <div
        className={`
          fixed top-20 left-1/2 transform -translate-x-1/2
          w-full max-w-2xl
          bg-white dark:bg-gray-900
          border border-gray-200 dark:border-gray-700
          rounded-lg shadow-2xl
          z-50
          overflow-hidden
          ${className}
        `}
        role="dialog"
        aria-label="Command palette"
      >
        {/* Search Input */}
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-3">
            <span className="text-xl text-gray-400">🔍</span>
            <input
              ref={inputRef}
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type a command or search..."
              className="
                flex-1 text-sm
                bg-transparent
                text-gray-900 dark:text-gray-100
                placeholder-gray-400 dark:placeholder-gray-500
                focus:outline-none
              "
              aria-label="Search commands"
              aria-autocomplete="list"
              aria-controls="command-list"
              aria-activedescendant={`command-${selectedIndex}`}
            />
            <kbd
              className="
                px-2 py-1 text-xs font-mono
                bg-gray-200 dark:bg-gray-700
                text-gray-600 dark:text-gray-400
                rounded border border-gray-300 dark:border-gray-600
              "
            >
              ESC
            </kbd>
          </div>
        </div>

        {/* Command List */}
        <div
          ref={listRef}
          id="command-list"
          role="listbox"
          className="
            max-h-96 overflow-y-auto
            p-2
          "
        >
          {filteredCommands.length === 0 ? (
            <div className="text-center py-8">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                No commands found
              </p>
              <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
                Try a different search term
              </p>
            </div>
          ) : (
            filteredCommands.map((command, index) => (
              <CommandItem
                key={command.id}
                command={command}
                isSelected={index === selectedIndex}
                onClick={() => handleCommandClick(command)}
                searchQuery={searchQuery}
              />
            ))
          )}
        </div>

        {/* Footer */}
        <div
          className="
            px-4 py-2 border-t border-gray-200 dark:border-gray-700
            bg-gray-50 dark:bg-gray-800
            flex items-center justify-between
            text-xs text-gray-500 dark:text-gray-400
          "
        >
          <div className="flex items-center gap-4">
            <span>
              {filteredCommands.length} command{filteredCommands.length !== 1 ? 's' : ''}
            </span>
            <div className="flex items-center gap-2">
              <kbd className="px-1.5 py-0.5 rounded bg-gray-200 dark:bg-gray-700">
                ↑↓
              </kbd>
              <span>Navigate</span>
            </div>
            <div className="flex items-center gap-2">
              <kbd className="px-1.5 py-0.5 rounded bg-gray-200 dark:bg-gray-700">
                ⏎
              </kbd>
              <span>Execute</span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <kbd className="px-1.5 py-0.5 rounded bg-gray-200 dark:bg-gray-700">
              Cmd
            </kbd>
            <span>+</span>
            <kbd className="px-1.5 py-0.5 rounded bg-gray-200 dark:bg-gray-700">
              K
            </kbd>
          </div>
        </div>
      </div>
    </>
  );
};
