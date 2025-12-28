/**
 * Force Heatmap Component
 * Heatmap visualization for force distribution
 */

import React, { useRef, useEffect } from 'react';
import { ForceHeatmapProps } from '../types';
import './Heatmap.css';

export const ForceHeatmap: React.FC<ForceHeatmapProps> = ({
  width,
  height,
  data,
  colorScale = ['#0000ff', '#00ffff', '#00ff00', '#ffff00', '#ff0000'],
  opacity = 0.6,
  blur = 25,
  showVectors = true,
  vectorScale = 1,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    canvas.width = width;
    canvas.height = height;

    ctx.clearRect(0, 0, width, height);

    drawForceHeatmap(ctx, data, width, height, colorScale, blur, opacity, showVectors, vectorScale);
  }, [width, height, data, colorScale, opacity, blur, showVectors, vectorScale]);

  return (
    <div className="heatmap-container force-heatmap">
      <canvas ref={canvasRef} className="heatmap-canvas" />
      <div className="force-legend">
        <h4>Force Magnitude</h4>
        <div className="legend-gradient" style={{
          background: `linear-gradient(to top, ${colorScale.join(', ')})`
        }} />
        <div className="legend-labels">
          <span>High</span>
          <span>Low</span>
        </div>
      </div>
    </div>
  );
};

function drawForceHeatmap(
  ctx: CanvasRenderingContext2D,
  data: any[],
  width: number,
  height: number,
  colorScale: string[],
  blur: number,
  opacity: number,
  showVectors: boolean,
  vectorScale: number
) {
  // Draw heatmap
  const tempCanvas = document.createElement('canvas');
  tempCanvas.width = width;
  tempCanvas.height = height;
  const tempCtx = tempCanvas.getContext('2d');
  if (!tempCtx) return;

  data.forEach(point => {
    const gradient = tempCtx.createRadialGradient(
      point.x,
      point.y,
      0,
      point.x,
      point.y,
      blur
    );

    const alpha = point.value * opacity;
    gradient.addColorStop(0, `rgba(255, 255, 255, ${alpha})`);
    gradient.addColorStop(1, 'rgba(255, 255, 255, 0)');

    tempCtx.fillStyle = gradient;
    tempCtx.fillRect(0, 0, width, height);
  });

  const imageData = tempCtx.getImageData(0, 0, width, height);
  const pixels = imageData.data;

  for (let i = 0; i < pixels.length; i += 4) {
    const intensity = pixels[i + 3] / 255;

    if (intensity > 0) {
      const color = getColorFromScale(intensity, colorScale);
      pixels[i] = color.r;
      pixels[i + 1] = color.g;
      pixels[i + 2] = color.b;
      pixels[i + 3] = intensity * 255 * opacity;
    }
  }

  ctx.putImageData(imageData, 0, 0);

  // Draw force vectors
  if (showVectors) {
    ctx.strokeStyle = '#fff';
    ctx.fillStyle = '#fff';
    ctx.lineWidth = 2;

    data.forEach(point => {
      // Calculate vector direction (simplified)
      const angle = Math.random() * Math.PI * 2; // In production, use actual force direction
      const length = point.value * 30 * vectorScale;

      const endX = point.x + Math.cos(angle) * length;
      const endY = point.y + Math.sin(angle) * length;

      // Draw arrow
      ctx.beginPath();
      ctx.moveTo(point.x, point.y);
      ctx.lineTo(endX, endY);
      ctx.stroke();

      // Draw arrowhead
      const headAngle = Math.PI / 6;
      const headLength = 8;

      ctx.beginPath();
      ctx.moveTo(endX, endY);
      ctx.lineTo(
        endX - headLength * Math.cos(angle - headAngle),
        endY - headLength * Math.sin(angle - headAngle)
      );
      ctx.moveTo(endX, endY);
      ctx.lineTo(
        endX - headLength * Math.cos(angle + headAngle),
        endY - headLength * Math.sin(angle + headAngle)
      );
      ctx.stroke();
    });
  }
}

function getColorFromScale(value: number, colorScale: string[]): { r: number; g: number; b: number } {
  const index = Math.floor(value * (colorScale.length - 1));
  const nextIndex = Math.min(index + 1, colorScale.length - 1);
  const t = value * (colorScale.length - 1) - index;

  const color1 = hexToRgb(colorScale[index]);
  const color2 = hexToRgb(colorScale[nextIndex]);

  return {
    r: Math.round(color1.r + (color2.r - color1.r) * t),
    g: Math.round(color1.g + (color2.g - color1.g) * t),
    b: Math.round(color1.b + (color2.b - color1.b) * t),
  };
}

function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16),
      }
    : { r: 0, g: 0, b: 0 };
}

export default ForceHeatmap;
