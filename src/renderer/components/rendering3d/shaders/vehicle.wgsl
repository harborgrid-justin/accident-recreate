/**
 * AccuScene Enterprise v0.3.0
 * Vehicle Shader with Damage Visualization - WebGPU
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

struct VehicleMaterial {
  baseColor: vec3<f32>,
  metallic: f32,
  roughness: f32,
  damageIntensity: f32,
  scratchIntensity: f32,
  dirtIntensity: f32,
}

struct DamageData {
  impactPoint: vec3<f32>,
  impactRadius: f32,
  impactStrength: f32,
  deformationAxis: vec3<f32>,
  crackPattern: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<uniform> model: ModelUniforms;
@group(1) @binding(0) var<uniform> vehicleMaterial: VehicleMaterial;
@group(1) @binding(1) var<storage, read> damagePoints: array<DamageData>;
@group(1) @binding(2) var paintTexture: texture_2d<f32>;
@group(1) @binding(3) var damageTexture: texture_2d<f32>;
@group(1) @binding(4) var scratchTexture: texture_2d<f32>;
@group(1) @binding(5) var dirtTexture: texture_2d<f32>;
@group(1) @binding(6) var normalMap: texture_2d<f32>;
@group(1) @binding(7) var textureSampler: sampler;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) tangent: vec4<f32>,
}

struct VertexOutput {
  @builtin(position) clipPosition: vec4<f32>,
  @location(0) worldPosition: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) tangent: vec3<f32>,
  @location(4) bitangent: vec3<f32>,
  @location(5) damageAmount: f32,
}

const PI: f32 = 3.14159265359;

// Noise function for procedural damage
fn hash(p: vec2<f32>) -> f32 {
  var p3 = fract(vec3<f32>(p.xyx) * 0.13);
  p3 += dot(p3, p3.yzx + 3.333);
  return fract((p3.x + p3.y) * p3.z);
}

fn noise(x: vec2<f32>) -> f32 {
  let i = floor(x);
  let f = fract(x);

  let a = hash(i);
  let b = hash(i + vec2<f32>(1.0, 0.0));
  let c = hash(i + vec2<f32>(0.0, 1.0));
  let d = hash(i + vec2<f32>(1.0, 1.0));

  let u = f * f * (3.0 - 2.0 * f);

  return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

// Calculate damage at a vertex position
fn calculateDamage(worldPos: vec3<f32>) -> f32 {
  var totalDamage = 0.0;

  for (var i = 0u; i < arrayLength(&damagePoints); i = i + 1u) {
    let damage = damagePoints[i];
    let dist = distance(worldPos, damage.impactPoint);

    if (dist < damage.impactRadius) {
      let falloff = 1.0 - (dist / damage.impactRadius);
      let damageValue = damage.impactStrength * falloff * falloff;
      totalDamage += damageValue;
    }
  }

  return clamp(totalDamage, 0.0, 1.0);
}

// Generate crack patterns
fn crackPattern(uv: vec2<f32>, damageAmount: f32) -> f32 {
  if (damageAmount < 0.3) {
    return 0.0;
  }

  let scale = 20.0 + damageAmount * 30.0;
  let n1 = noise(uv * scale);
  let n2 = noise(uv * scale * 2.0 + vec2<f32>(100.0));

  let cracks = smoothstep(0.6, 0.65, n1) * smoothstep(0.55, 0.6, n2);
  return cracks * damageAmount;
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  var out: VertexOutput;

  let worldPos = model.model * vec4<f32>(in.position, 1.0);
  out.worldPosition = worldPos.xyz;

  // Calculate damage at this vertex
  out.damageAmount = calculateDamage(worldPos.xyz);

  // Apply deformation based on damage
  var deformedPos = worldPos.xyz;
  for (var i = 0u; i < arrayLength(&damagePoints); i = i + 1u) {
    let damage = damagePoints[i];
    let dist = distance(worldPos.xyz, damage.impactPoint);

    if (dist < damage.impactRadius) {
      let falloff = 1.0 - (dist / damage.impactRadius);
      let deformation = damage.deformationAxis * damage.impactStrength * falloff * falloff * falloff;
      deformedPos += deformation * 0.5; // Scale deformation amount
    }
  }

  out.clipPosition = camera.viewProjection * vec4<f32>(deformedPos, 1.0);

  out.normal = normalize((model.normalMatrix * vec4<f32>(in.normal, 0.0)).xyz);
  out.tangent = normalize((model.normalMatrix * vec4<f32>(in.tangent.xyz, 0.0)).xyz);
  out.bitangent = cross(out.normal, out.tangent) * in.tangent.w;
  out.uv = in.uv;

  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // Sample base textures
  let paintColor = textureSample(paintTexture, textureSampler, in.uv);
  let damageMap = textureSample(damageTexture, textureSampler, in.uv);
  let scratchMap = textureSample(scratchTexture, textureSampler, in.uv * 5.0);
  let dirtMap = textureSample(dirtTexture, textureSampler, in.uv * 3.0);
  let normalSample = textureSample(normalMap, textureSampler, in.uv);

  // Reconstruct normal from normal map
  let TBN = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
  let tangentNormal = normalSample.xyz * 2.0 - 1.0;
  let N = normalize(TBN * tangentNormal);

  // View direction
  let V = normalize(camera.position - in.worldPosition);

  // Base color mixing
  var baseColor = paintColor.rgb * vehicleMaterial.baseColor;

  // Apply damage effects
  let damageValue = in.damageAmount * vehicleMaterial.damageIntensity;

  // Crack patterns
  let cracks = crackPattern(in.uv, damageValue);

  // Mix in damage texture
  baseColor = mix(baseColor, damageMap.rgb * 0.3, damageValue * 0.8);

  // Add cracks (darker lines)
  baseColor = mix(baseColor, vec3<f32>(0.1, 0.05, 0.0), cracks);

  // Scratches
  let scratchMask = scratchMap.r * vehicleMaterial.scratchIntensity;
  baseColor = mix(baseColor, vec3<f32>(0.4, 0.4, 0.4), scratchMask * 0.3);

  // Dirt accumulation
  let dirtMask = dirtMap.r * vehicleMaterial.dirtIntensity;
  baseColor = mix(baseColor, vec3<f32>(0.2, 0.15, 0.1), dirtMask * 0.5);

  // Modify roughness based on damage
  var roughness = vehicleMaterial.roughness;
  roughness = mix(roughness, 0.9, damageValue * 0.5); // Damaged areas are rougher

  // Modify metallic based on damage
  var metallic = vehicleMaterial.metallic;
  metallic = mix(metallic, 0.1, damageValue * 0.7); // Damaged areas less metallic

  // Simple lighting (can be replaced with full PBR)
  let lightDir = normalize(vec3<f32>(1.0, 2.0, 1.0));
  let lightColor = vec3<f32>(1.0, 0.98, 0.95);

  let NdotL = max(dot(N, lightDir), 0.0);
  let diffuse = baseColor * lightColor * NdotL;

  let H = normalize(V + lightDir);
  let NdotH = max(dot(N, H), 0.0);
  let specular = pow(NdotH, (1.0 - roughness) * 128.0) * metallic;

  let ambient = baseColor * 0.3;

  var finalColor = ambient + diffuse + vec3<f32>(specular);

  // Fresnel for reflections
  let fresnel = pow(1.0 - max(dot(N, V), 0.0), 5.0);
  finalColor += vec3<f32>(fresnel * metallic * 0.2);

  // Gamma correction
  finalColor = pow(finalColor, vec3<f32>(1.0 / 2.2));

  return vec4<f32>(finalColor, 1.0);
}
