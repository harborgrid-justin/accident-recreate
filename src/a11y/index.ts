/**
 * AccuScene Accessibility System
 *
 * Comprehensive WCAG 2.1 AA compliant accessibility library for AccuScene Enterprise.
 * Provides components, hooks, and utilities for building accessible applications.
 *
 * @module a11y
 */

// Components
export { A11yProvider, A11yContext } from './A11yProvider';
export { SkipLinks } from './components/SkipLinks';
export { LiveRegion } from './components/LiveRegion';
export { FocusTrap } from './components/FocusTrap';
export { VisuallyHidden } from './components/VisuallyHidden';
export { AccessibleIcon } from './components/AccessibleIcon';

// Hooks
export { useA11y } from './hooks/useA11y';
export { useFocusManagement } from './hooks/useFocusManagement';
export { useReducedMotion } from './hooks/useReducedMotion';
export { useKeyboardNav } from './hooks/useKeyboardNav';

// Utilities
export * from './utils/contrastChecker';
export * from './utils/ariaHelpers';

// Testing
export { runA11yAudit, type AuditReport } from './testing/a11yAudit';

// Types
export type {
  A11yConfig,
  A11ySettings,
  WcagLevel,
  ColorScheme,
  TextSize,
  AriaRole,
  AriaAttributes,
  ContrastResult,
  KeyboardShortcut,
  FocusableElement,
} from './types';
