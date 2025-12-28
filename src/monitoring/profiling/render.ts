/**
 * AccuScene Enterprise v0.2.0
 * Render Profiling
 *
 * React component and rendering performance profiling
 */

import { RenderProfile, ComponentProfile } from '../types';

export class RenderProfiler {
  private componentProfiles: Map<string, ComponentProfile> = new Map();
  private frameTimestamps: number[] = [];
  private longFrames: number = 0;
  private droppedFrames: number = 0;
  private readonly longFrameThreshold: number = 16.67; // 60 FPS threshold
  private rafId?: number;
  private isRunning: boolean = false;

  /**
   * Start render profiling
   */
  public start(): void {
    if (this.isRunning) {
      return;
    }

    this.isRunning = true;
    this.reset();
    this.startFrameMonitoring();
  }

  /**
   * Stop render profiling
   */
  public stop(): RenderProfile {
    this.isRunning = false;

    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = undefined;
    }

    return this.getProfile();
  }

  /**
   * Start monitoring frame rate
   */
  private startFrameMonitoring(): void {
    let lastTimestamp = performance.now();

    const measureFrame = (timestamp: number) => {
      if (!this.isRunning) {
        return;
      }

      const frameTime = timestamp - lastTimestamp;
      this.frameTimestamps.push(frameTime);

      // Check for long frames (>16.67ms for 60fps)
      if (frameTime > this.longFrameThreshold) {
        this.longFrames++;
      }

      // Check for dropped frames
      const expectedFrames = Math.round(frameTime / this.longFrameThreshold);
      if (expectedFrames > 1) {
        this.droppedFrames += expectedFrames - 1;
      }

      // Keep only last 100 frames
      if (this.frameTimestamps.length > 100) {
        this.frameTimestamps.shift();
      }

      lastTimestamp = timestamp;
      this.rafId = requestAnimationFrame(measureFrame);
    };

    this.rafId = requestAnimationFrame(measureFrame);
  }

  /**
   * Record component render
   */
  public recordComponentRender(componentName: string, renderTime: number): void {
    let profile = this.componentProfiles.get(componentName);

    if (!profile) {
      profile = {
        name: componentName,
        renderCount: 0,
        renderTime: 0,
        avgRenderTime: 0,
        maxRenderTime: 0
      };
      this.componentProfiles.set(componentName, profile);
    }

    profile.renderCount++;
    profile.renderTime += renderTime;
    profile.avgRenderTime = profile.renderTime / profile.renderCount;
    profile.maxRenderTime = Math.max(profile.maxRenderTime, renderTime);
  }

  /**
   * Get current profile
   */
  public getProfile(): RenderProfile {
    const fps = this.calculateFPS();
    const avgFrameTime = this.frameTimestamps.length > 0
      ? this.frameTimestamps.reduce((a, b) => a + b, 0) / this.frameTimestamps.length
      : 0;

    return {
      timestamp: Date.now(),
      fps,
      frameTime: avgFrameTime,
      longFrames: this.longFrames,
      droppedFrames: this.droppedFrames,
      components: Array.from(this.componentProfiles.values())
    };
  }

  /**
   * Calculate current FPS
   */
  private calculateFPS(): number {
    if (this.frameTimestamps.length < 2) {
      return 0;
    }

    const recentFrames = this.frameTimestamps.slice(-10);
    const avgFrameTime = recentFrames.reduce((a, b) => a + b, 0) / recentFrames.length;

    return avgFrameTime > 0 ? 1000 / avgFrameTime : 0;
  }

  /**
   * Reset profiler
   */
  public reset(): void {
    this.componentProfiles.clear();
    this.frameTimestamps = [];
    this.longFrames = 0;
    this.droppedFrames = 0;
  }

  /**
   * Get slow components
   */
  public getSlowComponents(threshold: number = 16): ComponentProfile[] {
    return Array.from(this.componentProfiles.values())
      .filter(profile => profile.avgRenderTime > threshold)
      .sort((a, b) => b.avgRenderTime - a.avgRenderTime);
  }

  /**
   * Get frequently rendered components
   */
  public getFrequentComponents(limit: number = 10): ComponentProfile[] {
    return Array.from(this.componentProfiles.values())
      .sort((a, b) => b.renderCount - a.renderCount)
      .slice(0, limit);
  }
}

/**
 * React Profiler Component wrapper
 */
export function ProfiledComponent<P extends object>(
  Component: React.ComponentType<P>,
  profiler: RenderProfiler
): React.ComponentType<P> {
  return (props: P) => {
    const startTime = performance.now();

    React.useEffect(() => {
      const endTime = performance.now();
      const renderTime = endTime - startTime;
      const componentName = Component.displayName || Component.name || 'Unknown';

      profiler.recordComponentRender(componentName, renderTime);
    });

    return React.createElement(Component, props);
  };
}

/**
 * React Profiler Hook
 */
export function useRenderProfile(componentName: string, profiler: RenderProfiler): void {
  const renderStartTime = React.useRef(performance.now());

  React.useEffect(() => {
    const renderTime = performance.now() - renderStartTime.current;
    profiler.recordComponentRender(componentName, renderTime);
    renderStartTime.current = performance.now();
  });
}

/**
 * Measure paint timing
 */
export class PaintTimingProfiler {
  /**
   * Get paint timing metrics
   */
  public static getPaintTiming(): {
    firstPaint?: number;
    firstContentfulPaint?: number;
    largestContentfulPaint?: number;
  } {
    if (typeof performance === 'undefined' || !performance.getEntriesByType) {
      return {};
    }

    const paintEntries = performance.getEntriesByType('paint');
    const result: any = {};

    for (const entry of paintEntries) {
      if (entry.name === 'first-paint') {
        result.firstPaint = entry.startTime;
      } else if (entry.name === 'first-contentful-paint') {
        result.firstContentfulPaint = entry.startTime;
      }
    }

    // Get LCP if available
    const lcpEntries = performance.getEntriesByType('largest-contentful-paint');
    if (lcpEntries.length > 0) {
      const lastEntry = lcpEntries[lcpEntries.length - 1];
      result.largestContentfulPaint = lastEntry.startTime;
    }

    return result;
  }

  /**
   * Get layout shift metrics
   */
  public static getLayoutShift(): number {
    if (typeof performance === 'undefined' || !performance.getEntriesByType) {
      return 0;
    }

    const layoutShiftEntries = performance.getEntriesByType('layout-shift');

    return layoutShiftEntries.reduce((score: number, entry: any) => {
      if (!entry.hadRecentInput) {
        return score + entry.value;
      }
      return score;
    }, 0);
  }
}

// React import for types
import * as React from 'react';
