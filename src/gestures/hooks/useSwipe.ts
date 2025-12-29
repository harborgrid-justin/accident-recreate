/**
 * Swipe detection hook
 */

import { useState, useCallback, useRef } from 'react';
import {
  UseSwipeResult,
  SwipeConfig,
  SwipeDirection,
  TouchPoint,
} from '../types';
import { convertToTouchPoint, calculateDistance, calculateAngle } from '../utils/touchMath';

interface UseSwipeOptions {
  config?: Partial<SwipeConfig>;
  onSwipe?: (direction: SwipeDirection, velocity: number) => void;
  onSwipeLeft?: () => void;
  onSwipeRight?: () => void;
  onSwipeUp?: () => void;
  onSwipeDown?: () => void;
  enabled?: boolean;
}

export function useSwipe(options: UseSwipeOptions = {}): UseSwipeResult {
  const {
    config = {},
    onSwipe,
    onSwipeLeft,
    onSwipeRight,
    onSwipeUp,
    onSwipeDown,
    enabled = true,
  } = options;

  const minDistance = config.minDistance ?? 50;
  const minVelocity = config.minVelocity ?? 100;
  const maxDurationMs = config.maxDurationMs ?? 1000;

  const [isSwiping, setIsSwiping] = useState(false);
  const [direction, setDirection] = useState<SwipeDirection | null>(null);

  const startPointRef = useRef<TouchPoint | null>(null);
  const velocitySamplesRef = useRef<Array<{ x: number; y: number; timestamp: number }>>([]);

  const getSwipeDirection = useCallback((angle: number): SwipeDirection => {
    const degrees = (angle * 180) / Math.PI;
    const normalized = ((degrees + 360) % 360);

    if (normalized >= 337.5 || normalized < 22.5) return SwipeDirection.Right;
    if (normalized >= 22.5 && normalized < 67.5) return SwipeDirection.UpRight;
    if (normalized >= 67.5 && normalized < 112.5) return SwipeDirection.Up;
    if (normalized >= 112.5 && normalized < 157.5) return SwipeDirection.UpLeft;
    if (normalized >= 157.5 && normalized < 202.5) return SwipeDirection.Left;
    if (normalized >= 202.5 && normalized < 247.5) return SwipeDirection.DownLeft;
    if (normalized >= 247.5 && normalized < 292.5) return SwipeDirection.Down;
    return SwipeDirection.DownRight;
  }, []);

  const calculateVelocity = useCallback((): number => {
    if (velocitySamplesRef.current.length < 2) {
      return 0;
    }

    const samples = velocitySamplesRef.current;
    const first = samples[0];
    const last = samples[samples.length - 1];

    const dt = (last.timestamp - first.timestamp) / 1000;
    if (dt === 0) return 0;

    const dx = last.x - first.x;
    const dy = last.y - first.y;
    const distance = Math.sqrt(dx * dx + dy * dy);

    return distance / dt;
  }, []);

  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || e.touches.length !== 1) return;

      const touch = convertToTouchPoint(e.touches[0]);
      startPointRef.current = touch;
      velocitySamplesRef.current = [{ x: touch.x, y: touch.y, timestamp: touch.timestamp }];
      setIsSwiping(false);
      setDirection(null);
    },
    [enabled]
  );

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || !startPointRef.current || e.touches.length !== 1) return;

      const touch = convertToTouchPoint(e.touches[0]);
      velocitySamplesRef.current.push({ x: touch.x, y: touch.y, timestamp: touch.timestamp });

      // Limit samples
      if (velocitySamplesRef.current.length > 10) {
        velocitySamplesRef.current.shift();
      }

      const distance = calculateDistance(startPointRef.current, touch);

      if (distance >= minDistance) {
        setIsSwiping(true);
      }
    },
    [enabled, minDistance]
  );

  const handleTouchEnd = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || !startPointRef.current) return;

      const touch = e.changedTouches[0];
      const endPoint = convertToTouchPoint(touch);

      const distance = calculateDistance(startPointRef.current, endPoint);
      const duration = endPoint.timestamp - startPointRef.current.timestamp;
      const velocity = calculateVelocity();

      // Check if it's a valid swipe
      if (
        distance >= minDistance &&
        velocity >= minVelocity &&
        duration <= maxDurationMs
      ) {
        const angle = calculateAngle(startPointRef.current, endPoint);
        const swipeDirection = getSwipeDirection(angle);

        setDirection(swipeDirection);
        onSwipe?.(swipeDirection, velocity);

        // Call specific direction handlers
        switch (swipeDirection) {
          case SwipeDirection.Left:
            onSwipeLeft?.();
            break;
          case SwipeDirection.Right:
            onSwipeRight?.();
            break;
          case SwipeDirection.Up:
            onSwipeUp?.();
            break;
          case SwipeDirection.Down:
            onSwipeDown?.();
            break;
        }
      }

      // Reset
      startPointRef.current = null;
      velocitySamplesRef.current = [];
      setIsSwiping(false);
    },
    [
      enabled,
      minDistance,
      minVelocity,
      maxDurationMs,
      getSwipeDirection,
      calculateVelocity,
      onSwipe,
      onSwipeLeft,
      onSwipeRight,
      onSwipeUp,
      onSwipeDown,
    ]
  );

  return {
    handlers: {
      onTouchStart: handleTouchStart,
      onTouchMove: handleTouchMove,
      onTouchEnd: handleTouchEnd,
    },
    state: {
      isSwiping,
      direction,
    },
  };
}
