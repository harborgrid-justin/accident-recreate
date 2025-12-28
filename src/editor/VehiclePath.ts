/**
 * Vehicle Path System
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Point, PathElement, ElementType, AnimationFrame, PathAnimation, Transform } from '../types/diagram';

export interface PathPoint extends Point {
  timestamp: number;
  speed?: number;
  heading?: number;
}

export class VehiclePath {
  private scale: number; // pixels per meter

  constructor(scale: number = 20) {
    this.scale = scale;
  }

  /**
   * Set scale
   */
  setScale(scale: number): void {
    this.scale = scale;
  }

  /**
   * Create a path element
   */
  createPath(
    points: Point[],
    options: {
      color?: string;
      strokeWidth?: number;
      dashed?: boolean;
      arrowStart?: boolean;
      arrowEnd?: boolean;
      label?: string;
    } = {}
  ): PathElement {
    const id = `path_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      id,
      type: ElementType.PATH,
      subType: 'path',
      transform: {
        position: { x: 0, y: 0 },
        rotation: 0,
        scale: { x: 1, y: 1 },
      },
      properties: {
        points,
        color: options.color || '#3B82F6',
        strokeWidth: options.strokeWidth || 3,
        dashed: options.dashed || false,
        arrowStart: options.arrowStart || false,
        arrowEnd: options.arrowEnd || true,
      },
      locked: false,
      visible: true,
      zIndex: -1, // Paths typically behind other elements
      label: options.label || 'Path',
    };
  }

  /**
   * Create a skid mark path
   */
  createSkidMark(points: Point[], label?: string): PathElement {
    return this.createPath(points, {
      color: '#1F2937',
      strokeWidth: 4,
      dashed: false,
      arrowStart: false,
      arrowEnd: true,
      label: label || 'Skid Mark',
    });
  }

  /**
   * Smooth path using Catmull-Rom spline
   */
  smoothPath(points: Point[], tension: number = 0.5): Point[] {
    if (points.length < 3) return points;

    const smoothed: Point[] = [];
    const segments = 10; // Number of points between each control point

    for (let i = 0; i < points.length - 1; i++) {
      const p0 = i === 0 ? points[i] : points[i - 1];
      const p1 = points[i];
      const p2 = points[i + 1];
      const p3 = i === points.length - 2 ? p2 : points[i + 2];

      for (let t = 0; t < segments; t++) {
        const tt = t / segments;
        const point = this.catmullRomSpline(p0, p1, p2, p3, tt, tension);
        smoothed.push(point);
      }
    }

    smoothed.push(points[points.length - 1]);
    return smoothed;
  }

  /**
   * Catmull-Rom spline interpolation
   */
  private catmullRomSpline(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    t: number,
    tension: number
  ): Point {
    const t2 = t * t;
    const t3 = t2 * t;

    const v0x = (p2.x - p0.x) * tension;
    const v0y = (p2.y - p0.y) * tension;
    const v1x = (p3.x - p1.x) * tension;
    const v1y = (p3.y - p1.y) * tension;

    return {
      x:
        (2 * p1.x - 2 * p2.x + v0x + v1x) * t3 +
        (-3 * p1.x + 3 * p2.x - 2 * v0x - v1x) * t2 +
        v0x * t +
        p1.x,
      y:
        (2 * p1.y - 2 * p2.y + v0y + v1y) * t3 +
        (-3 * p1.y + 3 * p2.y - 2 * v0y - v1y) * t2 +
        v0y * t +
        p1.y,
    };
  }

  /**
   * Calculate path length
   */
  calculatePathLength(points: Point[]): number {
    let length = 0;
    for (let i = 0; i < points.length - 1; i++) {
      const dx = points[i + 1].x - points[i].x;
      const dy = points[i + 1].y - points[i].y;
      length += Math.sqrt(dx * dx + dy * dy);
    }
    return length / this.scale; // Convert to meters
  }

  /**
   * Get point along path at specific distance
   */
  getPointAtDistance(points: Point[], distance: number): Point | null {
    const targetDistance = distance * this.scale;
    let currentDistance = 0;

    for (let i = 0; i < points.length - 1; i++) {
      const dx = points[i + 1].x - points[i].x;
      const dy = points[i + 1].y - points[i].y;
      const segmentLength = Math.sqrt(dx * dx + dy * dy);

      if (currentDistance + segmentLength >= targetDistance) {
        const t = (targetDistance - currentDistance) / segmentLength;
        return {
          x: points[i].x + dx * t,
          y: points[i].y + dy * t,
        };
      }

      currentDistance += segmentLength;
    }

    return points[points.length - 1];
  }

  /**
   * Get heading at point along path
   */
  getHeadingAtDistance(points: Point[], distance: number): number {
    const targetDistance = distance * this.scale;
    let currentDistance = 0;

    for (let i = 0; i < points.length - 1; i++) {
      const dx = points[i + 1].x - points[i].x;
      const dy = points[i + 1].y - points[i].y;
      const segmentLength = Math.sqrt(dx * dx + dy * dy);

      if (currentDistance + segmentLength >= targetDistance) {
        return (Math.atan2(dy, dx) * 180) / Math.PI;
      }

      currentDistance += segmentLength;
    }

    // Return heading of last segment
    const last = points.length - 1;
    const dx = points[last].x - points[last - 1].x;
    const dy = points[last].y - points[last - 1].y;
    return (Math.atan2(dy, dx) * 180) / Math.PI;
  }

  /**
   * Create animation frames for a vehicle following a path
   */
  createPathAnimation(
    vehicleId: string,
    path: PathElement,
    durationSeconds: number,
    initialTransform?: Transform
  ): PathAnimation {
    const points = path.properties.points;
    const pathLength = this.calculatePathLength(points);
    const framesPerSecond = 60;
    const totalFrames = durationSeconds * framesPerSecond;

    const frames: AnimationFrame[] = [];

    for (let i = 0; i <= totalFrames; i++) {
      const t = i / totalFrames;
      const distance = pathLength * t;
      const position = this.getPointAtDistance(points, distance);
      const heading = this.getHeadingAtDistance(points, distance);

      if (position) {
        frames.push({
          elementId: vehicleId,
          transform: {
            position,
            rotation: heading,
            scale: initialTransform?.scale || { x: 1, y: 1 },
          },
          timestamp: (i / framesPerSecond) * 1000,
        });
      }
    }

    return {
      pathId: path.id,
      duration: durationSeconds * 1000,
      frames,
      currentFrame: 0,
      playing: false,
      loop: false,
    };
  }

  /**
   * Get frame at specific time
   */
  getFrameAtTime(animation: PathAnimation, timeMs: number): AnimationFrame | null {
    if (animation.frames.length === 0) return null;

    // Handle looping
    let time = timeMs;
    if (animation.loop) {
      time = timeMs % animation.duration;
    }

    // Find the frame closest to the time
    for (let i = 0; i < animation.frames.length - 1; i++) {
      if (
        animation.frames[i].timestamp <= time &&
        animation.frames[i + 1].timestamp > time
      ) {
        // Interpolate between frames
        const t =
          (time - animation.frames[i].timestamp) /
          (animation.frames[i + 1].timestamp - animation.frames[i].timestamp);

        return {
          elementId: animation.frames[i].elementId,
          transform: this.interpolateTransform(
            animation.frames[i].transform,
            animation.frames[i + 1].transform,
            t
          ),
          timestamp: time,
        };
      }
    }

    // Return last frame if beyond duration
    return animation.frames[animation.frames.length - 1];
  }

  /**
   * Interpolate between two transforms
   */
  private interpolateTransform(t1: Transform, t2: Transform, t: number): Transform {
    return {
      position: {
        x: t1.position.x + (t2.position.x - t1.position.x) * t,
        y: t1.position.y + (t2.position.y - t1.position.y) * t,
      },
      rotation: t1.rotation + (t2.rotation - t1.rotation) * t,
      scale: {
        x: t1.scale.x + (t2.scale.x - t1.scale.x) * t,
        y: t1.scale.y + (t2.scale.y - t1.scale.y) * t,
      },
    };
  }

  /**
   * Simplify path using Ramer-Douglas-Peucker algorithm
   */
  simplifyPath(points: Point[], tolerance: number): Point[] {
    if (points.length <= 2) return points;

    const tolerancePixels = tolerance * this.scale;

    return this.ramerDouglasPeucker(points, tolerancePixels);
  }

  /**
   * Ramer-Douglas-Peucker algorithm implementation
   */
  private ramerDouglasPeucker(points: Point[], epsilon: number): Point[] {
    if (points.length <= 2) return points;

    // Find point with maximum distance
    let maxDistance = 0;
    let maxIndex = 0;

    const first = points[0];
    const last = points[points.length - 1];

    for (let i = 1; i < points.length - 1; i++) {
      const distance = this.perpendicularDistance(points[i], first, last);
      if (distance > maxDistance) {
        maxDistance = distance;
        maxIndex = i;
      }
    }

    // If max distance is greater than epsilon, recursively simplify
    if (maxDistance > epsilon) {
      const left = this.ramerDouglasPeucker(points.slice(0, maxIndex + 1), epsilon);
      const right = this.ramerDouglasPeucker(points.slice(maxIndex), epsilon);

      return [...left.slice(0, -1), ...right];
    } else {
      return [first, last];
    }
  }

  /**
   * Calculate perpendicular distance from point to line
   */
  private perpendicularDistance(point: Point, lineStart: Point, lineEnd: Point): number {
    const dx = lineEnd.x - lineStart.x;
    const dy = lineEnd.y - lineStart.y;
    const lengthSquared = dx * dx + dy * dy;

    if (lengthSquared === 0) {
      const pdx = point.x - lineStart.x;
      const pdy = point.y - lineStart.y;
      return Math.sqrt(pdx * pdx + pdy * pdy);
    }

    const t = Math.max(
      0,
      Math.min(1, ((point.x - lineStart.x) * dx + (point.y - lineStart.y) * dy) / lengthSquared)
    );

    const projectionX = lineStart.x + t * dx;
    const projectionY = lineStart.y + t * dy;

    const distX = point.x - projectionX;
    const distY = point.y - projectionY;

    return Math.sqrt(distX * distX + distY * distY);
  }

  /**
   * Reverse path direction
   */
  reversePath(path: PathElement): PathElement {
    return {
      ...path,
      properties: {
        ...path.properties,
        points: [...path.properties.points].reverse(),
        arrowStart: path.properties.arrowEnd,
        arrowEnd: path.properties.arrowStart,
      },
    };
  }

  /**
   * Merge multiple paths into one
   */
  mergePaths(paths: PathElement[]): PathElement | null {
    if (paths.length === 0) return null;
    if (paths.length === 1) return paths[0];

    const allPoints: Point[] = [];
    paths.forEach((path) => {
      allPoints.push(...path.properties.points);
    });

    return this.createPath(allPoints, {
      color: paths[0].properties.color,
      strokeWidth: paths[0].properties.strokeWidth,
      dashed: paths[0].properties.dashed,
      arrowStart: paths[0].properties.arrowStart,
      arrowEnd: paths[paths.length - 1].properties.arrowEnd,
      label: 'Merged Path',
    });
  }

  /**
   * Calculate curvature at each point
   */
  calculateCurvature(points: Point[]): number[] {
    if (points.length < 3) return [];

    const curvatures: number[] = [0]; // First point has no curvature

    for (let i = 1; i < points.length - 1; i++) {
      const p0 = points[i - 1];
      const p1 = points[i];
      const p2 = points[i + 1];

      // Calculate angle change
      const angle1 = Math.atan2(p1.y - p0.y, p1.x - p0.x);
      const angle2 = Math.atan2(p2.y - p1.y, p2.x - p1.x);
      let angleChange = angle2 - angle1;

      // Normalize angle to -π to π
      while (angleChange > Math.PI) angleChange -= 2 * Math.PI;
      while (angleChange < -Math.PI) angleChange += 2 * Math.PI;

      curvatures.push(Math.abs(angleChange));
    }

    curvatures.push(0); // Last point has no curvature

    return curvatures;
  }
}

/**
 * Create a singleton instance
 */
export const vehiclePath = new VehiclePath();
