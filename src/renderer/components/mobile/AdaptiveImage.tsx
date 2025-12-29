/**
 * AccuScene Enterprise v0.3.0
 * Adaptive Image Component
 *
 * Responsive image with srcset and lazy loading
 */

import React, { useState, useEffect, CSSProperties } from 'react';
import { ResponsiveImageProps, ImageSource } from './types';

/**
 * Adaptive image component with responsive sources
 * Automatically selects appropriate image size based on viewport
 *
 * @example
 * ```tsx
 * <AdaptiveImage
 *   src="image.jpg"
 *   alt="Evidence photo"
 *   sources={[
 *     { src: 'image-400.jpg', width: 400, height: 300 },
 *     { src: 'image-800.jpg', width: 800, height: 600 },
 *     { src: 'image-1200.jpg', width: 1200, height: 900 },
 *   ]}
 *   loading="lazy"
 *   objectFit="cover"
 * />
 * ```
 */
export const AdaptiveImage: React.FC<ResponsiveImageProps> = ({
  src,
  alt,
  sources = [],
  loading = 'lazy',
  objectFit = 'cover',
  placeholder,
  onLoad,
  onError,
  className = '',
}) => {
  const [isLoaded, setIsLoaded] = useState(false);
  const [hasError, setHasError] = useState(false);
  const [currentSrc, setCurrentSrc] = useState(placeholder || src);

  useEffect(() => {
    // Preload image
    const img = new Image();
    img.src = src;

    img.onload = () => {
      setCurrentSrc(src);
      setIsLoaded(true);
      onLoad?.();
    };

    img.onerror = () => {
      setHasError(true);
      onError?.();
    };
  }, [src, onLoad, onError]);

  const generateSrcSet = (): string => {
    if (sources.length === 0) return '';

    return sources
      .map((source) => `${source.src} ${source.width}w`)
      .join(', ');
  };

  const generateSizes = (): string => {
    if (sources.length === 0) return '';

    // Default responsive sizes
    return `
      (max-width: 640px) 100vw,
      (max-width: 1024px) 50vw,
      33vw
    `.trim();
  };

  const containerStyles: CSSProperties = {
    position: 'relative',
    width: '100%',
    height: '100%',
    overflow: 'hidden',
    backgroundColor: '#f0f0f0',
  };

  const imageStyles: CSSProperties = {
    width: '100%',
    height: '100%',
    objectFit,
    opacity: isLoaded ? 1 : 0,
    transition: 'opacity 0.3s ease',
  };

  const placeholderStyles: CSSProperties = {
    position: 'absolute',
    top: 0,
    left: 0,
    width: '100%',
    height: '100%',
    objectFit: 'cover',
    filter: 'blur(10px)',
    transform: 'scale(1.1)',
    opacity: isLoaded ? 0 : 1,
    transition: 'opacity 0.3s ease',
  };

  const errorStyles: CSSProperties = {
    position: 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    color: '#8E8E93',
    fontSize: '0.875rem',
    textAlign: 'center',
  };

  const skeletonStyles: CSSProperties = {
    position: 'absolute',
    top: 0,
    left: 0,
    width: '100%',
    height: '100%',
    background: 'linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%)',
    backgroundSize: '200% 100%',
    animation: 'shimmer 1.5s infinite',
  };

  if (hasError) {
    return (
      <div
        className={`adaptive-image adaptive-image--error ${className}`}
        style={containerStyles}
        data-testid="adaptive-image"
      >
        <div style={errorStyles}>
          <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>📷</div>
          <div>Failed to load image</div>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`adaptive-image ${isLoaded ? 'adaptive-image--loaded' : ''} ${className}`}
      style={containerStyles}
      data-testid="adaptive-image"
    >
      {/* Loading skeleton */}
      {!isLoaded && !placeholder && (
        <div className="adaptive-image__skeleton" style={skeletonStyles} />
      )}

      {/* Placeholder blur */}
      {placeholder && !isLoaded && (
        <img
          src={placeholder}
          alt=""
          style={placeholderStyles}
          aria-hidden="true"
        />
      )}

      {/* Main image */}
      <img
        src={currentSrc}
        alt={alt}
        srcSet={generateSrcSet()}
        sizes={generateSizes()}
        loading={loading}
        style={imageStyles}
        className="adaptive-image__img"
      />

      <style>{`
        @keyframes shimmer {
          0% {
            background-position: -200% 0;
          }
          100% {
            background-position: 200% 0;
          }
        }

        .adaptive-image {
          -webkit-user-select: none;
          user-select: none;
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .adaptive-image {
            background-color: #2c2c2e;
          }

          .adaptive-image__skeleton {
            background: linear-gradient(90deg, #2c2c2e 25%, #38383a 50%, #2c2c2e 75%);
            background-size: 200% 100%;
          }

          .adaptive-image--error {
            color: #98989d;
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .adaptive-image__img,
          .adaptive-image__skeleton {
            animation: none !important;
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default AdaptiveImage;
