/**
 * AccuScene Enterprise v0.3.0
 * Touch Canvas Component
 *
 * Touch-optimized canvas with multi-touch gestures and drawing capabilities
 */

import React, { useRef, useEffect, useState, useCallback, CSSProperties } from 'react';
import { CanvasState, DrawingTool, Point } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface TouchCanvasProps {
  width?: number | string;
  height?: number | string;
  tool?: DrawingTool;
  onDraw?: (points: Point[]) => void;
  onTransform?: (state: CanvasState) => void;
  backgroundColor?: string;
  enableZoom?: boolean;
  enablePan?: boolean;
  maxZoom?: number;
  minZoom?: number;
  className?: string;
}

/**
 * Touch-optimized canvas for drawing and manipulation
 * Supports pinch-to-zoom, pan, and drawing operations
 *
 * @example
 * ```tsx
 * <TouchCanvas
 *   width="100%"
 *   height="500px"
 *   tool={{ type: 'pen', color: '#000', width: 2 }}
 *   enableZoom
 *   enablePan
 *   onDraw={(points) => console.log('Drew path:', points)}
 * />
 * ```
 */
export const TouchCanvas: React.FC<TouchCanvasProps> = ({
  width = '100%',
  height = '400px',
  tool = { type: 'pen', color: '#000000', width: 2 },
  onDraw,
  onTransform,
  backgroundColor = '#ffffff',
  enableZoom = true,
  enablePan = true,
  maxZoom = 5,
  minZoom = 0.5,
  className = '',
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const contextRef = useRef<CanvasRenderingContext2D | null>(null);

  const [canvasState, setCanvasState] = useState<CanvasState>({
    scale: 1,
    offsetX: 0,
    offsetY: 0,
    rotation: 0,
    width: 0,
    height: 0,
  });

  const [isDrawing, setIsDrawing] = useState(false);
  const [currentPath, setCurrentPath] = useState<Point[]>([]);

  const touchStartRef = useRef<{ touches: Touch[]; time: number } | null>(null);
  const lastDistanceRef = useRef<number>(0);
  const lastTouchesRef = useRef<TouchList | null>(null);

  // Initialize canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();

    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;

    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) return;

    ctx.scale(dpr, dpr);
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';

    contextRef.current = ctx;

    setCanvasState((prev) => ({
      ...prev,
      width: rect.width,
      height: rect.height,
    }));

    // Clear canvas
    ctx.fillStyle = backgroundColor;
    ctx.fillRect(0, 0, rect.width, rect.height);
  }, [backgroundColor]);

  const getDistance = (touch1: Touch, touch2: Touch): number => {
    const dx = touch2.clientX - touch1.clientX;
    const dy = touch2.clientY - touch1.clientY;
    return Math.sqrt(dx * dx + dy * dy);
  };

  const getTouchPoint = (touch: Touch): Point => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };

    const rect = canvas.getBoundingClientRect();
    return {
      x: (touch.clientX - rect.left - canvasState.offsetX) / canvasState.scale,
      y: (touch.clientY - rect.top - canvasState.offsetY) / canvasState.scale,
    };
  };

  const drawPoint = useCallback(
    (point: Point) => {
      const ctx = contextRef.current;
      if (!ctx) return;

      ctx.save();
      ctx.translate(canvasState.offsetX, canvasState.offsetY);
      ctx.scale(canvasState.scale, canvasState.scale);

      ctx.strokeStyle = tool.color || '#000000';
      ctx.lineWidth = tool.width || 2;
      ctx.globalAlpha = tool.opacity || 1;

      if (currentPath.length === 0) {
        ctx.beginPath();
        ctx.moveTo(point.x, point.y);
      } else {
        const lastPoint = currentPath[currentPath.length - 1];
        ctx.beginPath();
        ctx.moveTo(lastPoint.x, lastPoint.y);
        ctx.lineTo(point.x, point.y);
        ctx.stroke();
      }

      ctx.restore();
    },
    [canvasState, currentPath, tool]
  );

  const handleTouchStart = (e: TouchEvent) => {
    e.preventDefault();

    const touches = e.touches;
    touchStartRef.current = {
      touches: Array.from(touches),
      time: Date.now(),
    };

    if (touches.length === 1 && tool.type === 'pen') {
      // Start drawing
      setIsDrawing(true);
      const point = getTouchPoint(touches[0]);
      setCurrentPath([point]);
      HapticFeedback.light();
    } else if (touches.length === 2 && (enableZoom || enablePan)) {
      // Prepare for pinch/pan
      lastDistanceRef.current = getDistance(touches[0], touches[1]);
      lastTouchesRef.current = touches;
      HapticFeedback.light();
    }
  };

  const handleTouchMove = (e: TouchEvent) => {
    e.preventDefault();

    const touches = e.touches;

    if (touches.length === 1 && isDrawing && tool.type === 'pen') {
      // Continue drawing
      const point = getTouchPoint(touches[0]);
      setCurrentPath((prev) => [...prev, point]);
      drawPoint(point);
    } else if (touches.length === 2 && lastTouchesRef.current) {
      // Handle pinch zoom and pan
      const currentDistance = getDistance(touches[0], touches[1]);
      const lastDistance = lastDistanceRef.current;

      if (enableZoom && Math.abs(currentDistance - lastDistance) > 5) {
        // Pinch to zoom
        const scaleChange = currentDistance / lastDistance;
        const newScale = Math.max(
          minZoom,
          Math.min(maxZoom, canvasState.scale * scaleChange)
        );

        setCanvasState((prev) => {
          const updated = { ...prev, scale: newScale };
          onTransform?.(updated);
          return updated;
        });

        lastDistanceRef.current = currentDistance;
        HapticFeedback.selection();
      }

      if (enablePan) {
        // Pan gesture
        const lastCenter = {
          x: (lastTouchesRef.current[0].clientX + lastTouchesRef.current[1].clientX) / 2,
          y: (lastTouchesRef.current[0].clientY + lastTouchesRef.current[1].clientY) / 2,
        };

        const currentCenter = {
          x: (touches[0].clientX + touches[1].clientX) / 2,
          y: (touches[0].clientY + touches[1].clientY) / 2,
        };

        const deltaX = currentCenter.x - lastCenter.x;
        const deltaY = currentCenter.y - lastCenter.y;

        setCanvasState((prev) => {
          const updated = {
            ...prev,
            offsetX: prev.offsetX + deltaX,
            offsetY: prev.offsetY + deltaY,
          };
          onTransform?.(updated);
          return updated;
        });
      }

      lastTouchesRef.current = touches;
    }
  };

  const handleTouchEnd = (e: TouchEvent) => {
    e.preventDefault();

    if (isDrawing && currentPath.length > 0) {
      onDraw?.(currentPath);
      setIsDrawing(false);
      setCurrentPath([]);
      HapticFeedback.light();
    }

    if (e.touches.length === 0) {
      touchStartRef.current = null;
      lastTouchesRef.current = null;
    }
  };

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    canvas.addEventListener('touchstart', handleTouchStart, { passive: false });
    canvas.addEventListener('touchmove', handleTouchMove, { passive: false });
    canvas.addEventListener('touchend', handleTouchEnd, { passive: false });
    canvas.addEventListener('touchcancel', handleTouchEnd, { passive: false });

    return () => {
      canvas.removeEventListener('touchstart', handleTouchStart);
      canvas.removeEventListener('touchmove', handleTouchMove);
      canvas.removeEventListener('touchend', handleTouchEnd);
      canvas.removeEventListener('touchcancel', handleTouchEnd);
    };
  }, [isDrawing, currentPath, canvasState, tool, enableZoom, enablePan]);

  const canvasStyles: CSSProperties = {
    width,
    height,
    backgroundColor,
    touchAction: 'none',
    userSelect: 'none',
    WebkitUserSelect: 'none',
    display: 'block',
    cursor: tool.type === 'pen' ? 'crosshair' : 'grab',
  };

  return (
    <div
      className={`touch-canvas-wrapper ${className}`}
      style={{ width, height, position: 'relative', overflow: 'hidden' }}
      data-testid="touch-canvas"
    >
      <canvas
        ref={canvasRef}
        className="touch-canvas"
        style={canvasStyles}
        aria-label="Touch drawing canvas"
      />

      <style>{`
        .touch-canvas-wrapper {
          -webkit-user-select: none;
          -webkit-touch-callout: none;
        }

        .touch-canvas {
          transform-origin: center;
        }

        /* Prevent default touch behaviors */
        .touch-canvas * {
          touch-action: none;
        }
      `}</style>
    </div>
  );
};

export default TouchCanvas;
