/**
 * AccuScene Enterprise v0.3.0 - Command Palette Hook
 */

import { useState, useCallback, useEffect, useMemo } from 'react';
import { Command } from '../types';

export interface UseCommandPaletteReturn {
  isOpen: boolean;
  open: () => void;
  close: () => void;
  toggle: () => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  filteredCommands: Command[];
  selectedIndex: number;
  setSelectedIndex: (index: number) => void;
  executeCommand: (command: Command) => void;
  executeSelected: () => void;
  navigateUp: () => void;
  navigateDown: () => void;
}

// Fuzzy search function
function fuzzyMatch(text: string, query: string): boolean {
  const lowerText = text.toLowerCase();
  const lowerQuery = query.toLowerCase();

  let queryIndex = 0;
  for (let i = 0; i < lowerText.length && queryIndex < lowerQuery.length; i++) {
    if (lowerText[i] === lowerQuery[queryIndex]) {
      queryIndex++;
    }
  }

  return queryIndex === lowerQuery.length;
}

function calculateScore(command: Command, query: string): number {
  let score = 0;
  const lowerQuery = query.toLowerCase();

  // Exact match in name
  if (command.name.toLowerCase() === lowerQuery) {
    score += 100;
  }

  // Starts with query
  if (command.name.toLowerCase().startsWith(lowerQuery)) {
    score += 50;
  }

  // Contains query
  if (command.name.toLowerCase().includes(lowerQuery)) {
    score += 25;
  }

  // Tag match
  if (command.tags.some((tag) => tag.toLowerCase().includes(lowerQuery))) {
    score += 15;
  }

  // Fuzzy match
  if (fuzzyMatch(command.name, query)) {
    score += 10;
  }

  return score;
}

export function useCommandPalette(commands: Command[]): UseCommandPaletteReturn {
  const [isOpen, setIsOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);

  const open = useCallback(() => setIsOpen(true), []);
  const close = useCallback(() => {
    setIsOpen(false);
    setSearchQuery('');
    setSelectedIndex(0);
  }, []);
  const toggle = useCallback(() => setIsOpen((prev) => !prev), []);

  const filteredCommands = useMemo(() => {
    if (!searchQuery.trim()) {
      return commands;
    }

    const scored = commands
      .map((command) => ({
        command,
        score: calculateScore(command, searchQuery),
      }))
      .filter(({ score }) => score > 0)
      .sort((a, b) => b.score - a.score);

    return scored.map(({ command }) => command);
  }, [commands, searchQuery]);

  const executeCommand = useCallback(
    async (command: Command) => {
      try {
        await command.action();
        close();
      } catch (error) {
        console.error('Error executing command:', error);
      }
    },
    [close]
  );

  const executeSelected = useCallback(() => {
    if (filteredCommands[selectedIndex]) {
      executeCommand(filteredCommands[selectedIndex]);
    }
  }, [filteredCommands, selectedIndex, executeCommand]);

  const navigateUp = useCallback(() => {
    setSelectedIndex((prev) => Math.max(0, prev - 1));
  }, []);

  const navigateDown = useCallback(() => {
    setSelectedIndex((prev) =>
      Math.min(filteredCommands.length - 1, prev + 1)
    );
  }, [filteredCommands.length]);

  // Reset selected index when filtered commands change
  useEffect(() => {
    setSelectedIndex(0);
  }, [filteredCommands]);

  // Keyboard shortcut to open palette (Cmd+K / Ctrl+K)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        toggle();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [toggle]);

  return {
    isOpen,
    open,
    close,
    toggle,
    searchQuery,
    setSearchQuery,
    filteredCommands,
    selectedIndex,
    setSelectedIndex,
    executeCommand,
    executeSelected,
    navigateUp,
    navigateDown,
  };
}
