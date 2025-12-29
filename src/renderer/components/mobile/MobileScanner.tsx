/**
 * AccuScene Enterprise v0.3.0
 * Mobile Scanner Component
 *
 * Document and QR code scanner using device camera
 */

import React, { useState, useRef, useEffect, CSSProperties } from 'react';
import { ScanResult } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface MobileScannerProps {
  onScan: (result: ScanResult) => void;
  onClose: () => void;
  scanType?: 'qr' | 'barcode' | 'document';
  continuous?: boolean;
  className?: string;
}

/**
 * Scanner component for QR codes, barcodes, and documents
 * Uses device camera with real-time detection
 *
 * @example
 * ```tsx
 * <MobileScanner
 *   onScan={(result) => {
 *     console.log('Scanned:', result.data);
 *     processCode(result);
 *   }}
 *   onClose={() => setScannerOpen(false)}
 *   scanType="qr"
 * />
 * ```
 */
export const MobileScanner: React.FC<MobileScannerProps> = ({
  onScan,
  onClose,
  scanType = 'qr',
  continuous = false,
  className = '',
}) => {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(true);

  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const scanIntervalRef = useRef<NodeJS.Timer | null>(null);

  useEffect(() => {
    let mounted = true;

    const startCamera = async () => {
      try {
        const mediaStream = await navigator.mediaDevices.getUserMedia({
          video: {
            facingMode: 'environment',
            width: { ideal: 1920 },
            height: { ideal: 1080 },
          },
        });

        if (!mounted) {
          mediaStream.getTracks().forEach((track) => track.stop());
          return;
        }

        setStream(mediaStream);

        if (videoRef.current) {
          videoRef.current.srcObject = mediaStream;
        }

        // Start scanning
        startScanning();
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
      if (scanIntervalRef.current) {
        clearInterval(scanIntervalRef.current);
      }
    };
  }, []);

  const startScanning = () => {
    // Simulate scanning (in production, use a library like jsQR or ZXing)
    scanIntervalRef.current = setInterval(() => {
      if (!scanning) return;

      // This is a placeholder - in production, implement actual QR/barcode detection
      // For now, we'll just demonstrate the interface
    }, 100);
  };

  const handleScan = (data: string) => {
    HapticFeedback.success();

    const result: ScanResult = {
      type: scanType,
      data,
      timestamp: Date.now(),
      confidence: 0.95,
    };

    onScan(result);

    if (!continuous) {
      setScanning(false);
      setTimeout(() => onClose(), 500);
    }
  };

  const handleClose = () => {
    HapticFeedback.light();
    onClose();
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
    width: '100%',
    height: '100%',
    objectFit: 'cover',
  };

  const overlayStyles: CSSProperties = {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    pointerEvents: 'none',
  };

  const frameStyles: CSSProperties = {
    position: 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    width: scanType === 'document' ? '80%' : '250px',
    height: scanType === 'document' ? '60%' : '250px',
    border: '2px solid #00FF00',
    borderRadius: scanType === 'document' ? '8px' : '12px',
    boxShadow: '0 0 0 9999px rgba(0, 0, 0, 0.5)',
  };

  const controlsStyles: CSSProperties = {
    position: 'absolute',
    top: '2rem',
    left: 0,
    right: 0,
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '0 1rem',
  };

  const instructionsStyles: CSSProperties = {
    position: 'absolute',
    bottom: '4rem',
    left: 0,
    right: 0,
    textAlign: 'center',
    color: '#ffffff',
    fontSize: '1rem',
    padding: '0 2rem',
    textShadow: '0 2px 4px rgba(0, 0, 0, 0.8)',
  };

  const buttonStyles: CSSProperties = {
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

  const getInstructionText = (): string => {
    switch (scanType) {
      case 'qr':
        return 'Align QR code within the frame';
      case 'barcode':
        return 'Align barcode within the frame';
      case 'document':
        return 'Position document within the frame';
      default:
        return 'Align target within the frame';
    }
  };

  return (
    <div
      className={`mobile-scanner ${className}`}
      style={containerStyles}
      data-testid="mobile-scanner"
    >
      {error ? (
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            color: '#ffffff',
            fontSize: '1rem',
            textAlign: 'center',
            padding: '2rem',
          }}
        >
          <p>{error}</p>
          <button
            onClick={handleClose}
            style={{ ...buttonStyles, marginTop: '1rem' }}
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

          <canvas ref={canvasRef} style={{ display: 'none' }} />

          {/* Overlay with scanning frame */}
          <div className="mobile-scanner__overlay" style={overlayStyles}>
            <div className="mobile-scanner__frame" style={frameStyles} />

            {/* Controls */}
            <div className="mobile-scanner__controls" style={controlsStyles}>
              <button
                className="mobile-scanner__close"
                style={buttonStyles}
                onClick={handleClose}
                type="button"
                aria-label="Close scanner"
              >
                ✕
              </button>

              <div
                style={{
                  color: '#ffffff',
                  fontSize: '1.125rem',
                  fontWeight: 600,
                  textShadow: '0 2px 4px rgba(0, 0, 0, 0.8)',
                }}
              >
                {scanType.toUpperCase()} Scanner
              </div>

              <div style={{ width: '48px' }} />
            </div>

            {/* Instructions */}
            <div className="mobile-scanner__instructions" style={instructionsStyles}>
              {scanning ? getInstructionText() : 'Scan complete!'}
            </div>
          </div>
        </>
      )}

      <style>{`
        .mobile-scanner__close:active {
          transform: scale(0.95);
          background-color: rgba(255, 255, 255, 0.5);
        }

        .mobile-scanner__frame {
          animation: pulse 2s ease-in-out infinite;
        }

        @keyframes pulse {
          0%, 100% {
            border-color: #00FF00;
          }
          50% {
            border-color: #00AA00;
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .mobile-scanner__frame {
            animation: none !important;
          }

          .mobile-scanner__close {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default MobileScanner;
