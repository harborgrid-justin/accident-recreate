/**
 * Trajectory Component
 * 3D trajectory path visualization
 */

import React, { useEffect, useRef } from 'react';
import { TrajectoryProps } from '../types';

export const Trajectory: React.FC<TrajectoryProps> = ({
  points,
  color = '#0088ff',
  opacity = 0.8,
  showVelocityGradient = false,
  showKeyframes = false,
}) => {
  const trajectoryRef = useRef<any>(null);
  const keyframeMarkersRef = useRef<any[]>([]);

  useEffect(() => {
    const createTrajectory = async () => {
      const THREE = await import('three');

      if (points.length < 2) return;

      // Create trajectory line
      const positions: number[] = [];
      const colors: number[] = [];

      points.forEach(point => {
        positions.push(...point.position);

        if (showVelocityGradient) {
          // Color based on velocity
          const normalizedVelocity = Math.min(point.velocity / 100, 1);
          const r = normalizedVelocity;
          const g = 1 - normalizedVelocity;
          const b = 0.5;
          colors.push(r, g, b);
        }
      });

      const geometry = new THREE.BufferGeometry();
      geometry.setAttribute(
        'position',
        new THREE.Float32BufferAttribute(positions, 3)
      );

      if (showVelocityGradient) {
        geometry.setAttribute(
          'color',
          new THREE.Float32BufferAttribute(colors, 3)
        );
      }

      const material = new THREE.LineBasicMaterial({
        color: showVelocityGradient ? 0xffffff : color,
        vertexColors: showVelocityGradient,
        opacity: opacity,
        transparent: true,
        linewidth: 2,
      });

      const line = new THREE.Line(geometry, material);
      trajectoryRef.current = line;

      // Create keyframe markers
      if (showKeyframes) {
        const markers: any[] = [];
        points.forEach((point, index) => {
          if (index % 10 === 0) { // Show every 10th keyframe
            const markerGeometry = new THREE.SphereGeometry(0.1, 8, 8);
            const markerMaterial = new THREE.MeshBasicMaterial({
              color: color,
              transparent: true,
              opacity: opacity,
            });
            const marker = new THREE.Mesh(markerGeometry, markerMaterial);
            marker.position.set(...point.position);
            markers.push(marker);
          }
        });
        keyframeMarkersRef.current = markers;
      }
    };

    createTrajectory();

    return () => {
      if (trajectoryRef.current) {
        trajectoryRef.current.geometry?.dispose();
        trajectoryRef.current.material?.dispose();
      }
      keyframeMarkersRef.current.forEach(marker => {
        marker.geometry?.dispose();
        marker.material?.dispose();
      });
    };
  }, [points, color, opacity, showVelocityGradient, showKeyframes]);

  return null;
};

export default Trajectory;
