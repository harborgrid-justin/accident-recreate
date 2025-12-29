/**
 * AccuScene Enterprise v0.3.0
 * useBreakpoint Hook
 *
 * React hook for responsive breakpoint detection and media queries
 */

import { useState, useEffect, useMemo } from 'react';
import { Breakpoint, BreakpointConfig, DEFAULT_BREAKPOINTS } from '../types';

interface UseBreakpointResult {
  currentBreakpoint: Breakpoint;
  width: number;
  height: number;
  isXs: boolean;
  isSm: boolean;
  isMd: boolean;
  isLg: boolean;
  isXl: boolean;
  is2xl: boolean;
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  greaterThan: (breakpoint: Breakpoint) => boolean;
  lessThan: (breakpoint: Breakpoint) => boolean;
}

/**
 * Hook to detect and respond to viewport breakpoints
 * Supports mobile-first responsive design patterns
 *
 * @param customBreakpoints - Optional custom breakpoint configuration
 * @returns Breakpoint detection utilities and current viewport information
 *
 * @example
 * ```tsx
 * const { isMobile, isTablet, currentBreakpoint, greaterThan } = useBreakpoint();
 *
 * if (isMobile) {
 *   return <MobileView />;
 * }
 *
 * if (greaterThan('md')) {
 *   return <DesktopView />;
 * }
 * ```
 */
export function useBreakpoint(
  customBreakpoints?: Partial<BreakpointConfig>
): UseBreakpointResult {
  const breakpoints: BreakpointConfig = useMemo(
    () => ({ ...DEFAULT_BREAKPOINTS, ...customBreakpoints }),
    [customBreakpoints]
  );

  const [dimensions, setDimensions] = useState({
    width: typeof window !== 'undefined' ? window.innerWidth : 0,
    height: typeof window !== 'undefined' ? window.innerHeight : 0,
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    let timeoutId: NodeJS.Timeout;

    const handleResize = () => {
      // Debounce resize events for performance
      clearTimeout(timeoutId);
      timeoutId = setTimeout(() => {
        setDimensions({
          width: window.innerWidth,
          height: window.innerHeight,
        });
      }, 150);
    };

    window.addEventListener('resize', handleResize);

    // Handle orientation change separately for better mobile support
    window.addEventListener('orientationchange', handleResize);

    return () => {
      clearTimeout(timeoutId);
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('orientationchange', handleResize);
    };
  }, []);

  const currentBreakpoint = useMemo((): Breakpoint => {
    const { width } = dimensions;

    if (width >= breakpoints['2xl']) return '2xl';
    if (width >= breakpoints.xl) return 'xl';
    if (width >= breakpoints.lg) return 'lg';
    if (width >= breakpoints.md) return 'md';
    if (width >= breakpoints.sm) return 'sm';
    return 'xs';
  }, [dimensions, breakpoints]);

  const greaterThan = (breakpoint: Breakpoint): boolean => {
    return dimensions.width >= breakpoints[breakpoint];
  };

  const lessThan = (breakpoint: Breakpoint): boolean => {
    return dimensions.width < breakpoints[breakpoint];
  };

  return {
    currentBreakpoint,
    width: dimensions.width,
    height: dimensions.height,
    isXs: currentBreakpoint === 'xs',
    isSm: currentBreakpoint === 'sm',
    isMd: currentBreakpoint === 'md',
    isLg: currentBreakpoint === 'lg',
    isXl: currentBreakpoint === 'xl',
    is2xl: currentBreakpoint === '2xl',
    isMobile: lessThan('md'), // < 768px
    isTablet: greaterThan('md') && lessThan('lg'), // 768px - 1024px
    isDesktop: greaterThan('lg'), // >= 1024px
    greaterThan,
    lessThan,
  };
}
