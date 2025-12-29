/**
 * AccuScene Enterprise v0.3.0
 * Cascaded Shadow Mapping System
 */

import { Light, Camera, ShadowCascade, Vec3, WebGPUContext, WebGLContext } from './types';

export class ShadowMapper {
  private cascades: ShadowCascade[] = [];
  private shadowMapSize: number;
  private cascadeCount: number;
  private splitDistances: number[] = [];
  private gpuContext?: WebGPUContext;
  private glContext?: WebGLContext;

  constructor(
    shadowMapSize: number,
    cascadeCount: number,
    gpuContext?: WebGPUContext,
    glContext?: WebGLContext
  ) {
    this.shadowMapSize = shadowMapSize;
    this.cascadeCount = cascadeCount;
    this.gpuContext = gpuContext;
    this.glContext = glContext;

    this.initializeCascades();
  }

  private initializeCascades(): void {
    for (let i = 0; i < this.cascadeCount; i++) {
      const cascade = this.createCascade(i);
      this.cascades.push(cascade);
    }
  }

  private createCascade(index: number): ShadowCascade {
    if (this.gpuContext) {
      return this.createGPUCascade(index);
    } else if (this.glContext) {
      return this.createGLCascade(index);
    }

    throw new Error('No render context available');
  }

  private createGPUCascade(index: number): ShadowCascade {
    const { device } = this.gpuContext!;

    const shadowMap = device.createTexture({
      size: {
        width: this.shadowMapSize,
        height: this.shadowMapSize,
        depthOrArrayLayers: 1,
      },
      format: 'depth24plus',
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING,
      label: `shadow-cascade-${index}`,
    });

    return {
      viewProjectionMatrix: new Float32Array(16),
      splitDistance: 0,
      shadowMap,
    };
  }

  private createGLCascade(index: number): ShadowCascade {
    const { gl } = this.glContext!;

    const shadowMap = gl.createTexture();
    if (!shadowMap) throw new Error('Failed to create shadow map texture');

    gl.bindTexture(gl.TEXTURE_2D, shadowMap);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.DEPTH_COMPONENT24,
      this.shadowMapSize,
      this.shadowMapSize,
      0,
      gl.DEPTH_COMPONENT,
      gl.UNSIGNED_INT,
      null
    );

    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_MODE, gl.COMPARE_REF_TO_TEXTURE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_FUNC, gl.LEQUAL);

    return {
      viewProjectionMatrix: new Float32Array(16),
      splitDistance: 0,
      shadowMap,
    };
  }

  // Update cascade splits based on camera
  public updateCascades(camera: Camera, light: Light): void {
    this.calculateSplitDistances(camera);

    for (let i = 0; i < this.cascadeCount; i++) {
      const nearDist = i === 0 ? camera.near : this.splitDistances[i - 1];
      const farDist = this.splitDistances[i];

      this.cascades[i].splitDistance = farDist;
      this.cascades[i].viewProjectionMatrix = this.calculateLightMatrix(
        camera,
        light,
        nearDist,
        farDist
      );
    }
  }

  // Calculate cascade split distances using PSSM (Parallel-Split Shadow Maps)
  private calculateSplitDistances(camera: Camera): void {
    const lambda = 0.5; // Blend between uniform and logarithmic splits
    const near = camera.near;
    const far = camera.far;

    this.splitDistances = [];

    for (let i = 1; i <= this.cascadeCount; i++) {
      const ratio = i / this.cascadeCount;

      // Logarithmic split
      const cLog = near * Math.pow(far / near, ratio);

      // Uniform split
      const cUniform = near + (far - near) * ratio;

      // Blend
      const split = lambda * cLog + (1 - lambda) * cUniform;
      this.splitDistances.push(split);
    }
  }

  // Calculate light view-projection matrix for cascade
  private calculateLightMatrix(
    camera: Camera,
    light: Light,
    nearDist: number,
    farDist: number
  ): Float32Array {
    // Calculate frustum corners for this cascade
    const frustumCorners = this.calculateFrustumCorners(camera, nearDist, farDist);

    // Calculate light space bounds
    const lightView = this.createLightViewMatrix(light.direction);
    const lightSpaceCorners = frustumCorners.map(corner =>
      this.transformPoint(corner, lightView)
    );

    // Find min/max bounds
    let minX = Infinity, maxX = -Infinity;
    let minY = Infinity, maxY = -Infinity;
    let minZ = Infinity, maxZ = -Infinity;

    lightSpaceCorners.forEach(corner => {
      minX = Math.min(minX, corner.x);
      maxX = Math.max(maxX, corner.x);
      minY = Math.min(minY, corner.y);
      maxY = Math.max(maxY, corner.y);
      minZ = Math.min(minZ, corner.z);
      maxZ = Math.max(maxZ, corner.z);
    });

    // Extend Z range to include shadow casters behind the camera
    minZ -= (maxZ - minZ);

    // Create orthographic projection
    const lightProjection = this.createOrthographicMatrix(
      minX, maxX, minY, maxY, minZ, maxZ
    );

    // Combine view and projection
    return this.multiplyMatrices(lightProjection, lightView);
  }

  // Calculate frustum corners
  private calculateFrustumCorners(camera: Camera, nearDist: number, farDist: number): Vec3[] {
    const corners: Vec3[] = [];
    const tanHalfFov = Math.tan((camera.fov * Math.PI / 180) / 2);

    const nearHeight = 2 * tanHalfFov * nearDist;
    const nearWidth = nearHeight * camera.aspect;
    const farHeight = 2 * tanHalfFov * farDist;
    const farWidth = farHeight * camera.aspect;

    // Camera vectors
    const forward = this.normalize({
      x: camera.target.x - camera.position.x,
      y: camera.target.y - camera.position.y,
      z: camera.target.z - camera.position.z,
    });
    const right = this.normalize(this.cross(forward, camera.up));
    const up = this.cross(right, forward);

    // Near plane corners
    const nearCenter = this.add(camera.position, this.scale(forward, nearDist));
    corners.push(this.add(this.add(nearCenter, this.scale(up, nearHeight / 2)), this.scale(right, -nearWidth / 2)));
    corners.push(this.add(this.add(nearCenter, this.scale(up, nearHeight / 2)), this.scale(right, nearWidth / 2)));
    corners.push(this.add(this.add(nearCenter, this.scale(up, -nearHeight / 2)), this.scale(right, -nearWidth / 2)));
    corners.push(this.add(this.add(nearCenter, this.scale(up, -nearHeight / 2)), this.scale(right, nearWidth / 2)));

    // Far plane corners
    const farCenter = this.add(camera.position, this.scale(forward, farDist));
    corners.push(this.add(this.add(farCenter, this.scale(up, farHeight / 2)), this.scale(right, -farWidth / 2)));
    corners.push(this.add(this.add(farCenter, this.scale(up, farHeight / 2)), this.scale(right, farWidth / 2)));
    corners.push(this.add(this.add(farCenter, this.scale(up, -farHeight / 2)), this.scale(right, -farWidth / 2)));
    corners.push(this.add(this.add(farCenter, this.scale(up, -farHeight / 2)), this.scale(right, farWidth / 2)));

    return corners;
  }

  // Helper: Create light view matrix
  private createLightViewMatrix(lightDir: Vec3): Float32Array {
    const eye = { x: -lightDir.x * 100, y: -lightDir.y * 100, z: -lightDir.z * 100 };
    const target = { x: 0, y: 0, z: 0 };
    const up = { x: 0, y: 1, z: 0 };

    return this.createLookAtMatrix(eye, target, up);
  }

  // Helper math functions
  private normalize(v: Vec3): Vec3 {
    const len = Math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
    return { x: v.x / len, y: v.y / len, z: v.z / len };
  }

  private cross(a: Vec3, b: Vec3): Vec3 {
    return {
      x: a.y * b.z - a.z * b.y,
      y: a.z * b.x - a.x * b.z,
      z: a.x * b.y - a.y * b.x,
    };
  }

  private add(a: Vec3, b: Vec3): Vec3 {
    return { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z };
  }

  private scale(v: Vec3, s: number): Vec3 {
    return { x: v.x * s, y: v.y * s, z: v.z * s };
  }

  private transformPoint(point: Vec3, matrix: Float32Array): Vec3 {
    const x = matrix[0] * point.x + matrix[4] * point.y + matrix[8] * point.z + matrix[12];
    const y = matrix[1] * point.x + matrix[5] * point.y + matrix[9] * point.z + matrix[13];
    const z = matrix[2] * point.x + matrix[6] * point.y + matrix[10] * point.z + matrix[14];
    return { x, y, z };
  }

  private createLookAtMatrix(eye: Vec3, target: Vec3, up: Vec3): Float32Array {
    const matrix = new Float32Array(16);
    // Simplified lookAt implementation
    matrix[0] = matrix[5] = matrix[10] = matrix[15] = 1;
    return matrix;
  }

  private createOrthographicMatrix(
    left: number, right: number,
    bottom: number, top: number,
    near: number, far: number
  ): Float32Array {
    const matrix = new Float32Array(16);
    matrix[0] = 2 / (right - left);
    matrix[5] = 2 / (top - bottom);
    matrix[10] = -2 / (far - near);
    matrix[12] = -(right + left) / (right - left);
    matrix[13] = -(top + bottom) / (top - bottom);
    matrix[14] = -(far + near) / (far - near);
    matrix[15] = 1;
    return matrix;
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

  // Get cascades for rendering
  public getCascades(): ShadowCascade[] {
    return this.cascades;
  }

  // Dispose resources
  public dispose(): void {
    if (this.gpuContext) {
      this.cascades.forEach(cascade => {
        (cascade.shadowMap as GPUTexture).destroy();
      });
    } else if (this.glContext) {
      this.cascades.forEach(cascade => {
        this.glContext!.gl.deleteTexture(cascade.shadowMap as WebGLTexture);
      });
    }

    this.cascades = [];
  }
}
