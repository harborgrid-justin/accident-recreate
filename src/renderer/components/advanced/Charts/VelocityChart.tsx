/**
 * VelocityChart Component
 * Visualizes velocity over time with optional acceleration
 */

import React, { useRef, useEffect } from 'react';
import { VelocityChartProps } from '../types';
import './Charts.css';

export const VelocityChart: React.FC<VelocityChartProps> = ({
  data,
  title = 'Velocity Profile',
  showLegend = true,
  height = 300,
  showAcceleration = false,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    canvas.width = canvas.offsetWidth;
    canvas.height = height;

    drawVelocityChart(ctx, data, canvas.width, canvas.height, showAcceleration);
  }, [data, height, showAcceleration]);

  return (
    <div className="chart-container">
      {title && <h3 className="chart-title">{title}</h3>}
      <canvas ref={canvasRef} className="chart-canvas" />
      {showLegend && (
        <div className="chart-legend">
          {data.datasets.map((dataset, index) => (
            <div key={index} className="legend-item">
              <span
                className="legend-color"
                style={{ backgroundColor: dataset.color || '#00ff88' }}
              />
              <span className="legend-label">{dataset.label}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

function drawVelocityChart(
  ctx: CanvasRenderingContext2D,
  data: any,
  width: number,
  height: number,
  showAcceleration: boolean
) {
  const padding = 40;
  const chartWidth = width - padding * 2;
  const chartHeight = height - padding * 2;

  let maxValue = 0;
  data.datasets.forEach((dataset: any) => {
    const max = Math.max(...dataset.data);
    if (max > maxValue) maxValue = max;
  });

  // Draw axes
  ctx.strokeStyle = '#404060';
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(padding, padding);
  ctx.lineTo(padding, height - padding);
  ctx.lineTo(width - padding, height - padding);
  ctx.stroke();

  // Draw grid
  ctx.strokeStyle = '#2a2a3e';
  ctx.lineWidth = 1;
  for (let i = 0; i <= 5; i++) {
    const y = padding + (chartHeight / 5) * i;
    ctx.beginPath();
    ctx.moveTo(padding, y);
    ctx.lineTo(width - padding, y);
    ctx.stroke();
  }

  // Draw velocity lines
  data.datasets.forEach((dataset: any) => {
    ctx.strokeStyle = dataset.color || '#00ff88';
    ctx.lineWidth = 3;
    ctx.lineJoin = 'round';

    ctx.beginPath();
    dataset.data.forEach((value: number, index: number) => {
      const x = padding + (chartWidth / (dataset.data.length - 1)) * index;
      const y = height - padding - (value / maxValue) * chartHeight;

      if (index === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    });
    ctx.stroke();

    // Draw points
    ctx.fillStyle = dataset.color || '#00ff88';
    dataset.data.forEach((value: number, index: number) => {
      const x = padding + (chartWidth / (dataset.data.length - 1)) * index;
      const y = height - padding - (value / maxValue) * chartHeight;

      ctx.beginPath();
      ctx.arc(x, y, 4, 0, Math.PI * 2);
      ctx.fill();
    });
  });

  // Draw labels
  ctx.fillStyle = '#aaa';
  ctx.font = '12px sans-serif';
  ctx.textAlign = 'center';

  data.labels.forEach((label: string, index: number) => {
    const x = padding + (chartWidth / (data.labels.length - 1)) * index;
    ctx.fillText(label, x, height - padding + 20);
  });

  ctx.textAlign = 'right';
  for (let i = 0; i <= 5; i++) {
    const value = (maxValue / 5) * (5 - i);
    const y = padding + (chartHeight / 5) * i;
    ctx.fillText(`${value.toFixed(1)} m/s`, padding - 10, y + 4);
  }
}

export default VelocityChart;
