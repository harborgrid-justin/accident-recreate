/**
 * AccuScene Enterprise v0.3.0
 * GPU Instanced Mesh Renderer for Many Similar Objects
 */

import React, { useEffect, useRef } from 'react';
import { Mesh, InstanceData, WebGPUContext, WebGLContext } from './types';

interface InstancedMeshRendererProps {
  mesh: Mesh;
  instances: InstanceData[];
  gpuContext?: WebGPUContext;
  glContext?: WebGLContext;
  maxInstances?: number;
}

export const InstancedMeshRenderer: React.FC<InstancedMeshRendererProps> = ({
  mesh,
  instances,
  gpuContext,
  glContext,
  maxInstances = 10000,
}) => {
  const instanceBufferRef = useRef<GPUBuffer | WebGLBuffer | null>(null);

  useEffect(() => {
    if (gpuContext) {
      setupInstancedRenderingGPU();
    } else if (glContext) {
      setupInstancedRenderingGL();
    }

    return () => {
      cleanup();
    };
  }, [gpuContext, glContext]);

  useEffect(() => {
    updateInstanceData();
  }, [instances]);

  const setupInstancedRenderingGPU = () => {
    if (!gpuContext) return;

    const { device } = gpuContext;

    // Create instance buffer
    instanceBufferRef.current = device.createBuffer({
      size: maxInstances * 64, // 4x4 matrix = 16 floats * 4 bytes
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
  };

  const setupInstancedRenderingGL = () => {
    if (!glContext) return;

    const { gl } = glContext;

    // Create instance buffer
    instanceBufferRef.current = gl.createBuffer();
  };

  const updateInstanceData = () => {
    if (!instanceBufferRef.current) return;

    const instanceMatrices = new Float32Array(instances.length * 16);

    instances.forEach((instance, index) => {
      const matrix = createTransformMatrix(instance.transform);
      instanceMatrices.set(matrix, index * 16);
    });

    if (gpuContext && instanceBufferRef.current instanceof GPUBuffer) {
      gpuContext.device.queue.writeBuffer(
        instanceBufferRef.current,
        0,
        instanceMatrices
      );
    } else if (glContext) {
      const { gl } = glContext;
      gl.bindBuffer(gl.ARRAY_BUFFER, instanceBufferRef.current as WebGLBuffer);
      gl.bufferData(gl.ARRAY_BUFFER, instanceMatrices, gl.DYNAMIC_DRAW);
    }
  };

  const createTransformMatrix = (transform: any): Float32Array => {
    // Simplified transform matrix creation
    const matrix = new Float32Array(16);
    matrix[0] = matrix[5] = matrix[10] = matrix[15] = 1;

    // Apply scale
    matrix[0] = transform.scale.x;
    matrix[5] = transform.scale.y;
    matrix[10] = transform.scale.z;

    // Apply translation
    matrix[12] = transform.position.x;
    matrix[13] = transform.position.y;
    matrix[14] = transform.position.z;

    return matrix;
  };

  const cleanup = () => {
    if (gpuContext && instanceBufferRef.current instanceof GPUBuffer) {
      (instanceBufferRef.current as GPUBuffer).destroy();
    } else if (glContext && instanceBufferRef.current) {
      glContext.gl.deleteBuffer(instanceBufferRef.current as WebGLBuffer);
    }
  };

  return null;
};

export default InstancedMeshRenderer;
