import React, { useRef, useEffect, useMemo } from 'react';
import * as d3 from 'd3';
import { LineChartProps, TooltipData } from '../types';
import { DEFAULT_CHART_CONFIG, DEFAULT_INTERACTION } from '../index';
import ChartContainer from '../components/ChartContainer';
import ChartLegend from '../components/ChartLegend';
import ChartTooltip from '../components/ChartTooltip';
import { useLegendVisibility, useChartTooltip } from '../hooks/useChartData';

/**
 * Advanced line chart with zoom, pan, and interactive features
 */
const LineChart: React.FC<LineChartProps> = ({
  data,
  config = {},
  interaction = {},
  showPoints = true,
  curve = 'linear',
  onPointClick,
  onZoomChange,
  className = '',
  style = {},
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const mergedConfig = { ...DEFAULT_CHART_CONFIG, ...config };
  const mergedInteraction = { ...DEFAULT_INTERACTION, ...interaction };

  const { visibility, toggleSeries, visibleSeries } = useLegendVisibility(data.series);
  const { tooltip, showTooltip, hideTooltip } = useChartTooltip<TooltipData>();

  const legendItems = useMemo(() => {
    return data.series.map((series, idx) => ({
      name: series.name,
      color: series.color || mergedConfig.colors.primary[idx % mergedConfig.colors.primary.length],
      visible: visibility[series.name] !== false,
      chartType: series.chartType,
    }));
  }, [data.series, visibility, mergedConfig.colors.primary]);

  const curveFunction = useMemo(() => {
    switch (curve) {
      case 'monotone':
        return d3.curveMonotoneX;
      case 'step':
        return d3.curveStep;
      case 'basis':
        return d3.curveBasis;
      default:
        return d3.curveLinear;
    }
  }, [curve]);

  useEffect(() => {
    if (!svgRef.current || !containerRef.current || visibleSeries.length === 0) return;

    const svg = d3.select(svgRef.current);
    const container = containerRef.current;
    const { width: containerWidth } = container.getBoundingClientRect();

    const width = mergedConfig.responsive ? containerWidth : mergedConfig.width;
    const height = mergedConfig.height;
    const margin = mergedConfig.margin;

    const boundedWidth = width - margin.left - margin.right;
    const boundedHeight = height - margin.top - margin.bottom;

    // Clear previous content
    svg.selectAll('*').remove();

    // Set SVG dimensions
    svg.attr('width', width).attr('height', height);

    // Create main group
    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Get all data points from visible series
    const allPoints = visibleSeries.flatMap((s) => s.data);

    if (allPoints.length === 0) return;

    // Create scales
    const xExtent = d3.extent(allPoints, (d) => d.x) as [number, number];
    const yExtent = d3.extent(allPoints, (d) => d.y) as [number, number];

    const xScale = d3
      .scaleLinear()
      .domain(data.xAxis.min !== undefined && data.xAxis.max !== undefined
        ? [data.xAxis.min, data.xAxis.max]
        : xExtent
      )
      .range([0, boundedWidth])
      .nice();

    const yScale = d3
      .scaleLinear()
      .domain(data.yAxis.min !== undefined && data.yAxis.max !== undefined
        ? [data.yAxis.min, data.yAxis.max]
        : yExtent
      )
      .range([boundedHeight, 0])
      .nice();

    // Add grid
    if (data.xAxis.grid) {
      g.append('g')
        .attr('class', 'grid-x')
        .attr('transform', `translate(0,${boundedHeight})`)
        .call(
          d3
            .axisBottom(xScale)
            .tickSize(-boundedHeight)
            .tickFormat(() => '')
        )
        .call((g) => g.select('.domain').remove())
        .call((g) =>
          g
            .selectAll('.tick line')
            .attr('stroke', mergedConfig.colors.grid)
            .attr('stroke-opacity', 0.5)
        );
    }

    if (data.yAxis.grid) {
      g.append('g')
        .attr('class', 'grid-y')
        .call(
          d3
            .axisLeft(yScale)
            .tickSize(-boundedWidth)
            .tickFormat(() => '')
        )
        .call((g) => g.select('.domain').remove())
        .call((g) =>
          g
            .selectAll('.tick line')
            .attr('stroke', mergedConfig.colors.grid)
            .attr('stroke-opacity', 0.5)
        );
    }

    // Create line generator
    const lineGenerator = d3
      .line<{ x: number; y: number }>()
      .x((d) => xScale(d.x))
      .y((d) => yScale(d.y))
      .curve(curveFunction);

    // Draw lines for each series
    visibleSeries.forEach((series, idx) => {
      const color = series.color || mergedConfig.colors.primary[idx % mergedConfig.colors.primary.length];

      // Line path
      const path = g
        .append('path')
        .datum(series.data)
        .attr('class', `line-${series.name.replace(/\s+/g, '-')}`)
        .attr('fill', 'none')
        .attr('stroke', color)
        .attr('stroke-width', 2)
        .attr('d', lineGenerator);

      // Animate line drawing
      if (mergedConfig.animation.enabled) {
        const totalLength = (path.node() as SVGPathElement).getTotalLength();
        path
          .attr('stroke-dasharray', `${totalLength} ${totalLength}`)
          .attr('stroke-dashoffset', totalLength)
          .transition()
          .duration(mergedConfig.animation.duration)
          .ease(d3.easeCubicInOut)
          .attr('stroke-dashoffset', 0);
      }

      // Draw points
      if (showPoints) {
        g.selectAll(`.point-${series.name.replace(/\s+/g, '-')}`)
          .data(series.data)
          .join('circle')
          .attr('class', `point-${series.name.replace(/\s+/g, '-')}`)
          .attr('cx', (d) => xScale(d.x))
          .attr('cy', (d) => yScale(d.y))
          .attr('r', 0)
          .attr('fill', color)
          .attr('stroke', 'white')
          .attr('stroke-width', 2)
          .style('cursor', 'pointer')
          .on('mouseover', function (event, d) {
            if (mergedInteraction.tooltip) {
              const rect = (event.target as SVGElement).getBoundingClientRect();
              showTooltip(rect.left + rect.width / 2, rect.top, {
                x: d.x,
                y: d.y,
                label: d.label || `${d.x}`,
                value: d.y,
                color,
                series: series.name,
              });
              d3.select(this).transition().duration(100).attr('r', 6);
            }
          })
          .on('mouseout', function () {
            hideTooltip();
            d3.select(this).transition().duration(100).attr('r', 4);
          })
          .on('click', function (_event, d) {
            if (onPointClick) {
              onPointClick(d, series);
            }
          })
          .transition()
          .duration(mergedConfig.animation.enabled ? mergedConfig.animation.duration : 0)
          .delay((_d, i) => i * 20)
          .attr('r', 4);
      }
    });

    // Add axes
    const xAxis = g
      .append('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${boundedHeight})`)
      .call(d3.axisBottom(xScale))
      .call((g) =>
        g
          .selectAll('text')
          .attr('fill', mergedConfig.colors.text)
          .style('font-family', mergedConfig.font.family)
          .style('font-size', `${mergedConfig.font.size}px`)
      )
      .call((g) => g.select('.domain').attr('stroke', mergedConfig.colors.axis));

    const yAxis = g
      .append('g')
      .attr('class', 'y-axis')
      .call(d3.axisLeft(yScale))
      .call((g) =>
        g
          .selectAll('text')
          .attr('fill', mergedConfig.colors.text)
          .style('font-family', mergedConfig.font.family)
          .style('font-size', `${mergedConfig.font.size}px`)
      )
      .call((g) => g.select('.domain').attr('stroke', mergedConfig.colors.axis));

    // Add axis labels
    if (data.xAxis.label) {
      g.append('text')
        .attr('class', 'x-axis-label')
        .attr('x', boundedWidth / 2)
        .attr('y', boundedHeight + margin.bottom - 5)
        .attr('text-anchor', 'middle')
        .attr('fill', mergedConfig.colors.text)
        .style('font-family', mergedConfig.font.family)
        .style('font-size', `${mergedConfig.font.size + 2}px`)
        .text(data.xAxis.label);
    }

    if (data.yAxis.label) {
      g.append('text')
        .attr('class', 'y-axis-label')
        .attr('transform', 'rotate(-90)')
        .attr('x', -boundedHeight / 2)
        .attr('y', -margin.left + 15)
        .attr('text-anchor', 'middle')
        .attr('fill', mergedConfig.colors.text)
        .style('font-family', mergedConfig.font.family)
        .style('font-size', `${mergedConfig.font.size + 2}px`)
        .text(data.yAxis.label);
    }

    // Add zoom behavior
    if (mergedInteraction.zoom) {
      const zoom = d3
        .zoom<SVGSVGElement, unknown>()
        .scaleExtent([1, 10])
        .translateExtent([
          [0, 0],
          [width, height],
        ])
        .extent([
          [0, 0],
          [width, height],
        ])
        .on('zoom', (event) => {
          const transform = event.transform;

          const newXScale = transform.rescaleX(xScale);
          const newYScale = transform.rescaleY(yScale);

          xAxis.call(d3.axisBottom(newXScale) as any);
          yAxis.call(d3.axisLeft(newYScale) as any);

          g.selectAll('path[class^="line-"]').attr(
            'd',
            lineGenerator.x((d) => newXScale(d.x)).y((d) => newYScale(d.y)) as any
          );

          g.selectAll('circle[class^="point-"]')
            .attr('cx', (d: any) => newXScale(d.x))
            .attr('cy', (d: any) => newYScale(d.y));

          if (onZoomChange) {
            onZoomChange({
              xDomain: newXScale.domain() as [number, number],
              yDomain: newYScale.domain() as [number, number],
            });
          }
        });

      svg.call(zoom as any);
    }
  }, [
    data,
    visibleSeries,
    mergedConfig,
    mergedInteraction,
    curveFunction,
    showPoints,
    onPointClick,
    onZoomChange,
    showTooltip,
    hideTooltip,
  ]);

  return (
    <div ref={containerRef} className={className} style={style}>
      <ChartContainer config={mergedConfig} title={data.title}>
        <svg ref={svgRef} />
      </ChartContainer>
      {mergedInteraction.legend && (
        <ChartLegend
          items={legendItems}
          config={mergedConfig}
          onToggle={toggleSeries}
        />
      )}
      {mergedInteraction.tooltip && <ChartTooltip data={tooltip} config={mergedConfig} />}
    </div>
  );
};

export default LineChart;
