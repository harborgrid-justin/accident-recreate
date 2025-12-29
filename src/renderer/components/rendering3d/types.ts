/**
 * AccuScene Enterprise v0.3.0 - Advanced 3D Rendering Pipeline
 * Type Definitions
 */

export interface Vec2 {
  x: number;
  y: number;
}

export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

export interface Vec4 {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface Quaternion {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface Transform {
  position: Vec3;
  rotation: Quaternion;
  scale: Vec3;
}

export interface AABB {
  min: Vec3;
  max: Vec3;
}

export interface Sphere {
  center: Vec3;
  radius: number;
}

export interface Frustum {
  planes: Array<{ normal: Vec3; distance: number }>;
}

export interface Material {
  albedo: Vec3;
  metallic: number;
  roughness: number;
  ao: number;
  emissive?: Vec3;
  albedoMap?: GPUTexture | WebGLTexture;
  normalMap?: GPUTexture | WebGLTexture;
  metallicRoughnessMap?: GPUTexture | WebGLTexture;
  aoMap?: GPUTexture | WebGLTexture;
  emissiveMap?: GPUTexture | WebGLTexture;
}

export interface Mesh {
  vertices: Float32Array;
  normals: Float32Array;
  uvs: Float32Array;
  tangents?: Float32Array;
  indices: Uint32Array;
  bounds: AABB;
}

export interface RenderableObject {
  id: string;
  mesh: Mesh;
  material: Material;
  transform: Transform;
  visible: boolean;
  castShadow: boolean;
  receiveShadow: boolean;
  layer: number;
}

export interface InstanceData {
  transform: Transform;
  color?: Vec4;
  customData?: Float32Array;
}

export interface Light {
  type: 'directional' | 'point' | 'spot';
  position: Vec3;
  direction: Vec3;
  color: Vec3;
  intensity: number;
  range?: number;
  innerAngle?: number;
  outerAngle?: number;
  castShadow: boolean;
}

export interface Camera {
  position: Vec3;
  target: Vec3;
  up: Vec3;
  fov: number;
  aspect: number;
  near: number;
  far: number;
  viewMatrix?: Float32Array;
  projectionMatrix?: Float32Array;
  viewProjectionMatrix?: Float32Array;
}

export interface RenderPipelineConfig {
  mode: 'forward' | 'deferred';
  msaa: boolean;
  msaaSamples: 1 | 2 | 4 | 8;
  hdr: boolean;
  shadows: boolean;
  shadowMapSize: number;
  shadowCascades: number;
  ssao: boolean;
  bloom: boolean;
  postProcessing: boolean;
}

export interface WebGPUContext {
  adapter: GPUAdapter;
  device: GPUDevice;
  context: GPUCanvasContext;
  format: GPUTextureFormat;
  capabilities: {
    maxTextureSize: number;
    maxBufferSize: number;
    maxBindGroups: number;
    maxComputeWorkgroups: number;
  };
}

export interface WebGLContext {
  gl: WebGL2RenderingContext;
  extensions: {
    drawBuffers?: WEBGL_draw_buffers;
    depthTexture?: WEBGL_depth_texture;
    textureFloat?: OES_texture_float;
    textureFloatLinear?: OES_texture_float_linear;
    textureHalfFloat?: OES_texture_half_float;
    anisotropic?: EXT_texture_filter_anisotropic;
    vao?: OES_vertex_array_object;
  };
}

export interface Shader {
  id: string;
  type: 'webgpu' | 'webgl';
  source: string;
  compiled?: boolean;
  webgpuModule?: GPUShaderModule;
  webglProgram?: WebGLProgram;
}

export interface RenderPass {
  name: string;
  enabled: boolean;
  execute: (context: RenderContext) => void;
}

export interface RenderContext {
  gpuContext?: WebGPUContext;
  glContext?: WebGLContext;
  camera: Camera;
  lights: Light[];
  renderables: RenderableObject[];
  deltaTime: number;
  frameNumber: number;
}

export interface RenderStats {
  fps: number;
  frameTime: number;
  drawCalls: number;
  triangles: number;
  vertices: number;
  instances: number;
  textures: number;
  buffers: number;
  memoryUsed: number;
  gpuTime: number;
}

export interface LODLevel {
  distance: number;
  mesh: Mesh;
  screenCoverage?: number;
}

export interface LODGroup {
  id: string;
  levels: LODLevel[];
  currentLevel: number;
  bounds: Sphere;
}

export interface Particle {
  position: Vec3;
  velocity: Vec3;
  acceleration: Vec3;
  color: Vec4;
  size: number;
  life: number;
  maxLife: number;
}

export interface ParticleEmitter {
  id: string;
  position: Vec3;
  rate: number;
  maxParticles: number;
  particleLife: number;
  initialVelocity: Vec3;
  velocityVariation: Vec3;
  gravity: Vec3;
  size: number;
  sizeVariation: number;
  color: Vec4;
  colorVariation: Vec4;
}

export interface Decal {
  id: string;
  position: Vec3;
  normal: Vec3;
  tangent: Vec3;
  size: Vec2;
  rotation: number;
  texture: GPUTexture | WebGLTexture;
  opacity: number;
}

export interface ShadowCascade {
  viewProjectionMatrix: Float32Array;
  splitDistance: number;
  shadowMap: GPUTexture | WebGLTexture;
}

export interface EnvironmentMapData {
  radianceMap: GPUTexture | WebGLTexture;
  irradianceMap: GPUTexture | WebGLTexture;
  brdfLUT: GPUTexture | WebGLTexture;
  intensity: number;
}

export interface GizmoConfig {
  type: 'translate' | 'rotate' | 'scale';
  size: number;
  highlight: 'x' | 'y' | 'z' | 'xy' | 'xz' | 'yz' | null;
}

export interface GridConfig {
  size: number;
  divisions: number;
  color: Vec4;
  fadeDistance: number;
  infinite: boolean;
}

export interface OcclusionQuery {
  id: string;
  object: RenderableObject;
  query?: GPUQuerySet | WebGLQuery;
  visible: boolean;
  lastChecked: number;
}

export interface TextureDescriptor {
  width: number;
  height: number;
  format: GPUTextureFormat | number;
  usage: GPUTextureUsageFlags | number;
  mipLevelCount?: number;
  sampleCount?: number;
  dimension?: '1d' | '2d' | '3d' | 'cube';
}

export interface BufferDescriptor {
  size: number;
  usage: GPUBufferUsageFlags | number;
  mappedAtCreation?: boolean;
}

export interface CameraControlState {
  distance: number;
  azimuth: number;
  elevation: number;
  target: Vec3;
  damping: number;
  zoomSpeed: number;
  rotateSpeed: number;
  panSpeed: number;
}

export interface RenderQuality {
  level: 'low' | 'medium' | 'high' | 'ultra';
  targetFPS: number;
  adaptiveQuality: boolean;
  shadowQuality: number;
  textureQuality: number;
  particleQuality: number;
  postProcessing: boolean;
}

export interface VolumetricConfig {
  enabled: boolean;
  samples: number;
  density: number;
  color: Vec3;
  lightShafts: boolean;
  scattering: number;
}

export interface OutlineConfig {
  enabled: boolean;
  color: Vec4;
  thickness: number;
  objects: string[];
}

export type RenderBackend = 'webgpu' | 'webgl2';

export interface RenderingCapabilities {
  backend: RenderBackend;
  maxAnisotropy: number;
  maxTextureSize: number;
  maxCubeMapSize: number;
  maxRenderTargets: number;
  maxVertexAttributes: number;
  maxUniformBufferBindings: number;
  supportedFormats: string[];
  computeShaders: boolean;
  multiDrawIndirect: boolean;
  conservativeRaster: boolean;
}
