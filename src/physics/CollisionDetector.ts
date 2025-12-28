/**
 * CollisionDetector - Collision Detection System
 * Implements AABB, polygon collision (SAT), and collision point calculations
 */

import { Vector2D } from './Vector2D';

export interface AABB {
  min: Vector2D;
  max: Vector2D;
}

export interface Polygon {
  vertices: Vector2D[];
  position: Vector2D;
  rotation: number;
}

export interface CollisionResult {
  colliding: boolean;
  penetrationDepth: number;
  normal: Vector2D;
  contactPoint: Vector2D;
  contactPoints: Vector2D[];
}

export class CollisionDetector {
  /**
   * Test AABB collision between two axis-aligned bounding boxes
   */
  static testAABB(aabb1: AABB, aabb2: AABB): boolean {
    return !(
      aabb1.max.x < aabb2.min.x ||
      aabb1.min.x > aabb2.max.x ||
      aabb1.max.y < aabb2.min.y ||
      aabb1.min.y > aabb2.max.y
    );
  }

  /**
   * Create AABB from polygon vertices
   */
  static createAABB(vertices: Vector2D[]): AABB {
    if (vertices.length === 0) {
      return {
        min: new Vector2D(0, 0),
        max: new Vector2D(0, 0)
      };
    }

    let minX = vertices[0].x;
    let minY = vertices[0].y;
    let maxX = vertices[0].x;
    let maxY = vertices[0].y;

    for (let i = 1; i < vertices.length; i++) {
      minX = Math.min(minX, vertices[i].x);
      minY = Math.min(minY, vertices[i].y);
      maxX = Math.max(maxX, vertices[i].x);
      maxY = Math.max(maxY, vertices[i].y);
    }

    return {
      min: new Vector2D(minX, minY),
      max: new Vector2D(maxX, maxY)
    };
  }

  /**
   * Get world-space vertices of a polygon
   */
  static getWorldVertices(polygon: Polygon): Vector2D[] {
    return polygon.vertices.map(v => {
      const rotated = v.rotate(polygon.rotation);
      return rotated.add(polygon.position);
    });
  }

  /**
   * Get edge normals for SAT
   */
  private static getEdgeNormals(vertices: Vector2D[]): Vector2D[] {
    const normals: Vector2D[] = [];

    for (let i = 0; i < vertices.length; i++) {
      const v1 = vertices[i];
      const v2 = vertices[(i + 1) % vertices.length];
      const edge = v2.subtract(v1);
      const normal = edge.perpendicular().normalize();
      normals.push(normal);
    }

    return normals;
  }

  /**
   * Project polygon onto axis and return min/max
   */
  private static projectPolygon(vertices: Vector2D[], axis: Vector2D): { min: number; max: number } {
    let min = vertices[0].dot(axis);
    let max = min;

    for (let i = 1; i < vertices.length; i++) {
      const projection = vertices[i].dot(axis);
      min = Math.min(min, projection);
      max = Math.max(max, projection);
    }

    return { min, max };
  }

  /**
   * Test if two projection ranges overlap
   */
  private static projectionsOverlap(proj1: { min: number; max: number }, proj2: { min: number; max: number }): number {
    if (proj1.max < proj2.min || proj2.max < proj1.min) {
      return 0; // No overlap
    }

    // Calculate overlap amount
    const overlap1 = proj1.max - proj2.min;
    const overlap2 = proj2.max - proj1.min;
    return Math.min(overlap1, overlap2);
  }

  /**
   * Separating Axis Theorem (SAT) collision detection
   * Returns detailed collision information
   */
  static testPolygonCollision(poly1: Polygon, poly2: Polygon): CollisionResult {
    const vertices1 = this.getWorldVertices(poly1);
    const vertices2 = this.getWorldVertices(poly2);

    // Get all potential separating axes (edge normals from both polygons)
    const normals1 = this.getEdgeNormals(vertices1);
    const normals2 = this.getEdgeNormals(vertices2);
    const axes = [...normals1, ...normals2];

    let minPenetration = Infinity;
    let collisionNormal = new Vector2D(0, 0);

    // Test each axis
    for (const axis of axes) {
      const proj1 = this.projectPolygon(vertices1, axis);
      const proj2 = this.projectPolygon(vertices2, axis);

      const overlap = this.projectionsOverlap(proj1, proj2);

      if (overlap === 0) {
        // Found separating axis - no collision
        return {
          colliding: false,
          penetrationDepth: 0,
          normal: new Vector2D(0, 0),
          contactPoint: new Vector2D(0, 0),
          contactPoints: []
        };
      }

      // Track minimum penetration
      if (overlap < minPenetration) {
        minPenetration = overlap;
        collisionNormal = axis.clone();

        // Ensure normal points from poly1 to poly2
        const centerDiff = poly2.position.subtract(poly1.position);
        if (collisionNormal.dot(centerDiff) < 0) {
          collisionNormal = collisionNormal.negate();
        }
      }
    }

    // Collision detected - calculate contact points
    const contactPoints = this.findContactPoints(vertices1, vertices2, collisionNormal);
    const contactPoint = this.calculateAverageContactPoint(contactPoints);

    return {
      colliding: true,
      penetrationDepth: minPenetration,
      normal: collisionNormal,
      contactPoint,
      contactPoints
    };
  }

  /**
   * Find contact points between two polygons
   */
  private static findContactPoints(vertices1: Vector2D[], vertices2: Vector2D[], normal: Vector2D): Vector2D[] {
    const contactPoints: Vector2D[] = [];
    const tolerance = 0.01;

    // Find reference and incident edges
    const { refVertices, incVertices } = this.findReferenceEdge(vertices1, vertices2, normal);

    // Clip incident edge against reference edge
    for (const v of incVertices) {
      // Check if vertex is on or behind the reference edge
      for (let i = 0; i < refVertices.length; i++) {
        const v1 = refVertices[i];
        const v2 = refVertices[(i + 1) % refVertices.length];
        const edge = v2.subtract(v1);
        const edgeNormal = edge.perpendicular().normalize();

        const toVertex = v.subtract(v1);
        const distance = toVertex.dot(edgeNormal);

        if (distance <= tolerance) {
          contactPoints.push(v);
          break;
        }
      }
    }

    // If no contact points found, use closest vertices
    if (contactPoints.length === 0) {
      const closest = this.findClosestVertices(vertices1, vertices2);
      contactPoints.push(...closest);
    }

    return contactPoints;
  }

  /**
   * Find reference and incident edges for contact point calculation
   */
  private static findReferenceEdge(
    vertices1: Vector2D[],
    vertices2: Vector2D[],
    normal: Vector2D
  ): { refVertices: Vector2D[]; incVertices: Vector2D[] } {
    // For simplicity, use all vertices
    return {
      refVertices: vertices1,
      incVertices: vertices2
    };
  }

  /**
   * Find closest vertices between two polygons
   */
  private static findClosestVertices(vertices1: Vector2D[], vertices2: Vector2D[]): Vector2D[] {
    let minDist = Infinity;
    const closest: Vector2D[] = [];

    for (const v1 of vertices1) {
      for (const v2 of vertices2) {
        const dist = v1.distanceToSquared(v2);
        if (dist < minDist) {
          minDist = dist;
          closest.length = 0;
          closest.push(v1, v2);
        }
      }
    }

    return closest;
  }

  /**
   * Calculate average contact point from multiple contact points
   */
  private static calculateAverageContactPoint(contactPoints: Vector2D[]): Vector2D {
    if (contactPoints.length === 0) {
      return new Vector2D(0, 0);
    }

    let sumX = 0;
    let sumY = 0;

    for (const point of contactPoints) {
      sumX += point.x;
      sumY += point.y;
    }

    return new Vector2D(
      sumX / contactPoints.length,
      sumY / contactPoints.length
    );
  }

  /**
   * Test circle vs circle collision
   */
  static testCircleCollision(
    center1: Vector2D,
    radius1: number,
    center2: Vector2D,
    radius2: number
  ): CollisionResult {
    const diff = center2.subtract(center1);
    const distance = diff.magnitude();
    const radiusSum = radius1 + radius2;

    if (distance >= radiusSum) {
      return {
        colliding: false,
        penetrationDepth: 0,
        normal: new Vector2D(0, 0),
        contactPoint: new Vector2D(0, 0),
        contactPoints: []
      };
    }

    const penetration = radiusSum - distance;
    const normal = distance > 0 ? diff.normalize() : new Vector2D(1, 0);
    const contactPoint = center1.add(normal.multiply(radius1 - penetration / 2));

    return {
      colliding: true,
      penetrationDepth: penetration,
      normal,
      contactPoint,
      contactPoints: [contactPoint]
    };
  }

  /**
   * Test point inside polygon
   */
  static pointInPolygon(point: Vector2D, vertices: Vector2D[]): boolean {
    let inside = false;

    for (let i = 0, j = vertices.length - 1; i < vertices.length; j = i++) {
      const vi = vertices[i];
      const vj = vertices[j];

      const intersect = ((vi.y > point.y) !== (vj.y > point.y)) &&
        (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x);

      if (intersect) inside = !inside;
    }

    return inside;
  }

  /**
   * Calculate closest point on line segment to a point
   */
  static closestPointOnSegment(point: Vector2D, segmentStart: Vector2D, segmentEnd: Vector2D): Vector2D {
    const segment = segmentEnd.subtract(segmentStart);
    const toPoint = point.subtract(segmentStart);

    const segmentLengthSquared = segment.magnitudeSquared();

    if (segmentLengthSquared === 0) {
      return segmentStart.clone();
    }

    const t = Math.max(0, Math.min(1, toPoint.dot(segment) / segmentLengthSquared));
    return segmentStart.add(segment.multiply(t));
  }

  /**
   * Ray cast against polygon
   */
  static rayCast(
    rayOrigin: Vector2D,
    rayDirection: Vector2D,
    vertices: Vector2D[]
  ): { hit: boolean; distance: number; point: Vector2D; normal: Vector2D } | null {
    let closestHit: { distance: number; point: Vector2D; normal: Vector2D } | null = null;
    const rayDir = rayDirection.normalize();

    for (let i = 0; i < vertices.length; i++) {
      const v1 = vertices[i];
      const v2 = vertices[(i + 1) % vertices.length];

      const hit = this.raySegmentIntersection(rayOrigin, rayDir, v1, v2);

      if (hit && (closestHit === null || hit.distance < closestHit.distance)) {
        const edge = v2.subtract(v1);
        const normal = edge.perpendicular().normalize();
        closestHit = { ...hit, normal };
      }
    }

    if (closestHit) {
      return { hit: true, ...closestHit };
    }

    return null;
  }

  /**
   * Ray-segment intersection
   */
  private static raySegmentIntersection(
    rayOrigin: Vector2D,
    rayDirection: Vector2D,
    segmentStart: Vector2D,
    segmentEnd: Vector2D
  ): { distance: number; point: Vector2D } | null {
    const v1 = rayOrigin;
    const v2 = rayOrigin.add(rayDirection);
    const v3 = segmentStart;
    const v4 = segmentEnd;

    const denom = (v1.x - v2.x) * (v3.y - v4.y) - (v1.y - v2.y) * (v3.x - v4.x);

    if (Math.abs(denom) < 0.0001) {
      return null; // Parallel
    }

    const t = ((v1.x - v3.x) * (v3.y - v4.y) - (v1.y - v3.y) * (v3.x - v4.x)) / denom;
    const u = -((v1.x - v2.x) * (v1.y - v3.y) - (v1.y - v2.y) * (v1.x - v3.x)) / denom;

    if (t >= 0 && u >= 0 && u <= 1) {
      const point = v1.add(rayDirection.multiply(t));
      return { distance: t, point };
    }

    return null;
  }
}
