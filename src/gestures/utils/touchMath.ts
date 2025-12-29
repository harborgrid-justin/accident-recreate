/**
 * Touch Point Mathematics and Calculations
 */

import { TouchPoint, Point2D, VelocityVector, BoundingBox } from '../types';

/**
 * Convert React Touch to TouchPoint
 */
export function convertToTouchPoint(touch: React.Touch, timestamp?: number): TouchPoint {
  return {
    id: touch.identifier,
    x: touch.clientX,
    y: touch.clientY,
    timestamp: timestamp || Date.now(),
    pressure: touch.force || 1.0,
    radiusX: touch.radiusX || 1.0,
    radiusY: touch.radiusY || 1.0,
  };
}

/**
 * Convert multiple React Touches to TouchPoints
 */
export function convertTouchList(touchList: React.TouchList): TouchPoint[] {
  const timestamp = Date.now();
  return Array.from(touchList).map((touch) => convertToTouchPoint(touch, timestamp));
}

/**
 * Calculate distance between two points
 */
export function calculateDistance(p1: Point2D, p2: Point2D): number {
  const dx = p2.x - p1.x;
  const dy = p2.y - p1.y;
  return Math.sqrt(dx * dx + dy * dy);
}

/**
 * Calculate angle between two points (in radians)
 */
export function calculateAngle(p1: Point2D, p2: Point2D): number {
  return Math.atan2(p2.y - p1.y, p2.x - p1.x);
}

/**
 * Calculate angle in degrees
 */
export function calculateAngleDegrees(p1: Point2D, p2: Point2D): number {
  return (calculateAngle(p1, p2) * 180) / Math.PI;
}

/**
 * Calculate center point between two points
 */
export function calculateCenter(p1: Point2D, p2: Point2D): Point2D {
  return {
    x: (p1.x + p2.x) / 2,
    y: (p1.y + p2.y) / 2,
  };
}

/**
 * Calculate centroid of multiple points
 */
export function calculateCentroid(points: Point2D[]): Point2D {
  if (points.length === 0) {
    return { x: 0, y: 0 };
  }

  const sum = points.reduce(
    (acc, p) => ({
      x: acc.x + p.x,
      y: acc.y + p.y,
    }),
    { x: 0, y: 0 }
  );

  return {
    x: sum.x / points.length,
    y: sum.y / points.length,
  };
}

/**
 * Calculate velocity between two points
 */
export function calculateVelocity(
  p1: TouchPoint,
  p2: TouchPoint
): VelocityVector {
  const dt = (p2.timestamp - p1.timestamp) / 1000; // Convert to seconds

  if (dt === 0) {
    return { x: 0, y: 0, magnitude: 0 };
  }

  const vx = (p2.x - p1.x) / dt;
  const vy = (p2.y - p1.y) / dt;
  const magnitude = Math.sqrt(vx * vx + vy * vy);

  return { x: vx, y: vy, magnitude };
}

/**
 * Calculate velocity from a series of touch points
 */
export function calculateAverageVelocity(points: TouchPoint[]): VelocityVector {
  if (points.length < 2) {
    return { x: 0, y: 0, magnitude: 0 };
  }

  const velocities: VelocityVector[] = [];

  for (let i = 1; i < points.length; i++) {
    velocities.push(calculateVelocity(points[i - 1], points[i]));
  }

  const sum = velocities.reduce(
    (acc, v) => ({
      x: acc.x + v.x,
      y: acc.y + v.y,
      magnitude: acc.magnitude + v.magnitude,
    }),
    { x: 0, y: 0, magnitude: 0 }
  );

  return {
    x: sum.x / velocities.length,
    y: sum.y / velocities.length,
    magnitude: sum.magnitude / velocities.length,
  };
}

/**
 * Calculate scale factor between two touch pairs
 */
export function calculateScale(
  touch1Start: Point2D,
  touch2Start: Point2D,
  touch1End: Point2D,
  touch2End: Point2D
): number {
  const startDistance = calculateDistance(touch1Start, touch2Start);
  const endDistance = calculateDistance(touch1End, touch2End);

  if (startDistance === 0) {
    return 1;
  }

  return endDistance / startDistance;
}

/**
 * Calculate rotation angle between two touch pairs (in radians)
 */
export function calculateRotation(
  touch1Start: Point2D,
  touch2Start: Point2D,
  touch1End: Point2D,
  touch2End: Point2D
): number {
  const startAngle = calculateAngle(touch1Start, touch2Start);
  const endAngle = calculateAngle(touch1End, touch2End);

  let rotation = endAngle - startAngle;

  // Normalize to -PI to PI
  while (rotation > Math.PI) rotation -= 2 * Math.PI;
  while (rotation < -Math.PI) rotation += 2 * Math.PI;

  return rotation;
}

/**
 * Calculate rotation in degrees
 */
export function calculateRotationDegrees(
  touch1Start: Point2D,
  touch2Start: Point2D,
  touch1End: Point2D,
  touch2End: Point2D
): number {
  return (calculateRotation(touch1Start, touch2Start, touch1End, touch2End) * 180) / Math.PI;
}

/**
 * Calculate bounding box for a set of points
 */
export function calculateBoundingBox(points: Point2D[]): BoundingBox {
  if (points.length === 0) {
    return { minX: 0, minY: 0, maxX: 0, maxY: 0, width: 0, height: 0 };
  }

  const minX = Math.min(...points.map((p) => p.x));
  const minY = Math.min(...points.map((p) => p.y));
  const maxX = Math.max(...points.map((p) => p.x));
  const maxY = Math.max(...points.map((p) => p.y));

  return {
    minX,
    minY,
    maxX,
    maxY,
    width: maxX - minX,
    height: maxY - minY,
  };
}

/**
 * Check if point is within a circular area
 */
export function isWithinRadius(
  point: Point2D,
  center: Point2D,
  radius: number
): boolean {
  return calculateDistance(point, center) <= radius;
}

/**
 * Check if point is within a rectangular area
 */
export function isWithinBounds(point: Point2D, bounds: BoundingBox): boolean {
  return (
    point.x >= bounds.minX &&
    point.x <= bounds.maxX &&
    point.y >= bounds.minY &&
    point.y <= bounds.maxY
  );
}

/**
 * Interpolate between two points
 */
export function interpolate(p1: Point2D, p2: Point2D, t: number): Point2D {
  return {
    x: p1.x + (p2.x - p1.x) * t,
    y: p1.y + (p2.y - p1.y) * t,
  };
}

/**
 * Smooth a series of points using moving average
 */
export function smoothPoints(points: Point2D[], windowSize: number = 3): Point2D[] {
  if (points.length < windowSize) {
    return [...points];
  }

  const smoothed: Point2D[] = [];
  const halfWindow = Math.floor(windowSize / 2);

  for (let i = 0; i < points.length; i++) {
    const start = Math.max(0, i - halfWindow);
    const end = Math.min(points.length, i + halfWindow + 1);
    const window = points.slice(start, end);

    const avg = calculateCentroid(window);
    smoothed.push(avg);
  }

  return smoothed;
}

/**
 * Predict next point based on velocity
 */
export function predictNextPoint(
  currentPoint: TouchPoint,
  velocity: VelocityVector,
  deltaTimeMs: number
): Point2D {
  const dt = deltaTimeMs / 1000; // Convert to seconds

  return {
    x: currentPoint.x + velocity.x * dt,
    y: currentPoint.y + velocity.y * dt,
  };
}

/**
 * Calculate angular velocity
 */
export function calculateAngularVelocity(
  angle1: number,
  angle2: number,
  timestamp1: number,
  timestamp2: number
): number {
  const dt = (timestamp2 - timestamp1) / 1000; // Convert to seconds

  if (dt === 0) {
    return 0;
  }

  let deltaAngle = angle2 - angle1;

  // Normalize to -PI to PI
  while (deltaAngle > Math.PI) deltaAngle -= 2 * Math.PI;
  while (deltaAngle < -Math.PI) deltaAngle += 2 * Math.PI;

  return deltaAngle / dt;
}

/**
 * Clamp a value between min and max
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/**
 * Linear interpolation
 */
export function lerp(start: number, end: number, t: number): number {
  return start + (end - start) * t;
}

/**
 * Exponential smoothing
 */
export function exponentialSmooth(
  current: number,
  target: number,
  smoothing: number
): number {
  return current + (target - current) * smoothing;
}

/**
 * Calculate momentum decay
 */
export function applyMomentumDecay(velocity: number, decay: number): number {
  return velocity * decay;
}

/**
 * Normalize angle to -PI to PI range
 */
export function normalizeAngle(angle: number): number {
  let normalized = angle;
  while (normalized > Math.PI) normalized -= 2 * Math.PI;
  while (normalized < -Math.PI) normalized += 2 * Math.PI;
  return normalized;
}

/**
 * Convert radians to degrees
 */
export function radiansToDegrees(radians: number): number {
  return (radians * 180) / Math.PI;
}

/**
 * Convert degrees to radians
 */
export function degreesToRadians(degrees: number): number {
  return (degrees * Math.PI) / 180;
}
