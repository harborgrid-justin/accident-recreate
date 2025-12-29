/**
 * useResponsive Hook
 *
 * Manages responsive breakpoint detection and provides utilities
 * for responsive behavior
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { Breakpoint, BREAKPOINT_WIDTHS, GridConfig } from '../types';

/**
 * Get breakpoint from window width
 */
export function getBreakpointFromWidth(width: number): Breakpoint {
  if (width < 576) return Breakpoint.Mobile;
  if (width < 768) return Breakpoint.MobileLandscape;
  if (width < 992) return Breakpoint.Tablet;
  if (width < 1200) return Breakpoint.Desktop;
  if (width < 1600) return Breakpoint.DesktopLarge;
  return Breakpoint.DesktopXL;
}

/**
 * Get default grid config for breakpoint
 */
export function getDefaultGridConfig(breakpoint: Breakpoint): GridConfig {
  const configs: Record<Breakpoint, GridConfig> = {
    [Breakpoint.Mobile]: {
      columns: 4,
      rowHeight: 60,
      horizontalGap: 8,
      verticalGap: 8,
      containerPadding: 12,
    },
    [Breakpoint.MobileLandscape]: {
      columns: 6,
      rowHeight: 70,
      horizontalGap: 12,
      verticalGap: 12,
      containerPadding: 16,
    },
    [Breakpoint.Tablet]: {
      columns: 8,
      rowHeight: 80,
      horizontalGap: 16,
      verticalGap: 16,
      containerPadding: 20,
    },
    [Breakpoint.Desktop]: {
      columns: 12,
      rowHeight: 80,
      horizontalGap: 16,
      verticalGap: 16,
      containerPadding: 24,
    },
    [Breakpoint.DesktopLarge]: {
      columns: 12,
      rowHeight: 80,
      horizontalGap: 16,
      verticalGap: 16,
      containerPadding: 24,
    },
    [Breakpoint.DesktopXL]: {
      columns: 16,
      rowHeight: 90,
      horizontalGap: 20,
      verticalGap: 20,
      containerPadding: 32,
    },
  };

  return configs[breakpoint];
}

/**
 * Hook return type
 */
interface UseResponsiveReturn {
  breakpoint: Breakpoint;
  width: number;
  height: number;
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  isLargeDesktop: boolean;
  gridConfig: GridConfig;
  matchesBreakpoint: (bp: Breakpoint) => boolean;
  isAtLeast: (bp: Breakpoint) => boolean;
  isAtMost: (bp: Breakpoint) => boolean;
}

/**
 * Custom hook for responsive behavior
 */
export function useResponsive(): UseResponsiveReturn {
  const [windowSize, setWindowSize] = useState({
    width: typeof window !== 'undefined' ? window.innerWidth : 1200,
    height: typeof window !== 'undefined' ? window.innerHeight : 800,
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const handleResize = () => {
      setWindowSize({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    // Add debouncing for better performance
    let timeoutId: NodeJS.Timeout;
    const debouncedResize = () => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(handleResize, 150);
    };

    window.addEventListener('resize', debouncedResize);

    return () => {
      clearTimeout(timeoutId);
      window.removeEventListener('resize', debouncedResize);
    };
  }, []);

  const breakpoint = useMemo(
    () => getBreakpointFromWidth(windowSize.width),
    [windowSize.width]
  );

  const gridConfig = useMemo(() => getDefaultGridConfig(breakpoint), [breakpoint]);

  const isMobile = breakpoint === Breakpoint.Mobile;
  const isTablet = breakpoint === Breakpoint.Tablet || breakpoint === Breakpoint.MobileLandscape;
  const isDesktop =
    breakpoint === Breakpoint.Desktop ||
    breakpoint === Breakpoint.DesktopLarge ||
    breakpoint === Breakpoint.DesktopXL;
  const isLargeDesktop =
    breakpoint === Breakpoint.DesktopLarge || breakpoint === Breakpoint.DesktopXL;

  const matchesBreakpoint = useCallback(
    (bp: Breakpoint) => breakpoint === bp,
    [breakpoint]
  );

  const breakpointOrder = [
    Breakpoint.Mobile,
    Breakpoint.MobileLandscape,
    Breakpoint.Tablet,
    Breakpoint.Desktop,
    Breakpoint.DesktopLarge,
    Breakpoint.DesktopXL,
  ];

  const isAtLeast = useCallback(
    (bp: Breakpoint) => {
      const currentIndex = breakpointOrder.indexOf(breakpoint);
      const targetIndex = breakpointOrder.indexOf(bp);
      return currentIndex >= targetIndex;
    },
    [breakpoint]
  );

  const isAtMost = useCallback(
    (bp: Breakpoint) => {
      const currentIndex = breakpointOrder.indexOf(breakpoint);
      const targetIndex = breakpointOrder.indexOf(bp);
      return currentIndex <= targetIndex;
    },
    [breakpoint]
  );

  return {
    breakpoint,
    width: windowSize.width,
    height: windowSize.height,
    isMobile,
    isTablet,
    isDesktop,
    isLargeDesktop,
    gridConfig,
    matchesBreakpoint,
    isAtLeast,
    isAtMost,
  };
}

/**
 * Hook for media queries
 */
export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() => {
    if (typeof window === 'undefined') return false;
    return window.matchMedia(query).matches;
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const mediaQuery = window.matchMedia(query);
    const handler = (event: MediaQueryListEvent) => setMatches(event.matches);

    // Modern browsers
    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handler);
      return () => mediaQuery.removeEventListener('change', handler);
    }
    // Legacy browsers
    else {
      mediaQuery.addListener(handler);
      return () => mediaQuery.removeListener(handler);
    }
  }, [query]);

  return matches;
}

/**
 * Hook for orientation detection
 */
export function useOrientation(): 'portrait' | 'landscape' {
  const [orientation, setOrientation] = useState<'portrait' | 'landscape'>(() => {
    if (typeof window === 'undefined') return 'landscape';
    return window.innerWidth > window.innerHeight ? 'landscape' : 'portrait';
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const handleOrientationChange = () => {
      setOrientation(window.innerWidth > window.innerHeight ? 'landscape' : 'portrait');
    };

    window.addEventListener('resize', handleOrientationChange);
    return () => window.removeEventListener('resize', handleOrientationChange);
  }, []);

  return orientation;
}

/**
 * Hook for touch device detection
 */
export function useTouch(): boolean {
  const [isTouch, setIsTouch] = useState(() => {
    if (typeof window === 'undefined') return false;
    return 'ontouchstart' in window || navigator.maxTouchPoints > 0;
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const checkTouch = () => {
      setIsTouch('ontouchstart' in window || navigator.maxTouchPoints > 0);
    };

    checkTouch();
  }, []);

  return isTouch;
}

export default useResponsive;
