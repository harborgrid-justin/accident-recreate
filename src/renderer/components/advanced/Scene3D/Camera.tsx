/**
 * Camera Component
 * Manages camera controls and perspectives
 */

import React, { useEffect, useRef } from 'react';
import { CameraProps } from '../types';

export const Camera: React.FC<CameraProps> = ({
  position = [0, 5, 10],
  fov = 75,
  near = 0.1,
  far = 1000,
  lookAt = [0, 0, 0],
  enableControls = true,
  controlsType = 'orbit',
  autoRotate = false,
  autoRotateSpeed = 1,
}) => {
  const cameraRef = useRef<any>(null);
  const controlsRef = useRef<any>(null);

  useEffect(() => {
    const initCamera = async () => {
      const THREE = await import('three');

      // Create perspective camera
      const camera = new THREE.PerspectiveCamera(
        fov,
        window.innerWidth / window.innerHeight,
        near,
        far
      );
      camera.position.set(...position);
      camera.lookAt(...lookAt);
      cameraRef.current = camera;

      // Initialize controls if enabled
      if (enableControls) {
        let Controls;
        switch (controlsType) {
          case 'fly':
            // Would import FlyControls in production
            break;
          case 'first-person':
            // Would import FirstPersonControls in production
            break;
          case 'orbit':
          default:
            // Would import OrbitControls in production
            // For now, just a placeholder
            break;
        }

        if (Controls) {
          const controls = new (Controls as any)(camera);
          if (autoRotate && 'autoRotate' in controls) {
            (controls as any).autoRotate = true;
            (controls as any).autoRotateSpeed = autoRotateSpeed;
          }
          controlsRef.current = controls;
        }
      }
    };

    initCamera();

    return () => {
      if (controlsRef.current && 'dispose' in controlsRef.current) {
        controlsRef.current.dispose();
      }
    };
  }, [position, fov, near, far, lookAt, enableControls, controlsType, autoRotate, autoRotateSpeed]);

  // Camera components don't render DOM elements in R3F
  return null;
};

export default Camera;
