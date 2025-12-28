/**
 * Environment Component
 * Environment mapping and HDRI backgrounds
 */

import React, { useEffect, useRef } from 'react';
import { EnvironmentProps } from '../types';

export const Environment: React.FC<EnvironmentProps> = ({
  preset = 'sunset',
  background = true,
  blur = 0,
}) => {
  const envMapRef = useRef<any>(null);

  useEffect(() => {
    const loadEnvironment = async () => {
      const THREE = await import('three');

      // In production, this would load actual HDRI environment maps
      // For now, we'll create a simple gradient skybox

      const envMapUrls = getEnvironmentUrls(preset);

      // Create cube texture loader
      const loader = new THREE.CubeTextureLoader();

      // Load environment map (placeholder)
      try {
        const cubeTexture = loader.load(envMapUrls);

        if (blur > 0) {
          // Apply blur to environment map
          cubeTexture.minFilter = THREE.LinearMipMapLinearFilter;
        }

        envMapRef.current = cubeTexture;
      } catch (error) {
        console.warn('Failed to load environment map:', error);
      }
    };

    loadEnvironment();

    return () => {
      if (envMapRef.current) {
        envMapRef.current.dispose();
      }
    };
  }, [preset, background, blur]);

  return null;
};

// Helper function to get environment map URLs based on preset
function getEnvironmentUrls(preset: string): string[] {
  // In production, these would point to actual HDRI files
  const basePath = '/assets/environments/' + preset;
  return [
    `${basePath}/px.png`,
    `${basePath}/nx.png`,
    `${basePath}/py.png`,
    `${basePath}/ny.png`,
    `${basePath}/pz.png`,
    `${basePath}/nz.png`,
  ];
}

export default Environment;
