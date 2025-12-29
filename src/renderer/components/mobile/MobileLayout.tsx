/**
 * AccuScene Enterprise v0.3.0
 * Mobile Layout Component
 *
 * Responsive layout container with breakpoint support and safe area handling
 */

import React, { CSSProperties, useMemo } from 'react';
import { MobileLayoutProps } from './types';
import { useBreakpoint } from './hooks/useBreakpoint';

/**
 * Mobile-first responsive layout container
 * Handles safe areas, responsive breakpoints, and layout composition
 *
 * @example
 * ```tsx
 * <MobileLayout
 *   header={<Header />}
 *   footer={<MobileNavigation />}
 *   drawer={<MobileDrawer />}
 *   safeArea
 * >
 *   <MainContent />
 * </MobileLayout>
 * ```
 */
export const MobileLayout: React.FC<MobileLayoutProps> = ({
  children,
  header,
  footer,
  navigation,
  drawer,
  backgroundColor = '#ffffff',
  fullScreen = false,
  safeArea = true,
  className = '',
}) => {
  const { isMobile, currentBreakpoint } = useBreakpoint();

  const layoutStyles: CSSProperties = useMemo(
    () => ({
      display: 'flex',
      flexDirection: 'column',
      minHeight: fullScreen ? '100vh' : '100%',
      height: fullScreen ? '100vh' : 'auto',
      width: '100%',
      backgroundColor,
      position: 'relative',
      overflow: 'hidden',
      // Safe area insets for notched devices
      ...(safeArea && {
        paddingTop: 'env(safe-area-inset-top)',
        paddingBottom: 'env(safe-area-inset-bottom)',
        paddingLeft: 'env(safe-area-inset-left)',
        paddingRight: 'env(safe-area-inset-right)',
      }),
    }),
    [backgroundColor, fullScreen, safeArea]
  );

  const headerStyles: CSSProperties = useMemo(
    () => ({
      flexShrink: 0,
      width: '100%',
      zIndex: 100,
      position: 'sticky',
      top: 0,
    }),
    []
  );

  const mainStyles: CSSProperties = useMemo(
    () => ({
      flex: 1,
      width: '100%',
      overflow: 'auto',
      position: 'relative',
      WebkitOverflowScrolling: 'touch', // Smooth scrolling on iOS
      display: 'flex',
      flexDirection: 'column',
    }),
    []
  );

  const footerStyles: CSSProperties = useMemo(
    () => ({
      flexShrink: 0,
      width: '100%',
      zIndex: 100,
      position: 'sticky',
      bottom: 0,
    }),
    []
  );

  const contentWrapperStyles: CSSProperties = useMemo(
    () => ({
      flex: 1,
      width: '100%',
      maxWidth: isMobile ? '100%' : '1200px',
      margin: '0 auto',
      padding: isMobile ? '0.5rem' : '1rem',
      boxSizing: 'border-box',
    }),
    [isMobile]
  );

  return (
    <div
      className={`mobile-layout mobile-layout--${currentBreakpoint} ${className}`}
      style={layoutStyles}
      data-testid="mobile-layout"
    >
      {/* Drawer overlay (if provided) */}
      {drawer}

      {/* Header */}
      {header && (
        <header style={headerStyles} className="mobile-layout__header">
          {header}
        </header>
      )}

      {/* Main content area */}
      <main style={mainStyles} className="mobile-layout__main">
        <div style={contentWrapperStyles} className="mobile-layout__content">
          {children}
        </div>
      </main>

      {/* Navigation (typically bottom navigation on mobile) */}
      {navigation && (
        <nav className="mobile-layout__navigation" style={{ zIndex: 100 }}>
          {navigation}
        </nav>
      )}

      {/* Footer */}
      {footer && (
        <footer style={footerStyles} className="mobile-layout__footer">
          {footer}
        </footer>
      )}

      <style>{`
        .mobile-layout {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto,
            'Helvetica Neue', Arial, sans-serif;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
          text-size-adjust: 100%;
          -webkit-text-size-adjust: 100%;
        }

        .mobile-layout * {
          box-sizing: border-box;
        }

        .mobile-layout__main::-webkit-scrollbar {
          display: none;
        }

        .mobile-layout__main {
          -ms-overflow-style: none;
          scrollbar-width: none;
        }

        /* Prevent pull-to-refresh on Chrome mobile */
        .mobile-layout {
          overscroll-behavior-y: contain;
        }

        /* Optimize for touch */
        .mobile-layout button,
        .mobile-layout a,
        .mobile-layout [role="button"] {
          min-height: 44px;
          min-width: 44px;
          touch-action: manipulation;
        }

        /* Responsive breakpoints */
        @media (max-width: 640px) {
          .mobile-layout--xs .mobile-layout__content {
            padding: 0.5rem;
          }
        }

        @media (min-width: 640px) and (max-width: 768px) {
          .mobile-layout--sm .mobile-layout__content {
            padding: 0.75rem;
          }
        }

        @media (min-width: 768px) {
          .mobile-layout--md .mobile-layout__content,
          .mobile-layout--lg .mobile-layout__content,
          .mobile-layout--xl .mobile-layout__content,
          .mobile-layout--2xl .mobile-layout__content {
            padding: 1rem;
          }
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .mobile-layout {
            color-scheme: dark;
          }
        }

        /* Reduce motion for accessibility */
        @media (prefers-reduced-motion: reduce) {
          .mobile-layout * {
            animation-duration: 0.01ms !important;
            animation-iteration-count: 1 !important;
            transition-duration: 0.01ms !important;
          }
        }
      `}</style>
    </div>
  );
};

export default MobileLayout;
