/**
 * AR Component
 * Augmented Reality preview container
 */

import React, { useEffect, useRef } from 'react';
import { ARProps } from '../types';
import { useAR } from '../hooks/useAR';
import './AR.css';

export const AR: React.FC<ARProps> = ({
  enabled,
  scene,
  camera,
  markerDetection = true,
  trackingMode = 'world',
  onTrackingStatusChange,
}) => {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const {
    isSupported,
    isTracking,
    isInitialized,
    error,
    cameraStream,
    startAR,
    stopAR,
  } = useAR({
    trackingMode,
    markerDetection,
    onTrackingStart: () => onTrackingStatusChange?.(true),
    onTrackingLost: () => onTrackingStatusChange?.(false),
  });

  useEffect(() => {
    if (enabled && isSupported) {
      startAR();
    } else if (!enabled && isInitialized) {
      stopAR();
    }
  }, [enabled, isSupported, isInitialized]);

  useEffect(() => {
    if (videoRef.current && cameraStream) {
      videoRef.current.srcObject = cameraStream;
    }
  }, [cameraStream]);

  if (!isSupported) {
    return (
      <div className="ar-container ar-not-supported">
        <div className="ar-message">
          <h3>AR Not Supported</h3>
          <p>Your device or browser does not support AR features.</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="ar-container ar-error">
        <div className="ar-message">
          <h3>AR Error</h3>
          <p>{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`ar-container ${isTracking ? 'tracking' : ''}`}>
      <video
        ref={videoRef}
        className="ar-video"
        autoPlay
        playsInline
        muted
      />
      <canvas ref={canvasRef} className="ar-canvas" />

      {isTracking && (
        <div className="ar-tracking-indicator">
          <div className="tracking-dot" />
          <span>Tracking Active</span>
        </div>
      )}

      {!isTracking && isInitialized && (
        <div className="ar-instructions">
          <p>Point camera at the scene to begin tracking</p>
        </div>
      )}
    </div>
  );
};

export default AR;
