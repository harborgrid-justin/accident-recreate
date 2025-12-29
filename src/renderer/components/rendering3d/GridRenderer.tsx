/**
 * AccuScene Enterprise v0.3.0
 * Infinite Ground Grid with Anti-Aliasing
 */

import React, { useEffect, useRef } from 'react';
import { GridConfig, WebGPUContext, WebGLContext } from './types';

interface GridRendererProps {
  config: GridConfig;
  gpuContext?: WebGPUContext;
  glContext?: WebGLContext;
}

export const GridRenderer: React.FC<GridRendererProps> = ({
  config,
  gpuContext,
  glContext,
}) => {
  const gridBufferRef = useRef<GPUBuffer | WebGLBuffer | null>(null);

  useEffect(() => {
    createGridMesh();
    return cleanup;
  }, [config, gpuContext, glContext]);

  const createGridMesh = () => {
    if (config.infinite) {
      createInfiniteGrid();
    } else {
      createFiniteGrid();
    }
  };

  const createInfiniteGrid = () => {
    // Create infinite grid using shader-based approach
    if (gpuContext) {
      createInfiniteGridGPU();
    } else if (glContext) {
      createInfiniteGridGL();
    }
  };

  const createFiniteGrid = () => {
    const vertices: number[] = [];
    const step = config.size / config.divisions;

    for (let i = 0; i <= config.divisions; i++) {
      const pos = -config.size / 2 + i * step;

      // X lines
      vertices.push(-config.size / 2, 0, pos);
      vertices.push(config.size / 2, 0, pos);

      // Z lines
      vertices.push(pos, 0, -config.size / 2);
      vertices.push(pos, 0, config.size / 2);
    }

    if (gpuContext) {
      uploadGridGPU(new Float32Array(vertices));
    } else if (glContext) {
      uploadGridGL(new Float32Array(vertices));
    }
  };

  const createInfiniteGridGPU = () => {
    // Full-screen quad for shader-based infinite grid
  };

  const createInfiniteGridGL = () => {
    // Full-screen quad for shader-based infinite grid
  };

  const uploadGridGPU = (vertices: Float32Array) => {
    if (!gpuContext) return;
    const { device } = gpuContext;

    gridBufferRef.current = device.createBuffer({
      size: vertices.byteLength,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });

    device.queue.writeBuffer(gridBufferRef.current, 0, vertices);
  };

  const uploadGridGL = (vertices: Float32Array) => {
    if (!glContext) return;
    const { gl } = glContext;

    gridBufferRef.current = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, gridBufferRef.current);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);
  };

  const cleanup = () => {
    if (gpuContext && gridBufferRef.current) {
      (gridBufferRef.current as GPUBuffer).destroy();
    } else if (glContext && gridBufferRef.current) {
      glContext.gl.deleteBuffer(gridBufferRef.current as WebGLBuffer);
    }
  };

  return null;
};

export default GridRenderer;
