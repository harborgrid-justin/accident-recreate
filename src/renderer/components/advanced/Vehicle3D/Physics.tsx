/**
 * Physics Overlay Component
 * Displays physics vectors and center of mass
 */

import React, { useEffect, useRef } from 'react';
import { PhysicsOverlayProps } from '../types';

export const PhysicsOverlay: React.FC<PhysicsOverlayProps> = ({
  centerOfMass,
  forceVectors = [],
  velocityVector,
  showLabels = true,
}) => {
  const visualizationsRef = useRef<any[]>([]);

  useEffect(() => {
    const createPhysicsVisualization = async () => {
      const THREE = await import('three');
      const objects: any[] = [];

      // Center of mass indicator
      if (centerOfMass) {
        const geometry = new THREE.SphereGeometry(0.1, 16, 16);
        const material = new THREE.MeshBasicMaterial({ color: '#00ffff' });
        const sphere = new THREE.Mesh(geometry, material);
        sphere.position.set(...centerOfMass);
        objects.push(sphere);
      }

      // Force vectors
      forceVectors.forEach(vector => {
        const dir = new THREE.Vector3(...vector.direction).normalize();
        const origin = new THREE.Vector3(...vector.origin);
        const length = vector.magnitude;
        const color = vector.color || '#ff00ff';

        const arrow = new THREE.ArrowHelper(dir, origin, length, color, 0.3, 0.2);
        objects.push(arrow);
      });

      // Velocity vector
      if (velocityVector) {
        const dir = new THREE.Vector3(...velocityVector.direction).normalize();
        const origin = new THREE.Vector3(0, 0, 0);
        const length = velocityVector.magnitude;

        const arrow = new THREE.ArrowHelper(dir, origin, length, '#00ff00', 0.4, 0.3);
        objects.push(arrow);
      }

      visualizationsRef.current = objects;
    };

    createPhysicsVisualization();

    return () => {
      visualizationsRef.current.forEach(obj => {
        if (obj.geometry) obj.geometry.dispose();
        if (obj.material) obj.material.dispose();
      });
    };
  }, [centerOfMass, forceVectors, velocityVector, showLabels]);

  return null;
};

export default PhysicsOverlay;
