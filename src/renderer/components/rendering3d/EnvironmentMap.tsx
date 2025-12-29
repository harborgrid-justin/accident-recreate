/**
 * AccuScene Enterprise v0.3.0
 * HDR Environment Mapping and Reflections
 */

import React, { useEffect, useRef } from 'react';
import { EnvironmentMapData, WebGPUContext, WebGLContext } from './types';

interface EnvironmentMapProps {
  hdrUrl?: string;
  cubeMapUrls?: string[];
  intensity?: number;
  gpuContext?: WebGPUContext;
  glContext?: WebGLContext;
  onLoaded?: (data: EnvironmentMapData) => void;
}

export const EnvironmentMap: React.FC<EnvironmentMapProps> = ({
  hdrUrl,
  cubeMapUrls,
  intensity = 1.0,
  gpuContext,
  glContext,
  onLoaded,
}) => {
  const envDataRef = useRef<EnvironmentMapData | null>(null);

  useEffect(() => {
    if (cubeMapUrls) {
      loadCubeMap(cubeMapUrls);
    } else if (hdrUrl) {
      loadHDR(hdrUrl);
    }

    return () => {
      cleanup();
    };
  }, [hdrUrl, cubeMapUrls]);

  const loadCubeMap = async (urls: string[]) => {
    if (urls.length !== 6) {
      console.error('Environment map requires 6 cube faces');
      return;
    }

    try {
      const images = await Promise.all(
        urls.map(url => loadImage(url))
      );

      if (gpuContext) {
        createGPUCubeMap(images);
      } else if (glContext) {
        createGLCubeMap(images);
      }
    } catch (err) {
      console.error('Failed to load environment map:', err);
    }
  };

  const loadHDR = async (url: string) => {
    // HDR loading would require HDR image parser
    console.log('HDR loading:', url);
  };

  const loadImage = (url: string): Promise<HTMLImageElement> => {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.crossOrigin = 'anonymous';
      img.onload = () => resolve(img);
      img.onerror = reject;
      img.src = url;
    });
  };

  const createGPUCubeMap = (images: HTMLImageElement[]) => {
    if (!gpuContext) return;

    const { device } = gpuContext;
    const size = images[0].width;

    // Create radiance map (for reflections)
    const radianceMap = device.createTexture({
      size: { width: size, height: size, depthOrArrayLayers: 6 },
      format: 'rgba16float',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
      dimension: '2d',
    });

    // Upload cube faces
    images.forEach((image, face) => {
      const canvas = document.createElement('canvas');
      canvas.width = canvas.height = size;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      ctx.drawImage(image, 0, 0);
      const imageData = ctx.getImageData(0, 0, size, size);

      // Convert to Float16
      const float16Data = new Uint16Array(size * size * 4);
      // Simplified conversion - real implementation would use proper Float16 conversion
      for (let i = 0; i < imageData.data.length; i++) {
        float16Data[i] = imageData.data[i] * 256;
      }

      device.queue.writeTexture(
        { texture: radianceMap, origin: { x: 0, y: 0, z: face } },
        float16Data,
        { bytesPerRow: size * 8, rowsPerImage: size },
        { width: size, height: size }
      );
    });

    // Create irradiance map (for diffuse lighting)
    const irradianceMap = generateIrradianceMap(radianceMap);

    // Create BRDF LUT
    const brdfLUT = generateBRDFLUT();

    envDataRef.current = {
      radianceMap,
      irradianceMap,
      brdfLUT,
      intensity,
    };

    onLoaded?.(envDataRef.current);
  };

  const createGLCubeMap = (images: HTMLImageElement[]) => {
    if (!glContext) return;

    const { gl } = glContext;

    const radianceMap = gl.createTexture();
    if (!radianceMap) return;

    gl.bindTexture(gl.TEXTURE_CUBE_MAP, radianceMap);

    const faces = [
      gl.TEXTURE_CUBE_MAP_POSITIVE_X,
      gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
      gl.TEXTURE_CUBE_MAP_POSITIVE_Y,
      gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
      gl.TEXTURE_CUBE_MAP_POSITIVE_Z,
      gl.TEXTURE_CUBE_MAP_NEGATIVE_Z,
    ];

    images.forEach((image, i) => {
      gl.texImage2D(faces[i], 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
    });

    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.generateMipmap(gl.TEXTURE_CUBE_MAP);

    // Simplified - would create irradiance and BRDF LUT
    envDataRef.current = {
      radianceMap,
      irradianceMap: radianceMap, // Placeholder
      brdfLUT: radianceMap as any, // Placeholder
      intensity,
    };

    onLoaded?.(envDataRef.current);
  };

  const generateIrradianceMap = (radianceMap: GPUTexture): GPUTexture => {
    if (!gpuContext) throw new Error('No GPU context');

    const { device } = gpuContext;

    // Create irradiance map (typically 32x32)
    const irradianceMap = device.createTexture({
      size: { width: 32, height: 32, depthOrArrayLayers: 6 },
      format: 'rgba16float',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
      dimension: '2d',
    });

    // Real implementation would use compute shader to convolve environment map
    return irradianceMap;
  };

  const generateBRDFLUT = (): GPUTexture => {
    if (!gpuContext) throw new Error('No GPU context');

    const { device } = gpuContext;

    // Create BRDF LUT (typically 512x512)
    const brdfLUT = device.createTexture({
      size: { width: 512, height: 512 },
      format: 'rg16float',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
    });

    // Real implementation would compute BRDF integration
    return brdfLUT;
  };

  const cleanup = () => {
    if (gpuContext && envDataRef.current) {
      (envDataRef.current.radianceMap as GPUTexture).destroy();
      (envDataRef.current.irradianceMap as GPUTexture).destroy();
      (envDataRef.current.brdfLUT as GPUTexture).destroy();
    } else if (glContext && envDataRef.current) {
      const { gl } = glContext;
      gl.deleteTexture(envDataRef.current.radianceMap as WebGLTexture);
      gl.deleteTexture(envDataRef.current.irradianceMap as WebGLTexture);
      gl.deleteTexture(envDataRef.current.brdfLUT as WebGLTexture);
    }
  };

  return null;
};

export default EnvironmentMap;
