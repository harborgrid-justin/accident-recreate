/**
 * Heatmap Component
 * Base heatmap visualization component
 */

import React, { useRef, useEffect } from 'react';
import { HeatmapProps } from '../types';
import './Heatmap.css';

export const Heatmap: React.FC<HeatmapProps> = ({
  width,
  height,
  data,
  colorScale = ['#0000ff', '#00ffff', '#00ff00', '#ffff00', '#ff0000'],
  opacity = 0.7,
  blur = 15,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    canvas.width = width;
    canvas.height = height;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Draw heatmap
    drawHeatmap(ctx, data, width, height, colorScale, blur, opacity);
  }, [width, height, data, colorScale, opacity, blur]);

  return (
    <div className="heatmap-container">
      <canvas ref={canvasRef} className="heatmap-canvas" />
    </div>
  );
};

function drawHeatmap(
  ctx: CanvasRenderingContext2D,
  data: any[],
  width: number,
  height: number,
  colorScale: string[],
  blur: number,
  opacity: number
) {
  // Create temporary canvas for heatmap calculation
  const tempCanvas = document.createElement('canvas');
  tempCanvas.width = width;
  tempCanvas.height = height;
  const tempCtx = tempCanvas.getContext('2d');
  if (!tempCtx) return;

  // Draw gradient circles for each data point
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
    gradient.addColorStop(0, `rgba(0, 0, 0, ${alpha})`);
    gradient.addColorStop(1, 'rgba(0, 0, 0, 0)');

    tempCtx.fillStyle = gradient;
    tempCtx.fillRect(0, 0, width, height);
  });

  // Get image data and apply color mapping
  const imageData = tempCtx.getImageData(0, 0, width, height);
  const pixels = imageData.data;

  for (let i = 0; i < pixels.length; i += 4) {
    const intensity = pixels[i + 3] / 255; // Use alpha channel as intensity

    if (intensity > 0) {
      const color = getColorFromScale(intensity, colorScale);
      pixels[i] = color.r;
      pixels[i + 1] = color.g;
      pixels[i + 2] = color.b;
      pixels[i + 3] = intensity * 255;
    }
  }

  ctx.putImageData(imageData, 0, 0);
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

export default Heatmap;
