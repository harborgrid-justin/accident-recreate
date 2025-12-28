import React, { useRef, useEffect } from 'react';
import * as d3 from 'd3';
import { HeatmapChartProps } from '../types';
import { DEFAULT_CHART_CONFIG } from '../index';
import ChartContainer from '../components/ChartContainer';

/**
 * Heatmap visualization with color gradients
 */
const HeatmapChart: React.FC<HeatmapChartProps> = ({
  data,
  config = {},
  colorScheme,
  showValues = false,
  className = '',
  style = {},
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const mergedConfig = { ...DEFAULT_CHART_CONFIG, ...config };

  useEffect(() => {
    if (!svgRef.current || !containerRef.current || !data.data.length) return;

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

    // Get unique x and y values
    const xLabels = data.xLabels;
    const yLabels = data.yLabels;

    // Create scales
    const xScale = d3
      .scaleBand()
      .domain(xLabels)
      .range([0, boundedWidth])
      .padding(0.05);

    const yScale = d3
      .scaleBand()
      .domain(yLabels)
      .range([0, boundedHeight])
      .padding(0.05);

    // Get value extent for color scale
    const values = data.data.map((d) => d.value);
    const valueExtent = d3.extent(values) as [number, number];

    // Create color scale
    let colorScale: d3.ScaleSequential<string>;

    if (colorScheme) {
      colorScale = d3
        .scaleSequential()
        .domain(valueExtent)
        .interpolator(d3.interpolateRgbBasis(colorScheme));
    } else {
      // Default color schemes based on type
      switch (data.colorScale) {
        case 'diverging':
          colorScale = d3
            .scaleSequential(d3.interpolateRdBu)
            .domain([valueExtent[1], valueExtent[0]]); // Reverse for diverging
          break;
        case 'categorical':
          colorScale = d3
            .scaleSequential(d3.interpolateViridis)
            .domain(valueExtent);
          break;
        default:
          colorScale = d3
            .scaleSequential(d3.interpolateBlues)
            .domain(valueExtent);
      }
    }

    // Draw cells
    const cells = g
      .selectAll('.cell')
      .data(data.data)
      .join('rect')
      .attr('class', 'cell')
      .attr('x', (d) => xScale(d.xLabel || String(d.x))!)
      .attr('y', (d) => yScale(d.yLabel || String(d.y))!)
      .attr('width', xScale.bandwidth())
      .attr('height', yScale.bandwidth())
      .attr('fill', (d) => colorScale(d.value))
      .attr('stroke', 'white')
      .attr('stroke-width', 1)
      .style('cursor', 'pointer');

    // Animate cells
    if (mergedConfig.animation.enabled) {
      cells
        .attr('opacity', 0)
        .transition()
        .duration(mergedConfig.animation.duration)
        .delay((_d, i) => i * 5)
        .attr('opacity', 1);
    }

    // Add hover effects
    cells
      .on('mouseover', function () {
        d3.select(this)
          .attr('stroke', mergedConfig.colors.text)
          .attr('stroke-width', 2);
      })
      .on('mouseout', function () {
        d3.select(this)
          .attr('stroke', 'white')
          .attr('stroke-width', 1);
      });

    // Add values if requested
    if (showValues) {
      g.selectAll('.cell-text')
        .data(data.data)
        .join('text')
        .attr('class', 'cell-text')
        .attr('x', (d) => xScale(d.xLabel || String(d.x))! + xScale.bandwidth() / 2)
        .attr('y', (d) => yScale(d.yLabel || String(d.y))! + yScale.bandwidth() / 2)
        .attr('text-anchor', 'middle')
        .attr('dominant-baseline', 'middle')
        .attr('fill', (d) => {
          // Choose contrasting color based on cell brightness
          const rgb = d3.color(colorScale(d.value))?.rgb();
          if (!rgb) return 'black';
          const brightness = (rgb.r * 299 + rgb.g * 587 + rgb.b * 114) / 1000;
          return brightness > 128 ? 'black' : 'white';
        })
        .style('font-family', mergedConfig.font.family)
        .style('font-size', `${mergedConfig.font.size}px`)
        .style('pointer-events', 'none')
        .text((d) => d.value.toFixed(2));
    }

    // Add axes
    g.append('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${boundedHeight})`)
      .call(d3.axisBottom(xScale))
      .call((g) =>
        g
          .selectAll('text')
          .attr('fill', mergedConfig.colors.text)
          .style('font-family', mergedConfig.font.family)
          .style('font-size', `${mergedConfig.font.size}px`)
          .attr('transform', 'rotate(-45)')
          .attr('text-anchor', 'end')
      )
      .call((g) => g.select('.domain').remove())
      .call((g) => g.selectAll('.tick line').remove());

    g.append('g')
      .attr('class', 'y-axis')
      .call(d3.axisLeft(yScale))
      .call((g) =>
        g
          .selectAll('text')
          .attr('fill', mergedConfig.colors.text)
          .style('font-family', mergedConfig.font.family)
          .style('font-size', `${mergedConfig.font.size}px`)
      )
      .call((g) => g.select('.domain').remove())
      .call((g) => g.selectAll('.tick line').remove());

    // Add color legend
    const legendWidth = 20;
    const legendHeight = boundedHeight;

    const legendScale = d3
      .scaleLinear()
      .domain(valueExtent)
      .range([legendHeight, 0]);

    const legendAxis = d3.axisRight(legendScale).ticks(5);

    const legend = svg
      .append('g')
      .attr('class', 'legend')
      .attr('transform', `translate(${width - margin.right + 10},${margin.top})`);

    // Create gradient
    const defs = svg.append('defs');
    const gradient = defs
      .append('linearGradient')
      .attr('id', 'heatmap-gradient')
      .attr('x1', '0%')
      .attr('y1', '100%')
      .attr('x2', '0%')
      .attr('y2', '0%');

    const numStops = 10;
    for (let i = 0; i <= numStops; i++) {
      const t = i / numStops;
      const value = valueExtent[0] + t * (valueExtent[1] - valueExtent[0]);
      gradient
        .append('stop')
        .attr('offset', `${t * 100}%`)
        .attr('stop-color', colorScale(value));
    }

    legend
      .append('rect')
      .attr('width', legendWidth)
      .attr('height', legendHeight)
      .style('fill', 'url(#heatmap-gradient)')
      .attr('stroke', mergedConfig.colors.axis)
      .attr('stroke-width', 1);

    legend
      .append('g')
      .attr('transform', `translate(${legendWidth}, 0)`)
      .call(legendAxis)
      .call((g) =>
        g
          .selectAll('text')
          .attr('fill', mergedConfig.colors.text)
          .style('font-family', mergedConfig.font.family)
          .style('font-size', `${mergedConfig.font.size}px`)
      )
      .call((g) => g.select('.domain').attr('stroke', mergedConfig.colors.axis));
  }, [data, mergedConfig, colorScheme, showValues]);

  return (
    <div ref={containerRef} className={className} style={style}>
      <ChartContainer config={mergedConfig} title="Heatmap">
        <svg ref={svgRef} />
      </ChartContainer>
    </div>
  );
};

export default HeatmapChart;
