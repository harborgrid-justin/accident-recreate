import React, { useRef, useEffect, useMemo } from 'react';
import * as d3 from 'd3';
import { RadarChartProps } from '../types';
import { DEFAULT_CHART_CONFIG } from '../index';
import ChartContainer from '../components/ChartContainer';
import ChartLegend from '../components/ChartLegend';
import { useLegendVisibility } from '../hooks/useChartData';

/**
 * Radar/Spider chart for multivariate data
 */
const RadarChart: React.FC<RadarChartProps> = ({
  data,
  config = {},
  maxValue,
  levels = 5,
  className = '',
  style = {},
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const mergedConfig = { ...DEFAULT_CHART_CONFIG, ...config };

  const { visibility, toggleSeries, visibleSeries } = useLegendVisibility(
    data.map((s) => ({ ...s, chartType: 'radar' as const, data: [] }))
  );

  const legendItems = useMemo(() => {
    return data.map((series, idx) => ({
      name: series.name,
      color: series.color || mergedConfig.colors.primary[idx % mergedConfig.colors.primary.length],
      visible: visibility[series.name] !== false,
      chartType: 'radar' as const,
    }));
  }, [data, visibility, mergedConfig.colors.primary]);

  const filteredData = useMemo(() => {
    return data.filter((s) => visibility[s.name] !== false);
  }, [data, visibility]);

  useEffect(() => {
    if (!svgRef.current || !containerRef.current || !filteredData.length) return;

    const svg = d3.select(svgRef.current);
    const container = containerRef.current;
    const { width: containerWidth } = container.getBoundingClientRect();

    const width = mergedConfig.responsive ? containerWidth : mergedConfig.width;
    const height = mergedConfig.height;
    const margin = mergedConfig.margin;

    svg.selectAll('*').remove();
    svg.attr('width', width).attr('height', height);

    const centerX = width / 2;
    const centerY = height / 2;
    const radius = Math.min(width - margin.left - margin.right, height - margin.top - margin.bottom) / 2 - 40;

    const g = svg
      .append('g')
      .attr('transform', `translate(${centerX},${centerY})`);

    // Get all axes
    const axes = filteredData[0].data.map((d) => d.axis);
    const total = axes.length;

    // Calculate max value
    const dataMax = maxValue !== undefined
      ? maxValue
      : d3.max(filteredData.flatMap((s) => s.data.map((d) => d.value))) || 100;

    // Create radial scale
    const radialScale = d3.scaleLinear().domain([0, dataMax]).range([0, radius]);

    // Draw concentric circles (levels)
    const levelStep = dataMax / levels;
    for (let level = 1; level <= levels; level++) {
      const levelRadius = radialScale(level * levelStep);

      g.append('circle')
        .attr('cx', 0)
        .attr('cy', 0)
        .attr('r', levelRadius)
        .attr('fill', 'none')
        .attr('stroke', mergedConfig.colors.grid)
        .attr('stroke-width', 1)
        .attr('opacity', 0.5);

      // Add level labels
      g.append('text')
        .attr('x', 5)
        .attr('y', -levelRadius)
        .attr('fill', mergedConfig.colors.axis)
        .style('font-family', mergedConfig.font.family)
        .style('font-size', `${mergedConfig.font.size - 2}px`)
        .text((level * levelStep).toFixed(0));
    }

    // Draw axis lines and labels
    axes.forEach((axis, i) => {
      const angle = (Math.PI * 2 * i) / total - Math.PI / 2;
      const lineX = Math.cos(angle) * radius;
      const lineY = Math.sin(angle) * radius;

      // Axis line
      g.append('line')
        .attr('x1', 0)
        .attr('y1', 0)
        .attr('x2', lineX)
        .attr('y2', lineY)
        .attr('stroke', mergedConfig.colors.axis)
        .attr('stroke-width', 1.5);

      // Axis label
      const labelX = Math.cos(angle) * (radius + 25);
      const labelY = Math.sin(angle) * (radius + 25);

      g.append('text')
        .attr('x', labelX)
        .attr('y', labelY)
        .attr('text-anchor', 'middle')
        .attr('dominant-baseline', 'middle')
        .attr('fill', mergedConfig.colors.text)
        .style('font-family', mergedConfig.font.family)
        .style('font-size', `${mergedConfig.font.size}px`)
        .style('font-weight', 'bold')
        .text(axis);
    });

    // Function to calculate point coordinates
    const angleToCoordinate = (angle: number, value: number) => {
      const x = Math.cos(angle) * radialScale(value);
      const y = Math.sin(angle) * radialScale(value);
      return { x, y };
    };

    // Draw data for each series
    filteredData.forEach((series, idx) => {
      const color = series.color || mergedConfig.colors.primary[idx % mergedConfig.colors.primary.length];

      // Create path coordinates
      const coordinates = series.data.map((d, i) => {
        const angle = (Math.PI * 2 * i) / total - Math.PI / 2;
        return angleToCoordinate(angle, d.value);
      });

      // Create line generator
      const lineGenerator = d3
        .lineRadial<{ x: number; y: number }>()
        .angle((_d, i) => (Math.PI * 2 * i) / total - Math.PI / 2)
        .radius((d, i) => radialScale(series.data[i].value))
        .curve(d3.curveLinearClosed);

      // Draw area
      g.append('path')
        .datum(series.data.map((d) => ({ x: 0, y: 0 }))) // Dummy data for shape
        .attr('class', `radar-area-${series.name.replace(/\s+/g, '-')}`)
        .attr('d', lineGenerator as any)
        .attr('fill', color)
        .attr('fill-opacity', 0.2)
        .attr('stroke', color)
        .attr('stroke-width', 2);

      // Draw points
      series.data.forEach((d, i) => {
        const angle = (Math.PI * 2 * i) / total - Math.PI / 2;
        const coord = angleToCoordinate(angle, d.value);

        g.append('circle')
          .attr('cx', 0)
          .attr('cy', 0)
          .attr('r', 5)
          .attr('fill', color)
          .attr('stroke', 'white')
          .attr('stroke-width', 2)
          .style('cursor', 'pointer')
          .append('title')
          .text(`${d.axis}: ${d.value}`);

        // Animate
        if (mergedConfig.animation.enabled) {
          g.select(`.radar-area-${series.name.replace(/\s+/g, '-')}`)
            .transition()
            .duration(mergedConfig.animation.duration)
            .attrTween('d', function () {
              return function (t: number) {
                const interpolatedData = series.data.map((point) => ({
                  x: 0,
                  y: point.value * t,
                }));
                return lineGenerator(interpolatedData as any) || '';
              };
            });

          g.selectAll('circle')
            .filter(function () {
              return d3.select(this).attr('cx') === '0';
            })
            .transition()
            .duration(mergedConfig.animation.duration)
            .delay(i * 50)
            .attr('cx', coord.x)
            .attr('cy', coord.y);
        } else {
          g.selectAll('circle').attr('cx', (_, i) => {
            const angle = (Math.PI * 2 * i) / total - Math.PI / 2;
            const value = series.data[i % series.data.length].value;
            return angleToCoordinate(angle, value).x;
          }).attr('cy', (_, i) => {
            const angle = (Math.PI * 2 * i) / total - Math.PI / 2;
            const value = series.data[i % series.data.length].value;
            return angleToCoordinate(angle, value).y;
          });
        }
      });
    });
  }, [filteredData, mergedConfig, maxValue, levels]);

  return (
    <div ref={containerRef} className={className} style={style}>
      <ChartContainer config={mergedConfig} title="Radar Chart">
        <svg ref={svgRef} />
      </ChartContainer>
      <ChartLegend items={legendItems} config={mergedConfig} onToggle={toggleSeries} />
    </div>
  );
};

export default RadarChart;
