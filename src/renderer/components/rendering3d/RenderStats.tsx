/**
 * AccuScene Enterprise v0.3.0
 * Performance HUD (FPS, Draw Calls, Memory)
 */

import React from 'react';
import { RenderStats as Stats } from './types';

interface RenderStatsProps {
  stats: Stats;
  position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
  visible?: boolean;
}

export const RenderStats: React.FC<RenderStatsProps> = ({
  stats,
  position = 'top-left',
  visible = true,
}) => {
  if (!visible) return null;

  const positionStyles: Record<string, React.CSSProperties> = {
    'top-left': { top: 10, left: 10 },
    'top-right': { top: 10, right: 10 },
    'bottom-left': { bottom: 10, left: 10 },
    'bottom-right': { bottom: 10, right: 10 },
  };

  const formatMemory = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const getFPSColor = (fps: number): string => {
    if (fps >= 60) return '#00ff00';
    if (fps >= 30) return '#ffff00';
    return '#ff0000';
  };

  return (
    <div
      style={{
        position: 'absolute',
        ...positionStyles[position],
        padding: '10px',
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        color: '#fff',
        fontFamily: 'monospace',
        fontSize: '12px',
        borderRadius: '4px',
        pointerEvents: 'none',
        zIndex: 1000,
        minWidth: '200px',
      }}
    >
      <div style={{ marginBottom: '8px', fontWeight: 'bold', fontSize: '14px' }}>
        Render Stats
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '4px 12px' }}>
        <span>FPS:</span>
        <span style={{ color: getFPSColor(stats.fps), fontWeight: 'bold' }}>
          {stats.fps}
        </span>

        <span>Frame:</span>
        <span>{stats.frameTime.toFixed(2)} ms</span>

        <span>GPU:</span>
        <span>{stats.gpuTime.toFixed(2)} ms</span>

        <span style={{ marginTop: '8px', gridColumn: '1 / -1', borderTop: '1px solid #444', paddingTop: '8px' }}>
          Geometry
        </span>

        <span>Draws:</span>
        <span>{stats.drawCalls}</span>

        <span>Triangles:</span>
        <span>{stats.triangles.toLocaleString()}</span>

        <span>Vertices:</span>
        <span>{stats.vertices.toLocaleString()}</span>

        <span>Instances:</span>
        <span>{stats.instances.toLocaleString()}</span>

        <span style={{ marginTop: '8px', gridColumn: '1 / -1', borderTop: '1px solid #444', paddingTop: '8px' }}>
          Resources
        </span>

        <span>Textures:</span>
        <span>{stats.textures}</span>

        <span>Buffers:</span>
        <span>{stats.buffers}</span>

        <span>Memory:</span>
        <span>{formatMemory(stats.memoryUsed)}</span>
      </div>

      {/* Performance indicator */}
      <div style={{ marginTop: '12px', paddingTop: '8px', borderTop: '1px solid #444' }}>
        <div style={{ fontSize: '10px', marginBottom: '4px' }}>Performance</div>
        <div style={{ display: 'flex', height: '4px', backgroundColor: '#333', borderRadius: '2px', overflow: 'hidden' }}>
          <div
            style={{
              width: `${Math.min(100, (stats.fps / 60) * 100)}%`,
              backgroundColor: getFPSColor(stats.fps),
              transition: 'width 0.3s',
            }}
          />
        </div>
      </div>
    </div>
  );
};

export default RenderStats;
