import React, { useRef, useEffect } from 'react';
import * as d3 from 'd3';
import { sankey, sankeyLinkHorizontal, SankeyGraph, SankeyNode, SankeyLink } from 'd3-sankey';
import { SankeyDiagramProps } from '../types';
import { DEFAULT_CHART_CONFIG } from '../index';
import ChartContainer from '../components/ChartContainer';

interface D3SankeyNode extends SankeyNode<any, any> {
  id: string;
  label: string;
  color?: string;
}

interface D3SankeyLink extends SankeyLink<D3SankeyNode, any> {
  source: string | D3SankeyNode;
  target: string | D3SankeyNode;
  value: number;
  color?: string;
}

/**
 * Sankey diagram for flow visualization
 */
const SankeyDiagram: React.FC<SankeyDiagramProps> = ({
  data,
  config = {},
  nodeWidth = 20,
  nodePadding = 10,
  className = '',
  style = {},
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const mergedConfig = { ...DEFAULT_CHART_CONFIG, ...config };

  useEffect(() => {
    if (!svgRef.current || !containerRef.current || !data.nodes.length || !data.links.length) return;

    const svg = d3.select(svgRef.current);
    const container = containerRef.current;
    const { width: containerWidth } = container.getBoundingClientRect();

    const width = mergedConfig.responsive ? containerWidth : mergedConfig.width;
    const height = mergedConfig.height;
    const margin = mergedConfig.margin;

    const boundedWidth = width - margin.left - margin.right;
    const boundedHeight = height - margin.top - margin.bottom;

    svg.selectAll('*').remove();
    svg.attr('width', width).attr('height', height);

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Create sankey generator
    const sankeyGenerator = sankey<D3SankeyNode, D3SankeyLink>()
      .nodeWidth(nodeWidth)
      .nodePadding(nodePadding)
      .extent([
        [0, 0],
        [boundedWidth, boundedHeight],
      ])
      .nodeId((d: any) => d.id);

    // Prepare data
    const nodes: D3SankeyNode[] = data.nodes.map((n) => ({
      ...n,
      id: n.id,
      label: n.label,
      color: n.color,
    }));

    const links: D3SankeyLink[] = data.links.map((l) => ({
      source: l.source,
      target: l.target,
      value: l.value,
      color: l.color,
    }));

    const sankeyData = sankeyGenerator({
      nodes,
      links,
    } as SankeyGraph<D3SankeyNode, D3SankeyLink>);

    // Color scale
    const colorScale = d3
      .scaleOrdinal<string>()
      .domain(data.nodes.map((n) => n.id))
      .range(mergedConfig.colors.primary);

    // Draw links
    const linkGroup = g
      .append('g')
      .attr('class', 'links')
      .attr('fill', 'none');

    const linkPaths = linkGroup
      .selectAll('.link')
      .data(sankeyData.links)
      .join('path')
      .attr('class', 'link')
      .attr('d', sankeyLinkHorizontal())
      .attr('stroke', (d) => {
        if (d.color) return d.color;
        const sourceNode = d.source as D3SankeyNode;
        return sourceNode.color || colorScale(sourceNode.id);
      })
      .attr('stroke-width', (d) => Math.max(1, d.width || 0))
      .attr('opacity', 0.5)
      .style('cursor', 'pointer');

    // Add link interactivity
    linkPaths
      .on('mouseover', function (event, d) {
        d3.select(this).attr('opacity', 0.8);

        // Show tooltip
        const sourceNode = d.source as D3SankeyNode;
        const targetNode = d.target as D3SankeyNode;

        g.append('text')
          .attr('class', 'link-tooltip')
          .attr('x', event.offsetX)
          .attr('y', event.offsetY - 10)
          .attr('text-anchor', 'middle')
          .attr('fill', mergedConfig.colors.text)
          .style('font-family', mergedConfig.font.family)
          .style('font-size', `${mergedConfig.font.size}px`)
          .style('pointer-events', 'none')
          .text(`${sourceNode.label} → ${targetNode.label}: ${d.value}`);
      })
      .on('mouseout', function () {
        d3.select(this).attr('opacity', 0.5);
        g.selectAll('.link-tooltip').remove();
      });

    // Animate links
    if (mergedConfig.animation.enabled) {
      linkPaths
        .attr('stroke-dasharray', function () {
          const length = (this as SVGPathElement).getTotalLength();
          return `${length} ${length}`;
        })
        .attr('stroke-dashoffset', function () {
          return (this as SVGPathElement).getTotalLength();
        })
        .transition()
        .duration(mergedConfig.animation.duration * 2)
        .ease(d3.easeCubicInOut)
        .attr('stroke-dashoffset', 0);
    }

    // Draw nodes
    const nodeGroup = g.append('g').attr('class', 'nodes');

    const nodeRects = nodeGroup
      .selectAll('.node')
      .data(sankeyData.nodes)
      .join('g')
      .attr('class', 'node')
      .style('cursor', 'pointer');

    nodeRects
      .append('rect')
      .attr('x', (d) => d.x0 || 0)
      .attr('y', (d) => d.y0 || 0)
      .attr('width', (d) => (d.x1 || 0) - (d.x0 || 0))
      .attr('height', (d) => (d.y1 || 0) - (d.y0 || 0))
      .attr('fill', (d) => d.color || colorScale(d.id))
      .attr('stroke', 'white')
      .attr('stroke-width', 2)
      .on('mouseover', function () {
        d3.select(this).attr('opacity', 0.8);
      })
      .on('mouseout', function () {
        d3.select(this).attr('opacity', 1);
      });

    // Animate nodes
    if (mergedConfig.animation.enabled) {
      nodeRects
        .selectAll('rect')
        .attr('height', 0)
        .transition()
        .duration(mergedConfig.animation.duration)
        .delay((_d, i) => i * 50)
        .attr('height', (d: any) => (d.y1 || 0) - (d.y0 || 0));
    }

    // Add node labels
    nodeRects
      .append('text')
      .attr('x', (d) => {
        const x0 = d.x0 || 0;
        const x1 = d.x1 || 0;
        return x0 < boundedWidth / 2 ? x1 + 6 : x0 - 6;
      })
      .attr('y', (d) => ((d.y0 || 0) + (d.y1 || 0)) / 2)
      .attr('text-anchor', (d) => {
        const x0 = d.x0 || 0;
        return x0 < boundedWidth / 2 ? 'start' : 'end';
      })
      .attr('dominant-baseline', 'middle')
      .attr('fill', mergedConfig.colors.text)
      .style('font-family', mergedConfig.font.family)
      .style('font-size', `${mergedConfig.font.size}px`)
      .style('font-weight', 'bold')
      .text((d) => d.label);

    // Add node values
    nodeRects
      .append('text')
      .attr('x', (d) => ((d.x0 || 0) + (d.x1 || 0)) / 2)
      .attr('y', (d) => ((d.y0 || 0) + (d.y1 || 0)) / 2)
      .attr('text-anchor', 'middle')
      .attr('dominant-baseline', 'middle')
      .attr('fill', 'white')
      .style('font-family', mergedConfig.font.family)
      .style('font-size', `${mergedConfig.font.size - 2}px`)
      .style('pointer-events', 'none')
      .text((d) => d.value?.toFixed(0) || '');

    // Add title
    if (mergedConfig.animation.enabled) {
      nodeRects
        .selectAll('text')
        .attr('opacity', 0)
        .transition()
        .duration(mergedConfig.animation.duration)
        .delay(mergedConfig.animation.duration)
        .attr('opacity', 1);
    }
  }, [data, mergedConfig, nodeWidth, nodePadding]);

  return (
    <div ref={containerRef} className={className} style={style}>
      <ChartContainer config={mergedConfig} title="Sankey Diagram">
        <svg ref={svgRef} />
      </ChartContainer>
    </div>
  );
};

export default SankeyDiagram;
