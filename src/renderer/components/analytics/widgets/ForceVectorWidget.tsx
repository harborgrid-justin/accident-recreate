/**
 * AccuScene Enterprise v0.3.0 - Force Vector Widget
 * 3D visualization of force vectors at impact points
 */

import React, { useRef, useMemo } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Grid, PerspectiveCamera } from '@react-three/drei';
import * as THREE from 'three';
import { WidgetProps, AnalyticsData, ForceVector } from '../types';

interface ArrowProps {
  vector: ForceVector;
}

const ForceArrow: React.FC<ArrowProps> = ({ vector }) => {
  const arrowRef = useRef<THREE.Group>(null);

  // Calculate arrow direction and length
  const direction = useMemo(() => {
    return new THREE.Vector3(
      vector.direction.x,
      vector.direction.y,
      vector.direction.z
    ).normalize();
  }, [vector.direction]);

  const length = useMemo(() => {
    return Math.min(vector.magnitude / 1000, 5); // Scale down for visualization
  }, [vector.magnitude]);

  // Color based on force type
  const color = useMemo(() => {
    if (vector.color) return vector.color;
    switch (vector.type) {
      case 'impact':
        return '#ef4444';
      case 'friction':
        return '#f59e0b';
      case 'normal':
        return '#3b82f6';
      case 'drag':
        return '#8b5cf6';
      default:
        return '#10b981';
    }
  }, [vector.type, vector.color]);

  // Pulse animation for impact forces
  useFrame(({ clock }) => {
    if (arrowRef.current && vector.type === 'impact') {
      const scale = 1 + Math.sin(clock.getElapsedTime() * 2) * 0.1;
      arrowRef.current.scale.setScalar(scale);
    }
  });

  return (
    <group
      ref={arrowRef}
      position={[vector.origin.x, vector.origin.y, vector.origin.z]}
    >
      {/* Arrow shaft */}
      <mesh>
        <cylinderGeometry args={[0.05, 0.05, length, 8]} />
        <meshStandardMaterial color={color} />
      </mesh>

      {/* Arrow head */}
      <mesh position={[0, length / 2, 0]}>
        <coneGeometry args={[0.15, 0.3, 8]} />
        <meshStandardMaterial color={color} />
      </mesh>

      {/* Rotate to point in direction */}
      <group
        rotation={[
          Math.acos(direction.y),
          Math.atan2(direction.x, direction.z),
          0,
        ]}
      />

      {/* Label */}
      <mesh position={[0, length + 0.5, 0]}>
        <sphereGeometry args={[0.1, 16, 16]} />
        <meshBasicMaterial color={color} />
      </mesh>
    </group>
  );
};

const Scene: React.FC<{ vectors: ForceVector[] }> = ({ vectors }) => {
  return (
    <>
      <PerspectiveCamera makeDefault position={[10, 10, 10]} />
      <OrbitControls
        enablePan={true}
        enableZoom={true}
        enableRotate={true}
        maxDistance={50}
        minDistance={2}
      />

      {/* Lighting */}
      <ambientLight intensity={0.5} />
      <directionalLight position={[10, 10, 5]} intensity={1} />
      <directionalLight position={[-10, -10, -5]} intensity={0.5} />

      {/* Grid */}
      <Grid
        args={[20, 20]}
        cellSize={1}
        cellThickness={0.5}
        cellColor="#334155"
        sectionSize={5}
        sectionThickness={1}
        sectionColor="#475569"
        fadeDistance={30}
        fadeStrength={1}
        followCamera={false}
      />

      {/* Coordinate axes */}
      <axesHelper args={[5]} />

      {/* Force vectors */}
      {vectors.map((vector) => (
        <ForceArrow key={vector.id} vector={vector} />
      ))}
    </>
  );
};

const ForceVectorWidget: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  // Get force vectors from data
  const forceVectors = useMemo(() => {
    return data?.forceVectors || [];
  }, [data]);

  // Calculate statistics
  const stats = useMemo(() => {
    if (forceVectors.length === 0) return null;

    const totalMagnitude = forceVectors.reduce(
      (sum, v) => sum + v.magnitude,
      0
    );
    const avgMagnitude = totalMagnitude / forceVectors.length;
    const maxMagnitude = Math.max(...forceVectors.map((v) => v.magnitude));

    return {
      count: forceVectors.length,
      total: totalMagnitude,
      average: avgMagnitude,
      max: maxMagnitude,
    };
  }, [forceVectors]);

  if (forceVectors.length === 0) {
    return (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#94a3b8',
        }}
      >
        No force vector data available
      </div>
    );
  }

  return (
    <div style={{ width: '100%', height: '100%', position: 'relative' }}>
      {/* 3D Canvas */}
      <Canvas
        style={{ background: '#0f172a' }}
        gl={{ antialias: true, alpha: false }}
      >
        <Scene vectors={forceVectors} />
      </Canvas>

      {/* Statistics Overlay */}
      {stats && (
        <div
          style={{
            position: 'absolute',
            top: '1rem',
            right: '1rem',
            backgroundColor: 'rgba(30, 41, 59, 0.9)',
            border: '1px solid #334155',
            borderRadius: '4px',
            padding: '0.75rem',
            fontSize: '0.75rem',
            color: '#f1f5f9',
            backdropFilter: 'blur(4px)',
          }}
        >
          <div style={{ fontWeight: 600, marginBottom: '0.5rem' }}>
            Force Statistics
          </div>
          <div style={{ display: 'grid', gap: '0.25rem' }}>
            <div>
              <span style={{ color: '#94a3b8' }}>Count:</span>{' '}
              {stats.count}
            </div>
            <div>
              <span style={{ color: '#94a3b8' }}>Max:</span>{' '}
              {stats.max.toFixed(0)} N
            </div>
            <div>
              <span style={{ color: '#94a3b8' }}>Avg:</span>{' '}
              {stats.average.toFixed(0)} N
            </div>
            <div>
              <span style={{ color: '#94a3b8' }}>Total:</span>{' '}
              {stats.total.toFixed(0)} N
            </div>
          </div>
        </div>
      )}

      {/* Legend */}
      <div
        style={{
          position: 'absolute',
          bottom: '1rem',
          left: '1rem',
          backgroundColor: 'rgba(30, 41, 59, 0.9)',
          border: '1px solid #334155',
          borderRadius: '4px',
          padding: '0.75rem',
          fontSize: '0.75rem',
          color: '#f1f5f9',
          backdropFilter: 'blur(4px)',
        }}
      >
        <div style={{ fontWeight: 600, marginBottom: '0.5rem' }}>
          Force Types
        </div>
        <div style={{ display: 'grid', gap: '0.25rem' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                backgroundColor: '#ef4444',
                borderRadius: '2px',
              }}
            />
            Impact
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                backgroundColor: '#f59e0b',
                borderRadius: '2px',
              }}
            />
            Friction
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                backgroundColor: '#3b82f6',
                borderRadius: '2px',
              }}
            />
            Normal
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                backgroundColor: '#8b5cf6',
                borderRadius: '2px',
              }}
            />
            Drag
          </div>
        </div>
      </div>

      {/* Controls Help */}
      <div
        style={{
          position: 'absolute',
          top: '1rem',
          left: '1rem',
          backgroundColor: 'rgba(30, 41, 59, 0.9)',
          border: '1px solid #334155',
          borderRadius: '4px',
          padding: '0.5rem 0.75rem',
          fontSize: '0.65rem',
          color: '#94a3b8',
          backdropFilter: 'blur(4px)',
        }}
      >
        🖱️ Click + Drag: Rotate | Scroll: Zoom | Right Click: Pan
      </div>
    </div>
  );
};

export default ForceVectorWidget;
