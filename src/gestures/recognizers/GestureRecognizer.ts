/**
 * Core Gesture Recognizer
 */

import {
  TouchPoint,
  GestureEvent,
  GestureConfig,
  DEFAULT_GESTURE_CONFIG,
  GesturePhase,
  GesturePriority,
} from '../types';

export interface GestureRecognizerState {
  phase: GesturePhase;
  priority: GesturePriority;
  isActive: boolean;
  startTime: number | null;
  lastUpdate: number | null;
}

export abstract class BaseGestureRecognizer {
  protected config: GestureConfig;
  protected state: GestureRecognizerState;
  protected touchHistory: TouchPoint[] = [];
  protected maxHistorySize: number = 100;

  constructor(config: Partial<GestureConfig> = {}) {
    this.config = this.mergeConfig(config);
    this.state = {
      phase: GesturePhase.Failed,
      priority: GesturePriority.Normal,
      isActive: false,
      startTime: null,
      lastUpdate: null,
    };
  }

  protected mergeConfig(config: Partial<GestureConfig>): GestureConfig {
    return {
      tap: { ...DEFAULT_GESTURE_CONFIG.tap, ...config.tap },
      swipe: { ...DEFAULT_GESTURE_CONFIG.swipe, ...config.swipe },
      pinch: { ...DEFAULT_GESTURE_CONFIG.pinch, ...config.pinch },
      rotate: { ...DEFAULT_GESTURE_CONFIG.rotate, ...config.rotate },
      pan: { ...DEFAULT_GESTURE_CONFIG.pan, ...config.pan },
      longPress: { ...DEFAULT_GESTURE_CONFIG.longPress, ...config.longPress },
      general: { ...DEFAULT_GESTURE_CONFIG.general, ...config.general },
    };
  }

  abstract process(touches: TouchPoint[]): GestureEvent | null;
  abstract reset(): void;
  abstract getName(): string;

  protected updatePhase(newPhase: GesturePhase): void {
    const now = Date.now();

    if (newPhase === GesturePhase.Began) {
      this.state.startTime = now;
      this.state.isActive = true;
    } else if (
      newPhase === GesturePhase.Ended ||
      newPhase === GesturePhase.Cancelled ||
      newPhase === GesturePhase.Failed
    ) {
      this.state.isActive = false;
    }

    this.state.phase = newPhase;
    this.state.lastUpdate = now;
  }

  protected addToHistory(touches: TouchPoint[]): void {
    this.touchHistory.push(...touches);

    // Limit history size
    while (this.touchHistory.length > this.maxHistorySize) {
      this.touchHistory.shift();
    }
  }

  protected getHistorySince(timeMs: number): TouchPoint[] {
    const now = Date.now();
    return this.touchHistory.filter((touch) => now - touch.timestamp <= timeMs);
  }

  protected clearHistory(): void {
    this.touchHistory = [];
  }

  public getState(): GestureRecognizerState {
    return { ...this.state };
  }

  public isActive(): boolean {
    return this.state.isActive;
  }

  public getPriority(): GesturePriority {
    return this.state.priority;
  }

  public updateConfig(config: Partial<GestureConfig>): void {
    this.config = this.mergeConfig(config);
  }
}

/**
 * Gesture State Machine
 */
export class GestureStateMachine {
  private recognizers: Map<string, BaseGestureRecognizer> = new Map();
  private activeGestures: Set<string> = new Set();
  private config: GestureConfig;

  constructor(config: Partial<GestureConfig> = {}) {
    this.config = { ...DEFAULT_GESTURE_CONFIG, ...config };
  }

  registerRecognizer(name: string, recognizer: BaseGestureRecognizer): void {
    this.recognizers.set(name, recognizer);
  }

  unregisterRecognizer(name: string): void {
    this.recognizers.delete(name);
    this.activeGestures.delete(name);
  }

  process(touches: TouchPoint[]): GestureEvent[] {
    const events: GestureEvent[] = [];

    // Process each recognizer
    for (const [name, recognizer] of this.recognizers.entries()) {
      const event = recognizer.process(touches);

      if (event) {
        events.push(event);

        if (recognizer.isActive()) {
          this.activeGestures.add(name);
        } else {
          this.activeGestures.delete(name);
        }
      }
    }

    // Resolve conflicts if enabled
    if (this.config.general.enableConflictResolution && this.hasConflicts()) {
      return this.resolveConflicts(events);
    }

    return events;
  }

  private hasConflicts(): boolean {
    return this.activeGestures.size > 1;
  }

  private resolveConflicts(events: GestureEvent[]): GestureEvent[] {
    if (events.length <= 1) {
      return events;
    }

    // Find highest priority gesture
    let highestPriority = GesturePriority.Low;
    let winningEvent: GestureEvent | null = null;

    for (const event of events) {
      const recognizer = this.getRecognizerForEvent(event);
      if (recognizer) {
        const priority = recognizer.getPriority();
        if (priority > highestPriority) {
          highestPriority = priority;
          winningEvent = event;
        }
      }
    }

    return winningEvent ? [winningEvent] : [];
  }

  private getRecognizerForEvent(event: GestureEvent): BaseGestureRecognizer | null {
    const type = event.type;

    if (type.includes('tap')) {
      return this.recognizers.get('tap') || null;
    } else if (type.includes('swipe')) {
      return this.recognizers.get('swipe') || null;
    } else if (type.includes('pinch')) {
      return this.recognizers.get('pinch') || null;
    } else if (type.includes('rotate')) {
      return this.recognizers.get('rotate') || null;
    } else if (type.includes('pan')) {
      return this.recognizers.get('pan') || null;
    } else if (type.includes('longPress')) {
      return this.recognizers.get('longPress') || null;
    }

    return null;
  }

  reset(): void {
    for (const recognizer of this.recognizers.values()) {
      recognizer.reset();
    }
    this.activeGestures.clear();
  }

  getActiveGestures(): string[] {
    return Array.from(this.activeGestures);
  }

  updateConfig(config: Partial<GestureConfig>): void {
    this.config = { ...this.config, ...config };
    for (const recognizer of this.recognizers.values()) {
      recognizer.updateConfig(config);
    }
  }
}

/**
 * Velocity Tracker
 */
export class VelocityTracker {
  private samples: Array<{ x: number; y: number; timestamp: number }> = [];
  private maxSamples: number;

  constructor(maxSamples: number = 5) {
    this.maxSamples = maxSamples;
  }

  addSample(x: number, y: number, timestamp: number): void {
    this.samples.push({ x, y, timestamp });

    while (this.samples.length > this.maxSamples) {
      this.samples.shift();
    }
  }

  getVelocity(): { x: number; y: number; magnitude: number } {
    if (this.samples.length < 2) {
      return { x: 0, y: 0, magnitude: 0 };
    }

    const first = this.samples[0];
    const last = this.samples[this.samples.length - 1];

    const dt = (last.timestamp - first.timestamp) / 1000; // Convert to seconds

    if (dt === 0) {
      return { x: 0, y: 0, magnitude: 0 };
    }

    const vx = (last.x - first.x) / dt;
    const vy = (last.y - first.y) / dt;
    const magnitude = Math.sqrt(vx * vx + vy * vy);

    return { x: vx, y: vy, magnitude };
  }

  reset(): void {
    this.samples = [];
  }
}
