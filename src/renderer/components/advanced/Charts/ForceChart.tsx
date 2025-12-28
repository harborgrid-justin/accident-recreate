/**
 * ForceChart Component
 * Visualizes force data over time
 */

import React, { useRef, useEffect } from 'react';
import { ForceChartProps } from '../types';
import './Charts.css';

export const ForceChart: React.FC<ForceChartProps> = ({
  data,
  title = 'Force Analysis',
  showLegend = true,
  height = 300,
  unit = 'N',
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Set canvas size
    canvas.width = canvas.offsetWidth;
    canvas.height = height;

    // Draw chart
    drawChart(ctx, data, canvas.width, canvas.height, unit);
  }, [data, height, unit]);

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
                style={{ backgroundColor: dataset.color || '#0088ff' }}
              />
              <span className="legend-label">{dataset.label}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

function drawChart(
  ctx: CanvasRenderingContext2D,
  data: any,
  width: number,
  height: number,
  unit: string
) {
  const padding = 40;
  const chartWidth = width - padding * 2;
  const chartHeight = height - padding * 2;

  // Find max value
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

  // Draw data
  data.datasets.forEach((dataset: any, datasetIndex: number) => {
    ctx.strokeStyle = dataset.color || '#0088ff';
    ctx.fillStyle = dataset.backgroundColor || 'rgba(0, 136, 255, 0.2)';
    ctx.lineWidth = 2;

    const points: [number, number][] = [];

    dataset.data.forEach((value: number, index: number) => {
      const x = padding + (chartWidth / (dataset.data.length - 1)) * index;
      const y = height - padding - (value / maxValue) * chartHeight;
      points.push([x, y]);
    });

    // Draw line
    ctx.beginPath();
    ctx.moveTo(points[0][0], points[0][1]);
    points.forEach(([x, y]) => {
      ctx.lineTo(x, y);
    });
    ctx.stroke();

    // Fill area if specified
    if (dataset.fill) {
      ctx.lineTo(points[points.length - 1][0], height - padding);
      ctx.lineTo(points[0][0], height - padding);
      ctx.closePath();
      ctx.fill();
    }
  });

  // Draw labels
  ctx.fillStyle = '#aaa';
  ctx.font = '12px sans-serif';
  ctx.textAlign = 'center';

  data.labels.forEach((label: string, index: number) => {
    const x = padding + (chartWidth / (data.labels.length - 1)) * index;
    ctx.fillText(label, x, height - padding + 20);
  });

  // Draw Y-axis labels
  ctx.textAlign = 'right';
  for (let i = 0; i <= 5; i++) {
    const value = (maxValue / 5) * (5 - i);
    const y = padding + (chartHeight / 5) * i;
    ctx.fillText(`${value.toFixed(0)} ${unit}`, padding - 10, y + 4);
  }
}

export default ForceChart;
