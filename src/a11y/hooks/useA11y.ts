/**
 * Accessibility Settings Hook
 *
 * Provides access to global accessibility settings and utilities.
 */

import { useCallback } from 'react';
import { useA11yContext } from '../A11yProvider';
import type { A11yConfig, KeyboardShortcut } from '../types';

export interface UseA11yReturn {
  config: A11yConfig;
  updateConfig: (settings: Partial<A11yConfig>) => void;
  registerShortcut: (shortcut: KeyboardShortcut) => void;
  unregisterShortcut: (key: string) => void;
  announce: (message: string, priority?: 'polite' | 'assertive') => void;
  announceSuccess: (message: string) => void;
  announceError: (message: string) => void;
  announceWarning: (message: string) => void;
  announceInfo: (message: string) => void;
  isReducedMotion: boolean;
  isHighContrast: boolean;
  textScale: number;
}

const TEXT_SIZE_SCALES = {
  small: 0.875,
  medium: 1.0,
  large: 1.25,
  'extra-large': 1.5,
} as const;

export const useA11y = (): UseA11yReturn => {
  const context = useA11yContext();

  const announceSuccess = useCallback(
    (message: string) => {
      context.announce(message, 'polite');
    },
    [context]
  );

  const announceError = useCallback(
    (message: string) => {
      context.announce(message, 'assertive');
    },
    [context]
  );

  const announceWarning = useCallback(
    (message: string) => {
      context.announce(message, 'assertive');
    },
    [context]
  );

  const announceInfo = useCallback(
    (message: string) => {
      context.announce(message, 'polite');
    },
    [context]
  );

  const isReducedMotion = context.config.motionPreference === 'reduce';
  const isHighContrast = context.config.colorScheme === 'high-contrast';
  const textScale = TEXT_SIZE_SCALES[context.config.textSize];

  return {
    config: context.config,
    updateConfig: context.updateConfig,
    registerShortcut: context.registerShortcut,
    unregisterShortcut: context.unregisterShortcut,
    announce: context.announce,
    announceSuccess,
    announceError,
    announceWarning,
    announceInfo,
    isReducedMotion,
    isHighContrast,
    textScale,
  };
};

export default useA11y;
