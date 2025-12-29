/**
 * AccuScene Enterprise v0.3.0 - Energy Flow Widget
 * Sankey diagram visualization of energy transfer between components
 */

import React, { useMemo } from 'react';
import { WidgetProps, AnalyticsData, EnergyTransfer } from '../types';

interface SankeyNode {
  id: string;
  label: string;
  value: number;
  color: string;
}

interface SankeyLink {
  source: string;
  target: string;
  value: number;
  type: string;
}

const EnergyFlowWidget: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  // Process energy transfer data
  const { nodes, links } = useMemo(() => {
    if (!data?.energyTransfers || data.energyTransfers.length === 0) {
      return { nodes: [], links: [] };
    }

    const nodeMap = new Map<string, SankeyNode>();
    const linkArray: SankeyLink[] = [];

    // Build nodes and links
    data.energyTransfers.forEach((transfer) => {
      // Source node
      if (!nodeMap.has(transfer.source)) {
        nodeMap.set(transfer.source, {
          id: transfer.source,
          label: transfer.source,
          value: 0,
          color: getEnergyColor(transfer.type),
        });
      }
      nodeMap.get(transfer.source)!.value += transfer.amount;

      // Target node
      if (!nodeMap.has(transfer.target)) {
        nodeMap.set(transfer.target, {
          id: transfer.target,
          label: transfer.target,
          value: 0,
          color: getEnergyColor(transfer.type),
        });
      }

      // Link
      linkArray.push({
        source: transfer.source,
        target: transfer.target,
        value: transfer.amount,
        type: transfer.type,
      });
    });

    return {
      nodes: Array.from(nodeMap.values()),
      links: linkArray,
    };
  }, [data]);

  // Calculate statistics
  const stats = useMemo(() => {
    if (links.length === 0) return null;

    const totalEnergy = links.reduce((sum, link) => sum + link.value, 0);
    const byType = links.reduce((acc, link) => {
      acc[link.type] = (acc[link.type] || 0) + link.value;
      return acc;
    }, {} as Record<string, number>);

    return {
      total: totalEnergy,
      byType,
      transferCount: links.length,
    };
  }, [links]);

  if (nodes.length === 0) {
    return (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#94a3b8',
        }}
      >
        No energy transfer data available
      </div>
    );
  }

  // Simple Sankey-style visualization
  const nodeHeight = 60;
  const nodeWidth = 120;
  const nodeSpacing = 100;

  // Arrange nodes in columns
  const sourceNodes = nodes.filter((n) =>
    links.some((l) => l.source === n.id && !links.some((l2) => l2.target === n.id))
  );
  const targetNodes = nodes.filter((n) =>
    links.some((l) => l.target === n.id && !links.some((l2) => l2.source === n.id))
  );
  const intermediateNodes = nodes.filter(
    (n) => !sourceNodes.includes(n) && !targetNodes.includes(n)
  );

  const columns = [sourceNodes, intermediateNodes, targetNodes].filter(
    (col) => col.length > 0
  );

  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        gap: '1rem',
      }}
    >
      {/* Statistics */}
      {stats && (
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
            gap: '0.75rem',
          }}
        >
          <div
            style={{
              backgroundColor: '#1e293b',
              padding: '0.75rem',
              borderRadius: '4px',
              border: '1px solid #334155',
            }}
          >
            <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
              Total Energy
            </div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: '#f1f5f9' }}>
              {stats.total.toFixed(0)}
              <span style={{ fontSize: '0.75rem', marginLeft: '0.25rem' }}>
                J
              </span>
            </div>
          </div>
          {Object.entries(stats.byType).map(([type, value]) => (
            <div
              key={type}
              style={{
                backgroundColor: '#1e293b',
                padding: '0.75rem',
                borderRadius: '4px',
                border: '1px solid #334155',
              }}
            >
              <div style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                {type.charAt(0).toUpperCase() + type.slice(1)}
              </div>
              <div
                style={{
                  fontSize: '1.25rem',
                  fontWeight: 600,
                  color: getEnergyColor(type as any),
                }}
              >
                {value.toFixed(0)}
                <span style={{ fontSize: '0.75rem', marginLeft: '0.25rem' }}>
                  J
                </span>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Flow Diagram */}
      <div
        style={{
          flex: 1,
          position: 'relative',
          overflow: 'auto',
          backgroundColor: '#1e293b',
          borderRadius: '4px',
          border: '1px solid #334155',
        }}
      >
        <svg
          width="100%"
          height="100%"
          style={{ minWidth: '600px', minHeight: '400px' }}
        >
          {/* Draw links */}
          {links.map((link, index) => {
            const sourceNode = nodes.find((n) => n.id === link.source);
            const targetNode = nodes.find((n) => n.id === link.target);
            if (!sourceNode || !targetNode) return null;

            const sourceCol = columns.findIndex((col) =>
              col.some((n) => n.id === sourceNode.id)
            );
            const targetCol = columns.findIndex((col) =>
              col.some((n) => n.id === targetNode.id)
            );
            const sourceIndex = columns[sourceCol].indexOf(sourceNode);
            const targetIndex = columns[targetCol].indexOf(targetNode);

            const x1 = 100 + sourceCol * (nodeWidth + nodeSpacing) + nodeWidth;
            const y1 =
              100 + sourceIndex * (nodeHeight + 40) + nodeHeight / 2;
            const x2 = 100 + targetCol * (nodeWidth + nodeSpacing);
            const y2 =
              100 + targetIndex * (nodeHeight + 40) + nodeHeight / 2;

            const midX = (x1 + x2) / 2;

            return (
              <g key={index}>
                <path
                  d={`M ${x1} ${y1} C ${midX} ${y1}, ${midX} ${y2}, ${x2} ${y2}`}
                  stroke={getEnergyColor(link.type as any)}
                  strokeWidth={Math.max(2, link.value / 10000)}
                  fill="none"
                  opacity={0.6}
                />
                <text
                  x={midX}
                  y={(y1 + y2) / 2 - 5}
                  fill="#94a3b8"
                  fontSize="10"
                  textAnchor="middle"
                >
                  {link.value.toFixed(0)} J
                </text>
              </g>
            );
          })}

          {/* Draw nodes */}
          {columns.map((column, colIndex) =>
            column.map((node, nodeIndex) => {
              const x = 100 + colIndex * (nodeWidth + nodeSpacing);
              const y = 100 + nodeIndex * (nodeHeight + 40);

              return (
                <g key={node.id}>
                  <rect
                    x={x}
                    y={y}
                    width={nodeWidth}
                    height={nodeHeight}
                    fill={node.color}
                    rx={4}
                    opacity={0.8}
                  />
                  <text
                    x={x + nodeWidth / 2}
                    y={y + nodeHeight / 2 - 8}
                    fill="#f1f5f9"
                    fontSize="12"
                    fontWeight="600"
                    textAnchor="middle"
                  >
                    {node.label}
                  </text>
                  <text
                    x={x + nodeWidth / 2}
                    y={y + nodeHeight / 2 + 8}
                    fill="#cbd5e1"
                    fontSize="10"
                    textAnchor="middle"
                  >
                    {node.value.toFixed(0)} J
                  </text>
                </g>
              );
            })
          )}
        </svg>
      </div>

      {/* Legend */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          gap: '1rem',
          fontSize: '0.75rem',
          color: '#94a3b8',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <div
            style={{
              width: '12px',
              height: '12px',
              backgroundColor: getEnergyColor('kinetic'),
              borderRadius: '2px',
            }}
          />
          Kinetic
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <div
            style={{
              width: '12px',
              height: '12px',
              backgroundColor: getEnergyColor('potential'),
              borderRadius: '2px',
            }}
          />
          Potential
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <div
            style={{
              width: '12px',
              height: '12px',
              backgroundColor: getEnergyColor('heat'),
              borderRadius: '2px',
            }}
          />
          Heat
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <div
            style={{
              width: '12px',
              height: '12px',
              backgroundColor: getEnergyColor('deformation'),
              borderRadius: '2px',
            }}
          />
          Deformation
        </div>
      </div>
    </div>
  );
};

// Helper function to get color based on energy type
function getEnergyColor(type: 'kinetic' | 'potential' | 'heat' | 'deformation'): string {
  switch (type) {
    case 'kinetic':
      return '#3b82f6';
    case 'potential':
      return '#10b981';
    case 'heat':
      return '#ef4444';
    case 'deformation':
      return '#f59e0b';
    default:
      return '#8b5cf6';
  }
}

export default EnergyFlowWidget;
