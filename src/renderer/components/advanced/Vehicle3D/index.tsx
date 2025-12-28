/**
 * Vehicle3D Component
 * 3D vehicle model with interactions
 */

import React, { useEffect, useRef, useState } from 'react';
import { Vehicle3DProps } from '../types';
import './Vehicle3D.css';

export const Vehicle3D: React.FC<Vehicle3DProps> = ({
  vehicleId,
  modelUrl,
  position,
  rotation,
  scale = [1, 1, 1],
  color = '#ff0000',
  showDamage = false,
  showPhysics = false,
  showTrajectory = false,
  interactive = true,
  onClick,
  onHover,
}) => {
  const meshRef = useRef<any>(null);
  const [hovered, setHovered] = useState(false);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    const loadVehicle = async () => {
      const THREE = await import('three');

      // Load vehicle model
      if (modelUrl) {
        // In production, use GLTFLoader or similar
        // For now, create a simple box as placeholder
        const geometry = new THREE.BoxGeometry(4, 1.5, 2);
        const material = new THREE.MeshStandardMaterial({
          color: color,
          metalness: 0.7,
          roughness: 0.3,
        });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(...position);
        mesh.rotation.set(...rotation);
        mesh.scale.set(...scale);
        mesh.castShadow = true;
        mesh.receiveShadow = true;

        meshRef.current = mesh;
        setLoaded(true);
      } else {
        // Create default vehicle shape
        const geometry = new THREE.BoxGeometry(4, 1.5, 2);
        const material = new THREE.MeshStandardMaterial({
          color: color,
          metalness: 0.7,
          roughness: 0.3,
        });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(...position);
        mesh.rotation.set(...rotation);
        mesh.scale.set(...scale);
        mesh.castShadow = true;
        mesh.receiveShadow = true;

        meshRef.current = mesh;
        setLoaded(true);
      }
    };

    loadVehicle();

    return () => {
      if (meshRef.current) {
        meshRef.current.geometry?.dispose();
        meshRef.current.material?.dispose();
      }
    };
  }, [vehicleId, modelUrl, position, rotation, scale, color]);

  const handlePointerOver = () => {
    if (!interactive) return;
    setHovered(true);
    onHover?.(vehicleId, true);
  };

  const handlePointerOut = () => {
    if (!interactive) return;
    setHovered(false);
    onHover?.(vehicleId, false);
  };

  const handleClick = () => {
    if (!interactive) return;
    onClick?.(vehicleId);
  };

  return (
    <div
      className={`vehicle-3d ${hovered ? 'hovered' : ''} ${loaded ? 'loaded' : 'loading'}`}
      onPointerEnter={handlePointerOver}
      onPointerLeave={handlePointerOut}
      onClick={handleClick}
    >
      {!loaded && <div className="vehicle-loading">Loading vehicle...</div>}
    </div>
  );
};

export default Vehicle3D;
