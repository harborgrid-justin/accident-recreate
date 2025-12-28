/**
 * Main gesture detection hook
 */

import { useCallback, useRef, useState, useEffect } from 'react';
import {
  GestureEvent,
  GestureConfig,
  TouchPoint,
  UseGestureResult,
} from '../types';
import { GestureStateMachine } from '../recognizers/GestureRecognizer';
import { PinchRecognizer, RotateRecognizer } from '../recognizers/MultiTouchRecognizer';
import { convertTouchList } from '../utils/touchMath';

interface UseGestureOptions {
  config?: Partial<GestureConfig>;
  onGesture?: (event: GestureEvent) => void;
  enabled?: boolean;
}

export function useGesture(options: UseGestureOptions = {}): UseGestureResult {
  const { config = {}, onGesture, enabled = true } = options;

  const [isActive, setIsActive] = useState(false);
  const [activeGestures, setActiveGestures] = useState<string[]>([]);
  const [touchCount, setTouchCount] = useState(0);

  const stateMachineRef = useRef<GestureStateMachine | null>(null);
  const activeTouchesRef = useRef<TouchPoint[]>([]);

  // Initialize gesture state machine
  useEffect(() => {
    if (!stateMachineRef.current) {
      const machine = new GestureStateMachine(config);

      // Register recognizers
      const pinchRecognizer = new PinchRecognizer(config);
      const rotateRecognizer = new RotateRecognizer(config);

      machine.registerRecognizer('pinch', pinchRecognizer);
      machine.registerRecognizer('rotate', rotateRecognizer);

      stateMachineRef.current = machine;
    }

    return () => {
      stateMachineRef.current?.reset();
    };
  }, []);

  // Update config when it changes
  useEffect(() => {
    if (stateMachineRef.current && config) {
      stateMachineRef.current.updateConfig(config);
    }
  }, [config]);

  const processGestures = useCallback(
    (touches: TouchPoint[]) => {
      if (!enabled || !stateMachineRef.current) {
        return;
      }

      const events = stateMachineRef.current.process(touches);

      // Update active state
      const active = stateMachineRef.current.getActiveGestures();
      setActiveGestures(active);
      setIsActive(active.length > 0);

      // Dispatch events
      if (onGesture) {
        events.forEach((event) => onGesture(event));
      }
    },
    [enabled, onGesture]
  );

  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled) return;

      const touches = convertTouchList(e.touches);
      activeTouchesRef.current = touches;
      setTouchCount(touches.length);

      processGestures(touches);
    },
    [enabled, processGestures]
  );

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled) return;

      e.preventDefault(); // Prevent scrolling

      const touches = convertTouchList(e.touches);
      activeTouchesRef.current = touches;
      setTouchCount(touches.length);

      processGestures(touches);
    },
    [enabled, processGestures]
  );

  const handleTouchEnd = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled) return;

      const touches = convertTouchList(e.touches);
      activeTouchesRef.current = touches;
      setTouchCount(touches.length);

      processGestures(touches);

      // Reset if no more touches
      if (touches.length === 0) {
        setIsActive(false);
        setActiveGestures([]);
      }
    },
    [enabled, processGestures]
  );

  const handleTouchCancel = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled) return;

      activeTouchesRef.current = [];
      setTouchCount(0);
      setIsActive(false);
      setActiveGestures([]);

      stateMachineRef.current?.reset();
    },
    [enabled]
  );

  return {
    handlers: {
      onTouchStart: handleTouchStart,
      onTouchMove: handleTouchMove,
      onTouchEnd: handleTouchEnd,
      onTouchCancel: handleTouchCancel,
    },
    state: {
      isActive,
      activeGestures,
      touchCount,
    },
  };
}
