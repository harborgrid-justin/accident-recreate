/**
 * AccuScene Enterprise v0.3.0
 * Volumetric Fog and Light Shafts
 */

import React, { useEffect, useRef } from 'react';
import { VolumetricConfig, WebGPUContext, WebGLContext } from './types';

interface VolumetricEffectsProps {
  config: VolumetricConfig;
  gpuContext?: WebGPUContext;
  glContext?: WebGLContext;
}

export const VolumetricEffects: React.FC<VolumetricEffectsProps> = ({
  config,
  gpuContext,
  glContext,
}) => {
  const volumeTextureRef = useRef<GPUTexture | WebGLTexture | null>(null);

  useEffect(() => {
    if (!config.enabled) return;

    setupVolumetric();
    return cleanup;
  }, [config, gpuContext, glContext]);

  const setupVolumetric = () => {
    if (gpuContext) {
      setupVolumetricGPU();
    } else if (glContext) {
      setupVolumetricGL();
    }
  };

  const setupVolumetricGPU = () => {
    if (!gpuContext) return;
    const { device } = gpuContext;

    volumeTextureRef.current = device.createTexture({
      size: { width: 128, height: 128, depthOrArrayLayers: 64 },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
      dimension: '3d',
    });
  };

  const setupVolumetricGL = () => {
    if (!glContext) return;
    const { gl } = glContext;

    const texture = gl.createTexture();
    if (!texture) return;

    gl.bindTexture(gl.TEXTURE_3D, texture);
    gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 128, 128, 64, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

    volumeTextureRef.current = texture;
  };

  const cleanup = () => {
    if (gpuContext && volumeTextureRef.current) {
      (volumeTextureRef.current as GPUTexture).destroy();
    } else if (glContext && volumeTextureRef.current) {
      glContext.gl.deleteTexture(volumeTextureRef.current as WebGLTexture);
    }
  };

  return null;
};

export default VolumetricEffects;
