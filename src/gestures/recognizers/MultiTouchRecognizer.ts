/**
 * Multi-Touch Gesture Recognizer
 * Handles pinch, rotate, and multi-finger gestures
 */

import {
  TouchPoint,
  GestureEvent,
  GesturePhase,
  GesturePriority,
  PinchStartEvent,
  PinchMoveEvent,
  PinchEndEvent,
  RotateStartEvent,
  RotateMoveEvent,
  RotateEndEvent,
} from '../types';
import { BaseGestureRecognizer, VelocityTracker } from './GestureRecognizer';
import {
  calculateDistance,
  calculateCenter,
  calculateAngle,
  calculateAngularVelocity,
  radiansToDegrees,
  normalizeAngle,
} from '../utils/touchMath';

/**
 * Pinch Gesture Recognizer
 */
export class PinchRecognizer extends BaseGestureRecognizer {
  private initialDistance: number | null = null;
  private previousDistance: number | null = null;
  private currentScale: number = 1;
  private totalScale: number = 1;
  private velocityTracker = new VelocityTracker();
  private touch1: TouchPoint | null = null;
  private touch2: TouchPoint | null = null;

  constructor(config = {}) {
    super(config);
    this.state.priority = this.config.pinch.priority;
  }

  process(touches: TouchPoint[]): GestureEvent | null {
    if (touches.length === 2) {
      return this.handleTwoTouches(touches);
    } else if (touches.length < 2 && this.state.isActive) {
      return this.handlePinchEnd();
    }

    return null;
  }

  private handleTwoTouches(touches: TouchPoint[]): GestureEvent | null {
    const [touch1, touch2] = touches;
    const distance = calculateDistance(touch1, touch2);
    const center = calculateCenter(touch1, touch2);

    if (!this.state.isActive) {
      // Start pinch
      this.initialDistance = distance;
      this.previousDistance = distance;
      this.currentScale = 1;
      this.totalScale = 1;
      this.touch1 = touch1;
      this.touch2 = touch2;
      this.velocityTracker.reset();
      this.velocityTracker.addSample(distance, 0, touch1.timestamp);
      this.updatePhase(GesturePhase.Began);

      const event: PinchStartEvent = {
        type: 'pinchStart',
        center: {
          id: 0,
          x: center.x,
          y: center.y,
          timestamp: touch1.timestamp,
          pressure: 1,
          radiusX: 1,
          radiusY: 1,
        },
        initialDistance: distance,
        touch1,
        touch2,
      };

      return event;
    } else {
      // Continue pinch
      this.previousDistance = distance;
      this.currentScale = this.calculateScale(distance);
      this.totalScale = this.currentScale;
      this.touch1 = touch1;
      this.touch2 = touch2;
      this.velocityTracker.addSample(distance, 0, touch1.timestamp);
      this.updatePhase(GesturePhase.Changed);

      // Check if scale change is significant
      if (Math.abs(this.currentScale - 1) >= this.config.pinch.minScaleDelta) {
        const velocity = this.velocityTracker.getVelocity().magnitude;

        const event: PinchMoveEvent = {
          type: 'pinchMove',
          center: {
            id: 0,
            x: center.x,
            y: center.y,
            timestamp: touch1.timestamp,
            pressure: 1,
            radiusX: 1,
            radiusY: 1,
          },
          scale: this.currentScale,
          distance,
          velocity,
          touch1,
          touch2,
        };

        return event;
      }
    }

    return null;
  }

  private handlePinchEnd(): GestureEvent | null {
    if (!this.state.isActive) {
      return null;
    }

    const center = this.touch1 && this.touch2
      ? calculateCenter(this.touch1, this.touch2)
      : { x: 0, y: 0 };

    this.updatePhase(GesturePhase.Ended);

    const event: PinchEndEvent = {
      type: 'pinchEnd',
      center: {
        id: 0,
        x: center.x,
        y: center.y,
        timestamp: Date.now(),
        pressure: 1,
        radiusX: 1,
        radiusY: 1,
      },
      finalScale: this.currentScale,
      totalScaleChange: this.totalScale - 1,
    };

    this.reset();
    return event;
  }

  private calculateScale(currentDistance: number): number {
    if (!this.initialDistance || this.initialDistance === 0) {
      return 1;
    }

    const scale = (currentDistance / this.initialDistance) * this.config.pinch.sensitivity;
    return Math.max(
      this.config.pinch.minScale,
      Math.min(this.config.pinch.maxScale, scale)
    );
  }

  reset(): void {
    this.initialDistance = null;
    this.previousDistance = null;
    this.currentScale = 1;
    this.totalScale = 1;
    this.velocityTracker.reset();
    this.touch1 = null;
    this.touch2 = null;
    this.updatePhase(GesturePhase.Failed);
    this.clearHistory();
  }

  getName(): string {
    return 'pinch';
  }
}

/**
 * Rotate Gesture Recognizer
 */
export class RotateRecognizer extends BaseGestureRecognizer {
  private initialAngle: number | null = null;
  private previousAngle: number | null = null;
  private currentRotation: number = 0;
  private totalRotation: number = 0;
  private touch1: TouchPoint | null = null;
  private touch2: TouchPoint | null = null;
  private angleHistory: Array<{ angle: number; timestamp: number }> = [];

  constructor(config = {}) {
    super(config);
    this.state.priority = this.config.rotate.priority;
  }

  process(touches: TouchPoint[]): GestureEvent | null {
    if (touches.length === 2) {
      return this.handleTwoTouches(touches);
    } else if (touches.length < 2 && this.state.isActive) {
      return this.handleRotateEnd();
    }

    return null;
  }

  private handleTwoTouches(touches: TouchPoint[]): GestureEvent | null {
    const [touch1, touch2] = touches;
    const angle = calculateAngle(touch1, touch2);
    const center = calculateCenter(touch1, touch2);

    if (!this.state.isActive) {
      // Start rotation
      this.initialAngle = angle;
      this.previousAngle = angle;
      this.currentRotation = 0;
      this.totalRotation = 0;
      this.touch1 = touch1;
      this.touch2 = touch2;
      this.angleHistory = [{ angle, timestamp: touch1.timestamp }];
      this.updatePhase(GesturePhase.Began);

      const event: RotateStartEvent = {
        type: 'rotateStart',
        center: {
          id: 0,
          x: center.x,
          y: center.y,
          timestamp: touch1.timestamp,
          pressure: 1,
          radiusX: 1,
          radiusY: 1,
        },
        initialAngle: radiansToDegrees(angle),
        touch1,
        touch2,
      };

      return event;
    } else {
      // Continue rotation
      const delta = this.calculateDeltaAngle(angle);
      this.currentRotation = delta;
      this.totalRotation += delta;
      this.previousAngle = angle;
      this.touch1 = touch1;
      this.touch2 = touch2;
      this.angleHistory.push({ angle, timestamp: touch1.timestamp });

      // Limit history size
      if (this.angleHistory.length > 10) {
        this.angleHistory.shift();
      }

      this.updatePhase(GesturePhase.Changed);

      // Check if rotation is significant
      if (Math.abs(radiansToDegrees(delta)) >= this.config.rotate.minAngleDelta) {
        const angularVelocity = this.calculateAngularVelocity();

        const event: RotateMoveEvent = {
          type: 'rotateMove',
          center: {
            id: 0,
            x: center.x,
            y: center.y,
            timestamp: touch1.timestamp,
            pressure: 1,
            radiusX: 1,
            radiusY: 1,
          },
          angle: radiansToDegrees(angle),
          deltaAngle: radiansToDegrees(delta) * this.config.rotate.sensitivity,
          angularVelocity,
          touch1,
          touch2,
        };

        return event;
      }
    }

    return null;
  }

  private handleRotateEnd(): GestureEvent | null {
    if (!this.state.isActive) {
      return null;
    }

    const center = this.touch1 && this.touch2
      ? calculateCenter(this.touch1, this.touch2)
      : { x: 0, y: 0 };

    const finalAngle = this.previousAngle !== null
      ? radiansToDegrees(this.previousAngle)
      : 0;

    this.updatePhase(GesturePhase.Ended);

    const event: RotateEndEvent = {
      type: 'rotateEnd',
      center: {
        id: 0,
        x: center.x,
        y: center.y,
        timestamp: Date.now(),
        pressure: 1,
        radiusX: 1,
        radiusY: 1,
      },
      finalAngle,
      totalRotation: radiansToDegrees(this.totalRotation),
    };

    this.reset();
    return event;
  }

  private calculateDeltaAngle(currentAngle: number): number {
    if (this.previousAngle === null) {
      return 0;
    }

    return normalizeAngle(currentAngle - this.previousAngle);
  }

  private calculateAngularVelocity(): number {
    if (this.angleHistory.length < 2) {
      return 0;
    }

    const first = this.angleHistory[0];
    const last = this.angleHistory[this.angleHistory.length - 1];

    return calculateAngularVelocity(
      first.angle,
      last.angle,
      first.timestamp,
      last.timestamp
    );
  }

  reset(): void {
    this.initialAngle = null;
    this.previousAngle = null;
    this.currentRotation = 0;
    this.totalRotation = 0;
    this.touch1 = null;
    this.touch2 = null;
    this.angleHistory = [];
    this.updatePhase(GesturePhase.Failed);
    this.clearHistory();
  }

  getName(): string {
    return 'rotate';
  }
}
