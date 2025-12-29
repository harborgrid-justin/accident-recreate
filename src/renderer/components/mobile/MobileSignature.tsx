/**
 * AccuScene Enterprise v0.3.0
 * Mobile Signature Component
 *
 * Touch signature capture with smooth drawing
 */

import React, { useRef, useState, useCallback, useEffect, CSSProperties } from 'react';
import { SignatureData, SignatureOptions, Point } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface MobileSignatureProps {
  onSave: (signature: SignatureData) => void;
  onCancel: () => void;
  options?: SignatureOptions;
  width?: number | string;
  height?: number | string;
  className?: string;
}

/**
 * Signature pad for touch-based signatures
 * Captures smooth signatures with pressure sensitivity simulation
 *
 * @example
 * ```tsx
 * <MobileSignature
 *   onSave={(signature) => {
 *     console.log('Signature captured:', signature.dataUrl);
 *     saveSignature(signature);
 *   }}
 *   onCancel={() => setSignatureOpen(false)}
 *   width="100%"
 *   height="300px"
 * />
 * ```
 */
export const MobileSignature: React.FC<MobileSignatureProps> = ({
  onSave,
  onCancel,
  options = {},
  width = '100%',
  height = '300px',
  className = '',
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const contextRef = useRef<CanvasRenderingContext2D | null>(null);

  const [isDrawing, setIsDrawing] = useState(false);
  const [points, setPoints] = useState<Point[]>([]);
  const [isEmpty, setIsEmpty] = useState(true);

  const lastPointRef = useRef<Point | null>(null);
  const lastVelocityRef = useRef<number>(0);
  const lastWidthRef = useRef<number>(0);

  const defaultOptions: Required<SignatureOptions> = {
    penColor: '#000000',
    penWidth: 2,
    backgroundColor: '#ffffff',
    minStrokeWidth: 0.5,
    maxStrokeWidth: 2.5,
    velocityFilterWeight: 0.7,
    ...options,
  };

  // Initialize canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();

    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.scale(dpr, dpr);
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';

    contextRef.current = ctx;

    // Clear with background color
    ctx.fillStyle = defaultOptions.backgroundColor;
    ctx.fillRect(0, 0, rect.width, rect.height);
  }, [defaultOptions.backgroundColor]);

  const getTouchPoint = (touch: Touch): Point => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };

    const rect = canvas.getBoundingClientRect();
    return {
      x: touch.clientX - rect.left,
      y: touch.clientY - rect.top,
    };
  };

  const getStrokeWidth = (velocity: number): number => {
    const { minStrokeWidth, maxStrokeWidth, velocityFilterWeight } = defaultOptions;

    // Calculate width based on velocity (slower = thicker)
    const newWidth = Math.max(
      minStrokeWidth,
      Math.min(maxStrokeWidth, maxStrokeWidth - velocity * 0.1)
    );

    // Apply velocity filter for smoothness
    return (
      lastWidthRef.current * velocityFilterWeight +
      newWidth * (1 - velocityFilterWeight)
    );
  };

  const drawPoint = useCallback(
    (point: Point, velocity: number) => {
      const ctx = contextRef.current;
      if (!ctx) return;

      const strokeWidth = getStrokeWidth(velocity);
      lastWidthRef.current = strokeWidth;

      ctx.strokeStyle = defaultOptions.penColor;
      ctx.lineWidth = strokeWidth;

      if (!lastPointRef.current) {
        ctx.beginPath();
        ctx.moveTo(point.x, point.y);
      } else {
        ctx.beginPath();
        ctx.moveTo(lastPointRef.current.x, lastPointRef.current.y);
        ctx.lineTo(point.x, point.y);
        ctx.stroke();
      }

      lastPointRef.current = point;
    },
    [defaultOptions.penColor]
  );

  const handleTouchStart = (e: React.TouchEvent) => {
    e.preventDefault();
    HapticFeedback.light();

    const point = getTouchPoint(e.touches[0]);
    setIsDrawing(true);
    setIsEmpty(false);
    lastPointRef.current = point;
    lastVelocityRef.current = 0;
  };

  const handleTouchMove = (e: React.TouchEvent) => {
    if (!isDrawing) return;

    e.preventDefault();

    const point = getTouchPoint(e.touches[0]);
    const last = lastPointRef.current;

    if (last) {
      const dx = point.x - last.x;
      const dy = point.y - last.y;
      const distance = Math.sqrt(dx * dx + dy * dy);
      const velocity = distance;

      drawPoint(point, velocity);
      lastVelocityRef.current = velocity;
    }

    setPoints((prev) => [...prev, point]);
  };

  const handleTouchEnd = (e: React.TouchEvent) => {
    e.preventDefault();

    if (isDrawing) {
      setIsDrawing(false);
      lastPointRef.current = null;
      HapticFeedback.light();
    }
  };

  const clear = () => {
    const canvas = canvasRef.current;
    const ctx = contextRef.current;
    if (!canvas || !ctx) return;

    const rect = canvas.getBoundingClientRect();
    ctx.fillStyle = defaultOptions.backgroundColor;
    ctx.fillRect(0, 0, rect.width, rect.height);

    setPoints([]);
    setIsEmpty(true);
    lastPointRef.current = null;
    HapticFeedback.medium();
  };

  const save = async () => {
    if (isEmpty) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    HapticFeedback.success();

    // Convert to data URL
    const dataUrl = canvas.toDataURL('image/png');

    // Convert to blob
    canvas.toBlob((blob) => {
      if (!blob) return;

      const signature: SignatureData = {
        points,
        timestamp: Date.now(),
        blob,
        dataUrl,
      };

      onSave(signature);
    });
  };

  const cancel = () => {
    HapticFeedback.light();
    onCancel();
  };

  const containerStyles: CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: '1rem',
    width: '100%',
  };

  const canvasContainerStyles: CSSProperties = {
    width,
    height,
    border: '2px dashed #d0d0d0',
    borderRadius: '8px',
    overflow: 'hidden',
    backgroundColor: defaultOptions.backgroundColor,
    position: 'relative',
  };

  const canvasStyles: CSSProperties = {
    width: '100%',
    height: '100%',
    touchAction: 'none',
    display: 'block',
  };

  const placeholderStyles: CSSProperties = {
    position: 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    color: '#a0a0a0',
    fontSize: '1rem',
    pointerEvents: 'none',
    opacity: isEmpty ? 1 : 0,
    transition: 'opacity 0.2s ease',
  };

  const buttonsStyles: CSSProperties = {
    display: 'flex',
    gap: '0.75rem',
    justifyContent: 'flex-end',
  };

  const buttonStyles = (variant: 'primary' | 'secondary' | 'danger'): CSSProperties => {
    const colors = {
      primary: { bg: '#007AFF', color: '#ffffff' },
      secondary: { bg: '#f0f0f0', color: '#000000' },
      danger: { bg: '#FF3B30', color: '#ffffff' },
    };

    return {
      padding: '0.75rem 1.5rem',
      fontSize: '1rem',
      fontWeight: 500,
      border: 'none',
      borderRadius: '8px',
      backgroundColor: colors[variant].bg,
      color: colors[variant].color,
      cursor: 'pointer',
      minWidth: '44px',
      minHeight: '44px',
      transition: 'opacity 0.2s ease',
    };
  };

  return (
    <div
      className={`mobile-signature ${className}`}
      style={containerStyles}
      data-testid="mobile-signature"
    >
      <div className="mobile-signature__canvas-container" style={canvasContainerStyles}>
        <canvas
          ref={canvasRef}
          className="mobile-signature__canvas"
          style={canvasStyles}
          onTouchStart={handleTouchStart}
          onTouchMove={handleTouchMove}
          onTouchEnd={handleTouchEnd}
          onTouchCancel={handleTouchEnd}
        />

        <div className="mobile-signature__placeholder" style={placeholderStyles}>
          Sign here
        </div>
      </div>

      <div className="mobile-signature__buttons" style={buttonsStyles}>
        <button
          className="mobile-signature__clear"
          style={buttonStyles('secondary')}
          onClick={clear}
          disabled={isEmpty}
          type="button"
        >
          Clear
        </button>

        <button
          className="mobile-signature__cancel"
          style={buttonStyles('danger')}
          onClick={cancel}
          type="button"
        >
          Cancel
        </button>

        <button
          className="mobile-signature__save"
          style={buttonStyles('primary')}
          onClick={save}
          disabled={isEmpty}
          type="button"
        >
          Save
        </button>
      </div>

      <style>{`
        .mobile-signature__canvas {
          -webkit-user-select: none;
          -webkit-touch-callout: none;
        }

        .mobile-signature button:active:not(:disabled) {
          opacity: 0.8;
        }

        .mobile-signature button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .mobile-signature button:focus-visible {
          outline: 2px solid #007AFF;
          outline-offset: 2px;
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .mobile-signature__canvas-container {
            border-color: #48484a;
          }

          .mobile-signature__placeholder {
            color: #98989d;
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .mobile-signature__placeholder,
          .mobile-signature button {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default MobileSignature;
