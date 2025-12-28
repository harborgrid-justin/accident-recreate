/**
 * useScene3D Hook
 * Manages 3D scene state and interactions
 */

import { useRef, useEffect, useState, useCallback } from 'react';
import * as THREE from 'three';

export interface UseScene3DOptions {
  enableShadows?: boolean;
  backgroundColor?: string;
  fog?: {
    color: string;
    near: number;
    far: number;
  };
  onReady?: (scene: THREE.Scene) => void;
}

export interface UseScene3DReturn {
  sceneRef: React.RefObject<THREE.Scene>;
  cameraRef: React.RefObject<THREE.Camera>;
  rendererRef: React.RefObject<THREE.WebGLRenderer>;
  addObject: (object: THREE.Object3D) => void;
  removeObject: (object: THREE.Object3D) => void;
  getObject: (uuid: string) => THREE.Object3D | undefined;
  updateObject: (uuid: string, updates: Partial<THREE.Object3D>) => void;
  clearScene: () => void;
  takeScreenshot: () => string | null;
  setBackgroundColor: (color: string) => void;
  enableFog: (near: number, far: number, color?: string) => void;
  disableFog: () => void;
}

export function useScene3D(options: UseScene3DOptions = {}): UseScene3DReturn {
  const sceneRef = useRef<THREE.Scene>(null);
  const cameraRef = useRef<THREE.Camera>(null);
  const rendererRef = useRef<THREE.WebGLRenderer>(null);
  const [initialized, setInitialized] = useState(false);

  // Initialize scene
  useEffect(() => {
    if (!sceneRef.current) return;

    const scene = sceneRef.current;

    // Setup background
    if (options.backgroundColor) {
      scene.background = new THREE.Color(options.backgroundColor);
    }

    // Setup fog
    if (options.fog) {
      scene.fog = new THREE.Fog(
        options.fog.color,
        options.fog.near,
        options.fog.far
      );
    }

    // Enable shadows if requested
    if (options.enableShadows && rendererRef.current) {
      rendererRef.current.shadowMap.enabled = true;
      rendererRef.current.shadowMap.type = THREE.PCFSoftShadowMap;
    }

    setInitialized(true);
    options.onReady?.(scene);
  }, [options.backgroundColor, options.enableShadows]);

  const addObject = useCallback((object: THREE.Object3D) => {
    if (sceneRef.current) {
      sceneRef.current.add(object);
    }
  }, []);

  const removeObject = useCallback((object: THREE.Object3D) => {
    if (sceneRef.current) {
      sceneRef.current.remove(object);
    }
  }, []);

  const getObject = useCallback((uuid: string): THREE.Object3D | undefined => {
    if (!sceneRef.current) return undefined;
    return sceneRef.current.getObjectByProperty('uuid', uuid);
  }, []);

  const updateObject = useCallback((uuid: string, updates: Partial<THREE.Object3D>) => {
    const object = getObject(uuid);
    if (object) {
      Object.assign(object, updates);
    }
  }, [getObject]);

  const clearScene = useCallback(() => {
    if (!sceneRef.current) return;

    while (sceneRef.current.children.length > 0) {
      const object = sceneRef.current.children[0];
      sceneRef.current.remove(object);

      // Dispose of geometries and materials
      if (object instanceof THREE.Mesh) {
        object.geometry?.dispose();
        if (Array.isArray(object.material)) {
          object.material.forEach(mat => mat.dispose());
        } else {
          object.material?.dispose();
        }
      }
    }
  }, []);

  const takeScreenshot = useCallback((): string | null => {
    if (!rendererRef.current) return null;
    return rendererRef.current.domElement.toDataURL('image/png');
  }, []);

  const setBackgroundColor = useCallback((color: string) => {
    if (sceneRef.current) {
      sceneRef.current.background = new THREE.Color(color);
    }
  }, []);

  const enableFog = useCallback((near: number, far: number, color: string = '#ffffff') => {
    if (sceneRef.current) {
      sceneRef.current.fog = new THREE.Fog(color, near, far);
    }
  }, []);

  const disableFog = useCallback(() => {
    if (sceneRef.current) {
      sceneRef.current.fog = null;
    }
  }, []);

  return {
    sceneRef,
    cameraRef,
    rendererRef,
    addObject,
    removeObject,
    getObject,
    updateObject,
    clearScene,
    takeScreenshot,
    setBackgroundColor,
    enableFog,
    disableFog,
  };
}

export default useScene3D;
