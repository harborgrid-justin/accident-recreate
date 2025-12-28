/**
 * ImpactChart Component
 * Visualizes impact forces with peak highlighting
 */

import React, { useRef, useEffect } from 'react';
import { ImpactChartProps } from '../types';
import './Charts.css';

export const ImpactChart: React.FC<ImpactChartProps> = ({
  data,
  title = 'Impact Analysis',
  showLegend = true,
  height = 300,
  highlightPeak = true,
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

    drawImpactChart(ctx, data, canvas.width, canvas.height, highlightPeak);
  }, [data, height, highlightPeak]);

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
                style={{ backgroundColor: dataset.color || '#ff0000' }}
              />
              <span className="legend-label">{dataset.label}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

function drawImpactChart(
  ctx: CanvasRenderingContext2D,
  data: any,
  width: number,
  height: number,
  highlightPeak: boolean
) {
  const padding = 40;
  const chartWidth = width - padding * 2;
  const chartHeight = height - padding * 2;

  let maxValue = 0;
  let peakIndex = 0;

  data.datasets.forEach((dataset: any) => {
    dataset.data.forEach((value: number, index: number) => {
      if (value > maxValue) {
        maxValue = value;
        peakIndex = index;
      }
    });
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

  // Highlight peak region
  if (highlightPeak) {
    const peakX = padding + (chartWidth / (data.labels.length - 1)) * peakIndex;
    ctx.fillStyle = 'rgba(255, 0, 0, 0.1)';
    ctx.fillRect(peakX - 20, padding, 40, chartHeight);
  }

  // Draw bars
  data.datasets.forEach((dataset: any) => {
    const barWidth = chartWidth / dataset.data.length * 0.8;

    dataset.data.forEach((value: number, index: number) => {
      const x = padding + (chartWidth / (dataset.data.length - 1)) * index - barWidth / 2;
      const barHeight = (value / maxValue) * chartHeight;
      const y = height - padding - barHeight;

      // Gradient fill
      const gradient = ctx.createLinearGradient(x, y, x, height - padding);
      gradient.addColorStop(0, dataset.color || '#ff0000');
      gradient.addColorStop(1, dataset.backgroundColor || 'rgba(255, 0, 0, 0.3)');

      ctx.fillStyle = gradient;
      ctx.fillRect(x, y, barWidth, barHeight);

      // Highlight peak
      if (highlightPeak && index === peakIndex) {
        ctx.strokeStyle = '#fff';
        ctx.lineWidth = 2;
        ctx.strokeRect(x, y, barWidth, barHeight);

        // Draw peak value
        ctx.fillStyle = '#fff';
        ctx.font = 'bold 14px sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText(`${value.toFixed(0)}`, x + barWidth / 2, y - 10);
      }
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
    ctx.fillText(`${value.toFixed(0)} kN`, padding - 10, y + 4);
  }
}

export default ImpactChart;
