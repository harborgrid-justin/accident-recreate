/**
 * Pan/drag hook
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import { UsePanResult, PanConfig, TouchPoint } from '../types';
import { convertToTouchPoint } from '../utils/touchMath';

interface UsePanOptions {
  config?: Partial<PanConfig>;
  onPan?: (deltaX: number, deltaY: number) => void;
  onPanStart?: () => void;
  onPanEnd?: (velocityX: number, velocityY: number) => void;
  bounds?: {
    minX?: number;
    maxX?: number;
    minY?: number;
    maxY?: number;
  };
  initialPosition?: { x: number; y: number };
  enabled?: boolean;
}

export function usePan(options: UsePanOptions = {}): UsePanResult {
  const {
    config = {},
    onPan,
    onPanStart,
    onPanEnd,
    bounds = {},
    initialPosition = { x: 0, y: 0 },
    enabled = true,
  } = options;

  const minDistance = config.minDistance ?? 10;
  const enableHorizontal = config.enableHorizontal ?? true;
  const enableVertical = config.enableVertical ?? true;
  const enableMomentum = config.enableMomentum ?? true;
  const momentumDecay = config.momentumDecay ?? 0.95;

  const [position, setPosition] = useState(initialPosition);

  const isPanningRef = useRef(false);
  const startPointRef = useRef<TouchPoint | null>(null);
  const previousPointRef = useRef<TouchPoint | null>(null);
  const velocitySamplesRef = useRef<Array<{ x: number; y: number; timestamp: number }>>([]);
  const animationFrameRef = useRef<number | null>(null);
  const velocityRef = useRef({ x: 0, y: 0 });

  const applyBounds = useCallback(
    (x: number, y: number): { x: number; y: number } => {
      let boundedX = x;
      let boundedY = y;

      if (bounds.minX !== undefined) boundedX = Math.max(bounds.minX, boundedX);
      if (bounds.maxX !== undefined) boundedX = Math.min(bounds.maxX, boundedX);
      if (bounds.minY !== undefined) boundedY = Math.max(bounds.minY, boundedY);
      if (bounds.maxY !== undefined) boundedY = Math.min(bounds.maxY, boundedY);

      return { x: boundedX, y: boundedY };
    },
    [bounds]
  );

  const calculateVelocity = useCallback((): { x: number; y: number } => {
    if (velocitySamplesRef.current.length < 2) {
      return { x: 0, y: 0 };
    }

    const samples = velocitySamplesRef.current;
    const first = samples[0];
    const last = samples[samples.length - 1];

    const dt = (last.timestamp - first.timestamp) / 1000;
    if (dt === 0) return { x: 0, y: 0 };

    return {
      x: (last.x - first.x) / dt,
      y: (last.y - first.y) / dt,
    };
  }, []);

  const applyMomentum = useCallback(() => {
    if (!enableMomentum) return;

    const animate = () => {
      const speed = Math.sqrt(
        velocityRef.current.x ** 2 + velocityRef.current.y ** 2
      );

      if (speed < 0.5) {
        animationFrameRef.current = null;
        return;
      }

      velocityRef.current.x *= momentumDecay;
      velocityRef.current.y *= momentumDecay;

      const deltaX = enableHorizontal ? velocityRef.current.x / 60 : 0;
      const deltaY = enableVertical ? velocityRef.current.y / 60 : 0;

      setPosition((prev) => {
        const newPos = applyBounds(prev.x + deltaX, prev.y + deltaY);
        return newPos;
      });

      animationFrameRef.current = requestAnimationFrame(animate);
    };

    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
    }

    animationFrameRef.current = requestAnimationFrame(animate);
  }, [enableMomentum, momentumDecay, enableHorizontal, enableVertical, applyBounds]);

  useEffect(() => {
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || e.touches.length !== 1) return;

      // Cancel any ongoing momentum
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
        animationFrameRef.current = null;
      }

      const touch = convertToTouchPoint(e.touches[0]);
      startPointRef.current = touch;
      previousPointRef.current = touch;
      velocitySamplesRef.current = [{ x: touch.x, y: touch.y, timestamp: touch.timestamp }];
      isPanningRef.current = false;
    },
    [enabled]
  );

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || !startPointRef.current || e.touches.length !== 1) return;

      e.preventDefault();

      const touch = convertToTouchPoint(e.touches[0]);
      const previous = previousPointRef.current || startPointRef.current;

      let deltaX = enableHorizontal ? touch.x - previous.x : 0;
      let deltaY = enableVertical ? touch.y - previous.y : 0;

      // Check if we should start panning
      if (!isPanningRef.current) {
        const distance = Math.sqrt(
          (touch.x - startPointRef.current.x) ** 2 +
          (touch.y - startPointRef.current.y) ** 2
        );

        if (distance >= minDistance) {
          isPanningRef.current = true;
          onPanStart?.();
        }
      }

      if (isPanningRef.current) {
        setPosition((prev) => {
          const newPos = applyBounds(prev.x + deltaX, prev.y + deltaY);
          return newPos;
        });

        onPan?.(deltaX, deltaY);
      }

      previousPointRef.current = touch;
      velocitySamplesRef.current.push({ x: touch.x, y: touch.y, timestamp: touch.timestamp });

      // Limit samples
      if (velocitySamplesRef.current.length > 10) {
        velocitySamplesRef.current.shift();
      }
    },
    [enabled, enableHorizontal, enableVertical, minDistance, applyBounds, onPan, onPanStart]
  );

  const handleTouchEnd = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || !isPanningRef.current) return;

      const velocity = calculateVelocity();
      velocityRef.current = velocity;

      onPanEnd?.(velocity.x, velocity.y);

      // Apply momentum
      applyMomentum();

      // Reset
      startPointRef.current = null;
      previousPointRef.current = null;
      velocitySamplesRef.current = [];
      isPanningRef.current = false;
    },
    [enabled, calculateVelocity, onPanEnd, applyMomentum]
  );

  return {
    position,
    handlers: {
      onTouchStart: handleTouchStart,
      onTouchMove: handleTouchMove,
      onTouchEnd: handleTouchEnd,
    },
  };
}
