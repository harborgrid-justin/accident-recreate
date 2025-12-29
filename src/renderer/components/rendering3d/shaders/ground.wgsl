/**
 * AccuScene Enterprise v0.3.0
 * Ground Shader with Tire Marks and Debris - WebGPU
 */

struct CameraUniforms {
  viewProjection: mat4x4<f32>,
  position: vec3<f32>,
  time: f32,
}

struct ModelUniforms {
  model: mat4x4<f32>,
  normalMatrix: mat4x4<f32>,
}

struct GroundMaterial {
  baseColor: vec3<f32>,
  roughness: f32,
  tireMarkIntensity: f32,
  debrisIntensity: f32,
  wetness: f32,
  gridVisible: f32,
}

struct TireMark {
  start: vec3<f32>,
  width: f32,
  end: vec3<f32>,
  intensity: f32,
  direction: vec3<f32>,
  fade: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<uniform> model: ModelUniforms;
@group(1) @binding(0) var<uniform> groundMaterial: GroundMaterial;
@group(1) @binding(1) var<storage, read> tireMarks: array<TireMark>;
@group(1) @binding(2) var asphaltTexture: texture_2d<f32>;
@group(1) @binding(3) var asphaltNormal: texture_2d<f32>;
@group(1) @binding(4) var tireMarkTexture: texture_2d<f32>;
@group(1) @binding(5) var debrisTexture: texture_2d<f32>;
@group(1) @binding(6) var textureSampler: sampler;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
}

struct VertexOutput {
  @builtin(position) clipPosition: vec4<f32>,
  @location(0) worldPosition: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) viewDistance: f32,
}

const PI: f32 = 3.14159265359;
const GRID_SIZE: f32 = 1.0;

// Hash function for procedural patterns
fn hash21(p: vec2<f32>) -> f32 {
  var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

// Procedural noise
fn noise(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);

  return mix(
    mix(hash21(i + vec2<f32>(0.0, 0.0)), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
    mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
    u.y
  );
}

// Calculate distance from point to line segment
fn distanceToLineSegment(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>) -> f32 {
  let pa = p - a;
  let ba = b - a;
  let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
  return length(pa - ba * h);
}

// Calculate tire mark contribution
fn getTireMarkContribution(worldPos: vec3<f32>) -> f32 {
  var markIntensity = 0.0;

  for (var i = 0u; i < arrayLength(&tireMarks); i = i + 1u) {
    let mark = tireMarks[i];
    let dist = distanceToLineSegment(worldPos, mark.start, mark.end);

    if (dist < mark.width) {
      let edgeFalloff = 1.0 - smoothstep(mark.width * 0.6, mark.width, dist);
      let lengthFactor = length(mark.end - mark.start);
      let fade = mark.fade * mark.intensity;

      markIntensity += edgeFalloff * fade;
    }
  }

  return clamp(markIntensity, 0.0, 1.0);
}

// Grid pattern for reference
fn gridPattern(worldPos: vec2<f32>, scale: f32) -> f32 {
  let grid = abs(fract(worldPos / scale - 0.5) - 0.5) / fwidth(worldPos / scale);
  let line = min(grid.x, grid.y);
  return 1.0 - min(line, 1.0);
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  var out: VertexOutput;

  let worldPos = model.model * vec4<f32>(in.position, 1.0);
  out.worldPosition = worldPos.xyz;
  out.clipPosition = camera.viewProjection * worldPos;
  out.normal = normalize((model.normalMatrix * vec4<f32>(in.normal, 0.0)).xyz);
  out.uv = in.uv;
  out.viewDistance = length(camera.position - worldPos.xyz);

  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // Sample base textures with tiling
  let tiling = 10.0;
  let asphalt = textureSample(asphaltTexture, textureSampler, in.worldPosition.xz * tiling);
  let asphaltNorm = textureSample(asphaltNormal, textureSampler, in.worldPosition.xz * tiling);

  // Base color
  var baseColor = asphalt.rgb * groundMaterial.baseColor;

  // Reconstruct normal
  let tangentNormal = asphaltNorm.xyz * 2.0 - 1.0;
  let N = normalize(in.normal + tangentNormal * 0.5);

  // View direction
  let V = normalize(camera.position - in.worldPosition);

  // Tire marks
  let tireMarkValue = getTireMarkContribution(in.worldPosition);
  if (tireMarkValue > 0.01) {
    let tireMarkSample = textureSample(tireMarkTexture, textureSampler, in.worldPosition.xz * 2.0);
    let markColor = vec3<f32>(0.05, 0.05, 0.05) * tireMarkSample.rgb;
    baseColor = mix(baseColor, markColor, tireMarkValue * groundMaterial.tireMarkIntensity);
  }

  // Debris and weathering
  let debrisNoise = noise(in.worldPosition.xz * 5.0);
  if (debrisNoise > 0.7) {
    let debrisSample = textureSample(debrisTexture, textureSampler, in.worldPosition.xz * 3.0);
    baseColor = mix(baseColor, debrisSample.rgb * 0.4, groundMaterial.debrisIntensity * 0.5);
  }

  // Wetness effect
  let wetMask = noise(in.worldPosition.xz * 2.0) * groundMaterial.wetness;
  baseColor = mix(baseColor, baseColor * 0.7, wetMask);

  // Grid overlay (fades with distance)
  let gridSize = 1.0;
  let grid = gridPattern(in.worldPosition.xz, gridSize);
  let gridFade = 1.0 - clamp(in.viewDistance / 50.0, 0.0, 1.0);
  let gridColor = vec3<f32>(0.3, 0.5, 0.7) * 0.5;
  baseColor = mix(baseColor, gridColor, grid * gridFade * groundMaterial.gridVisible * 0.3);

  // Lighting
  let lightDir = normalize(vec3<f32>(1.0, 2.0, 1.0));
  let lightColor = vec3<f32>(1.0, 0.98, 0.95);

  let NdotL = max(dot(N, lightDir), 0.0);
  let diffuse = baseColor * lightColor * NdotL;

  // Modified roughness for wet areas
  var roughness = groundMaterial.roughness;
  roughness = mix(roughness, 0.1, wetMask);

  // Specular
  let H = normalize(V + lightDir);
  let NdotH = max(dot(N, H), 0.0);
  let specular = pow(NdotH, (1.0 - roughness) * 64.0) * 0.2;

  // Ambient
  let ambient = baseColor * 0.4;

  // Ambient occlusion from cracks and debris
  let ao = 1.0 - (debrisNoise * 0.2);

  var finalColor = (ambient + diffuse) * ao + vec3<f32>(specular);

  // Distance fog
  let fogColor = vec3<f32>(0.7, 0.75, 0.8);
  let fogDensity = 0.005;
  let fogFactor = 1.0 - exp(-in.viewDistance * fogDensity);
  finalColor = mix(finalColor, fogColor, fogFactor * 0.3);

  // Gamma correction
  finalColor = pow(finalColor, vec3<f32>(1.0 / 2.2));

  return vec4<f32>(finalColor, 1.0);
}
