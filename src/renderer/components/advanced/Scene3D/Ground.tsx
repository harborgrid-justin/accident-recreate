/**
 * Ground Component
 * Ground plane with optional grid
 */

import React, { useEffect, useRef } from 'react';
import { GroundProps } from '../types';

export const Ground: React.FC<GroundProps> = ({
  size = 100,
  gridSize = 10,
  color = '#2a2a3e',
  gridColor = '#404060',
  showGrid = true,
  receiveShadow = true,
}) => {
  const groundRef = useRef<any>(null);
  const gridRef = useRef<any>(null);

  useEffect(() => {
    const initGround = async () => {
      const THREE = await import('three');

      // Create ground plane
      const geometry = new THREE.PlaneGeometry(size, size);
      const material = new THREE.MeshStandardMaterial({
        color: color,
        roughness: 0.8,
        metalness: 0.2,
      });
      const ground = new THREE.Mesh(geometry, material);
      ground.rotation.x = -Math.PI / 2;
      ground.position.y = 0;
      ground.receiveShadow = receiveShadow;
      groundRef.current = ground;

      // Create grid if enabled
      if (showGrid) {
        const grid = new THREE.GridHelper(size, size / gridSize, gridColor, gridColor);
        grid.position.y = 0.01; // Slightly above ground to prevent z-fighting
        gridRef.current = grid;
      }
    };

    initGround();

    return () => {
      if (groundRef.current) {
        groundRef.current.geometry?.dispose();
        groundRef.current.material?.dispose();
      }
      if (gridRef.current) {
        gridRef.current.geometry?.dispose();
        gridRef.current.material?.dispose();
      }
    };
  }, [size, gridSize, color, gridColor, showGrid, receiveShadow]);

  return null;
};

export default Ground;
