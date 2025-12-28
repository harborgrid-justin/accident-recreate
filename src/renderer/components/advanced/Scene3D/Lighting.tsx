/**
 * Lighting Component
 * Comprehensive lighting system for 3D scenes
 */

import React, { useEffect, useRef } from 'react';
import { LightingProps } from '../types';

export const Lighting: React.FC<LightingProps> = ({
  ambient = { color: '#ffffff', intensity: 0.5 },
  directional = [],
  point = [],
  spot = [],
}) => {
  const lightsRef = useRef<any[]>([]);

  useEffect(() => {
    const initLights = async () => {
      const THREE = await import('three');
      const lights: any[] = [];

      // Ambient light
      if (ambient) {
        const ambientLight = new THREE.AmbientLight(
          ambient.color,
          ambient.intensity
        );
        lights.push(ambientLight);
      }

      // Directional lights
      directional.forEach(light => {
        const dirLight = new THREE.DirectionalLight(
          light.color,
          light.intensity
        );
        dirLight.position.set(...light.position);

        if (light.castShadow) {
          dirLight.castShadow = true;
          dirLight.shadow.mapSize.width = 2048;
          dirLight.shadow.mapSize.height = 2048;
          dirLight.shadow.camera.near = 0.5;
          dirLight.shadow.camera.far = 500;
        }

        lights.push(dirLight);
      });

      // Point lights
      point.forEach(light => {
        const pointLight = new THREE.PointLight(
          light.color,
          light.intensity,
          light.distance || 0,
          light.decay || 2
        );
        pointLight.position.set(...light.position);
        lights.push(pointLight);
      });

      // Spot lights
      spot.forEach(light => {
        const spotLight = new THREE.SpotLight(
          light.color,
          light.intensity
        );
        spotLight.position.set(...light.position);
        spotLight.angle = light.angle || Math.PI / 6;
        spotLight.penumbra = light.penumbra || 0.1;

        if (light.castShadow) {
          spotLight.castShadow = true;
          spotLight.shadow.mapSize.width = 2048;
          spotLight.shadow.mapSize.height = 2048;
        }

        lights.push(spotLight);
      });

      lightsRef.current = lights;
    };

    initLights();

    return () => {
      lightsRef.current.forEach(light => {
        if (light.dispose) {
          light.dispose();
        }
      });
    };
  }, [ambient, directional, point, spot]);

  return null;
};

export default Lighting;
