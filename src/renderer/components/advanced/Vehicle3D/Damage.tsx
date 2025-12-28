/**
 * Damage Component
 * Visualizes vehicle damage points and severity
 */

import React, { useEffect, useRef } from 'react';
import { DamageProps } from '../types';

export const Damage: React.FC<DamageProps> = ({
  damages,
  showLabels = true,
  colorScale = ['#00ff00', '#ffff00', '#ff6600', '#ff0000'],
}) => {
  const damagePointsRef = useRef<any[]>([]);

  useEffect(() => {
    const createDamageVisualization = async () => {
      const THREE = await import('three');
      const points: any[] = [];

      damages.forEach((damage, index) => {
        // Get color based on severity
        const colorIndex = Math.floor(damage.severity * (colorScale.length - 1));
        const color = colorScale[colorIndex];

        // Create damage indicator sphere
        const geometry = new THREE.SphereGeometry(damage.radius, 16, 16);
        const material = new THREE.MeshBasicMaterial({
          color: color,
          transparent: true,
          opacity: 0.6,
          wireframe: damage.type === 'scratch',
        });
        const sphere = new THREE.Mesh(geometry, material);
        sphere.position.set(...damage.position);

        points.push(sphere);

        // Create label if enabled
        if (showLabels) {
          // In production, would create text sprite or HTML overlay
        }
      });

      damagePointsRef.current = points;
    };

    createDamageVisualization();

    return () => {
      damagePointsRef.current.forEach(point => {
        point.geometry?.dispose();
        point.material?.dispose();
      });
    };
  }, [damages, showLabels, colorScale]);

  return null;
};

export default Damage;
