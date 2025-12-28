/**
 * Focus Management Hook
 *
 * Provides utilities for managing keyboard focus programmatically.
 */

import { useCallback, useRef, useState, useEffect } from 'react';
import type { FocusableElement } from '../types';

export interface UseFocusManagementReturn {
  focusedId: string | null;
  focus: (id: string) => void;
  blur: () => void;
  focusNext: () => void;
  focusPrevious: () => void;
  focusFirst: () => void;
  focusLast: () => void;
  trapFocus: (containerId: string) => void;
  releaseTrap: (containerId: string) => void;
  registerFocusable: (element: FocusableElement) => void;
  unregisterFocusable: (id: string) => void;
  getFocusableElements: () => FocusableElement[];
}

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'area[href]',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  'button:not([disabled])',
  'iframe',
  'object',
  'embed',
  '[contenteditable]',
  '[tabindex]:not([tabindex="-1"])',
].join(',');

export const useFocusManagement = (): UseFocusManagementReturn => {
  const [focusedId, setFocusedId] = useState<string | null>(null);
  const [focusableElements, setFocusableElements] = useState<FocusableElement[]>([]);
  const [activeTrap, setActiveTrap] = useState<string | null>(null);
  const focusHistoryRef = useRef<string[]>([]);

  // Get all focusable elements
  const getFocusableElements = useCallback((): FocusableElement[] => {
    const elements: FocusableElement[] = [];
    const domElements = document.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR);

    domElements.forEach((el, index) => {
      if (el.offsetWidth > 0 && el.offsetHeight > 0) {
        elements.push({
          id: el.id || `focusable-${index}`,
          element: el,
          tabIndex: parseInt(el.getAttribute('tabindex') || '0', 10),
        });
      }
    });

    // Sort by tab index
    elements.sort((a, b) => {
      if (a.tabIndex === b.tabIndex) return 0;
      if (a.tabIndex === -1) return 1;
      if (b.tabIndex === -1) return -1;
      return a.tabIndex - b.tabIndex;
    });

    return elements;
  }, []);

  // Focus an element by ID
  const focus = useCallback((id: string) => {
    const element = document.getElementById(id);
    if (element) {
      element.focus();
      setFocusedId(id);
      focusHistoryRef.current.push(id);

      // Keep history at max 50 items
      if (focusHistoryRef.current.length > 50) {
        focusHistoryRef.current.shift();
      }
    }
  }, []);

  // Blur current focus
  const blur = useCallback(() => {
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
    setFocusedId(null);
  }, []);

  // Focus next element
  const focusNext = useCallback(() => {
    const elements = getFocusableElements();
    if (elements.length === 0) return;

    const currentIndex = elements.findIndex((el) => el.id === focusedId);
    const nextIndex = (currentIndex + 1) % elements.length;
    focus(elements[nextIndex].id);
  }, [focusedId, getFocusableElements, focus]);

  // Focus previous element
  const focusPrevious = useCallback(() => {
    const elements = getFocusableElements();
    if (elements.length === 0) return;

    const currentIndex = elements.findIndex((el) => el.id === focusedId);
    const prevIndex = currentIndex <= 0 ? elements.length - 1 : currentIndex - 1;
    focus(elements[prevIndex].id);
  }, [focusedId, getFocusableElements, focus]);

  // Focus first element
  const focusFirst = useCallback(() => {
    const elements = getFocusableElements();
    if (elements.length > 0) {
      focus(elements[0].id);
    }
  }, [getFocusableElements, focus]);

  // Focus last element
  const focusLast = useCallback(() => {
    const elements = getFocusableElements();
    if (elements.length > 0) {
      focus(elements[elements.length - 1].id);
    }
  }, [getFocusableElements, focus]);

  // Trap focus within container
  const trapFocus = useCallback((containerId: string) => {
    setActiveTrap(containerId);
  }, []);

  // Release focus trap
  const releaseTrap = useCallback((containerId: string) => {
    setActiveTrap((current) => (current === containerId ? null : current));
  }, []);

  // Register focusable element
  const registerFocusable = useCallback((element: FocusableElement) => {
    setFocusableElements((prev) => {
      const filtered = prev.filter((el) => el.id !== element.id);
      return [...filtered, element].sort((a, b) => a.tabIndex - b.tabIndex);
    });
  }, []);

  // Unregister focusable element
  const unregisterFocusable = useCallback((id: string) => {
    setFocusableElements((prev) => prev.filter((el) => el.id !== id));
  }, []);

  // Track focus changes
  useEffect(() => {
    const handleFocusChange = () => {
      const activeElement = document.activeElement as HTMLElement;
      if (activeElement && activeElement.id) {
        setFocusedId(activeElement.id);
      }
    };

    document.addEventListener('focusin', handleFocusChange);
    document.addEventListener('focusout', handleFocusChange);

    return () => {
      document.removeEventListener('focusin', handleFocusChange);
      document.removeEventListener('focusout', handleFocusChange);
    };
  }, []);

  return {
    focusedId,
    focus,
    blur,
    focusNext,
    focusPrevious,
    focusFirst,
    focusLast,
    trapFocus,
    releaseTrap,
    registerFocusable,
    unregisterFocusable,
    getFocusableElements,
  };
};

export default useFocusManagement;
