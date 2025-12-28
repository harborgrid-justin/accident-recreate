/**
 * useAnimation Hook
 * Manages animations and transitions with React Spring patterns
 */

import { useRef, useEffect, useState, useCallback } from 'react';
import { AnimationConfig, SpringConfig } from '../types';

export interface UseAnimationOptions {
  autoplay?: boolean;
  loop?: boolean;
  onComplete?: () => void;
  onUpdate?: (progress: number) => void;
}

export interface UseAnimationReturn {
  progress: number;
  isPlaying: boolean;
  isComplete: boolean;
  play: () => void;
  pause: () => void;
  stop: () => void;
  restart: () => void;
  seek: (progress: number) => void;
  setSpeed: (speed: number) => void;
}

export function useAnimation(
  config: AnimationConfig,
  options: UseAnimationOptions = {}
): UseAnimationReturn {
  const [progress, setProgress] = useState(0);
  const [isPlaying, setIsPlaying] = useState(options.autoplay ?? false);
  const [isComplete, setIsComplete] = useState(false);
  const [speed, setSpeedState] = useState(1);

  const startTimeRef = useRef<number | null>(null);
  const pausedTimeRef = useRef<number>(0);
  const animationFrameRef = useRef<number | null>(null);

  const easeFunction = useCallback((t: number): number => {
    switch (config.easing) {
      case 'ease':
        return t < 0.5
          ? 2 * t * t
          : -1 + (4 - 2 * t) * t;
      case 'ease-in':
        return t * t;
      case 'ease-out':
        return t * (2 - t);
      case 'ease-in-out':
        return t < 0.5
          ? 2 * t * t
          : -1 + (4 - 2 * t) * t;
      case 'linear':
      default:
        return t;
    }
  }, [config.easing]);

  const animate = useCallback((timestamp: number) => {
    if (!startTimeRef.current) {
      startTimeRef.current = timestamp - pausedTimeRef.current;
    }

    const elapsed = (timestamp - startTimeRef.current) * speed;
    const delay = config.delay || 0;
    const duration = config.duration;

    if (elapsed < delay) {
      animationFrameRef.current = requestAnimationFrame(animate);
      return;
    }

    const adjustedElapsed = elapsed - delay;
    let rawProgress = Math.min(adjustedElapsed / duration, 1);
    const easedProgress = easeFunction(rawProgress);

    setProgress(easedProgress);
    options.onUpdate?.(easedProgress);

    if (rawProgress >= 1) {
      if (config.loop) {
        if (config.yoyo) {
          // Reverse animation
          startTimeRef.current = timestamp;
          pausedTimeRef.current = 0;
        } else {
          // Restart from beginning
          startTimeRef.current = null;
          pausedTimeRef.current = 0;
        }
        animationFrameRef.current = requestAnimationFrame(animate);
      } else {
        setIsComplete(true);
        setIsPlaying(false);
        options.onComplete?.();
      }
    } else {
      animationFrameRef.current = requestAnimationFrame(animate);
    }
  }, [config, speed, easeFunction, options]);

  const play = useCallback(() => {
    if (isComplete) {
      // Restart if complete
      startTimeRef.current = null;
      pausedTimeRef.current = 0;
      setProgress(0);
      setIsComplete(false);
    }
    setIsPlaying(true);
  }, [isComplete]);

  const pause = useCallback(() => {
    setIsPlaying(false);
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
    pausedTimeRef.current = progress * config.duration;
  }, [progress, config.duration]);

  const stop = useCallback(() => {
    setIsPlaying(false);
    setProgress(0);
    setIsComplete(false);
    startTimeRef.current = null;
    pausedTimeRef.current = 0;
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
  }, []);

  const restart = useCallback(() => {
    stop();
    setTimeout(() => play(), 0);
  }, [stop, play]);

  const seek = useCallback((newProgress: number) => {
    const clampedProgress = Math.max(0, Math.min(1, newProgress));
    setProgress(clampedProgress);
    pausedTimeRef.current = clampedProgress * config.duration;
    startTimeRef.current = null;
  }, [config.duration]);

  const setSpeed = useCallback((newSpeed: number) => {
    setSpeedState(Math.max(0.1, Math.min(10, newSpeed)));
  }, []);

  // Animation loop effect
  useEffect(() => {
    if (isPlaying) {
      animationFrameRef.current = requestAnimationFrame(animate);
    }

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [isPlaying, animate]);

  return {
    progress,
    isPlaying,
    isComplete,
    play,
    pause,
    stop,
    restart,
    seek,
    setSpeed,
  };
}

/**
 * Spring animation hook using physics-based animation
 */
export function useSpring(
  from: number,
  to: number,
  config: SpringConfig = {}
): number {
  const {
    mass = 1,
    tension = 170,
    friction = 26,
    clamp = false,
    precision = 0.01,
    velocity: initialVelocity = 0,
  } = config;

  const [value, setValue] = useState(from);
  const velocityRef = useRef(initialVelocity);
  const animationFrameRef = useRef<number | null>(null);

  useEffect(() => {
    const animate = () => {
      const distance = to - value;

      // Spring physics calculation
      const acceleration = (tension * distance - friction * velocityRef.current) / mass;
      velocityRef.current += acceleration * 0.016; // Assuming 60fps
      const newValue = value + velocityRef.current * 0.016;

      // Check if spring has settled
      if (Math.abs(distance) < precision && Math.abs(velocityRef.current) < precision) {
        setValue(to);
        velocityRef.current = 0;
        return;
      }

      setValue(clamp ? Math.max(from, Math.min(to, newValue)) : newValue);
      animationFrameRef.current = requestAnimationFrame(animate);
    };

    animationFrameRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [to, from, mass, tension, friction, clamp, precision]);

  return value;
}

export default useAnimation;
