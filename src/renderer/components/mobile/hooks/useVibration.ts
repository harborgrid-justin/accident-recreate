/**
 * AccuScene Enterprise v0.3.0
 * useVibration Hook
 *
 * React hook for haptic feedback and vibration control
 */

import { useCallback, useRef } from 'react';
import { HapticPattern, HapticOptions } from '../types';

interface UseVibrationResult {
  vibrate: (pattern?: number | number[]) => void;
  vibratePattern: (patternName: HapticPattern) => void;
  cancel: () => void;
  supported: boolean;
}

// Predefined haptic patterns (in ms)
const HAPTIC_PATTERNS: Record<HapticPattern, number | number[]> = {
  light: 10,
  medium: 20,
  heavy: 40,
  success: [10, 50, 10],
  warning: [20, 100, 20, 100, 20],
  error: [50, 100, 50, 100, 50],
  selection: 5,
};

/**
 * Hook for haptic feedback and vibration
 * Provides convenient methods for common haptic patterns
 *
 * @returns Vibration control methods
 *
 * @example
 * ```tsx
 * const { vibratePattern, vibrate, supported } = useVibration();
 *
 * // Use predefined patterns
 * const handleSuccess = () => {
 *   vibratePattern('success');
 * };
 *
 * // Custom vibration
 * const handleCustom = () => {
 *   vibrate([100, 50, 100, 50, 200]);
 * };
 *
 * if (!supported) {
 *   console.log('Vibration not supported on this device');
 * }
 * ```
 */
export function useVibration(): UseVibrationResult {
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  const supported =
    typeof navigator !== 'undefined' && 'vibrate' in navigator;

  const vibrate = useCallback(
    (pattern: number | number[] = 10): void => {
      if (!supported) {
        console.warn('Vibration API not supported');
        return;
      }

      try {
        // Cancel any ongoing vibration
        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current);
          timeoutRef.current = null;
        }

        // Execute vibration
        navigator.vibrate(pattern);

        // If pattern is an array, calculate total duration for cleanup
        if (Array.isArray(pattern)) {
          const totalDuration = pattern.reduce((sum, val) => sum + val, 0);
          timeoutRef.current = setTimeout(() => {
            timeoutRef.current = null;
          }, totalDuration);
        }
      } catch (error) {
        console.error('Vibration failed:', error);
      }
    },
    [supported]
  );

  const vibratePattern = useCallback(
    (patternName: HapticPattern): void => {
      const pattern = HAPTIC_PATTERNS[patternName];
      vibrate(pattern);
    },
    [vibrate]
  );

  const cancel = useCallback((): void => {
    if (!supported) return;

    try {
      navigator.vibrate(0);
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }
    } catch (error) {
      console.error('Failed to cancel vibration:', error);
    }
  }, [supported]);

  return {
    vibrate,
    vibratePattern,
    cancel,
    supported,
  };
}

/**
 * Convenience hook for common haptic feedback patterns
 * Automatically handles non-supported devices
 */
export function useHapticFeedback() {
  const { vibratePattern, supported } = useVibration();

  return {
    light: () => vibratePattern('light'),
    medium: () => vibratePattern('medium'),
    heavy: () => vibratePattern('heavy'),
    success: () => vibratePattern('success'),
    warning: () => vibratePattern('warning'),
    error: () => vibratePattern('error'),
    selection: () => vibratePattern('selection'),
    supported,
  };
}
