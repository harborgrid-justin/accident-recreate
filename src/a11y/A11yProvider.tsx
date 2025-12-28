/**
 * Accessibility Context Provider
 *
 * Provides global accessibility settings and utilities throughout the application.
 */

import React, { createContext, useContext, useState, useCallback, useEffect, ReactNode } from 'react';
import type { A11yConfig, A11ySettings, KeyboardShortcut } from './types';

interface A11yContextValue {
  config: A11yConfig;
  updateConfig: (settings: Partial<A11yConfig>) => void;
  registerShortcut: (shortcut: KeyboardShortcut) => void;
  unregisterShortcut: (key: string) => void;
  shortcuts: KeyboardShortcut[];
  announce: (message: string, priority?: 'polite' | 'assertive') => void;
}

const defaultConfig: A11yConfig = {
  wcagLevel: 'AA',
  screenReaderEnabled: true,
  keyboardNavEnabled: true,
  focusIndicatorsEnabled: true,
  colorScheme: 'auto',
  motionPreference: 'no-preference',
  textSize: 'medium',
  reduceTransparency: false,
  captionsEnabled: false,
  audioDescriptionsEnabled: false,
  timeoutDuration: 0,
  language: 'en',
  textDirection: 'ltr',
};

export const A11yContext = createContext<A11yContextValue | undefined>(undefined);

export interface A11yProviderProps {
  children: ReactNode;
  initialConfig?: A11ySettings;
  onConfigChange?: (config: A11yConfig) => void;
}

export const A11yProvider: React.FC<A11yProviderProps> = ({
  children,
  initialConfig = {},
  onConfigChange,
}) => {
  const [config, setConfig] = useState<A11yConfig>({
    ...defaultConfig,
    ...initialConfig,
  });

  const [shortcuts, setShortcuts] = useState<KeyboardShortcut[]>([]);

  // Update config
  const updateConfig = useCallback((settings: Partial<A11yConfig>) => {
    setConfig((prev) => {
      const newConfig = { ...prev, ...settings };
      onConfigChange?.(newConfig);
      return newConfig;
    });
  }, [onConfigChange]);

  // Register keyboard shortcut
  const registerShortcut = useCallback((shortcut: KeyboardShortcut) => {
    setShortcuts((prev) => {
      const filtered = prev.filter((s) => s.key !== shortcut.key);
      return [...filtered, shortcut];
    });
  }, []);

  // Unregister keyboard shortcut
  const unregisterShortcut = useCallback((key: string) => {
    setShortcuts((prev) => prev.filter((s) => s.key !== key));
  }, []);

  // Announce to screen readers
  const announce = useCallback((message: string, priority: 'polite' | 'assertive' = 'polite') => {
    const announcement = new CustomEvent('a11y-announce', {
      detail: { message, priority },
    });
    window.dispatchEvent(announcement);
  }, []);

  // Handle keyboard shortcuts
  useEffect(() => {
    if (!config.keyboardNavEnabled) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      const matchingShortcut = shortcuts.find((shortcut) => {
        const ctrlMatch = shortcut.ctrl === undefined || shortcut.ctrl === event.ctrlKey;
        const altMatch = shortcut.alt === undefined || shortcut.alt === event.altKey;
        const shiftMatch = shortcut.shift === undefined || shortcut.shift === event.shiftKey;
        const metaMatch = shortcut.meta === undefined || shortcut.meta === event.metaKey;
        const keyMatch = shortcut.key === event.key;

        return ctrlMatch && altMatch && shiftMatch && metaMatch && keyMatch;
      });

      if (matchingShortcut) {
        event.preventDefault();
        matchingShortcut.action();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [shortcuts, config.keyboardNavEnabled]);

  // Apply document-level settings
  useEffect(() => {
    const root = document.documentElement;

    // Set text size
    const textSizeScale = {
      small: 0.875,
      medium: 1.0,
      large: 1.25,
      'extra-large': 1.5,
    }[config.textSize];
    root.style.setProperty('--a11y-text-scale', textSizeScale.toString());

    // Set color scheme
    root.setAttribute('data-color-scheme', config.colorScheme);

    // Set text direction
    root.setAttribute('dir', config.textDirection);

    // Set language
    root.setAttribute('lang', config.language);

    // Motion preference
    if (config.motionPreference === 'reduce') {
      root.style.setProperty('--a11y-motion-duration', '0.01ms');
    } else {
      root.style.removeProperty('--a11y-motion-duration');
    }

    // Transparency
    if (config.reduceTransparency) {
      root.style.setProperty('--a11y-transparency', '1');
    } else {
      root.style.removeProperty('--a11y-transparency');
    }
  }, [config]);

  // Detect system preferences
  useEffect(() => {
    // Detect prefers-reduced-motion
    const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    const handleMotionChange = (e: MediaQueryListEvent | MediaQueryList) => {
      if (config.motionPreference === 'no-preference') {
        updateConfig({
          motionPreference: e.matches ? 'reduce' : 'no-preference',
        });
      }
    };
    handleMotionChange(motionQuery);
    motionQuery.addEventListener('change', handleMotionChange);

    // Detect color scheme preference
    const colorQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleColorChange = (e: MediaQueryListEvent | MediaQueryList) => {
      if (config.colorScheme === 'auto') {
        updateConfig({
          colorScheme: e.matches ? 'dark' : 'light',
        });
      }
    };
    handleColorChange(colorQuery);
    colorQuery.addEventListener('change', handleColorChange);

    return () => {
      motionQuery.removeEventListener('change', handleMotionChange);
      colorQuery.removeEventListener('change', handleColorChange);
    };
  }, [config.motionPreference, config.colorScheme, updateConfig]);

  const value: A11yContextValue = {
    config,
    updateConfig,
    registerShortcut,
    unregisterShortcut,
    shortcuts,
    announce,
  };

  return <A11yContext.Provider value={value}>{children}</A11yContext.Provider>;
};

export const useA11yContext = (): A11yContextValue => {
  const context = useContext(A11yContext);
  if (!context) {
    throw new Error('useA11yContext must be used within A11yProvider');
  }
  return context;
};
