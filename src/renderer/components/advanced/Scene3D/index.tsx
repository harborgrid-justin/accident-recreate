/**
 * Scene3D Component
 * Main 3D scene container using React Three Fiber patterns
 */

import React, { Suspense, useRef, useEffect } from 'react';
import { Scene3DProps } from '../types';
import './Scene3D.css';

export const Scene3D: React.FC<Scene3DProps> = ({
  children,
  backgroundColor = '#1a1a2e',
  fog,
  onSceneReady,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sceneRef = useRef<any>(null);
  const cameraRef = useRef<any>(null);
  const rendererRef = useRef<any>(null);
  const animationFrameRef = useRef<number | null>(null);

  useEffect(() => {
    if (!containerRef.current || !canvasRef.current) return;

    // In a real implementation, this would initialize Three.js
    // For now, we'll use a placeholder that demonstrates the pattern

    const initScene = async () => {
      const THREE = await import('three');

      // Create scene
      const scene = new THREE.Scene();
      scene.background = new THREE.Color(backgroundColor);
      sceneRef.current = scene;

      // Add fog if specified
      if (fog) {
        scene.fog = new THREE.Fog(
          fog.color,
          fog.near,
          fog.far
        );
      }

      // Create camera
      const camera = new THREE.PerspectiveCamera(
        75,
        containerRef.current!.clientWidth / containerRef.current!.clientHeight,
        0.1,
        1000
      );
      camera.position.set(0, 5, 10);
      camera.lookAt(0, 0, 0);
      cameraRef.current = camera;

      // Create renderer
      const renderer = new THREE.WebGLRenderer({
        canvas: canvasRef.current!,
        antialias: true,
        alpha: true,
      });
      renderer.setSize(
        containerRef.current!.clientWidth,
        containerRef.current!.clientHeight
      );
      renderer.setPixelRatio(window.devicePixelRatio);
      renderer.shadowMap.enabled = true;
      renderer.shadowMap.type = THREE.PCFSoftShadowMap;
      rendererRef.current = renderer;

      // Animation loop
      const animate = () => {
        animationFrameRef.current = requestAnimationFrame(animate);
        renderer.render(scene, camera);
      };
      animate();

      // Notify parent
      onSceneReady?.(scene);
    };

    initScene();

    // Handle resize
    const handleResize = () => {
      if (!containerRef.current || !cameraRef.current || !rendererRef.current) return;

      const width = containerRef.current.clientWidth;
      const height = containerRef.current.clientHeight;

      cameraRef.current.aspect = width / height;
      cameraRef.current.updateProjectionMatrix();
      rendererRef.current.setSize(width, height);
    };

    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      if (rendererRef.current) {
        rendererRef.current.dispose();
      }
    };
  }, [backgroundColor, fog, onSceneReady]);

  return (
    <div ref={containerRef} className="scene-3d-container">
      <canvas ref={canvasRef} className="scene-3d-canvas" />
      <Suspense fallback={<div className="scene-3d-loading">Loading 3D Scene...</div>}>
        {children}
      </Suspense>
    </div>
  );
};

export default Scene3D;
