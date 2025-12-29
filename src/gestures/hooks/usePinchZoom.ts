/**
 * Pinch to zoom hook
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import { UsePinchZoomResult, PinchConfig, TouchPoint } from '../types';
import { PinchRecognizer } from '../recognizers/MultiTouchRecognizer';
import { convertTouchList } from '../utils/touchMath';

interface UsePinchZoomOptions {
  config?: Partial<PinchConfig>;
  onPinch?: (scale: number) => void;
  onPinchStart?: () => void;
  onPinchEnd?: (finalScale: number) => void;
  minScale?: number;
  maxScale?: number;
  initialScale?: number;
  enabled?: boolean;
}

export function usePinchZoom(options: UsePinchZoomOptions = {}): UsePinchZoomResult {
  const {
    config = {},
    onPinch,
    onPinchStart,
    onPinchEnd,
    minScale = 0.5,
    maxScale = 3,
    initialScale = 1,
    enabled = true,
  } = options;

  const [scale, setScale] = useState(initialScale);
  const recognizerRef = useRef<PinchRecognizer | null>(null);
  const baseScaleRef = useRef(initialScale);

  useEffect(() => {
    if (!recognizerRef.current) {
      recognizerRef.current = new PinchRecognizer({
        pinch: {
          minScaleDelta: 0.01,
          maxScale,
          minScale,
          sensitivity: 1,
          priority: 2,
          allowSimultaneousRotation: false,
          ...config,
        },
      } as any);
    }
  }, []);

  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || !recognizerRef.current) return;

      const touches = convertTouchList(e.touches);

      if (touches.length === 2) {
        baseScaleRef.current = scale;
        onPinchStart?.();
      }

      recognizerRef.current.process(touches);
    },
    [enabled, scale, onPinchStart]
  );

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || !recognizerRef.current) return;

      e.preventDefault();

      const touches = convertTouchList(e.touches);
      const event = recognizerRef.current.process(touches);

      if (event && event.type === 'pinchMove') {
        const newScale = baseScaleRef.current * event.scale;
        const clampedScale = Math.max(minScale, Math.min(maxScale, newScale));

        setScale(clampedScale);
        onPinch?.(clampedScale);
      }
    },
    [enabled, minScale, maxScale, onPinch]
  );

  const handleTouchEnd = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || !recognizerRef.current) return;

      const touches = convertTouchList(e.touches);
      const event = recognizerRef.current.process(touches);

      if (event && event.type === 'pinchEnd') {
        onPinchEnd?.(scale);
      }
    },
    [enabled, scale, onPinchEnd]
  );

  return {
    scale,
    handlers: {
      onTouchStart: handleTouchStart,
      onTouchMove: handleTouchMove,
      onTouchEnd: handleTouchEnd,
    },
  };
}
