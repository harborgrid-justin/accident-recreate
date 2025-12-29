/**
 * Keyboard Navigation Hook
 *
 * Provides keyboard navigation utilities and event handlers.
 * Implements WCAG 2.1 Success Criterion 2.1.1 (Keyboard)
 */

import { useEffect, useCallback, useRef } from 'react';
import type { KeyboardShortcut } from '../types';
import { useA11yContext } from '../A11yProvider';

export interface KeyboardNavOptions {
  enabled?: boolean;
  captureTab?: boolean;
  captureArrows?: boolean;
  captureEscape?: boolean;
  onTab?: (event: KeyboardEvent) => void;
  onShiftTab?: (event: KeyboardEvent) => void;
  onArrowUp?: (event: KeyboardEvent) => void;
  onArrowDown?: (event: KeyboardEvent) => void;
  onArrowLeft?: (event: KeyboardEvent) => void;
  onArrowRight?: (event: KeyboardEvent) => void;
  onEnter?: (event: KeyboardEvent) => void;
  onSpace?: (event: KeyboardEvent) => void;
  onEscape?: (event: KeyboardEvent) => void;
  onHome?: (event: KeyboardEvent) => void;
  onEnd?: (event: KeyboardEvent) => void;
}

export interface UseKeyboardNavReturn {
  handleKeyDown: (event: React.KeyboardEvent) => void;
  registerShortcut: (shortcut: KeyboardShortcut) => void;
  unregisterShortcut: (key: string) => void;
  isKeyboardUser: boolean;
}

export const useKeyboardNav = (
  options: KeyboardNavOptions = {}
): UseKeyboardNavReturn => {
  const {
    enabled = true,
    captureTab = false,
    captureArrows = false,
    captureEscape = false,
    onTab,
    onShiftTab,
    onArrowUp,
    onArrowDown,
    onArrowLeft,
    onArrowRight,
    onEnter,
    onSpace,
    onEscape,
    onHome,
    onEnd,
  } = options;

  const { registerShortcut: contextRegisterShortcut, unregisterShortcut: contextUnregisterShortcut } = useA11yContext();
  const isKeyboardUser = useRef(false);

  // Handle keyboard event
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (!enabled) return;

      // Mark as keyboard user
      isKeyboardUser.current = true;

      const { key, shiftKey } = event;

      switch (key) {
        case 'Tab':
          if (captureTab) {
            event.preventDefault();
          }
          if (shiftKey && onShiftTab) {
            onShiftTab(event.nativeEvent);
          } else if (!shiftKey && onTab) {
            onTab(event.nativeEvent);
          }
          break;

        case 'ArrowUp':
          if (captureArrows && onArrowUp) {
            event.preventDefault();
            onArrowUp(event.nativeEvent);
          }
          break;

        case 'ArrowDown':
          if (captureArrows && onArrowDown) {
            event.preventDefault();
            onArrowDown(event.nativeEvent);
          }
          break;

        case 'ArrowLeft':
          if (captureArrows && onArrowLeft) {
            event.preventDefault();
            onArrowLeft(event.nativeEvent);
          }
          break;

        case 'ArrowRight':
          if (captureArrows && onArrowRight) {
            event.preventDefault();
            onArrowRight(event.nativeEvent);
          }
          break;

        case 'Enter':
          if (onEnter) {
            onEnter(event.nativeEvent);
          }
          break;

        case ' ':
        case 'Space':
          if (onSpace) {
            event.preventDefault();
            onSpace(event.nativeEvent);
          }
          break;

        case 'Escape':
        case 'Esc':
          if (captureEscape && onEscape) {
            event.preventDefault();
            onEscape(event.nativeEvent);
          }
          break;

        case 'Home':
          if (onHome) {
            event.preventDefault();
            onHome(event.nativeEvent);
          }
          break;

        case 'End':
          if (onEnd) {
            event.preventDefault();
            onEnd(event.nativeEvent);
          }
          break;
      }
    },
    [
      enabled,
      captureTab,
      captureArrows,
      captureEscape,
      onTab,
      onShiftTab,
      onArrowUp,
      onArrowDown,
      onArrowLeft,
      onArrowRight,
      onEnter,
      onSpace,
      onEscape,
      onHome,
      onEnd,
    ]
  );

  // Detect mouse usage (keyboard user flag reset)
  useEffect(() => {
    const handleMouseDown = () => {
      isKeyboardUser.current = false;
    };

    window.addEventListener('mousedown', handleMouseDown);
    return () => window.removeEventListener('mousedown', handleMouseDown);
  }, []);

  // Add keyboard user class to body
  useEffect(() => {
    const handleKeyDown = () => {
      document.body.classList.add('keyboard-user');
    };

    const handleMouseDown = () => {
      document.body.classList.remove('keyboard-user');
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('mousedown', handleMouseDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('mousedown', handleMouseDown);
    };
  }, []);

  return {
    handleKeyDown,
    registerShortcut: contextRegisterShortcut,
    unregisterShortcut: contextUnregisterShortcut,
    isKeyboardUser: isKeyboardUser.current,
  };
};

/**
 * Hook for managing roving tabindex (e.g., in lists, menus)
 */
export const useRovingTabIndex = (itemCount: number, initialIndex = 0) => {
  const [focusedIndex, setFocusedIndex] = React.useState(initialIndex);

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      switch (event.key) {
        case 'ArrowDown':
        case 'ArrowRight':
          event.preventDefault();
          setFocusedIndex((prev) => (prev + 1) % itemCount);
          break;

        case 'ArrowUp':
        case 'ArrowLeft':
          event.preventDefault();
          setFocusedIndex((prev) => (prev - 1 + itemCount) % itemCount);
          break;

        case 'Home':
          event.preventDefault();
          setFocusedIndex(0);
          break;

        case 'End':
          event.preventDefault();
          setFocusedIndex(itemCount - 1);
          break;
      }
    },
    [itemCount]
  );

  const getTabIndex = useCallback(
    (index: number) => (index === focusedIndex ? 0 : -1),
    [focusedIndex]
  );

  return {
    focusedIndex,
    setFocusedIndex,
    handleKeyDown,
    getTabIndex,
  };
};

export default useKeyboardNav;
