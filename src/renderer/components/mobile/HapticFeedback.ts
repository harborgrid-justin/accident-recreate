/**
 * AccuScene Enterprise v0.3.0
 * Haptic Feedback Utilities
 *
 * Comprehensive haptic feedback system for mobile interfaces
 */

import { HapticPattern, HapticOptions } from './types';

/**
 * Haptic feedback manager class
 * Provides static methods for triggering haptic feedback across the application
 */
export class HapticFeedback {
  private static supported: boolean =
    typeof navigator !== 'undefined' && 'vibrate' in navigator;

  private static patterns: Record<HapticPattern, number | number[]> = {
    light: 10,
    medium: 20,
    heavy: 40,
    success: [10, 50, 10],
    warning: [20, 100, 20, 100, 20],
    error: [50, 100, 50, 100, 50],
    selection: 5,
  };

  /**
   * Check if haptic feedback is supported
   */
  static isSupported(): boolean {
    return this.supported;
  }

  /**
   * Trigger haptic feedback with a predefined pattern
   */
  static trigger(pattern: HapticPattern, options?: HapticOptions): void {
    if (!this.supported) {
      return;
    }

    try {
      const vibrationPattern = options?.pattern
        ? this.patterns[options.pattern]
        : this.patterns[pattern];

      // Apply intensity multiplier if provided
      let adjustedPattern = vibrationPattern;
      if (options?.intensity && options.intensity !== 1) {
        if (Array.isArray(vibrationPattern)) {
          adjustedPattern = vibrationPattern.map((val) =>
            Math.round(val * options.intensity!)
          );
        } else {
          adjustedPattern = Math.round(vibrationPattern * options.intensity);
        }
      }

      // Apply duration if provided
      if (options?.duration) {
        adjustedPattern = options.duration;
      }

      navigator.vibrate(adjustedPattern);
    } catch (error) {
      console.error('Haptic feedback failed:', error);
    }
  }

  /**
   * Light haptic feedback (10ms)
   * Use for subtle interactions like hovering or selecting
   */
  static light(): void {
    this.trigger('light');
  }

  /**
   * Medium haptic feedback (20ms)
   * Use for standard button presses
   */
  static medium(): void {
    this.trigger('medium');
  }

  /**
   * Heavy haptic feedback (40ms)
   * Use for important actions or confirmations
   */
  static heavy(): void {
    this.trigger('heavy');
  }

  /**
   * Success pattern (double pulse)
   * Use for successful operations
   */
  static success(): void {
    this.trigger('success');
  }

  /**
   * Warning pattern (triple pulse)
   * Use for warnings or important notifications
   */
  static warning(): void {
    this.trigger('warning');
  }

  /**
   * Error pattern (strong triple pulse)
   * Use for errors or failed operations
   */
  static error(): void {
    this.trigger('error');
  }

  /**
   * Selection feedback (very light, 5ms)
   * Use for list item selection or scrolling
   */
  static selection(): void {
    this.trigger('selection');
  }

  /**
   * Custom vibration pattern
   */
  static custom(pattern: number | number[]): void {
    if (!this.supported) {
      return;
    }

    try {
      navigator.vibrate(pattern);
    } catch (error) {
      console.error('Custom haptic feedback failed:', error);
    }
  }

  /**
   * Cancel any ongoing vibration
   */
  static cancel(): void {
    if (!this.supported) {
      return;
    }

    try {
      navigator.vibrate(0);
    } catch (error) {
      console.error('Failed to cancel vibration:', error);
    }
  }

  /**
   * Impact feedback for collision or boundary events
   */
  static impact(intensity: 'light' | 'medium' | 'heavy' = 'medium'): void {
    const patterns = {
      light: 15,
      medium: 25,
      heavy: 45,
    };
    this.custom(patterns[intensity]);
  }

  /**
   * Notification feedback
   */
  static notification(type: 'success' | 'warning' | 'error' = 'success'): void {
    switch (type) {
      case 'success':
        this.success();
        break;
      case 'warning':
        this.warning();
        break;
      case 'error':
        this.error();
        break;
    }
  }

  /**
   * Continuous feedback for dragging
   */
  static startContinuous(interval: number = 50): NodeJS.Timer {
    if (!this.supported) {
      return setInterval(() => {}, interval);
    }

    return setInterval(() => {
      this.selection();
    }, interval);
  }

  /**
   * Stop continuous feedback
   */
  static stopContinuous(timerId: NodeJS.Timer): void {
    clearInterval(timerId);
    this.cancel();
  }

  /**
   * Haptic feedback for scrolling snap points
   */
  static snap(): void {
    this.custom(8);
  }

  /**
   * Haptic feedback for reaching boundaries
   */
  static boundary(): void {
    this.custom([30, 20, 30]);
  }

  /**
   * Haptic feedback for toggle switches
   */
  static toggle(state: boolean): void {
    this.custom(state ? [10, 30, 10] : [10, 50, 20]);
  }

  /**
   * Haptic feedback for slider adjustments
   */
  static sliderChange(): void {
    this.selection();
  }

  /**
   * Haptic feedback for keyboard input
   */
  static keyPress(): void {
    this.custom(3);
  }

  /**
   * Haptic feedback for destructive actions
   */
  static destructive(): void {
    this.custom([40, 30, 40, 30, 60]);
  }

  /**
   * Register a custom pattern
   */
  static registerPattern(name: string, pattern: number | number[]): void {
    (this.patterns as any)[name] = pattern;
  }

  /**
   * Get all available patterns
   */
  static getPatterns(): Record<string, number | number[]> {
    return { ...this.patterns };
  }
}

/**
 * Decorator for adding haptic feedback to class methods
 */
export function withHaptic(pattern: HapticPattern = 'medium') {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    const originalMethod = descriptor.value;

    descriptor.value = function (...args: any[]) {
      HapticFeedback.trigger(pattern);
      return originalMethod.apply(this, args);
    };

    return descriptor;
  };
}

/**
 * Higher-order component for adding haptic feedback to button clicks
 */
export function withHapticClick<P extends object>(
  Component: React.ComponentType<P>,
  pattern: HapticPattern = 'medium'
): React.ComponentType<P> {
  return function HapticWrapper(props: P) {
    const handleClick = (event: React.MouseEvent | React.TouchEvent) => {
      HapticFeedback.trigger(pattern);
      if ((props as any).onClick) {
        (props as any).onClick(event);
      }
    };

    return <Component {...props} onClick={handleClick} />;
  };
}

export default HapticFeedback;
