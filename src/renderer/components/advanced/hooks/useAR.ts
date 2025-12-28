/**
 * useAR Hook
 * Manages AR capabilities and tracking
 */

import { useRef, useEffect, useState, useCallback } from 'react';
import * as THREE from 'three';

export interface UseAROptions {
  trackingMode?: 'world' | 'image' | 'face';
  markerDetection?: boolean;
  onTrackingStart?: () => void;
  onTrackingLost?: () => void;
  onMarkerDetected?: (markerId: string) => void;
}

export interface UseARReturn {
  isSupported: boolean;
  isTracking: boolean;
  isInitialized: boolean;
  error: string | null;
  cameraStream: MediaStream | null;
  arCamera: THREE.Camera | null;
  startAR: () => Promise<void>;
  stopAR: () => void;
  takeSnapshot: () => string | null;
  placeObject: (object: THREE.Object3D, position: THREE.Vector3) => void;
  removeObject: (object: THREE.Object3D) => void;
  getTrackedPose: () => THREE.Matrix4 | null;
}

export function useAR(options: UseAROptions = {}): UseARReturn {
  const [isSupported, setIsSupported] = useState(false);
  const [isTracking, setIsTracking] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [cameraStream, setCameraStream] = useState<MediaStream | null>(null);

  const arCameraRef = useRef<THREE.Camera | null>(null);
  const trackedObjectsRef = useRef<THREE.Object3D[]>([]);
  const poseMatrixRef = useRef<THREE.Matrix4>(new THREE.Matrix4());
  const videoElementRef = useRef<HTMLVideoElement | null>(null);

  // Check AR support
  useEffect(() => {
    const checkSupport = async () => {
      // Check for WebXR support
      if ('xr' in navigator) {
        try {
          const supported = await (navigator as any).xr.isSessionSupported('immersive-ar');
          setIsSupported(supported);
        } catch (err) {
          setIsSupported(false);
        }
      } else {
        // Fallback to basic camera access
        const hasCamera = !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia);
        setIsSupported(hasCamera);
      }
    };

    checkSupport();
  }, []);

  const startAR = useCallback(async () => {
    try {
      setError(null);

      // Request camera access
      const stream = await navigator.mediaDevices.getUserMedia({
        video: {
          facingMode: 'environment',
          width: { ideal: 1920 },
          height: { ideal: 1080 },
        },
        audio: false,
      });

      setCameraStream(stream);

      // Create video element for camera feed
      const video = document.createElement('video');
      video.srcObject = stream;
      video.autoplay = true;
      video.playsInline = true;
      videoElementRef.current = video;

      // Setup AR camera
      const camera = new THREE.PerspectiveCamera(
        75,
        window.innerWidth / window.innerHeight,
        0.1,
        1000
      );
      arCameraRef.current = camera;

      setIsInitialized(true);
      setIsTracking(true);
      options.onTrackingStart?.();

      // Start tracking simulation (in production, this would use a real AR library)
      simulateTracking();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to start AR';
      setError(errorMessage);
      console.error('AR initialization error:', err);
    }
  }, [options]);

  const stopAR = useCallback(() => {
    // Stop camera stream
    if (cameraStream) {
      cameraStream.getTracks().forEach(track => track.stop());
      setCameraStream(null);
    }

    // Cleanup video element
    if (videoElementRef.current) {
      videoElementRef.current.srcObject = null;
      videoElementRef.current = null;
    }

    // Clear tracked objects
    trackedObjectsRef.current = [];

    setIsTracking(false);
    setIsInitialized(false);
    options.onTrackingLost?.();
  }, [cameraStream, options]);

  const simulateTracking = useCallback(() => {
    // Simulated tracking updates (in production, use WebXR or AR.js)
    const updateTracking = () => {
      if (!isTracking) return;

      // Simulate pose updates
      const time = Date.now() * 0.001;
      const position = new THREE.Vector3(
        Math.sin(time * 0.5) * 0.1,
        0,
        Math.cos(time * 0.5) * 0.1
      );
      const rotation = new THREE.Euler(0, time * 0.2, 0);

      poseMatrixRef.current.makeRotationFromEuler(rotation);
      poseMatrixRef.current.setPosition(position);

      requestAnimationFrame(updateTracking);
    };

    updateTracking();
  }, [isTracking]);

  const takeSnapshot = useCallback((): string | null => {
    if (!videoElementRef.current) return null;

    const canvas = document.createElement('canvas');
    const video = videoElementRef.current;
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;

    const ctx = canvas.getContext('2d');
    if (!ctx) return null;

    ctx.drawImage(video, 0, 0);
    return canvas.toDataURL('image/png');
  }, []);

  const placeObject = useCallback((object: THREE.Object3D, position: THREE.Vector3) => {
    object.position.copy(position);
    trackedObjectsRef.current.push(object);
  }, []);

  const removeObject = useCallback((object: THREE.Object3D) => {
    const index = trackedObjectsRef.current.indexOf(object);
    if (index > -1) {
      trackedObjectsRef.current.splice(index, 1);
    }
  }, []);

  const getTrackedPose = useCallback((): THREE.Matrix4 | null => {
    if (!isTracking) return null;
    return poseMatrixRef.current.clone();
  }, [isTracking]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      stopAR();
    };
  }, []);

  return {
    isSupported,
    isTracking,
    isInitialized,
    error,
    cameraStream,
    arCamera: arCameraRef.current,
    startAR,
    stopAR,
    takeSnapshot,
    placeObject,
    removeObject,
    getTrackedPose,
  };
}

export default useAR;
