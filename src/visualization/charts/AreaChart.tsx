import React, { useRef, useEffect, useMemo } from 'react';
import * as d3 from 'd3';
import { AreaChartProps, TooltipData } from '../types';
import { DEFAULT_CHART_CONFIG, DEFAULT_INTERACTION } from '../index';
import ChartContainer from '../components/ChartContainer';
import ChartLegend from '../components/ChartLegend';
import ChartTooltip from '../components/ChartTooltip';
import { useLegendVisibility, useChartTooltip } from '../hooks/useChartData';

/**
 * Stacked area chart with smooth curves
 */
const AreaChart: React.FC<AreaChartProps> = ({
  data,
  config = {},
  interaction = {},
  stacked = false,
  curve = 'monotone',
  fillOpacity = 0.6,
  onPointClick,
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
      chartType: 'area' as const,
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

    svg.selectAll('*').remove();
    svg.attr('width', width).attr('height', height);

    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Prepare data for stacking
    let seriesData: any[];
    let yMax: number;

    if (stacked) {
      // Get all unique x values
      const xValues = Array.from(
        new Set(visibleSeries.flatMap((s) => s.data.map((d) => d.x)))
      ).sort((a, b) => a - b);

      // Create data structure for stacking
      const stackData = xValues.map((x) => {
        const point: any = { x };
        visibleSeries.forEach((series) => {
          const dataPoint = series.data.find((d) => d.x === x);
          point[series.name] = dataPoint ? dataPoint.y : 0;
        });
        return point;
      });

      // Stack the data
      const stack = d3.stack().keys(visibleSeries.map((s) => s.name));
      seriesData = stack(stackData);

      yMax = d3.max(seriesData, (layer) => d3.max(layer, (d) => d[1])) || 0;
    } else {
      const allPoints = visibleSeries.flatMap((s) => s.data);
      yMax = d3.max(allPoints, (d) => d.y) || 0;
    }

    // Create scales
    const xExtent = d3.extent(
      visibleSeries.flatMap((s) => s.data),
      (d) => d.x
    ) as [number, number];

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
      .domain([
        0,
        data.yAxis.max !== undefined ? data.yAxis.max : yMax,
      ])
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

    // Create area generator
    if (stacked) {
      const areaGenerator = d3
        .area<any>()
        .x((d) => xScale(d.data.x))
        .y0((d) => yScale(d[0]))
        .y1((d) => yScale(d[1]))
        .curve(curveFunction);

      // Draw stacked areas
      seriesData.forEach((layer, idx) => {
        const series = visibleSeries[idx];
        const color = series.color || mergedConfig.colors.primary[idx % mergedConfig.colors.primary.length];

        g.append('path')
          .datum(layer)
          .attr('class', `area-${series.name.replace(/\s+/g, '-')}`)
          .attr('fill', color)
          .attr('fill-opacity', fillOpacity)
          .attr('d', areaGenerator)
          .on('mouseover', function (event) {
            d3.select(this).attr('fill-opacity', fillOpacity + 0.2);
          })
          .on('mouseout', function () {
            d3.select(this).attr('fill-opacity', fillOpacity);
          });
      });
    } else {
      const areaGenerator = d3
        .area<{ x: number; y: number }>()
        .x((d) => xScale(d.x))
        .y0(boundedHeight)
        .y1((d) => yScale(d.y))
        .curve(curveFunction);

      // Draw individual areas
      visibleSeries.forEach((series, idx) => {
        const color = series.color || mergedConfig.colors.primary[idx % mergedConfig.colors.primary.length];

        const path = g
          .append('path')
          .datum(series.data)
          .attr('class', `area-${series.name.replace(/\s+/g, '-')}`)
          .attr('fill', color)
          .attr('fill-opacity', fillOpacity)
          .attr('d', areaGenerator);

        // Add stroke on top
        const lineGenerator = d3
          .line<{ x: number; y: number }>()
          .x((d) => xScale(d.x))
          .y((d) => yScale(d.y))
          .curve(curveFunction);

        g.append('path')
          .datum(series.data)
          .attr('class', `line-${series.name.replace(/\s+/g, '-')}`)
          .attr('fill', 'none')
          .attr('stroke', color)
          .attr('stroke-width', 2)
          .attr('d', lineGenerator);

        // Animate
        if (mergedConfig.animation.enabled) {
          path
            .attr('opacity', 0)
            .transition()
            .duration(mergedConfig.animation.duration)
            .attr('opacity', 1);
        }
      });
    }

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
      .call((g) => g.select('.domain').attr('stroke', mergedConfig.colors.axis));

    // Add axis labels
    if (data.xAxis.label) {
      g.append('text')
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
        .attr('transform', 'rotate(-90)')
        .attr('x', -boundedHeight / 2)
        .attr('y', -margin.left + 15)
        .attr('text-anchor', 'middle')
        .attr('fill', mergedConfig.colors.text)
        .style('font-family', mergedConfig.font.family)
        .style('font-size', `${mergedConfig.font.size + 2}px`)
        .text(data.yAxis.label);
    }
  }, [
    data,
    visibleSeries,
    mergedConfig,
    mergedInteraction,
    curveFunction,
    stacked,
    fillOpacity,
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

export default AreaChart;
