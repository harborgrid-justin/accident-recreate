/**
 * AccuScene Enterprise v0.3.0
 * Physically-Based Rendering (PBR) Shader - WebGPU
 */

// Uniform bindings
struct CameraUniforms {
  viewProjection: mat4x4<f32>,
  position: vec3<f32>,
  time: f32,
}

struct ModelUniforms {
  model: mat4x4<f32>,
  normalMatrix: mat4x4<f32>,
}

struct MaterialUniforms {
  albedo: vec3<f32>,
  metallic: f32,
  roughness: f32,
  ao: f32,
  emissive: vec3<f32>,
  _padding: f32,
}

struct Light {
  position: vec3<f32>,
  type: u32,
  direction: vec3<f32>,
  range: f32,
  color: vec3<f32>,
  intensity: f32,
}

struct LightingUniforms {
  lights: array<Light, 8>,
  numLights: u32,
  ambientColor: vec3<f32>,
  ambientIntensity: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<uniform> model: ModelUniforms;
@group(1) @binding(0) var<uniform> material: MaterialUniforms;
@group(1) @binding(1) var albedoTexture: texture_2d<f32>;
@group(1) @binding(2) var normalTexture: texture_2d<f32>;
@group(1) @binding(3) var metallicRoughnessTexture: texture_2d<f32>;
@group(1) @binding(4) var aoTexture: texture_2d<f32>;
@group(1) @binding(5) var textureSampler: sampler;
@group(2) @binding(0) var<uniform> lighting: LightingUniforms;
@group(2) @binding(1) var irradianceMap: texture_cube<f32>;
@group(2) @binding(2) var radianceMap: texture_cube<f32>;
@group(2) @binding(3) var brdfLUT: texture_2d<f32>;
@group(2) @binding(4) var shadowMap: texture_depth_2d_array;
@group(2) @binding(5) var shadowSampler: sampler_comparison;

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
}

// Constants
const PI: f32 = 3.14159265359;
const EPSILON: f32 = 0.0001;

// Vertex Shader
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  var out: VertexOutput;

  let worldPos = model.model * vec4<f32>(in.position, 1.0);
  out.worldPosition = worldPos.xyz;
  out.clipPosition = camera.viewProjection * worldPos;

  out.normal = normalize((model.normalMatrix * vec4<f32>(in.normal, 0.0)).xyz);
  out.tangent = normalize((model.normalMatrix * vec4<f32>(in.tangent.xyz, 0.0)).xyz);
  out.bitangent = cross(out.normal, out.tangent) * in.tangent.w;
  out.uv = in.uv;

  return out;
}

// PBR Functions
fn distributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
  let a = roughness * roughness;
  let a2 = a * a;
  let NdotH = max(dot(N, H), 0.0);
  let NdotH2 = NdotH * NdotH;

  let nom = a2;
  var denom = (NdotH2 * (a2 - 1.0) + 1.0);
  denom = PI * denom * denom;

  return nom / max(denom, EPSILON);
}

fn geometrySchlickGGX(NdotV: f32, roughness: f32) -> f32 {
  let r = (roughness + 1.0);
  let k = (r * r) / 8.0;

  let nom = NdotV;
  let denom = NdotV * (1.0 - k) + k;

  return nom / max(denom, EPSILON);
}

fn geometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
  let NdotV = max(dot(N, V), 0.0);
  let NdotL = max(dot(N, L), 0.0);
  let ggx2 = geometrySchlickGGX(NdotV, roughness);
  let ggx1 = geometrySchlickGGX(NdotL, roughness);

  return ggx1 * ggx2;
}

fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
  return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn fresnelSchlickRoughness(cosTheta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
  return F0 + (max(vec3<f32>(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn getNormalFromMap(uv: vec2<f32>, worldPos: vec3<f32>, normal: vec3<f32>, tangent: vec3<f32>, bitangent: vec3<f32>) -> vec3<f32> {
  var tangentNormal = textureSample(normalTexture, textureSampler, uv).xyz * 2.0 - 1.0;

  let TBN = mat3x3<f32>(tangent, bitangent, normal);
  return normalize(TBN * tangentNormal);
}

fn calculateShadow(worldPos: vec3<f32>, cascadeIndex: u32) -> f32 {
  // Simple shadow mapping - can be enhanced with PCF
  let shadowCoord = vec3<f32>(worldPos.xy * 0.5 + 0.5, worldPos.z);
  let shadow = textureSampleCompare(shadowMap, shadowSampler, shadowCoord.xy, i32(cascadeIndex), shadowCoord.z);
  return shadow;
}

// Fragment Shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // Sample textures
  let albedoSample = textureSample(albedoTexture, textureSampler, in.uv);
  let metallicRoughness = textureSample(metallicRoughnessTexture, textureSampler, in.uv);
  let ao = textureSample(aoTexture, textureSampler, in.uv).r;

  // Material properties
  let albedo = pow(albedoSample.rgb * material.albedo, vec3<f32>(2.2)); // sRGB to linear
  let metallic = metallicRoughness.b * material.metallic;
  let roughness = metallicRoughness.g * material.roughness;

  // Normal mapping
  let N = getNormalFromMap(in.uv, in.worldPosition, in.normal, in.tangent, in.bitangent);
  let V = normalize(camera.position - in.worldPosition);
  let R = reflect(-V, N);

  // Calculate reflectance at normal incidence
  var F0 = vec3<f32>(0.04);
  F0 = mix(F0, albedo, metallic);

  // Reflectance equation
  var Lo = vec3<f32>(0.0);

  for (var i = 0u; i < lighting.numLights; i = i + 1u) {
    let light = lighting.lights[i];

    // Calculate per-light radiance
    var L: vec3<f32>;
    var attenuation: f32 = 1.0;

    if (light.type == 0u) { // Directional
      L = normalize(-light.direction);
    } else if (light.type == 1u) { // Point
      L = normalize(light.position - in.worldPosition);
      let distance = length(light.position - in.worldPosition);
      attenuation = 1.0 / (distance * distance);
      attenuation *= clamp(1.0 - pow(distance / light.range, 4.0), 0.0, 1.0);
    } else { // Spot
      L = normalize(light.position - in.worldPosition);
      let distance = length(light.position - in.worldPosition);
      attenuation = 1.0 / (distance * distance);
    }

    let H = normalize(V + L);
    let radiance = light.color * light.intensity * attenuation;

    // Cook-Torrance BRDF
    let NDF = distributionGGX(N, H, roughness);
    let G = geometrySmith(N, V, L, roughness);
    let F = fresnelSchlick(max(dot(H, V), 0.0), F0);

    let numerator = NDF * G * F;
    let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + EPSILON;
    let specular = numerator / denominator;

    // Energy conservation
    let kS = F;
    var kD = vec3<f32>(1.0) - kS;
    kD *= 1.0 - metallic;

    let NdotL = max(dot(N, L), 0.0);
    Lo += (kD * albedo / PI + specular) * radiance * NdotL;
  }

  // Ambient lighting (IBL)
  let F = fresnelSchlickRoughness(max(dot(N, V), 0.0), F0, roughness);
  let kS = F;
  var kD = 1.0 - kS;
  kD *= 1.0 - metallic;

  let irradiance = textureSample(irradianceMap, textureSampler, N).rgb;
  let diffuse = irradiance * albedo;

  let prefilteredColor = textureSampleLevel(radianceMap, textureSampler, R, roughness * 4.0).rgb;
  let envBRDF = textureSample(brdfLUT, textureSampler, vec2<f32>(max(dot(N, V), 0.0), roughness)).rg;
  let specular = prefilteredColor * (F * envBRDF.x + envBRDF.y);

  let ambient = (kD * diffuse + specular) * ao * material.ao;

  var color = ambient + Lo;

  // Add emissive
  color += material.emissive;

  // HDR tonemapping (Reinhard)
  color = color / (color + vec3<f32>(1.0));

  // Gamma correction
  color = pow(color, vec3<f32>(1.0 / 2.2));

  return vec4<f32>(color, albedoSample.a);
}
