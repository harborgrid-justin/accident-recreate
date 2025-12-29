/**
 * AccuScene Enterprise v0.3.0
 * Hierarchical Frustum Culling System
 */

import { Camera, Frustum, AABB, Sphere, RenderableObject, Vec3 } from './types';

export class FrustumCuller {
  private frustum: Frustum;

  constructor() {
    this.frustum = { planes: [] };
  }

  // Extract frustum planes from view-projection matrix
  public extractFrustum(viewProjectionMatrix: Float32Array): void {
    const m = viewProjectionMatrix;

    this.frustum.planes = [
      // Left
      {
        normal: { x: m[3] + m[0], y: m[7] + m[4], z: m[11] + m[8] },
        distance: m[15] + m[12],
      },
      // Right
      {
        normal: { x: m[3] - m[0], y: m[7] - m[4], z: m[11] - m[8] },
        distance: m[15] - m[12],
      },
      // Bottom
      {
        normal: { x: m[3] + m[1], y: m[7] + m[5], z: m[11] + m[9] },
        distance: m[15] + m[13],
      },
      // Top
      {
        normal: { x: m[3] - m[1], y: m[7] - m[5], z: m[11] - m[9] },
        distance: m[15] - m[13],
      },
      // Near
      {
        normal: { x: m[3] + m[2], y: m[7] + m[6], z: m[11] + m[10] },
        distance: m[15] + m[14],
      },
      // Far
      {
        normal: { x: m[3] - m[2], y: m[7] - m[6], z: m[11] - m[10] },
        distance: m[15] - m[14],
      },
    ];

    // Normalize planes
    this.frustum.planes.forEach(plane => {
      const length = Math.sqrt(
        plane.normal.x * plane.normal.x +
        plane.normal.y * plane.normal.y +
        plane.normal.z * plane.normal.z
      );

      plane.normal.x /= length;
      plane.normal.y /= length;
      plane.normal.z /= length;
      plane.distance /= length;
    });
  }

  // Update frustum from camera
  public updateFromCamera(camera: Camera): void {
    if (camera.viewProjectionMatrix) {
      this.extractFrustum(camera.viewProjectionMatrix);
    } else {
      // Calculate view-projection matrix if not provided
      const viewProjection = this.calculateViewProjectionMatrix(camera);
      this.extractFrustum(viewProjection);
    }
  }

  // Test AABB against frustum
  public testAABB(aabb: AABB): boolean {
    for (const plane of this.frustum.planes) {
      const { normal, distance } = plane;

      // Get positive and negative vertices
      const px = normal.x >= 0 ? aabb.max.x : aabb.min.x;
      const py = normal.y >= 0 ? aabb.max.y : aabb.min.y;
      const pz = normal.z >= 0 ? aabb.max.z : aabb.min.z;

      const d = normal.x * px + normal.y * py + normal.z * pz + distance;

      if (d < 0) {
        return false; // Outside frustum
      }
    }

    return true; // Inside or intersecting frustum
  }

  // Test sphere against frustum
  public testSphere(sphere: Sphere): boolean {
    for (const plane of this.frustum.planes) {
      const { normal, distance } = plane;

      const d =
        normal.x * sphere.center.x +
        normal.y * sphere.center.y +
        normal.z * sphere.center.z +
        distance;

      if (d < -sphere.radius) {
        return false; // Outside frustum
      }
    }

    return true; // Inside or intersecting frustum
  }

  // Cull renderable objects
  public cullObjects(objects: RenderableObject[]): RenderableObject[] {
    return objects.filter(obj => {
      if (!obj.visible) return false;

      // Test bounding box
      return this.testAABB(obj.mesh.bounds);
    });
  }

  // Hierarchical culling with bounding volume hierarchy
  public cullHierarchical(
    objects: RenderableObject[],
    hierarchy?: Map<string, { bounds: AABB; children: string[] }>
  ): RenderableObject[] {
    if (!hierarchy) {
      return this.cullObjects(objects);
    }

    const visibleIds = new Set<string>();

    const testNode = (nodeId: string): boolean => {
      const node = hierarchy.get(nodeId);
      if (!node) return false;

      if (!this.testAABB(node.bounds)) {
        return false; // Node not visible
      }

      // Add all children to visible set
      node.children.forEach(childId => {
        const childNode = hierarchy.get(childId);
        if (childNode) {
          if (testNode(childId)) {
            visibleIds.add(childId);
          }
        } else {
          // Leaf node (actual object)
          visibleIds.add(childId);
        }
      });

      return true;
    };

    // Test root nodes
    hierarchy.forEach((node, id) => {
      if (!node.children.length) return; // Skip leaf nodes
      testNode(id);
    });

    // Filter objects
    return objects.filter(obj => visibleIds.has(obj.id));
  }

  // Calculate view-projection matrix from camera
  private calculateViewProjectionMatrix(camera: Camera): Float32Array {
    const view = this.createLookAtMatrix(camera.position, camera.target, camera.up);
    const projection = this.createPerspectiveMatrix(
      camera.fov,
      camera.aspect,
      camera.near,
      camera.far
    );

    return this.multiplyMatrices(projection, view);
  }

  // Create look-at matrix
  private createLookAtMatrix(eye: Vec3, target: Vec3, up: Vec3): Float32Array {
    const zAxis = this.normalize({
      x: eye.x - target.x,
      y: eye.y - target.y,
      z: eye.z - target.z,
    });

    const xAxis = this.normalize(this.cross(up, zAxis));
    const yAxis = this.cross(zAxis, xAxis);

    const matrix = new Float32Array(16);
    matrix[0] = xAxis.x;
    matrix[1] = yAxis.x;
    matrix[2] = zAxis.x;
    matrix[3] = 0;
    matrix[4] = xAxis.y;
    matrix[5] = yAxis.y;
    matrix[6] = zAxis.y;
    matrix[7] = 0;
    matrix[8] = xAxis.z;
    matrix[9] = yAxis.z;
    matrix[10] = zAxis.z;
    matrix[11] = 0;
    matrix[12] = -this.dot(xAxis, eye);
    matrix[13] = -this.dot(yAxis, eye);
    matrix[14] = -this.dot(zAxis, eye);
    matrix[15] = 1;

    return matrix;
  }

  // Create perspective projection matrix
  private createPerspectiveMatrix(
    fov: number,
    aspect: number,
    near: number,
    far: number
  ): Float32Array {
    const f = 1.0 / Math.tan((fov * Math.PI) / 360);
    const rangeInv = 1.0 / (near - far);

    const matrix = new Float32Array(16);
    matrix[0] = f / aspect;
    matrix[5] = f;
    matrix[10] = (near + far) * rangeInv;
    matrix[11] = -1;
    matrix[14] = near * far * rangeInv * 2;
    matrix[15] = 0;

    return matrix;
  }

  // Helper functions
  private normalize(v: Vec3): Vec3 {
    const len = Math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
    if (len === 0) return { x: 0, y: 1, z: 0 };
    return { x: v.x / len, y: v.y / len, z: v.z / len };
  }

  private cross(a: Vec3, b: Vec3): Vec3 {
    return {
      x: a.y * b.z - a.z * b.y,
      y: a.z * b.x - a.x * b.z,
      z: a.x * b.y - a.y * b.x,
    };
  }

  private dot(a: Vec3, b: Vec3): number {
    return a.x * b.x + a.y * b.y + a.z * b.z;
  }

  private multiplyMatrices(a: Float32Array, b: Float32Array): Float32Array {
    const result = new Float32Array(16);
    for (let i = 0; i < 4; i++) {
      for (let j = 0; j < 4; j++) {
        result[i * 4 + j] =
          a[i * 4] * b[j] +
          a[i * 4 + 1] * b[4 + j] +
          a[i * 4 + 2] * b[8 + j] +
          a[i * 4 + 3] * b[12 + j];
      }
    }
    return result;
  }
}
