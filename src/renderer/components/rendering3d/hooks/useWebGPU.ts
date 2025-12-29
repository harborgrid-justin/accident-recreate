/**
 * AccuScene Enterprise v0.3.0
 * WebGPU Context Hook
 */

import { useState, useEffect } from 'react';
import { WebGPUContext } from '../types';

export function useWebGPU(canvas: HTMLCanvasElement | null): {
  context: WebGPUContext | null;
  supported: boolean;
  error: string | null;
} {
  const [context, setContext] = useState<WebGPUContext | null>(null);
  const [supported, setSupported] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!canvas) return;

    let mounted = true;

    const initialize = async () => {
      // Check WebGPU support
      if (!navigator.gpu) {
        setSupported(false);
        setError('WebGPU is not supported in this browser');
        return;
      }

      setSupported(true);

      try {
        // Request adapter
        const adapter = await navigator.gpu.requestAdapter({
          powerPreference: 'high-performance',
        });

        if (!adapter) {
          throw new Error('No WebGPU adapter available');
        }

        // Request device
        const device = await adapter.requestDevice({
          requiredFeatures: [],
          requiredLimits: {
            maxTextureDimension2D: 4096,
            maxBufferSize: 256 * 1024 * 1024,
          },
        });

        // Get canvas context
        const gpuContext = canvas.getContext('webgpu');
        if (!gpuContext) {
          throw new Error('Failed to get WebGPU context');
        }

        // Configure context
        const format = navigator.gpu.getPreferredCanvasFormat();
        gpuContext.configure({
          device,
          format,
          alphaMode: 'premultiplied',
        });

        // Get capabilities
        const limits = device.limits;
        const capabilities = {
          maxTextureSize: limits.maxTextureDimension2D,
          maxBufferSize: Number(limits.maxBufferSize),
          maxBindGroups: limits.maxBindGroups,
          maxComputeWorkgroups: limits.maxComputeWorkgroupsPerDimension,
        };

        if (mounted) {
          setContext({
            adapter,
            device,
            context: gpuContext,
            format,
            capabilities,
          });
        }
      } catch (err) {
        if (mounted) {
          const errorMessage = err instanceof Error ? err.message : String(err);
          setError(`Failed to initialize WebGPU: ${errorMessage}`);
        }
      }
    };

    initialize();

    return () => {
      mounted = false;
      if (context) {
        context.device.destroy();
      }
    };
  }, [canvas]);

  return { context, supported, error };
}
