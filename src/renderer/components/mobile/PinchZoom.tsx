/**
 * AccuScene Enterprise v0.3.0
 * Pinch Zoom Component
 *
 * Pinch-to-zoom container with smooth animations
 */

import React, { useState, useRef, useCallback, CSSProperties, ReactNode } from 'react';
import { Point } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface PinchZoomProps {
  children: ReactNode;
  minZoom?: number;
  maxZoom?: number;
  initialZoom?: number;
  enablePan?: boolean;
  enableDoubleTapZoom?: boolean;
  doubleTapZoomFactor?: number;
  onZoomChange?: (zoom: number) => void;
  className?: string;
}

/**
 * Container that enables pinch-to-zoom and pan gestures
 * Optimized for smooth 60 FPS performance
 *
 * @example
 * ```tsx
 * <PinchZoom
 *   minZoom={0.5}
 *   maxZoom={4}
 *   enableDoubleTapZoom
 *   onZoomChange={(zoom) => console.log('Zoom:', zoom)}
 * >
 *   <img src="large-image.jpg" alt="Zoomable content" />
 * </PinchZoom>
 * ```
 */
export const PinchZoom: React.FC<PinchZoomProps> = ({
  children,
  minZoom = 0.5,
  maxZoom = 4,
  initialZoom = 1,
  enablePan = true,
  enableDoubleTapZoom = true,
  doubleTapZoomFactor = 2,
  onZoomChange,
  className = '',
}) => {
  const [zoom, setZoom] = useState(initialZoom);
  const [pan, setPan] = useState<Point>({ x: 0, y: 0 });
  const [isTransitioning, setIsTransitioning] = useState(false);

  const containerRef = useRef<HTMLDivElement>(null);
  const touchStartRef = useRef<{
    distance: number;
    center: Point;
    pan: Point;
    zoom: number;
    timestamp: number;
  } | null>(null);
  const lastTapRef = useRef<number>(0);

  const getDistance = (touch1: Touch, touch2: Touch): number => {
    const dx = touch2.clientX - touch1.clientX;
    const dy = touch2.clientY - touch1.clientY;
    return Math.sqrt(dx * dx + dy * dy);
  };

  const getCenter = (touch1: Touch, touch2: Touch): Point => {
    return {
      x: (touch1.clientX + touch2.clientX) / 2,
      y: (touch1.clientY + touch2.clientY) / 2,
    };
  };

  const constrainZoom = (value: number): number => {
    return Math.max(minZoom, Math.min(maxZoom, value));
  };

  const constrainPan = (panValue: Point, zoomValue: number): Point => {
    if (!containerRef.current) return panValue;

    const rect = containerRef.current.getBoundingClientRect();
    const maxPanX = (rect.width * (zoomValue - 1)) / 2;
    const maxPanY = (rect.height * (zoomValue - 1)) / 2;

    return {
      x: Math.max(-maxPanX, Math.min(maxPanX, panValue.x)),
      y: Math.max(-maxPanY, Math.min(maxPanY, panValue.y)),
    };
  };

  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      const touches = e.touches;

      if (touches.length === 2) {
        // Start pinch zoom
        const distance = getDistance(touches[0], touches[1]);
        const center = getCenter(touches[0], touches[1]);

        touchStartRef.current = {
          distance,
          center,
          pan,
          zoom,
          timestamp: Date.now(),
        };

        setIsTransitioning(false);
        HapticFeedback.light();
      } else if (touches.length === 1) {
        // Store for potential double-tap or pan
        const now = Date.now();
        const timeSinceLastTap = now - lastTapRef.current;

        if (enableDoubleTapZoom && timeSinceLastTap < 300) {
          // Double tap detected
          const newZoom = zoom === minZoom ? doubleTapZoomFactor : minZoom;
          setZoom(constrainZoom(newZoom));
          setPan({ x: 0, y: 0 });
          setIsTransitioning(true);
          onZoomChange?.(newZoom);
          HapticFeedback.medium();
          lastTapRef.current = 0;
        } else {
          lastTapRef.current = now;

          if (enablePan && zoom > 1) {
            touchStartRef.current = {
              distance: 0,
              center: { x: touches[0].clientX, y: touches[0].clientY },
              pan,
              zoom,
              timestamp: now,
            };
          }
        }
      }
    },
    [zoom, pan, enableDoubleTapZoom, enablePan, doubleTapZoomFactor, minZoom, onZoomChange]
  );

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!touchStartRef.current) return;

      const touches = e.touches;

      if (touches.length === 2) {
        // Pinch zoom
        e.preventDefault();

        const currentDistance = getDistance(touches[0], touches[1]);
        const currentCenter = getCenter(touches[0], touches[1]);

        const scale = currentDistance / touchStartRef.current.distance;
        const newZoom = constrainZoom(touchStartRef.current.zoom * scale);

        // Calculate pan to keep zoom centered
        const deltaX = currentCenter.x - touchStartRef.current.center.x;
        const deltaY = currentCenter.y - touchStartRef.current.center.y;

        const newPan = constrainPan(
          {
            x: touchStartRef.current.pan.x + deltaX,
            y: touchStartRef.current.pan.y + deltaY,
          },
          newZoom
        );

        setZoom(newZoom);
        setPan(newPan);
        onZoomChange?.(newZoom);

        // Haptic feedback on zoom milestones
        if (Math.abs(newZoom - touchStartRef.current.zoom) > 0.5) {
          HapticFeedback.selection();
          touchStartRef.current.zoom = newZoom;
        }
      } else if (touches.length === 1 && enablePan && zoom > 1) {
        // Pan gesture
        e.preventDefault();

        const deltaX = touches[0].clientX - touchStartRef.current.center.x;
        const deltaY = touches[0].clientY - touchStartRef.current.center.y;

        const newPan = constrainPan(
          {
            x: touchStartRef.current.pan.x + deltaX,
            y: touchStartRef.current.pan.y + deltaY,
          },
          zoom
        );

        setPan(newPan);
      }
    },
    [zoom, enablePan, onZoomChange]
  );

  const handleTouchEnd = useCallback(() => {
    if (touchStartRef.current) {
      HapticFeedback.light();
      touchStartRef.current = null;
    }
  }, []);

  const containerStyles: CSSProperties = {
    position: 'relative',
    width: '100%',
    height: '100%',
    overflow: 'hidden',
    touchAction: 'none',
    userSelect: 'none',
    WebkitUserSelect: 'none',
    WebkitTouchCallout: 'none',
  };

  const contentStyles: CSSProperties = {
    transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
    transformOrigin: 'center',
    transition: isTransitioning ? 'transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)' : 'none',
    willChange: 'transform',
    width: '100%',
    height: '100%',
  };

  return (
    <div
      ref={containerRef}
      className={`pinch-zoom ${className}`}
      style={containerStyles}
      onTouchStart={handleTouchStart}
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
      onTouchCancel={handleTouchEnd}
      data-testid="pinch-zoom"
      aria-label="Pinch to zoom content"
    >
      <div className="pinch-zoom__content" style={contentStyles}>
        {children}
      </div>

      <style>{`
        .pinch-zoom {
          -webkit-overflow-scrolling: touch;
        }

        .pinch-zoom__content {
          backface-visibility: hidden;
          -webkit-backface-visibility: hidden;
        }

        /* Optimize for 60 FPS */
        .pinch-zoom__content * {
          transform: translateZ(0);
          -webkit-transform: translateZ(0);
        }

        /* Reduce motion for accessibility */
        @media (prefers-reduced-motion: reduce) {
          .pinch-zoom__content {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default PinchZoom;
