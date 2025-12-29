/**
 * AccuScene Enterprise v0.3.0
 * Mesh Generation Utilities
 */

import { Mesh, AABB, Vec3 } from './types';

export class GeometryUtils {
  // Create box mesh
  public static createBox(width = 1, height = 1, depth = 1): Mesh {
    const w = width / 2;
    const h = height / 2;
    const d = depth / 2;

    const vertices = new Float32Array([
      // Front
      -w, -h, d,  w, -h, d,  w, h, d,  -w, h, d,
      // Back
      -w, -h, -d,  -w, h, -d,  w, h, -d,  w, -h, -d,
      // Top
      -w, h, -d,  -w, h, d,  w, h, d,  w, h, -d,
      // Bottom
      -w, -h, -d,  w, -h, -d,  w, -h, d,  -w, -h, d,
      // Right
      w, -h, -d,  w, h, -d,  w, h, d,  w, -h, d,
      // Left
      -w, -h, -d,  -w, -h, d,  -w, h, d,  -w, h, -d,
    ]);

    const normals = new Float32Array([
      // Front
      0, 0, 1,  0, 0, 1,  0, 0, 1,  0, 0, 1,
      // Back
      0, 0, -1,  0, 0, -1,  0, 0, -1,  0, 0, -1,
      // Top
      0, 1, 0,  0, 1, 0,  0, 1, 0,  0, 1, 0,
      // Bottom
      0, -1, 0,  0, -1, 0,  0, -1, 0,  0, -1, 0,
      // Right
      1, 0, 0,  1, 0, 0,  1, 0, 0,  1, 0, 0,
      // Left
      -1, 0, 0,  -1, 0, 0,  -1, 0, 0,  -1, 0, 0,
    ]);

    const uvs = new Float32Array([
      // Front
      0, 0,  1, 0,  1, 1,  0, 1,
      // Back
      0, 0,  1, 0,  1, 1,  0, 1,
      // Top
      0, 0,  1, 0,  1, 1,  0, 1,
      // Bottom
      0, 0,  1, 0,  1, 1,  0, 1,
      // Right
      0, 0,  1, 0,  1, 1,  0, 1,
      // Left
      0, 0,  1, 0,  1, 1,  0, 1,
    ]);

    const indices = new Uint32Array([
      0, 1, 2,  0, 2, 3,    // Front
      4, 5, 6,  4, 6, 7,    // Back
      8, 9, 10,  8, 10, 11,  // Top
      12, 13, 14,  12, 14, 15,  // Bottom
      16, 17, 18,  16, 18, 19,  // Right
      20, 21, 22,  20, 22, 23,  // Left
    ]);

    return {
      vertices,
      normals,
      uvs,
      indices,
      bounds: this.calculateBounds(vertices),
    };
  }

  // Create sphere mesh
  public static createSphere(radius = 1, segments = 32, rings = 16): Mesh {
    const vertices: number[] = [];
    const normals: number[] = [];
    const uvs: number[] = [];
    const indices: number[] = [];

    for (let ring = 0; ring <= rings; ring++) {
      const v = ring / rings;
      const theta = v * Math.PI;

      for (let segment = 0; segment <= segments; segment++) {
        const u = segment / segments;
        const phi = u * Math.PI * 2;

        const x = radius * Math.sin(theta) * Math.cos(phi);
        const y = radius * Math.cos(theta);
        const z = radius * Math.sin(theta) * Math.sin(phi);

        vertices.push(x, y, z);

        const nx = Math.sin(theta) * Math.cos(phi);
        const ny = Math.cos(theta);
        const nz = Math.sin(theta) * Math.sin(phi);

        normals.push(nx, ny, nz);
        uvs.push(u, v);
      }
    }

    for (let ring = 0; ring < rings; ring++) {
      for (let segment = 0; segment < segments; segment++) {
        const a = ring * (segments + 1) + segment;
        const b = a + segments + 1;

        indices.push(a, b, a + 1);
        indices.push(b, b + 1, a + 1);
      }
    }

    return {
      vertices: new Float32Array(vertices),
      normals: new Float32Array(normals),
      uvs: new Float32Array(uvs),
      indices: new Uint32Array(indices),
      bounds: this.calculateBounds(new Float32Array(vertices)),
    };
  }

  // Create plane mesh
  public static createPlane(width = 1, height = 1, widthSegments = 1, heightSegments = 1): Mesh {
    const vertices: number[] = [];
    const normals: number[] = [];
    const uvs: number[] = [];
    const indices: number[] = [];

    const w = width / 2;
    const h = height / 2;

    for (let y = 0; y <= heightSegments; y++) {
      const v = y / heightSegments;

      for (let x = 0; x <= widthSegments; x++) {
        const u = x / widthSegments;

        vertices.push(
          (u - 0.5) * width,
          0,
          (v - 0.5) * height
        );

        normals.push(0, 1, 0);
        uvs.push(u, v);
      }
    }

    for (let y = 0; y < heightSegments; y++) {
      for (let x = 0; x < widthSegments; x++) {
        const a = y * (widthSegments + 1) + x;
        const b = a + 1;
        const c = a + widthSegments + 1;
        const d = c + 1;

        indices.push(a, c, b);
        indices.push(b, c, d);
      }
    }

    return {
      vertices: new Float32Array(vertices),
      normals: new Float32Array(normals),
      uvs: new Float32Array(uvs),
      indices: new Uint32Array(indices),
      bounds: this.calculateBounds(new Float32Array(vertices)),
    };
  }

  // Create cylinder mesh
  public static createCylinder(
    radiusTop = 1,
    radiusBottom = 1,
    height = 1,
    radialSegments = 32,
    heightSegments = 1
  ): Mesh {
    const vertices: number[] = [];
    const normals: number[] = [];
    const uvs: number[] = [];
    const indices: number[] = [];

    const halfHeight = height / 2;

    for (let y = 0; y <= heightSegments; y++) {
      const v = y / heightSegments;
      const radius = v * radiusBottom + (1 - v) * radiusTop;

      for (let x = 0; x <= radialSegments; x++) {
        const u = x / radialSegments;
        const theta = u * Math.PI * 2;

        const px = radius * Math.cos(theta);
        const py = v * height - halfHeight;
        const pz = radius * Math.sin(theta);

        vertices.push(px, py, pz);

        const nx = Math.cos(theta);
        const nz = Math.sin(theta);
        const len = Math.sqrt(nx * nx + nz * nz);

        normals.push(nx / len, 0, nz / len);
        uvs.push(u, v);
      }
    }

    for (let y = 0; y < heightSegments; y++) {
      for (let x = 0; x < radialSegments; x++) {
        const a = y * (radialSegments + 1) + x;
        const b = a + radialSegments + 1;

        indices.push(a, b, a + 1);
        indices.push(b, b + 1, a + 1);
      }
    }

    return {
      vertices: new Float32Array(vertices),
      normals: new Float32Array(normals),
      uvs: new Float32Array(uvs),
      indices: new Uint32Array(indices),
      bounds: this.calculateBounds(new Float32Array(vertices)),
    };
  }

  // Calculate tangents for normal mapping
  public static calculateTangents(mesh: Mesh): Float32Array {
    const tangents = new Float32Array(mesh.vertices.length / 3 * 4);

    for (let i = 0; i < mesh.indices.length; i += 3) {
      const i0 = mesh.indices[i];
      const i1 = mesh.indices[i + 1];
      const i2 = mesh.indices[i + 2];

      const v0 = { x: mesh.vertices[i0 * 3], y: mesh.vertices[i0 * 3 + 1], z: mesh.vertices[i0 * 3 + 2] };
      const v1 = { x: mesh.vertices[i1 * 3], y: mesh.vertices[i1 * 3 + 1], z: mesh.vertices[i1 * 3 + 2] };
      const v2 = { x: mesh.vertices[i2 * 3], y: mesh.vertices[i2 * 3 + 1], z: mesh.vertices[i2 * 3 + 2] };

      const uv0 = { x: mesh.uvs[i0 * 2], y: mesh.uvs[i0 * 2 + 1] };
      const uv1 = { x: mesh.uvs[i1 * 2], y: mesh.uvs[i1 * 2 + 1] };
      const uv2 = { x: mesh.uvs[i2 * 2], y: mesh.uvs[i2 * 2 + 1] };

      const edge1 = { x: v1.x - v0.x, y: v1.y - v0.y, z: v1.z - v0.z };
      const edge2 = { x: v2.x - v0.x, y: v2.y - v0.y, z: v2.z - v0.z };

      const deltaUV1 = { x: uv1.x - uv0.x, y: uv1.y - uv0.y };
      const deltaUV2 = { x: uv2.x - uv0.x, y: uv2.y - uv0.y };

      const f = 1.0 / (deltaUV1.x * deltaUV2.y - deltaUV2.x * deltaUV1.y);

      const tangent = {
        x: f * (deltaUV2.y * edge1.x - deltaUV1.y * edge2.x),
        y: f * (deltaUV2.y * edge1.y - deltaUV1.y * edge2.y),
        z: f * (deltaUV2.y * edge1.z - deltaUV1.y * edge2.z),
      };

      tangents[i0 * 4] = tangents[i1 * 4] = tangents[i2 * 4] = tangent.x;
      tangents[i0 * 4 + 1] = tangents[i1 * 4 + 1] = tangents[i2 * 4 + 1] = tangent.y;
      tangents[i0 * 4 + 2] = tangents[i1 * 4 + 2] = tangents[i2 * 4 + 2] = tangent.z;
      tangents[i0 * 4 + 3] = tangents[i1 * 4 + 3] = tangents[i2 * 4 + 3] = 1.0;
    }

    return tangents;
  }

  // Calculate bounding box
  public static calculateBounds(vertices: Float32Array): AABB {
    let minX = Infinity, minY = Infinity, minZ = Infinity;
    let maxX = -Infinity, maxY = -Infinity, maxZ = -Infinity;

    for (let i = 0; i < vertices.length; i += 3) {
      const x = vertices[i];
      const y = vertices[i + 1];
      const z = vertices[i + 2];

      minX = Math.min(minX, x);
      minY = Math.min(minY, y);
      minZ = Math.min(minZ, z);

      maxX = Math.max(maxX, x);
      maxY = Math.max(maxY, y);
      maxZ = Math.max(maxZ, z);
    }

    return {
      min: { x: minX, y: minY, z: minZ },
      max: { x: maxX, y: maxY, z: maxZ },
    };
  }

  // Merge meshes
  public static mergeMeshes(meshes: Mesh[]): Mesh {
    let vertexCount = 0;
    let indexCount = 0;

    meshes.forEach(mesh => {
      vertexCount += mesh.vertices.length / 3;
      indexCount += mesh.indices.length;
    });

    const vertices = new Float32Array(vertexCount * 3);
    const normals = new Float32Array(vertexCount * 3);
    const uvs = new Float32Array(vertexCount * 2);
    const indices = new Uint32Array(indexCount);

    let vertexOffset = 0;
    let indexOffset = 0;
    let vertexIndex = 0;

    meshes.forEach(mesh => {
      vertices.set(mesh.vertices, vertexOffset);
      normals.set(mesh.normals, vertexOffset);
      uvs.set(mesh.uvs, (vertexOffset / 3) * 2);

      for (let i = 0; i < mesh.indices.length; i++) {
        indices[indexOffset + i] = mesh.indices[i] + vertexIndex;
      }

      vertexOffset += mesh.vertices.length;
      indexOffset += mesh.indices.length;
      vertexIndex += mesh.vertices.length / 3;
    });

    return {
      vertices,
      normals,
      uvs,
      indices,
      bounds: this.calculateBounds(vertices),
    };
  }
}
