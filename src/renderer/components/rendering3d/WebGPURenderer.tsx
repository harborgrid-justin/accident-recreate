/**
 * AccuScene Enterprise v0.3.0
 * WebGPU Renderer with WebGL2 Fallback
 */

import React, { useEffect, useRef, useState, useCallback } from 'react';
import { WebGPUContext, WebGLContext, RenderBackend, Camera, Light, RenderableObject, RenderStats } from './types';
import { useWebGPU } from './hooks/useWebGPU';
import { useRenderLoop } from './hooks/useRenderLoop';
import { RenderPipeline } from './RenderPipeline';
import { ShaderManager } from './ShaderManager';
import { TextureManager } from './TextureManager';

interface WebGPURendererProps {
  width: number;
  height: number;
  camera: Camera;
  lights: Light[];
  renderables: RenderableObject[];
  onStats?: (stats: RenderStats) => void;
  onReady?: (backend: RenderBackend) => void;
  className?: string;
  style?: React.CSSProperties;
}

export const WebGPURenderer: React.FC<WebGPURendererProps> = ({
  width,
  height,
  camera,
  lights,
  renderables,
  onStats,
  onReady,
  className,
  style,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [backend, setBackend] = useState<RenderBackend | null>(null);
  const [error, setError] = useState<string | null>(null);

  const gpuContextRef = useRef<WebGPUContext | null>(null);
  const glContextRef = useRef<WebGLContext | null>(null);
  const pipelineRef = useRef<RenderPipeline | null>(null);
  const shaderManagerRef = useRef<ShaderManager | null>(null);
  const textureManagerRef = useRef<TextureManager | null>(null);

  const statsRef = useRef<RenderStats>({
    fps: 0,
    frameTime: 0,
    drawCalls: 0,
    triangles: 0,
    vertices: 0,
    instances: 0,
    textures: 0,
    buffers: 0,
    memoryUsed: 0,
    gpuTime: 0,
  });

  const frameCountRef = useRef(0);
  const lastTimeRef = useRef(performance.now());
  const fpsUpdateRef = useRef(0);

  // Initialize WebGPU
  const initializeWebGPU = useCallback(async (canvas: HTMLCanvasElement): Promise<boolean> => {
    if (!navigator.gpu) {
      console.warn('WebGPU not supported, will fallback to WebGL2');
      return false;
    }

    try {
      const adapter = await navigator.gpu.requestAdapter({
        powerPreference: 'high-performance',
      });

      if (!adapter) {
        console.warn('No WebGPU adapter available');
        return false;
      }

      const device = await adapter.requestDevice({
        requiredFeatures: [],
        requiredLimits: {
          maxTextureDimension2D: 4096,
          maxBufferSize: 256 * 1024 * 1024, // 256MB
        },
      });

      const context = canvas.getContext('webgpu');
      if (!context) {
        console.warn('Failed to get WebGPU context');
        return false;
      }

      const format = navigator.gpu.getPreferredCanvasFormat();
      context.configure({
        device,
        format,
        alphaMode: 'premultiplied',
      });

      const limits = device.limits;
      gpuContextRef.current = {
        adapter,
        device,
        context,
        format,
        capabilities: {
          maxTextureSize: limits.maxTextureDimension2D,
          maxBufferSize: Number(limits.maxBufferSize),
          maxBindGroups: limits.maxBindGroups,
          maxComputeWorkgroups: limits.maxComputeWorkgroupsPerDimension,
        },
      };

      // Initialize managers
      shaderManagerRef.current = new ShaderManager('webgpu', device);
      textureManagerRef.current = new TextureManager('webgpu', device);
      pipelineRef.current = new RenderPipeline({
        mode: 'forward',
        msaa: true,
        msaaSamples: 4,
        hdr: true,
        shadows: true,
        shadowMapSize: 2048,
        shadowCascades: 3,
        ssao: true,
        bloom: true,
        postProcessing: true,
      }, gpuContextRef.current);

      console.log('WebGPU initialized successfully');
      setBackend('webgpu');
      onReady?.('webgpu');
      return true;
    } catch (err) {
      console.error('WebGPU initialization failed:', err);
      setError(`WebGPU init failed: ${err}`);
      return false;
    }
  }, [onReady]);

  // Initialize WebGL2 fallback
  const initializeWebGL = useCallback((canvas: HTMLCanvasElement): boolean => {
    try {
      const gl = canvas.getContext('webgl2', {
        alpha: false,
        antialias: true,
        depth: true,
        stencil: true,
        premultipliedAlpha: false,
        preserveDrawingBuffer: false,
        powerPreference: 'high-performance',
      });

      if (!gl) {
        throw new Error('WebGL2 not supported');
      }

      // Get extensions
      const extensions = {
        drawBuffers: gl.getExtension('WEBGL_draw_buffers'),
        depthTexture: gl.getExtension('WEBGL_depth_texture'),
        textureFloat: gl.getExtension('OES_texture_float'),
        textureFloatLinear: gl.getExtension('OES_texture_float_linear'),
        textureHalfFloat: gl.getExtension('OES_texture_half_float'),
        anisotropic: gl.getExtension('EXT_texture_filter_anisotropic'),
        vao: gl.getExtension('OES_vertex_array_object'),
      };

      glContextRef.current = { gl, extensions };

      // Initialize managers
      shaderManagerRef.current = new ShaderManager('webgl', undefined, gl);
      textureManagerRef.current = new TextureManager('webgl', undefined, gl);
      pipelineRef.current = new RenderPipeline({
        mode: 'forward',
        msaa: false,
        msaaSamples: 1,
        hdr: false,
        shadows: true,
        shadowMapSize: 1024,
        shadowCascades: 2,
        ssao: false,
        bloom: true,
        postProcessing: true,
      }, undefined, glContextRef.current);

      // Configure WebGL state
      gl.enable(gl.DEPTH_TEST);
      gl.enable(gl.CULL_FACE);
      gl.cullFace(gl.BACK);
      gl.frontFace(gl.CCW);
      gl.depthFunc(gl.LEQUAL);

      console.log('WebGL2 initialized successfully');
      setBackend('webgl2');
      onReady?.('webgl2');
      return true;
    } catch (err) {
      console.error('WebGL2 initialization failed:', err);
      setError(`WebGL2 init failed: ${err}`);
      return false;
    }
  }, [onReady]);

  // Initialize renderer
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    let mounted = true;

    const initialize = async () => {
      // Try WebGPU first
      const webgpuSuccess = await initializeWebGPU(canvas);

      if (!webgpuSuccess && mounted) {
        // Fallback to WebGL2
        initializeWebGL(canvas);
      }
    };

    initialize();

    return () => {
      mounted = false;

      // Cleanup
      if (gpuContextRef.current) {
        gpuContextRef.current.device.destroy();
      }

      if (glContextRef.current) {
        const ext = glContextRef.current.gl.getExtension('WEBGL_lose_context');
        ext?.loseContext();
      }

      pipelineRef.current?.dispose();
      shaderManagerRef.current?.dispose();
      textureManagerRef.current?.dispose();
    };
  }, [initializeWebGPU, initializeWebGL]);

  // Render loop
  const render = useCallback((deltaTime: number) => {
    if (!pipelineRef.current || !backend) return;

    const now = performance.now();
    frameCountRef.current++;

    // Update FPS
    fpsUpdateRef.current += deltaTime;
    if (fpsUpdateRef.current >= 1000) {
      statsRef.current.fps = Math.round((frameCountRef.current * 1000) / fpsUpdateRef.current);
      frameCountRef.current = 0;
      fpsUpdateRef.current = 0;
    }

    statsRef.current.frameTime = deltaTime;

    try {
      // Render frame
      pipelineRef.current.render({
        gpuContext: gpuContextRef.current ?? undefined,
        glContext: glContextRef.current ?? undefined,
        camera,
        lights,
        renderables,
        deltaTime,
        frameNumber: frameCountRef.current,
      });

      statsRef.current.drawCalls = pipelineRef.current.getDrawCallCount();
      statsRef.current.triangles = pipelineRef.current.getTriangleCount();
      statsRef.current.gpuTime = performance.now() - now;

      // Report stats
      if (onStats && frameCountRef.current % 60 === 0) {
        onStats({ ...statsRef.current });
      }
    } catch (err) {
      console.error('Render error:', err);
      setError(`Render error: ${err}`);
    }

    lastTimeRef.current = now;
  }, [backend, camera, lights, renderables, onStats]);

  useRenderLoop(render, backend !== null);

  // Handle resize
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    canvas.width = width * window.devicePixelRatio;
    canvas.height = height * window.devicePixelRatio;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;

    // Update camera aspect ratio
    camera.aspect = width / height;
  }, [width, height, camera]);

  if (error) {
    return (
      <div className={className} style={{ ...style, display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#1a1a1a', color: '#ff4444' }}>
        <div>
          <h3>Renderer Error</h3>
          <p>{error}</p>
        </div>
      </div>
    );
  }

  return (
    <canvas
      ref={canvasRef}
      className={className}
      style={style}
      width={width}
      height={height}
    />
  );
};
