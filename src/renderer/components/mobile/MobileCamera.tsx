/**
 * AccuScene Enterprise v0.3.0
 * Mobile Camera Component
 *
 * Camera capture for evidence photos with front/back camera switching
 */

import React, { useState, useRef, useEffect, CSSProperties } from 'react';
import { CameraFacing, CapturedMedia, CameraConstraints } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface MobileCameraProps {
  onCapture: (media: CapturedMedia) => void;
  onClose: () => void;
  facing?: CameraFacing;
  enableVideoRecording?: boolean;
  maxVideoDuration?: number;
  className?: string;
}

/**
 * Camera component for capturing photos and videos
 * Supports front/back camera switching and video recording
 *
 * @example
 * ```tsx
 * <MobileCamera
 *   onCapture={(media) => {
 *     console.log('Captured:', media);
 *     uploadEvidence(media);
 *   }}
 *   onClose={() => setCameraOpen(false)}
 *   enableVideoRecording
 * />
 * ```
 */
export const MobileCamera: React.FC<MobileCameraProps> = ({
  onCapture,
  onClose,
  facing = 'environment',
  enableVideoRecording = false,
  maxVideoDuration = 60,
  className = '',
}) => {
  const [currentFacing, setCurrentFacing] = useState<CameraFacing>(facing);
  const [isRecording, setIsRecording] = useState(false);
  const [recordingTime, setRecordingTime] = useState(0);
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [error, setError] = useState<string | null>(null);

  const videoRef = useRef<HTMLVideoElement>(null);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const recordedChunksRef = useRef<Blob[]>([]);
  const timerRef = useRef<NodeJS.Timer | null>(null);

  // Initialize camera
  useEffect(() => {
    let mounted = true;

    const startCamera = async () => {
      try {
        const constraints: MediaStreamConstraints = {
          video: {
            facingMode: currentFacing,
            width: { ideal: 1920 },
            height: { ideal: 1080 },
          },
          audio: enableVideoRecording,
        };

        const mediaStream = await navigator.mediaDevices.getUserMedia(constraints);

        if (!mounted) {
          mediaStream.getTracks().forEach((track) => track.stop());
          return;
        }

        setStream(mediaStream);

        if (videoRef.current) {
          videoRef.current.srcObject = mediaStream;
        }

        setError(null);
      } catch (err) {
        console.error('Camera access error:', err);
        setError('Unable to access camera. Please check permissions.');
      }
    };

    startCamera();

    return () => {
      mounted = false;
      if (stream) {
        stream.getTracks().forEach((track) => track.stop());
      }
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
    };
  }, [currentFacing, enableVideoRecording]);

  const capturePhoto = async () => {
    if (!videoRef.current || !stream) return;

    HapticFeedback.heavy();

    const canvas = document.createElement('canvas');
    const video = videoRef.current;

    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.drawImage(video, 0, 0);

    canvas.toBlob((blob) => {
      if (!blob) return;

      const capturedMedia: CapturedMedia = {
        id: `photo_${Date.now()}`,
        type: 'photo',
        blob,
        url: URL.createObjectURL(blob),
        timestamp: Date.now(),
        metadata: {
          width: canvas.width,
          height: canvas.height,
          size: blob.size,
        },
      };

      onCapture(capturedMedia);
    }, 'image/jpeg', 0.9);
  };

  const startVideoRecording = async () => {
    if (!stream) return;

    HapticFeedback.medium();

    try {
      const mediaRecorder = new MediaRecorder(stream, {
        mimeType: 'video/webm',
      });

      recordedChunksRef.current = [];

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          recordedChunksRef.current.push(event.data);
        }
      };

      mediaRecorder.onstop = () => {
        const blob = new Blob(recordedChunksRef.current, { type: 'video/webm' });

        const capturedMedia: CapturedMedia = {
          id: `video_${Date.now()}`,
          type: 'video',
          blob,
          url: URL.createObjectURL(blob),
          timestamp: Date.now(),
          metadata: {
            width: videoRef.current?.videoWidth || 0,
            height: videoRef.current?.videoHeight || 0,
            size: blob.size,
            duration: recordingTime,
          },
        };

        onCapture(capturedMedia);
        setRecordingTime(0);
      };

      mediaRecorder.start();
      mediaRecorderRef.current = mediaRecorder;
      setIsRecording(true);

      // Start timer
      timerRef.current = setInterval(() => {
        setRecordingTime((prev) => {
          const newTime = prev + 1;
          if (newTime >= maxVideoDuration) {
            stopVideoRecording();
          }
          return newTime;
        });
      }, 1000);
    } catch (err) {
      console.error('Recording error:', err);
      setError('Unable to start recording');
    }
  };

  const stopVideoRecording = () => {
    if (mediaRecorderRef.current && isRecording) {
      HapticFeedback.medium();
      mediaRecorderRef.current.stop();
      setIsRecording(false);

      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    }
  };

  const switchCamera = () => {
    HapticFeedback.selection();
    setCurrentFacing((prev) => (prev === 'user' ? 'environment' : 'user'));
  };

  const handleClose = () => {
    HapticFeedback.light();
    onClose();
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const containerStyles: CSSProperties = {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: '#000000',
    zIndex: 2000,
    display: 'flex',
    flexDirection: 'column',
  };

  const videoStyles: CSSProperties = {
    flex: 1,
    width: '100%',
    objectFit: 'cover',
  };

  const controlsStyles: CSSProperties = {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    padding: '2rem',
    background: 'linear-gradient(transparent, rgba(0, 0, 0, 0.6))',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '1rem',
  };

  const buttonRowStyles: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '2rem',
    width: '100%',
  };

  const captureButtonStyles: CSSProperties = {
    width: '72px',
    height: '72px',
    borderRadius: '50%',
    border: '4px solid #ffffff',
    backgroundColor: isRecording ? '#FF3B30' : '#ffffff',
    cursor: 'pointer',
    transition: 'transform 0.2s ease',
  };

  const iconButtonStyles: CSSProperties = {
    width: '48px',
    height: '48px',
    borderRadius: '50%',
    border: 'none',
    backgroundColor: 'rgba(255, 255, 255, 0.3)',
    color: '#ffffff',
    fontSize: '1.5rem',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    cursor: 'pointer',
  };

  const timerStyles: CSSProperties = {
    color: '#FF3B30',
    fontSize: '1.5rem',
    fontWeight: 600,
    fontVariantNumeric: 'tabular-nums',
  };

  const errorStyles: CSSProperties = {
    position: 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    color: '#ffffff',
    fontSize: '1rem',
    textAlign: 'center',
    padding: '2rem',
  };

  return (
    <div
      className={`mobile-camera ${className}`}
      style={containerStyles}
      data-testid="mobile-camera"
    >
      {error ? (
        <div style={errorStyles}>
          <p>{error}</p>
          <button
            onClick={handleClose}
            style={{
              ...iconButtonStyles,
              marginTop: '1rem',
            }}
            type="button"
          >
            ✕
          </button>
        </div>
      ) : (
        <>
          <video
            ref={videoRef}
            style={videoStyles}
            autoPlay
            playsInline
            muted
          />

          <div className="mobile-camera__controls" style={controlsStyles}>
            {isRecording && (
              <div className="mobile-camera__timer" style={timerStyles}>
                {formatTime(recordingTime)}
              </div>
            )}

            <div className="mobile-camera__buttons" style={buttonRowStyles}>
              {/* Close button */}
              <button
                className="mobile-camera__close"
                style={iconButtonStyles}
                onClick={handleClose}
                type="button"
                aria-label="Close camera"
              >
                ✕
              </button>

              {/* Capture button */}
              <button
                className="mobile-camera__capture"
                style={captureButtonStyles}
                onClick={
                  enableVideoRecording
                    ? isRecording
                      ? stopVideoRecording
                      : startVideoRecording
                    : capturePhoto
                }
                type="button"
                aria-label={
                  enableVideoRecording
                    ? isRecording
                      ? 'Stop recording'
                      : 'Start recording'
                    : 'Capture photo'
                }
              />

              {/* Switch camera button */}
              <button
                className="mobile-camera__switch"
                style={iconButtonStyles}
                onClick={switchCamera}
                type="button"
                aria-label="Switch camera"
              >
                ⟲
              </button>
            </div>

            {!isRecording && enableVideoRecording && (
              <div style={{ color: '#ffffff', fontSize: '0.875rem' }}>
                Tap to start recording
              </div>
            )}
          </div>
        </>
      )}

      <style>{`
        .mobile-camera__capture:active {
          transform: scale(0.9);
        }

        .mobile-camera__close:active,
        .mobile-camera__switch:active {
          transform: scale(0.95);
          background-color: rgba(255, 255, 255, 0.5);
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .mobile-camera__capture,
          .mobile-camera__close,
          .mobile-camera__switch {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default MobileCamera;
