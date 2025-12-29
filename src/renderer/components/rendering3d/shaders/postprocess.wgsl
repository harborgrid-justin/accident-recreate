/**
 * AccuScene Enterprise v0.3.0
 * Post-Processing Shader (Bloom, SSAO, Tone Mapping) - WebGPU
 */

struct PostProcessUniforms {
  resolution: vec2<f32>,
  time: f32,
  bloomIntensity: f32,
  bloomThreshold: f32,
  ssaoRadius: f32,
  ssaoIntensity: f32,
  vignetteIntensity: f32,
  exposure: f32,
  contrast: f32,
  saturation: f32,
  chromaticAberration: f32,
}

@group(0) @binding(0) var<uniform> uniforms: PostProcessUniforms;
@group(0) @binding(1) var sceneTexture: texture_2d<f32>;
@group(0) @binding(2) var depthTexture: texture_depth_2d;
@group(0) @binding(3) var normalTexture: texture_2d<f32>;
@group(0) @binding(4) var noiseTex: texture_2d<f32>;
@group(0) @binding(5) var textureSampler: sampler;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

const PI: f32 = 3.14159265359;
const SSAO_SAMPLES: u32 = 16u;
const BLUR_RADIUS: i32 = 4;

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertexIndex: u32) -> VertexOutput {
  var out: VertexOutput;

  // Full-screen triangle
  let x = f32((vertexIndex << 1u) & 2u);
  let y = f32(vertexIndex & 2u);

  out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
  out.uv = vec2<f32>(x, y);

  return out;
}

// Luminance calculation
fn luminance(color: vec3<f32>) -> f32 {
  return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// ACES Filmic tone mapping
fn acesToneMapping(color: vec3<f32>) -> vec3<f32> {
  let a = 2.51;
  let b = 0.03;
  let c = 2.43;
  let d = 0.59;
  let e = 0.14;
  return clamp((color * (a * color + b)) / (color * (c * color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// Reinhard tone mapping
fn reinhardToneMapping(color: vec3<f32>) -> vec3<f32> {
  return color / (color + vec3<f32>(1.0));
}

// Uncharted 2 tone mapping
fn uncharted2ToneMapping(color: vec3<f32>) -> vec3<f32> {
  let A = 0.15;
  let B = 0.50;
  let C = 0.10;
  let D = 0.20;
  let E = 0.02;
  let F = 0.30;
  let W = 11.2;

  let curr = ((color * (A * color + C * B) + D * E) / (color * (A * color + B) + D * F)) - E / F;
  let whiteScale = 1.0 / (((vec3<f32>(W) * (A * W + C * B) + D * E) / (vec3<f32>(W) * (A * W + B) + D * F)) - E / F);

  return curr * whiteScale;
}

// Hash function for SSAO
fn hash(n: f32) -> f32 {
  return fract(sin(n) * 43758.5453);
}

// Gaussian blur weight
fn gaussianWeight(x: f32, sigma: f32) -> f32 {
  return exp(-(x * x) / (2.0 * sigma * sigma)) / (sqrt(2.0 * PI) * sigma);
}

// SSAO calculation
fn calculateSSAO(uv: vec2<f32>, centerDepth: f32, centerNormal: vec3<f32>) -> f32 {
  var occlusion = 0.0;
  let radius = uniforms.ssaoRadius;
  let bias = 0.025;

  for (var i = 0u; i < SSAO_SAMPLES; i = i + 1u) {
    // Generate sample offset
    let angle = f32(i) / f32(SSAO_SAMPLES) * 2.0 * PI;
    let sampleDist = (f32(i) / f32(SSAO_SAMPLES)) * radius;

    let offset = vec2<f32>(cos(angle), sin(angle)) * sampleDist;
    let sampleUV = uv + offset / uniforms.resolution;

    // Sample depth
    let sampleDepth = textureSample(depthTexture, textureSampler, sampleUV);

    // Range check
    let rangeCheck = smoothstep(0.0, 1.0, radius / abs(centerDepth - sampleDepth));

    // Accumulate occlusion
    if (sampleDepth >= centerDepth + bias) {
      occlusion += rangeCheck;
    }
  }

  occlusion = 1.0 - (occlusion / f32(SSAO_SAMPLES));
  return pow(occlusion, uniforms.ssaoIntensity);
}

// Extract bright areas for bloom
@fragment
fn fs_bright_extract(in: VertexOutput) -> @location(0) vec4<f32> {
  let color = textureSample(sceneTexture, textureSampler, in.uv);
  let luma = luminance(color.rgb);

  if (luma > uniforms.bloomThreshold) {
    let bloomColor = color.rgb * (luma - uniforms.bloomThreshold);
    return vec4<f32>(bloomColor, 1.0);
  }

  return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

// Gaussian blur horizontal
@fragment
fn fs_blur_horizontal(in: VertexOutput) -> @location(0) vec4<f32> {
  var color = vec3<f32>(0.0);
  var weightSum = 0.0;
  let sigma = 2.0;

  for (var x = -BLUR_RADIUS; x <= BLUR_RADIUS; x = x + 1) {
    let offset = vec2<f32>(f32(x) / uniforms.resolution.x, 0.0);
    let sampleColor = textureSample(sceneTexture, textureSampler, in.uv + offset);
    let weight = gaussianWeight(f32(x), sigma);

    color += sampleColor.rgb * weight;
    weightSum += weight;
  }

  return vec4<f32>(color / weightSum, 1.0);
}

// Gaussian blur vertical
@fragment
fn fs_blur_vertical(in: VertexOutput) -> @location(0) vec4<f32> {
  var color = vec3<f32>(0.0);
  var weightSum = 0.0;
  let sigma = 2.0;

  for (var y = -BLUR_RADIUS; y <= BLUR_RADIUS; y = y + 1) {
    let offset = vec2<f32>(0.0, f32(y) / uniforms.resolution.y);
    let sampleColor = textureSample(sceneTexture, textureSampler, in.uv + offset);
    let weight = gaussianWeight(f32(y), sigma);

    color += sampleColor.rgb * weight;
    weightSum += weight;
  }

  return vec4<f32>(color / weightSum, 1.0);
}

// SSAO pass
@fragment
fn fs_ssao(in: VertexOutput) -> @location(0) vec4<f32> {
  let depth = textureSample(depthTexture, textureSampler, in.uv);
  let normal = textureSample(normalTexture, textureSampler, in.uv).xyz;

  let ssao = calculateSSAO(in.uv, depth, normal);

  return vec4<f32>(vec3<f32>(ssao), 1.0);
}

// Final composite with all post-processing
@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
  var uv = in.uv;

  // Chromatic aberration
  let aberration = uniforms.chromaticAberration;
  let r = textureSample(sceneTexture, textureSampler, uv + vec2<f32>(aberration, 0.0)).r;
  let g = textureSample(sceneTexture, textureSampler, uv).g;
  let b = textureSample(sceneTexture, textureSampler, uv - vec2<f32>(aberration, 0.0)).b;
  var color = vec3<f32>(r, g, b);

  // Exposure
  color *= uniforms.exposure;

  // Tone mapping
  color = acesToneMapping(color);

  // Contrast
  color = (color - 0.5) * uniforms.contrast + 0.5;
  color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

  // Saturation
  let gray = vec3<f32>(luminance(color));
  color = mix(gray, color, uniforms.saturation);

  // Vignette
  let vignetteDistance = length(uv - vec2<f32>(0.5));
  let vignette = smoothstep(0.8, 0.5, vignetteDistance);
  let vignetteAmount = mix(1.0, vignette, uniforms.vignetteIntensity);
  color *= vignetteAmount;

  // Gamma correction (already in sRGB if using sRGB textures, but for safety)
  color = pow(color, vec3<f32>(1.0 / 2.2));

  return vec4<f32>(color, 1.0);
}

// Bloom composite (adds bloom to scene)
@fragment
fn fs_bloom_composite(in: VertexOutput) -> @location(0) vec4<f32> {
  // This shader assumes sceneTexture is the original scene
  // and a second binding would have the blurred bloom
  let sceneColor = textureSample(sceneTexture, textureSampler, in.uv).rgb;

  // In a real implementation, you'd sample from a bloom texture here
  // For now, we'll return the scene color
  // let bloomColor = textureSample(bloomTexture, textureSampler, in.uv).rgb;
  // let finalColor = sceneColor + bloomColor * uniforms.bloomIntensity;

  return vec4<f32>(sceneColor, 1.0);
}

// Main post-process pass (simplified all-in-one)
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return fs_composite(in);
}
