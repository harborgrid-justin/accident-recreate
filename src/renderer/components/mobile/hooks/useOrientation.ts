/**
 * AccuScene Enterprise v0.3.0
 * useOrientation Hook
 *
 * React hook for device orientation detection and lock management
 */

import { useState, useEffect, useCallback } from 'react';
import { Orientation, OrientationState } from '../types';

interface UseOrientationResult extends OrientationState {
  lock: (orientation: OrientationType) => Promise<void>;
  unlock: () => Promise<void>;
  isPortrait: boolean;
  isLandscape: boolean;
  angle: number;
  supported: boolean;
}

/**
 * Hook to detect and manage device orientation
 * Supports orientation locking on supported devices
 *
 * @returns Orientation state and control methods
 *
 * @example
 * ```tsx
 * const { orientation, isLandscape, lock, unlock } = useOrientation();
 *
 * // Lock to landscape for video playback
 * const handleVideoFullscreen = async () => {
 *   await lock('landscape');
 * };
 *
 * // Unlock when done
 * const handleVideoExit = async () => {
 *   await unlock();
 * };
 * ```
 */
export function useOrientation(): UseOrientationResult {
  const getOrientation = (): Orientation => {
    if (typeof window === 'undefined') return 'portrait';

    // Check screen orientation API first
    if (window.screen?.orientation) {
      return window.screen.orientation.type.startsWith('portrait')
        ? 'portrait'
        : 'landscape';
    }

    // Fallback to matchMedia
    if (window.matchMedia) {
      return window.matchMedia('(orientation: portrait)').matches
        ? 'portrait'
        : 'landscape';
    }

    // Final fallback to dimensions
    return window.innerHeight > window.innerWidth ? 'portrait' : 'landscape';
  };

  const getAngle = (): number => {
    if (typeof window === 'undefined') return 0;

    if (window.screen?.orientation) {
      return window.screen.orientation.angle;
    }

    // Fallback to window.orientation (deprecated but still used on some devices)
    if ('orientation' in window) {
      return (window as any).orientation || 0;
    }

    return 0;
  };

  const [state, setState] = useState<OrientationState>({
    orientation: getOrientation(),
    angle: getAngle(),
    locked: false,
  });

  const handleOrientationChange = useCallback(() => {
    setState((prev) => ({
      ...prev,
      orientation: getOrientation(),
      angle: getAngle(),
    }));
  }, []);

  const lock = useCallback(
    async (orientation: OrientationType): Promise<void> => {
      if (typeof window === 'undefined') return;

      try {
        // Screen Orientation API
        if (window.screen?.orientation?.lock) {
          await window.screen.orientation.lock(orientation);
          setState((prev) => ({ ...prev, locked: true }));
        } else {
          console.warn('Screen Orientation API not supported');
        }
      } catch (error) {
        console.error('Failed to lock orientation:', error);
        throw error;
      }
    },
    []
  );

  const unlock = useCallback(async (): Promise<void> => {
    if (typeof window === 'undefined') return;

    try {
      // Screen Orientation API
      if (window.screen?.orientation?.unlock) {
        window.screen.orientation.unlock();
        setState((prev) => ({ ...prev, locked: false }));
      }
    } catch (error) {
      console.error('Failed to unlock orientation:', error);
      throw error;
    }
  }, []);

  useEffect(() => {
    if (typeof window === 'undefined') return;

    // Listen to orientation change events
    if (window.screen?.orientation) {
      window.screen.orientation.addEventListener('change', handleOrientationChange);
    }

    // Fallback to orientationchange event (deprecated but widely supported)
    window.addEventListener('orientationchange', handleOrientationChange);

    // Also listen to resize as a fallback
    window.addEventListener('resize', handleOrientationChange);

    return () => {
      if (window.screen?.orientation) {
        window.screen.orientation.removeEventListener('change', handleOrientationChange);
      }
      window.removeEventListener('orientationchange', handleOrientationChange);
      window.removeEventListener('resize', handleOrientationChange);
    };
  }, [handleOrientationChange]);

  const supported =
    typeof window !== 'undefined' &&
    window.screen?.orientation?.lock !== undefined;

  return {
    ...state,
    lock,
    unlock,
    isPortrait: state.orientation === 'portrait',
    isLandscape: state.orientation === 'landscape',
    supported,
  };
}
