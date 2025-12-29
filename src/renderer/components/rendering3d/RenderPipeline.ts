/**
 * AccuScene Enterprise v0.3.0
 * Configurable Render Pipeline (Forward/Deferred)
 */

import {
  RenderPipelineConfig,
  WebGPUContext,
  WebGLContext,
  RenderContext,
  RenderPass,
} from './types';

export class RenderPipeline {
  private config: RenderPipelineConfig;
  private gpuContext?: WebGPUContext;
  private glContext?: WebGLContext;
  private passes: RenderPass[] = [];
  private drawCallCount = 0;
  private triangleCount = 0;

  // Render targets
  private gBufferPosition?: GPUTexture | WebGLTexture;
  private gBufferNormal?: GPUTexture | WebGLTexture;
  private gBufferAlbedo?: GPUTexture | WebGLTexture;
  private gBufferDepth?: GPUTexture | WebGLTexture;
  private hdrFramebuffer?: GPUTexture | WebGLFramebuffer;
  private shadowMaps: (GPUTexture | WebGLTexture)[] = [];

  constructor(
    config: RenderPipelineConfig,
    gpuContext?: WebGPUContext,
    glContext?: WebGLContext
  ) {
    this.config = config;
    this.gpuContext = gpuContext;
    this.glContext = glContext;

    this.initializePipeline();
  }

  private initializePipeline(): void {
    if (this.config.mode === 'deferred') {
      this.initializeDeferredPipeline();
    } else {
      this.initializeForwardPipeline();
    }
  }

  private initializeForwardPipeline(): void {
    this.passes = [
      {
        name: 'shadow',
        enabled: this.config.shadows,
        execute: this.shadowPass.bind(this),
      },
      {
        name: 'geometry',
        enabled: true,
        execute: this.geometryPass.bind(this),
      },
      {
        name: 'transparent',
        enabled: true,
        execute: this.transparentPass.bind(this),
      },
      {
        name: 'postprocess',
        enabled: this.config.postProcessing,
        execute: this.postProcessPass.bind(this),
      },
    ];
  }

  private initializeDeferredPipeline(): void {
    this.createGBuffer();

    this.passes = [
      {
        name: 'shadow',
        enabled: this.config.shadows,
        execute: this.shadowPass.bind(this),
      },
      {
        name: 'gbuffer',
        enabled: true,
        execute: this.gBufferPass.bind(this),
      },
      {
        name: 'lighting',
        enabled: true,
        execute: this.lightingPass.bind(this),
      },
      {
        name: 'transparent',
        enabled: true,
        execute: this.transparentPass.bind(this),
      },
      {
        name: 'postprocess',
        enabled: this.config.postProcessing,
        execute: this.postProcessPass.bind(this),
      },
    ];
  }

  private createGBuffer(): void {
    if (this.gpuContext) {
      const { device, format } = this.gpuContext;

      // Create G-Buffer textures
      const size = { width: 1920, height: 1080 }; // Should be configurable

      this.gBufferPosition = device.createTexture({
        size,
        format: 'rgba16float',
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING,
      });

      this.gBufferNormal = device.createTexture({
        size,
        format: 'rgba16float',
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING,
      });

      this.gBufferAlbedo = device.createTexture({
        size,
        format: 'rgba8unorm',
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING,
      });

      this.gBufferDepth = device.createTexture({
        size,
        format: 'depth24plus',
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING,
      });
    } else if (this.glContext) {
      const { gl } = this.glContext;

      // Create WebGL framebuffer for G-Buffer
      const fb = gl.createFramebuffer();
      if (!fb) throw new Error('Failed to create framebuffer');

      gl.bindFramebuffer(gl.FRAMEBUFFER, fb);

      // Position texture
      const positionTex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, positionTex);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA16F, 1920, 1080, 0, gl.RGBA, gl.FLOAT, null);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, positionTex, 0);
      this.gBufferPosition = positionTex;

      // Normal texture
      const normalTex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, normalTex);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA16F, 1920, 1080, 0, gl.RGBA, gl.FLOAT, null);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT1, gl.TEXTURE_2D, normalTex, 0);
      this.gBufferNormal = normalTex;

      // Albedo texture
      const albedoTex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, albedoTex);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 1920, 1080, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT2, gl.TEXTURE_2D, albedoTex, 0);
      this.gBufferAlbedo = albedoTex;

      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    }
  }

  private shadowPass(context: RenderContext): void {
    this.drawCallCount = 0;

    // Render shadow maps for each shadow-casting light
    const shadowLights = context.lights.filter(light => light.castShadow);

    shadowLights.forEach((light, index) => {
      if (this.gpuContext) {
        this.renderShadowMapGPU(context, light, index);
      } else if (this.glContext) {
        this.renderShadowMapGL(context, light, index);
      }
    });
  }

  private renderShadowMapGPU(context: RenderContext, light: any, index: number): void {
    // WebGPU shadow map rendering
    // This is a simplified version - full implementation would be more complex
    this.drawCallCount += context.renderables.filter(r => r.castShadow).length;
  }

  private renderShadowMapGL(context: RenderContext, light: any, index: number): void {
    // WebGL shadow map rendering
    const { gl } = this.glContext!;

    // Create shadow framebuffer if needed
    if (!this.shadowMaps[index]) {
      const shadowTex = gl.createTexture();
      if (!shadowTex) return;

      gl.bindTexture(gl.TEXTURE_2D, shadowTex);
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.DEPTH_COMPONENT24,
        this.config.shadowMapSize,
        this.config.shadowMapSize,
        0,
        gl.DEPTH_COMPONENT,
        gl.UNSIGNED_INT,
        null
      );
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

      this.shadowMaps[index] = shadowTex;
    }

    this.drawCallCount += context.renderables.filter(r => r.castShadow).length;
  }

  private gBufferPass(context: RenderContext): void {
    // Deferred rendering: write to G-Buffer
    if (this.gpuContext) {
      // WebGPU G-Buffer rendering
    } else if (this.glContext) {
      // WebGL G-Buffer rendering
    }

    this.drawCallCount += context.renderables.filter(r => r.visible).length;
  }

  private geometryPass(context: RenderContext): void {
    // Forward rendering: render opaque geometry
    this.drawCallCount += context.renderables.filter(r => r.visible).length;

    if (this.gpuContext) {
      this.renderGeometryGPU(context);
    } else if (this.glContext) {
      this.renderGeometryGL(context);
    }
  }

  private renderGeometryGPU(context: RenderContext): void {
    // WebGPU geometry rendering
    // Full implementation would include actual rendering commands
  }

  private renderGeometryGL(context: RenderContext): void {
    const { gl } = this.glContext!;

    // Clear
    gl.clearColor(0.1, 0.1, 0.15, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    // Render each object
    context.renderables.forEach(obj => {
      if (!obj.visible) return;

      this.triangleCount += obj.mesh.indices.length / 3;
      // Actual rendering would happen here
    });
  }

  private lightingPass(context: RenderContext): void {
    // Deferred rendering: lighting pass
    this.drawCallCount += 1; // Full-screen quad
  }

  private transparentPass(context: RenderContext): void {
    // Render transparent objects (forward rendering)
    this.drawCallCount += context.renderables.filter(r => r.visible).length;
  }

  private postProcessPass(context: RenderContext): void {
    // Post-processing effects
    if (this.config.bloom) {
      this.drawCallCount += 3; // Bright extract + 2 blur passes
    }
    if (this.config.ssao) {
      this.drawCallCount += 2; // SSAO + blur
    }
    this.drawCallCount += 1; // Final composite
  }

  public render(context: RenderContext): void {
    this.drawCallCount = 0;
    this.triangleCount = 0;

    // Execute all enabled passes
    this.passes.forEach(pass => {
      if (pass.enabled) {
        pass.execute(context);
      }
    });
  }

  public getDrawCallCount(): number {
    return this.drawCallCount;
  }

  public getTriangleCount(): number {
    return this.triangleCount;
  }

  public dispose(): void {
    // Cleanup resources
    if (this.gpuContext) {
      // Destroy GPU textures
      (this.gBufferPosition as GPUTexture)?.destroy?.();
      (this.gBufferNormal as GPUTexture)?.destroy?.();
      (this.gBufferAlbedo as GPUTexture)?.destroy?.();
      (this.gBufferDepth as GPUTexture)?.destroy?.();
      (this.hdrFramebuffer as GPUTexture)?.destroy?.();
      this.shadowMaps.forEach(sm => (sm as GPUTexture)?.destroy?.());
    } else if (this.glContext) {
      const { gl } = this.glContext;
      // Delete WebGL textures
      gl.deleteTexture(this.gBufferPosition as WebGLTexture);
      gl.deleteTexture(this.gBufferNormal as WebGLTexture);
      gl.deleteTexture(this.gBufferAlbedo as WebGLTexture);
      gl.deleteFramebuffer(this.hdrFramebuffer as WebGLFramebuffer);
      this.shadowMaps.forEach(sm => gl.deleteTexture(sm as WebGLTexture));
    }
  }
}
