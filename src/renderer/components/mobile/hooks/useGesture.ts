/**
 * AccuScene Enterprise v0.3.0
 * useGesture Hook
 *
 * React hook for multi-touch gesture detection and handling
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import {
  GestureType,
  GestureHandlers,
  Point,
  SwipeDirection,
  TouchState,
} from '../types';

interface UseGestureConfig {
  // Tap configuration
  tapTimeout?: number; // ms
  doubleTapTimeout?: number; // ms
  longPressTimeout?: number; // ms

  // Swipe configuration
  swipeThreshold?: number; // pixels
  swipeVelocityThreshold?: number; // pixels/ms

  // Pinch configuration
  pinchThreshold?: number; // scale change threshold

  // Rotation configuration
  rotationThreshold?: number; // degrees

  // Pan/Drag configuration
  panThreshold?: number; // pixels

  // General
  preventDefaultEvents?: boolean;
}

const DEFAULT_CONFIG: Required<UseGestureConfig> = {
  tapTimeout: 300,
  doubleTapTimeout: 400,
  longPressTimeout: 500,
  swipeThreshold: 50,
  swipeVelocityThreshold: 0.3,
  pinchThreshold: 0.05,
  rotationThreshold: 5,
  panThreshold: 10,
  preventDefaultEvents: true,
};

/**
 * Hook for comprehensive gesture detection
 * Supports tap, double-tap, long-press, swipe, pinch, rotate, pan, and drag
 *
 * @param handlers - Gesture event handlers
 * @param config - Configuration options
 * @returns Ref to attach to the target element
 *
 * @example
 * ```tsx
 * const gestureRef = useGesture({
 *   onTap: (e, point) => console.log('Tapped at', point),
 *   onSwipe: (e, direction) => console.log('Swiped', direction),
 *   onPinch: (e, scale) => console.log('Pinched', scale),
 * });
 *
 * return <div ref={gestureRef}>Touch me!</div>;
 * ```
 */
export function useGesture(
  handlers: GestureHandlers = {},
  config: UseGestureConfig = {}
): React.RefObject<HTMLElement> {
  const elementRef = useRef<HTMLElement>(null);
  const configRef = useRef({ ...DEFAULT_CONFIG, ...config });

  const touchState = useRef<TouchState>({
    touching: false,
    touchCount: 0,
    touches: [],
    timestamp: 0,
  });

  const gestureState = useRef({
    startTouches: [] as Point[],
    lastTouches: [] as Point[],
    startTime: 0,
    lastTapTime: 0,
    tapCount: 0,
    longPressTimer: null as NodeJS.Timeout | null,
    initialDistance: 0,
    initialAngle: 0,
  });

  // Helper functions
  const getDistance = (p1: Point, p2: Point): number => {
    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;
    return Math.sqrt(dx * dx + dy * dy);
  };

  const getAngle = (p1: Point, p2: Point): number => {
    return Math.atan2(p2.y - p1.y, p2.x - p1.x) * (180 / Math.PI);
  };

  const getCenter = (points: Point[]): Point => {
    const sum = points.reduce(
      (acc, p) => ({ x: acc.x + p.x, y: acc.y + p.y }),
      { x: 0, y: 0 }
    );
    return {
      x: sum.x / points.length,
      y: sum.y / points.length,
    };
  };

  const getTouchPoints = (touches: TouchList): Point[] => {
    return Array.from(touches).map((touch) => ({
      x: touch.clientX,
      y: touch.clientY,
    }));
  };

  const clearLongPressTimer = () => {
    if (gestureState.current.longPressTimer) {
      clearTimeout(gestureState.current.longPressTimer);
      gestureState.current.longPressTimer = null;
    }
  };

  // Event handlers
  const handleTouchStart = useCallback(
    (event: TouchEvent) => {
      if (configRef.current.preventDefaultEvents) {
        event.preventDefault();
      }

      const touches = getTouchPoints(event.touches);
      const now = Date.now();

      touchState.current = {
        touching: true,
        touchCount: event.touches.length,
        touches,
        timestamp: now,
      };

      gestureState.current.startTouches = touches;
      gestureState.current.lastTouches = touches;
      gestureState.current.startTime = now;

      if (event.touches.length === 2) {
        const [p1, p2] = touches;
        gestureState.current.initialDistance = getDistance(p1, p2);
        gestureState.current.initialAngle = getAngle(p1, p2);
      }

      // Long press detection
      if (event.touches.length === 1 && handlers.onLongPress) {
        gestureState.current.longPressTimer = setTimeout(() => {
          handlers.onLongPress?.(event, touches[0]);
        }, configRef.current.longPressTimeout);
      }

      handlers.onGestureStart?.(event);
    },
    [handlers]
  );

  const handleTouchMove = useCallback(
    (event: TouchEvent) => {
      if (!touchState.current.touching) return;

      if (configRef.current.preventDefaultEvents) {
        event.preventDefault();
      }

      const touches = getTouchPoints(event.touches);
      touchState.current.touches = touches;

      // Clear long press if moved
      clearLongPressTimer();

      // Handle pan/drag
      if (event.touches.length === 1 && gestureState.current.startTouches.length === 1) {
        const start = gestureState.current.startTouches[0];
        const current = touches[0];
        const delta = {
          x: current.x - start.x,
          y: current.y - start.y,
        };

        if (
          Math.abs(delta.x) > configRef.current.panThreshold ||
          Math.abs(delta.y) > configRef.current.panThreshold
        ) {
          handlers.onPan?.(event, delta);
          handlers.onDrag?.(event, current);
        }
      }

      // Handle pinch
      if (event.touches.length === 2 && gestureState.current.startTouches.length === 2) {
        const [p1, p2] = touches;
        const currentDistance = getDistance(p1, p2);
        const scale = currentDistance / gestureState.current.initialDistance;
        const center = getCenter(touches);

        if (Math.abs(scale - 1) > configRef.current.pinchThreshold) {
          handlers.onPinch?.(event, scale, center);
        }

        // Handle rotation
        const currentAngle = getAngle(p1, p2);
        const rotation = currentAngle - gestureState.current.initialAngle;

        if (Math.abs(rotation) > configRef.current.rotationThreshold) {
          handlers.onRotate?.(event, rotation, center);
        }
      }

      gestureState.current.lastTouches = touches;
    },
    [handlers]
  );

  const handleTouchEnd = useCallback(
    (event: TouchEvent) => {
      if (!touchState.current.touching) return;

      const touches = getTouchPoints(event.changedTouches);
      const now = Date.now();
      const duration = now - gestureState.current.startTime;

      clearLongPressTimer();

      // Handle tap and double-tap
      if (
        duration < configRef.current.tapTimeout &&
        gestureState.current.startTouches.length === 1 &&
        touches.length === 1
      ) {
        const start = gestureState.current.startTouches[0];
        const end = touches[0];
        const distance = getDistance(start, end);

        // If minimal movement, it's a tap
        if (distance < configRef.current.panThreshold) {
          const timeSinceLastTap = now - gestureState.current.lastTapTime;

          if (
            timeSinceLastTap < configRef.current.doubleTapTimeout &&
            handlers.onDoubleTap
          ) {
            handlers.onDoubleTap(event, end);
            gestureState.current.tapCount = 0;
          } else {
            handlers.onTap?.(event, end);
            gestureState.current.lastTapTime = now;
            gestureState.current.tapCount = 1;
          }
        } else {
          // Handle swipe
          const deltaX = end.x - start.x;
          const deltaY = end.y - start.y;
          const velocity = distance / duration;

          if (
            distance > configRef.current.swipeThreshold &&
            velocity > configRef.current.swipeVelocityThreshold
          ) {
            let direction: SwipeDirection;
            if (Math.abs(deltaX) > Math.abs(deltaY)) {
              direction = deltaX > 0 ? 'right' : 'left';
            } else {
              direction = deltaY > 0 ? 'down' : 'up';
            }
            handlers.onSwipe?.(event, direction, distance);
          }
        }
      }

      touchState.current = {
        touching: false,
        touchCount: 0,
        touches: [],
        timestamp: now,
      };

      handlers.onGestureEnd?.(event);
    },
    [handlers]
  );

  const handleTouchCancel = useCallback(
    (event: TouchEvent) => {
      clearLongPressTimer();
      touchState.current = {
        touching: false,
        touchCount: 0,
        touches: [],
        timestamp: Date.now(),
      };
      handlers.onGestureEnd?.(event);
    },
    [handlers]
  );

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    element.addEventListener('touchstart', handleTouchStart, { passive: false });
    element.addEventListener('touchmove', handleTouchMove, { passive: false });
    element.addEventListener('touchend', handleTouchEnd, { passive: false });
    element.addEventListener('touchcancel', handleTouchCancel, { passive: false });

    return () => {
      clearLongPressTimer();
      element.removeEventListener('touchstart', handleTouchStart);
      element.removeEventListener('touchmove', handleTouchMove);
      element.removeEventListener('touchend', handleTouchEnd);
      element.removeEventListener('touchcancel', handleTouchCancel);
    };
  }, [handleTouchStart, handleTouchMove, handleTouchEnd, handleTouchCancel]);

  return elementRef;
}
