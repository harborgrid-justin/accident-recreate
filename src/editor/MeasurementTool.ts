/**
 * Measurement Tool
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Point, Measurement } from '../types/diagram';

export class MeasurementTool {
  private scale: number; // pixels per meter

  constructor(scale: number = 20) {
    this.scale = scale;
  }

  /**
   * Set scale (pixels per meter)
   */
  setScale(scale: number): void {
    this.scale = scale;
  }

  /**
   * Calculate distance between two points in meters
   */
  calculateDistance(p1: Point, p2: Point): number {
    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;
    const distanceInPixels = Math.sqrt(dx * dx + dy * dy);
    return distanceInPixels / this.scale;
  }

  /**
   * Calculate distance between multiple points (polyline)
   */
  calculatePolylineDistance(points: Point[]): number {
    if (points.length < 2) return 0;

    let totalDistance = 0;
    for (let i = 0; i < points.length - 1; i++) {
      totalDistance += this.calculateDistance(points[i], points[i + 1]);
    }

    return totalDistance;
  }

  /**
   * Calculate angle between three points (in degrees)
   * p2 is the vertex
   */
  calculateAngle(p1: Point, p2: Point, p3: Point): number {
    const angle1 = Math.atan2(p1.y - p2.y, p1.x - p2.x);
    const angle2 = Math.atan2(p3.y - p2.y, p3.x - p2.x);
    let angle = ((angle2 - angle1) * 180) / Math.PI;

    // Normalize to 0-360
    while (angle < 0) angle += 360;
    while (angle > 360) angle -= 360;

    return angle;
  }

  /**
   * Calculate heading angle from point 1 to point 2 (in degrees)
   * 0 degrees is North (up), clockwise
   */
  calculateHeading(p1: Point, p2: Point): number {
    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;
    let angle = (Math.atan2(dx, -dy) * 180) / Math.PI;

    // Normalize to 0-360
    while (angle < 0) angle += 360;
    while (angle >= 360) angle -= 360;

    return angle;
  }

  /**
   * Create distance measurement
   */
  createDistanceMeasurement(
    p1: Point,
    p2: Point,
    label?: string
  ): Measurement {
    const distance = this.calculateDistance(p1, p2);
    const id = `measurement_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      id,
      type: 'distance',
      points: [p1, p2],
      value: distance,
      unit: 'm',
      label: label || `${distance.toFixed(2)} m`,
      color: '#3B82F6',
    };
  }

  /**
   * Create polyline distance measurement
   */
  createPolylineMeasurement(
    points: Point[],
    label?: string
  ): Measurement {
    const distance = this.calculatePolylineDistance(points);
    const id = `measurement_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      id,
      type: 'distance',
      points,
      value: distance,
      unit: 'm',
      label: label || `${distance.toFixed(2)} m`,
      color: '#3B82F6',
    };
  }

  /**
   * Create angle measurement
   */
  createAngleMeasurement(
    p1: Point,
    p2: Point,
    p3: Point,
    label?: string
  ): Measurement {
    const angle = this.calculateAngle(p1, p2, p3);
    const id = `measurement_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      id,
      type: 'angle',
      points: [p1, p2, p3],
      value: angle,
      unit: '°',
      label: label || `${angle.toFixed(1)}°`,
      color: '#10B981',
    };
  }

  /**
   * Format distance for display
   */
  formatDistance(meters: number, precision: number = 2): string {
    if (meters < 1) {
      return `${(meters * 100).toFixed(precision)} cm`;
    } else if (meters >= 1000) {
      return `${(meters / 1000).toFixed(precision)} km`;
    } else {
      return `${meters.toFixed(precision)} m`;
    }
  }

  /**
   * Format angle for display
   */
  formatAngle(degrees: number, precision: number = 1): string {
    return `${degrees.toFixed(precision)}°`;
  }

  /**
   * Convert meters to pixels
   */
  metersToPixels(meters: number): number {
    return meters * this.scale;
  }

  /**
   * Convert pixels to meters
   */
  pixelsToMeters(pixels: number): number {
    return pixels / this.scale;
  }

  /**
   * Calculate area of a polygon (in square meters)
   */
  calculatePolygonArea(points: Point[]): number {
    if (points.length < 3) return 0;

    let area = 0;
    for (let i = 0; i < points.length; i++) {
      const j = (i + 1) % points.length;
      area += points[i].x * points[j].y;
      area -= points[j].x * points[i].y;
    }

    area = Math.abs(area / 2);
    const areaInMeters = area / (this.scale * this.scale);

    return areaInMeters;
  }

  /**
   * Calculate perimeter of a polygon (in meters)
   */
  calculatePolygonPerimeter(points: Point[]): number {
    if (points.length < 2) return 0;

    let perimeter = 0;
    for (let i = 0; i < points.length; i++) {
      const j = (i + 1) % points.length;
      perimeter += this.calculateDistance(points[i], points[j]);
    }

    return perimeter;
  }

  /**
   * Find closest point on a line segment to a given point
   */
  closestPointOnLine(point: Point, lineStart: Point, lineEnd: Point): Point {
    const dx = lineEnd.x - lineStart.x;
    const dy = lineEnd.y - lineStart.y;
    const lengthSquared = dx * dx + dy * dy;

    if (lengthSquared === 0) {
      return lineStart;
    }

    const t = Math.max(
      0,
      Math.min(
        1,
        ((point.x - lineStart.x) * dx + (point.y - lineStart.y) * dy) / lengthSquared
      )
    );

    return {
      x: lineStart.x + t * dx,
      y: lineStart.y + t * dy,
    };
  }

  /**
   * Calculate perpendicular distance from point to line
   */
  perpendicularDistance(point: Point, lineStart: Point, lineEnd: Point): number {
    const closestPoint = this.closestPointOnLine(point, lineStart, lineEnd);
    return this.calculateDistance(point, closestPoint);
  }

  /**
   * Snap point to grid
   */
  snapToGrid(point: Point, gridSize: number): Point {
    const gridSizeInPixels = gridSize * this.scale;
    return {
      x: Math.round(point.x / gridSizeInPixels) * gridSizeInPixels,
      y: Math.round(point.y / gridSizeInPixels) * gridSizeInPixels,
    };
  }

  /**
   * Calculate speed from distance and time
   */
  calculateSpeed(distanceMeters: number, timeSeconds: number): {
    metersPerSecond: number;
    kilometersPerHour: number;
    milesPerHour: number;
  } {
    const mps = distanceMeters / timeSeconds;
    return {
      metersPerSecond: mps,
      kilometersPerHour: mps * 3.6,
      milesPerHour: mps * 2.23694,
    };
  }

  /**
   * Calculate time from distance and speed
   */
  calculateTime(distanceMeters: number, speedMps: number): number {
    return distanceMeters / speedMps;
  }

  /**
   * Calculate stopping distance
   * Uses simplified physics: d = v²/(2μg)
   * where μ is friction coefficient, g is gravity
   */
  calculateStoppingDistance(
    speedMps: number,
    frictionCoefficient: number = 0.7
  ): number {
    const g = 9.81; // gravity in m/s²
    return (speedMps * speedMps) / (2 * frictionCoefficient * g);
  }

  /**
   * Calculate impact speed from skid marks
   * v = √(2μgd)
   */
  calculateImpactSpeed(
    skidDistanceMeters: number,
    frictionCoefficient: number = 0.7
  ): {
    metersPerSecond: number;
    kilometersPerHour: number;
    milesPerHour: number;
  } {
    const g = 9.81;
    const mps = Math.sqrt(2 * frictionCoefficient * g * skidDistanceMeters);

    return {
      metersPerSecond: mps,
      kilometersPerHour: mps * 3.6,
      milesPerHour: mps * 2.23694,
    };
  }

  /**
   * Calculate critical speed for a curve
   * v = √(μgr)
   */
  calculateCriticalSpeed(
    radiusMeters: number,
    frictionCoefficient: number = 0.7
  ): {
    metersPerSecond: number;
    kilometersPerHour: number;
    milesPerHour: number;
  } {
    const g = 9.81;
    const mps = Math.sqrt(frictionCoefficient * g * radiusMeters);

    return {
      metersPerSecond: mps,
      kilometersPerHour: mps * 3.6,
      milesPerHour: mps * 2.23694,
    };
  }
}

/**
 * Create a singleton instance
 */
export const measurementTool = new MeasurementTool();
