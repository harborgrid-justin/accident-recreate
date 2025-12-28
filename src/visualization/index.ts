/**
 * AccuScene Visualization Module
 * Advanced data visualization components and utilities
 */

// Type exports
export * from './types';

// Chart components
export { default as LineChart } from './charts/LineChart';
export { default as AreaChart } from './charts/AreaChart';
export { default as BarChart } from './charts/BarChart';
export { default as ScatterPlot } from './charts/ScatterPlot';
export { default as HeatmapChart } from './charts/HeatmapChart';
export { default as RadarChart } from './charts/RadarChart';
export { default as SankeyDiagram } from './charts/SankeyDiagram';

// Utility components
export { default as ChartContainer } from './components/ChartContainer';
export { default as ChartLegend } from './components/ChartLegend';
export { default as ChartTooltip } from './components/ChartTooltip';

// Hooks
export { default as useChartData } from './hooks/useChartData';

// Default configurations
export const DEFAULT_CHART_CONFIG: import('./types').ChartConfig = {
  width: 800,
  height: 600,
  margin: {
    top: 20,
    right: 20,
    bottom: 40,
    left: 60,
  },
  colors: {
    primary: [
      '#3B82F6', // Blue
      '#10B981', // Green
      '#F59E0B', // Amber
      '#EF4444', // Red
      '#8B5CF6', // Purple
      '#EC4899', // Pink
      '#06B6D4', // Cyan
      '#84CC16', // Lime
    ],
    background: '#FFFFFF',
    text: '#1F2937',
    grid: '#E5E7EB',
    axis: '#6B7280',
  },
  font: {
    family: 'Inter, sans-serif',
    size: 12,
    weight: 'normal',
  },
  animation: {
    enabled: true,
    duration: 300,
    easing: 'ease-in-out',
  },
  responsive: true,
};

export const DEFAULT_INTERACTION: import('./types').ChartInteraction = {
  zoom: true,
  pan: true,
  brush: false,
  tooltip: true,
  legend: true,
};
