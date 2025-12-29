/**
 * Reduced Motion Hook
 *
 * Detects and responds to prefers-reduced-motion preference.
 * Implements WCAG 2.1 Success Criterion 2.3.3 (Animation from Interactions)
 */

import { useState, useEffect } from 'react';
import { useA11yContext } from '../A11yProvider';

export interface UseReducedMotionReturn {
  prefersReducedMotion: boolean;
  shouldReduceMotion: boolean;
  getAnimationDuration: (defaultDuration: number) => number;
  getTransitionDuration: (defaultDuration: number) => number;
}

const REDUCED_MOTION_QUERY = '(prefers-reduced-motion: reduce)';

export const useReducedMotion = (): UseReducedMotionReturn => {
  const { config } = useA11yContext();
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);

  useEffect(() => {
    const mediaQuery = window.matchMedia(REDUCED_MOTION_QUERY);
    setPrefersReducedMotion(mediaQuery.matches);

    const handleChange = (event: MediaQueryListEvent | MediaQueryList) => {
      setPrefersReducedMotion(event.matches);
    };

    // Modern browsers
    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handleChange);
      return () => mediaQuery.removeEventListener('change', handleChange);
    }
    // Legacy browsers
    else if (mediaQuery.addListener) {
      mediaQuery.addListener(handleChange as any);
      return () => mediaQuery.removeListener(handleChange as any);
    }
  }, []);

  // Check if motion should be reduced (system preference OR app setting)
  const shouldReduceMotion =
    prefersReducedMotion || config.motionPreference === 'reduce';

  // Get animation duration (0 if reduced motion)
  const getAnimationDuration = (defaultDuration: number): number => {
    return shouldReduceMotion ? 0 : defaultDuration;
  };

  // Get transition duration (minimal if reduced motion)
  const getTransitionDuration = (defaultDuration: number): number => {
    return shouldReduceMotion ? Math.min(defaultDuration, 50) : defaultDuration;
  };

  return {
    prefersReducedMotion,
    shouldReduceMotion,
    getAnimationDuration,
    getTransitionDuration,
  };
};

/**
 * CSS-in-JS helper for animations
 */
export const getAnimationStyles = (
  shouldReduce: boolean,
  normalStyles: React.CSSProperties,
  reducedStyles: React.CSSProperties = {}
): React.CSSProperties => {
  return shouldReduce ? reducedStyles : normalStyles;
};

/**
 * Hook for animation-safe inline styles
 */
export const useAnimationStyles = (
  normalStyles: React.CSSProperties,
  reducedStyles: React.CSSProperties = {}
): React.CSSProperties => {
  const { shouldReduceMotion } = useReducedMotion();
  return getAnimationStyles(shouldReduceMotion, normalStyles, reducedStyles);
};

export default useReducedMotion;
