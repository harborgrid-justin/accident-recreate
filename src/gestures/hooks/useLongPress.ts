/**
 * Long press detection hook
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import { UseLongPressResult, LongPressConfig, TouchPoint } from '../types';
import { convertToTouchPoint, calculateDistance } from '../utils/touchMath';

interface UseLongPressOptions {
  config?: Partial<LongPressConfig>;
  onLongPress?: () => void;
  onLongPressStart?: () => void;
  onLongPressEnd?: (duration: number) => void;
  enabled?: boolean;
}

export function useLongPress(options: UseLongPressOptions = {}): UseLongPressResult {
  const {
    config = {},
    onLongPress,
    onLongPressStart,
    onLongPressEnd,
    enabled = true,
  } = options;

  const minDurationMs = config.minDurationMs ?? 500;
  const maxMovement = config.maxMovement ?? 10;

  const [isPressed, setIsPressed] = useState(false);
  const [duration, setDuration] = useState(0);

  const startPointRef = useRef<TouchPoint | null>(null);
  const startTimeRef = useRef<number | null>(null);
  const timerRef = useRef<NodeJS.Timeout | null>(null);
  const updateIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const hasTriggeredRef = useRef(false);

  const clearTimers = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    if (updateIntervalRef.current) {
      clearInterval(updateIntervalRef.current);
      updateIntervalRef.current = null;
    }
  }, []);

  useEffect(() => {
    return () => {
      clearTimers();
    };
  }, [clearTimers]);

  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || e.touches.length !== 1) return;

      const touch = convertToTouchPoint(e.touches[0]);
      startPointRef.current = touch;
      startTimeRef.current = Date.now();
      hasTriggeredRef.current = false;
      setIsPressed(true);
      setDuration(0);

      // Clear any existing timers
      clearTimers();

      // Set timer for long press
      timerRef.current = setTimeout(() => {
        hasTriggeredRef.current = true;
        onLongPressStart?.();
        onLongPress?.();

        // Start updating duration
        updateIntervalRef.current = setInterval(() => {
          if (startTimeRef.current) {
            const elapsed = Date.now() - startTimeRef.current;
            setDuration(elapsed);
          }
        }, 50);
      }, minDurationMs);
    },
    [enabled, minDurationMs, clearTimers, onLongPressStart, onLongPress]
  );

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || !startPointRef.current || e.touches.length !== 1) return;

      const touch = convertToTouchPoint(e.touches[0]);
      const distance = calculateDistance(startPointRef.current, touch);

      // Cancel if moved too much
      if (distance > maxMovement) {
        clearTimers();
        setIsPressed(false);
        setDuration(0);
        startPointRef.current = null;
        startTimeRef.current = null;
        hasTriggeredRef.current = false;
      }
    },
    [enabled, maxMovement, clearTimers]
  );

  const handleTouchEnd = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled) return;

      const finalDuration = startTimeRef.current
        ? Date.now() - startTimeRef.current
        : 0;

      if (hasTriggeredRef.current) {
        onLongPressEnd?.(finalDuration);
      }

      // Reset
      clearTimers();
      setIsPressed(false);
      setDuration(0);
      startPointRef.current = null;
      startTimeRef.current = null;
      hasTriggeredRef.current = false;
    },
    [enabled, clearTimers, onLongPressEnd]
  );

  return {
    handlers: {
      onTouchStart: handleTouchStart,
      onTouchMove: handleTouchMove,
      onTouchEnd: handleTouchEnd,
    },
    state: {
      isPressed,
      duration,
    },
  };
}
