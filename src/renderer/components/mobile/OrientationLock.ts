/**
 * AccuScene Enterprise v0.3.0
 * Orientation Lock Utilities
 *
 * Screen orientation management for mobile devices
 */

import { Orientation } from './types';

export type OrientationLockType =
  | 'any'
  | 'natural'
  | 'landscape'
  | 'portrait'
  | 'portrait-primary'
  | 'portrait-secondary'
  | 'landscape-primary'
  | 'landscape-secondary';

/**
 * Orientation Lock Manager
 * Provides utilities for locking and managing screen orientation
 */
export class OrientationLock {
  private static currentLock: OrientationLockType | null = null;
  private static listeners: Set<(orientation: Orientation) => void> = new Set();

  /**
   * Check if Screen Orientation API is supported
   */
  static isSupported(): boolean {
    return (
      typeof window !== 'undefined' &&
      window.screen?.orientation?.lock !== undefined
    );
  }

  /**
   * Get current orientation
   */
  static getCurrentOrientation(): Orientation {
    if (typeof window === 'undefined') {
      return 'portrait';
    }

    if (window.screen?.orientation) {
      return window.screen.orientation.type.startsWith('portrait')
        ? 'portrait'
        : 'landscape';
    }

    if (window.matchMedia) {
      return window.matchMedia('(orientation: portrait)').matches
        ? 'portrait'
        : 'landscape';
    }

    return window.innerHeight > window.innerWidth ? 'portrait' : 'landscape';
  }

  /**
   * Get current orientation angle
   */
  static getCurrentAngle(): number {
    if (typeof window === 'undefined') {
      return 0;
    }

    if (window.screen?.orientation) {
      return window.screen.orientation.angle;
    }

    // Fallback to deprecated window.orientation
    if ('orientation' in window) {
      return (window as any).orientation || 0;
    }

    return 0;
  }

  /**
   * Lock screen to specific orientation
   */
  static async lock(orientation: OrientationLockType): Promise<void> {
    if (!this.isSupported()) {
      console.warn('Screen Orientation API not supported');
      throw new Error('Screen Orientation API not supported');
    }

    try {
      await window.screen.orientation.lock(orientation);
      this.currentLock = orientation;
      console.log(`Orientation locked to: ${orientation}`);
    } catch (error) {
      console.error('Failed to lock orientation:', error);
      throw error;
    }
  }

  /**
   * Unlock screen orientation
   */
  static unlock(): void {
    if (!this.isSupported()) {
      console.warn('Screen Orientation API not supported');
      return;
    }

    try {
      window.screen.orientation.unlock();
      this.currentLock = null;
      console.log('Orientation unlocked');
    } catch (error) {
      console.error('Failed to unlock orientation:', error);
      throw error;
    }
  }

  /**
   * Lock to portrait mode
   */
  static async lockPortrait(): Promise<void> {
    return this.lock('portrait');
  }

  /**
   * Lock to landscape mode
   */
  static async lockLandscape(): Promise<void> {
    return this.lock('landscape');
  }

  /**
   * Lock to natural orientation
   */
  static async lockNatural(): Promise<void> {
    return this.lock('natural');
  }

  /**
   * Lock to primary portrait (normal portrait)
   */
  static async lockPortraitPrimary(): Promise<void> {
    return this.lock('portrait-primary');
  }

  /**
   * Lock to secondary portrait (upside down)
   */
  static async lockPortraitSecondary(): Promise<void> {
    return this.lock('portrait-secondary');
  }

  /**
   * Lock to primary landscape (normal landscape)
   */
  static async lockLandscapePrimary(): Promise<void> {
    return this.lock('landscape-primary');
  }

  /**
   * Lock to secondary landscape (reverse landscape)
   */
  static async lockLandscapeSecondary(): Promise<void> {
    return this.lock('landscape-secondary');
  }

  /**
   * Get current lock state
   */
  static getCurrentLock(): OrientationLockType | null {
    return this.currentLock;
  }

  /**
   * Check if orientation is currently locked
   */
  static isLocked(): boolean {
    return this.currentLock !== null;
  }

  /**
   * Add orientation change listener
   */
  static addChangeListener(callback: (orientation: Orientation) => void): void {
    this.listeners.add(callback);

    // Set up event listener if this is the first listener
    if (this.listeners.size === 1) {
      this.setupEventListeners();
    }
  }

  /**
   * Remove orientation change listener
   */
  static removeChangeListener(callback: (orientation: Orientation) => void): void {
    this.listeners.delete(callback);

    // Clean up event listener if no more listeners
    if (this.listeners.size === 0) {
      this.cleanupEventListeners();
    }
  }

  /**
   * Remove all listeners
   */
  static removeAllListeners(): void {
    this.listeners.clear();
    this.cleanupEventListeners();
  }

  /**
   * Internal method to setup event listeners
   */
  private static setupEventListeners(): void {
    if (typeof window === 'undefined') return;

    if (window.screen?.orientation) {
      window.screen.orientation.addEventListener('change', this.handleOrientationChange);
    }

    window.addEventListener('orientationchange', this.handleOrientationChange);
    window.addEventListener('resize', this.handleOrientationChange);
  }

  /**
   * Internal method to cleanup event listeners
   */
  private static cleanupEventListeners(): void {
    if (typeof window === 'undefined') return;

    if (window.screen?.orientation) {
      window.screen.orientation.removeEventListener(
        'change',
        this.handleOrientationChange
      );
    }

    window.removeEventListener('orientationchange', this.handleOrientationChange);
    window.removeEventListener('resize', this.handleOrientationChange);
  }

  /**
   * Internal orientation change handler
   */
  private static handleOrientationChange = (): void => {
    const currentOrientation = this.getCurrentOrientation();
    this.listeners.forEach((callback) => callback(currentOrientation));
  };

  /**
   * Request fullscreen and lock orientation
   * Useful for immersive experiences like video playback or games
   */
  static async requestFullscreenWithOrientation(
    element: HTMLElement,
    orientation: OrientationLockType
  ): Promise<void> {
    try {
      // Request fullscreen first
      if (element.requestFullscreen) {
        await element.requestFullscreen();
      } else if ((element as any).webkitRequestFullscreen) {
        await (element as any).webkitRequestFullscreen();
      } else if ((element as any).mozRequestFullScreen) {
        await (element as any).mozRequestFullScreen();
      } else if ((element as any).msRequestFullscreen) {
        await (element as any).msRequestFullscreen();
      }

      // Then lock orientation
      await this.lock(orientation);
    } catch (error) {
      console.error('Failed to enter fullscreen with orientation lock:', error);
      throw error;
    }
  }

  /**
   * Exit fullscreen and unlock orientation
   */
  static async exitFullscreenWithOrientation(): Promise<void> {
    try {
      // Unlock orientation first
      this.unlock();

      // Then exit fullscreen
      if (document.exitFullscreen) {
        await document.exitFullscreen();
      } else if ((document as any).webkitExitFullscreen) {
        await (document as any).webkitExitFullscreen();
      } else if ((document as any).mozCancelFullScreen) {
        await (document as any).mozCancelFullScreen();
      } else if ((document as any).msExitFullscreen) {
        await (document as any).msExitFullscreen();
      }
    } catch (error) {
      console.error('Failed to exit fullscreen:', error);
      throw error;
    }
  }

  /**
   * Get orientation type string from current state
   */
  static getOrientationType(): string {
    if (typeof window !== 'undefined' && window.screen?.orientation) {
      return window.screen.orientation.type;
    }
    return 'unknown';
  }

  /**
   * Check if device is in portrait mode
   */
  static isPortrait(): boolean {
    return this.getCurrentOrientation() === 'portrait';
  }

  /**
   * Check if device is in landscape mode
   */
  static isLandscape(): boolean {
    return this.getCurrentOrientation() === 'landscape';
  }
}

/**
 * React component wrapper for orientation locking
 * Automatically locks/unlocks orientation on mount/unmount
 */
export interface OrientationLockWrapperProps {
  children: React.ReactNode;
  orientation: OrientationLockType;
  onLockError?: (error: Error) => void;
}

export function OrientationLockWrapper({
  children,
  orientation,
  onLockError,
}: OrientationLockWrapperProps): JSX.Element {
  React.useEffect(() => {
    const lockOrientation = async () => {
      try {
        await OrientationLock.lock(orientation);
      } catch (error) {
        onLockError?.(error as Error);
      }
    };

    lockOrientation();

    return () => {
      OrientationLock.unlock();
    };
  }, [orientation, onLockError]);

  return <>{children}</>;
}

export default OrientationLock;
