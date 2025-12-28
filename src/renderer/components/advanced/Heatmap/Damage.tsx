/**
 * Damage Heatmap Component
 * Heatmap visualization for vehicle damage
 */

import React, { useRef, useEffect } from 'react';
import { DamageHeatmapProps } from '../types';
import './Heatmap.css';

export const DamageHeatmap: React.FC<DamageHeatmapProps> = ({
  width,
  height,
  data,
  colorScale = ['#00ff00', '#ffff00', '#ff6600', '#ff0000'],
  opacity = 0.7,
  blur = 20,
  vehicleOutline,
  showSeverityLabels = true,
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

    // Draw vehicle outline if provided
    if (vehicleOutline) {
      ctx.strokeStyle = '#404060';
      ctx.lineWidth = 2;
      const img = new Image();
      img.onload = () => {
        ctx.drawImage(img, 0, 0, width, height);
        drawDamageHeatmap(ctx, data, width, height, colorScale, blur, opacity, showSeverityLabels);
      };
      img.src = vehicleOutline;
    } else {
      drawDamageHeatmap(ctx, data, width, height, colorScale, blur, opacity, showSeverityLabels);
    }
  }, [width, height, data, colorScale, opacity, blur, vehicleOutline, showSeverityLabels]);

  return (
    <div className="heatmap-container damage-heatmap">
      <canvas ref={canvasRef} className="heatmap-canvas" />
      {showSeverityLabels && (
        <div className="severity-scale">
          <div className="scale-label">Low</div>
          <div className="scale-gradient" style={{
            background: `linear-gradient(to right, ${colorScale.join(', ')})`
          }} />
          <div className="scale-label">High</div>
        </div>
      )}
    </div>
  );
};

function drawDamageHeatmap(
  ctx: CanvasRenderingContext2D,
  data: any[],
  width: number,
  height: number,
  colorScale: string[],
  blur: number,
  opacity: number,
  showLabels: boolean
) {
  // Similar heatmap rendering as base component
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
    gradient.addColorStop(0, `rgba(255, 0, 0, ${alpha})`);
    gradient.addColorStop(1, 'rgba(255, 0, 0, 0)');

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
      pixels[i + 3] = intensity * 255;
    }
  }

  ctx.globalCompositeOperation = 'multiply';
  ctx.putImageData(imageData, 0, 0);
  ctx.globalCompositeOperation = 'source-over';

  // Draw labels for high severity points
  if (showLabels) {
    ctx.fillStyle = '#fff';
    ctx.strokeStyle = '#000';
    ctx.lineWidth = 2;
    ctx.font = 'bold 12px sans-serif';
    ctx.textAlign = 'center';

    data.filter(point => point.value > 0.7).forEach((point, index) => {
      ctx.strokeText(`${Math.round(point.value * 100)}%`, point.x, point.y - 10);
      ctx.fillText(`${Math.round(point.value * 100)}%`, point.x, point.y - 10);
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

export default DamageHeatmap;
