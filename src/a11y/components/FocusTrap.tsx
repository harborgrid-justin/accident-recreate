/**
 * Focus Trap Component
 *
 * Traps keyboard focus within a container (e.g., modal dialogs).
 * Implements WCAG 2.1 Success Criterion 2.1.2 (No Keyboard Trap - with escape)
 */

import React, { useEffect, useRef, ReactNode } from 'react';
import { useFocusManagement } from '../hooks/useFocusManagement';

export interface FocusTrapProps {
  active?: boolean;
  children: ReactNode;
  restoreFocus?: boolean;
  initialFocus?: React.RefObject<HTMLElement>;
  onEscape?: () => void;
  className?: string;
}

const FOCUSABLE_SELECTORS = [
  'a[href]',
  'area[href]',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  'button:not([disabled])',
  'iframe',
  'object',
  'embed',
  '[contenteditable]',
  '[tabindex]:not([tabindex="-1"])',
].join(',');

export const FocusTrap: React.FC<FocusTrapProps> = ({
  active = true,
  children,
  restoreFocus = true,
  initialFocus,
  onEscape,
  className = '',
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);
  const { trapFocus, releaseTrap } = useFocusManagement();

  // Get all focusable elements within the trap
  const getFocusableElements = (): HTMLElement[] => {
    if (!containerRef.current) return [];

    const elements = Array.from(
      containerRef.current.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTORS)
    );

    return elements.filter((el) => {
      // Filter out hidden elements
      return (
        el.offsetWidth > 0 &&
        el.offsetHeight > 0 &&
        !el.hasAttribute('hidden') &&
        getComputedStyle(el).visibility !== 'hidden'
      );
    });
  };

  // Focus first element
  const focusFirstElement = () => {
    if (initialFocus?.current) {
      initialFocus.current.focus();
      return;
    }

    const focusable = getFocusableElements();
    if (focusable.length > 0) {
      focusable[0].focus();
    }
  };

  // Handle Tab key navigation
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!active) return;

    // Handle Escape key
    if (e.key === 'Escape' && onEscape) {
      e.preventDefault();
      onEscape();
      return;
    }

    // Handle Tab key
    if (e.key === 'Tab') {
      const focusable = getFocusableElements();
      if (focusable.length === 0) return;

      const firstElement = focusable[0];
      const lastElement = focusable[focusable.length - 1];
      const activeElement = document.activeElement as HTMLElement;

      // Shift + Tab on first element -> focus last
      if (e.shiftKey && activeElement === firstElement) {
        e.preventDefault();
        lastElement.focus();
      }
      // Tab on last element -> focus first
      else if (!e.shiftKey && activeElement === lastElement) {
        e.preventDefault();
        firstElement.focus();
      }
    }
  };

  // Activate trap
  useEffect(() => {
    if (!active) return;

    // Save current focus
    previousFocusRef.current = document.activeElement as HTMLElement;

    // Focus first element
    requestAnimationFrame(() => {
      focusFirstElement();
    });

    // Add event listener
    document.addEventListener('keydown', handleKeyDown);

    // Trap focus in focus manager
    if (containerRef.current) {
      trapFocus(containerRef.current.id || 'focus-trap');
    }

    return () => {
      document.removeEventListener('keydown', handleKeyDown);

      // Release trap
      if (containerRef.current) {
        releaseTrap(containerRef.current.id || 'focus-trap');
      }

      // Restore focus
      if (restoreFocus && previousFocusRef.current) {
        previousFocusRef.current.focus();
      }
    };
  }, [active, restoreFocus]);

  // Monitor for new focusable elements
  useEffect(() => {
    if (!active || !containerRef.current) return;

    const observer = new MutationObserver(() => {
      const focusable = getFocusableElements();
      if (focusable.length > 0 && !focusable.includes(document.activeElement as HTMLElement)) {
        focusable[0].focus();
      }
    });

    observer.observe(containerRef.current, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['disabled', 'tabindex', 'hidden'],
    });

    return () => observer.disconnect();
  }, [active]);

  return (
    <div
      ref={containerRef}
      className={`focus-trap ${className}`}
      data-focus-trap={active ? 'active' : 'inactive'}
    >
      {children}
    </div>
  );
};

export default FocusTrap;
