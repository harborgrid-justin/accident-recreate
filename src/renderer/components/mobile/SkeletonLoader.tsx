/**
 * AccuScene Enterprise v0.3.0
 * Skeleton Loader Component
 *
 * Content placeholder skeleton loaders for loading states
 */

import React, { CSSProperties } from 'react';
import { SkeletonProps, SkeletonVariant } from './types';

/**
 * Skeleton loader for content placeholders
 * Shows animated loading state while content loads
 *
 * @example
 * ```tsx
 * // Text skeleton
 * <SkeletonLoader variant="text" count={3} />
 *
 * // Avatar skeleton
 * <SkeletonLoader variant="circular" width={48} height={48} />
 *
 * // Card skeleton
 * <SkeletonLoader variant="rectangular" width="100%" height={200} />
 * ```
 */
export const SkeletonLoader: React.FC<SkeletonProps> = ({
  variant = 'text',
  width = '100%',
  height,
  animation = 'wave',
  count = 1,
  className = '',
}) => {
  const getDefaultHeight = (): number | string => {
    if (height) return height;

    switch (variant) {
      case 'text':
        return '1em';
      case 'circular':
        return width;
      case 'rectangular':
      case 'rounded':
        return 200;
      default:
        return '1em';
    }
  };

  const getBaseStyles = (): CSSProperties => {
    const baseStyles: CSSProperties = {
      backgroundColor: '#e0e0e0',
      width,
      height: getDefaultHeight(),
      display: 'block',
    };

    switch (variant) {
      case 'text':
        return {
          ...baseStyles,
          borderRadius: '4px',
          marginBottom: '0.5em',
        };

      case 'circular':
        return {
          ...baseStyles,
          borderRadius: '50%',
        };

      case 'rectangular':
        return {
          ...baseStyles,
          borderRadius: 0,
        };

      case 'rounded':
        return {
          ...baseStyles,
          borderRadius: '8px',
        };

      default:
        return baseStyles;
    }
  };

  const getAnimationStyles = (): CSSProperties => {
    switch (animation) {
      case 'pulse':
        return {
          animation: 'skeletonPulse 1.5s ease-in-out infinite',
        };

      case 'wave':
        return {
          position: 'relative',
          overflow: 'hidden',
          '::after': {
            content: '""',
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background:
              'linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.4), transparent)',
            animation: 'skeletonWave 1.5s infinite',
          },
        };

      case 'none':
      default:
        return {};
    }
  };

  const skeletonStyles: CSSProperties = {
    ...getBaseStyles(),
    ...getAnimationStyles(),
  };

  const containerStyles: CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: variant === 'text' ? 0 : '0.5rem',
  };

  const skeletons = Array.from({ length: count }, (_, index) => (
    <div
      key={index}
      className={`skeleton-loader skeleton-loader--${variant} skeleton-loader--${animation} ${className}`}
      style={skeletonStyles}
      aria-busy="true"
      aria-label="Loading"
      data-testid="skeleton-loader"
    />
  ));

  if (count === 1) {
    return skeletons[0];
  }

  return (
    <div className="skeleton-loader-container" style={containerStyles}>
      {skeletons}

      <style>{`
        @keyframes skeletonPulse {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.5;
          }
        }

        @keyframes skeletonWave {
          0% {
            transform: translateX(-100%);
          }
          100% {
            transform: translateX(100%);
          }
        }

        .skeleton-loader--wave {
          position: relative;
          overflow: hidden;
        }

        .skeleton-loader--wave::after {
          content: '';
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: linear-gradient(
            90deg,
            transparent,
            rgba(255, 255, 255, 0.4),
            transparent
          );
          animation: skeletonWave 1.5s infinite;
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .skeleton-loader {
            background-color: #2c2c2e;
          }

          .skeleton-loader--wave::after {
            background: linear-gradient(
              90deg,
              transparent,
              rgba(255, 255, 255, 0.1),
              transparent
            );
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .skeleton-loader {
            animation: none !important;
          }

          .skeleton-loader--wave::after {
            animation: none !important;
          }
        }
      `}</style>
    </div>
  );
};

/**
 * Predefined skeleton layouts for common use cases
 */
export const SkeletonLayouts = {
  /**
   * Avatar + text skeleton (for list items)
   */
  ListItem: () => (
    <div style={{ display: 'flex', gap: '1rem', padding: '1rem' }}>
      <SkeletonLoader variant="circular" width={48} height={48} />
      <div style={{ flex: 1 }}>
        <SkeletonLoader variant="text" width="60%" />
        <SkeletonLoader variant="text" width="40%" />
      </div>
    </div>
  ),

  /**
   * Card skeleton with image and text
   */
  Card: () => (
    <div>
      <SkeletonLoader variant="rectangular" width="100%" height={200} />
      <div style={{ padding: '1rem' }}>
        <SkeletonLoader variant="text" width="80%" />
        <SkeletonLoader variant="text" width="60%" />
        <SkeletonLoader variant="text" width="90%" />
      </div>
    </div>
  ),

  /**
   * Article skeleton with title and paragraphs
   */
  Article: () => (
    <div style={{ padding: '1rem' }}>
      <SkeletonLoader variant="text" width="90%" height="2em" />
      <SkeletonLoader variant="text" width="60%" />
      <div style={{ marginTop: '1rem' }}>
        <SkeletonLoader variant="text" count={4} />
      </div>
    </div>
  ),

  /**
   * Profile skeleton with avatar and details
   */
  Profile: () => (
    <div style={{ padding: '1rem', textAlign: 'center' }}>
      <div style={{ display: 'flex', justifyContent: 'center', marginBottom: '1rem' }}>
        <SkeletonLoader variant="circular" width={80} height={80} />
      </div>
      <SkeletonLoader variant="text" width="40%" />
      <SkeletonLoader variant="text" width="60%" />
    </div>
  ),

  /**
   * Table row skeleton
   */
  TableRow: () => (
    <div style={{ display: 'flex', gap: '1rem', padding: '1rem' }}>
      <SkeletonLoader variant="text" width="20%" />
      <SkeletonLoader variant="text" width="30%" />
      <SkeletonLoader variant="text" width="25%" />
      <SkeletonLoader variant="text" width="25%" />
    </div>
  ),
};

export default SkeletonLoader;
