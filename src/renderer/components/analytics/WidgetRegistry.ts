/**
 * AccuScene Enterprise v0.3.0 - Widget Registry
 * Plugin-based widget registration and management system
 */

import { WidgetType, WidgetRegistryEntry } from './types';

// Import all widget components
import SpeedChart from './widgets/SpeedChart';
import ForceVectorWidget from './widgets/ForceVectorWidget';
import ImpactAnalysisWidget from './widgets/ImpactAnalysisWidget';
import EnergyFlowWidget from './widgets/EnergyFlowWidget';
import TrajectoryWidget from './widgets/TrajectoryWidget';
import DamageHeatmap from './widgets/DamageHeatmap';
import TimelineWidget from './widgets/TimelineWidget';
import StatisticsCard from './widgets/StatisticsCard';
import ComparisonWidget from './widgets/ComparisonWidget';
import DataTable from './widgets/DataTable';
import ReportSummary from './widgets/ReportSummary';

class WidgetRegistryClass {
  private registry: Map<WidgetType, WidgetRegistryEntry> = new Map();

  constructor() {
    this.registerDefaultWidgets();
  }

  /**
   * Register all default widgets
   */
  private registerDefaultWidgets(): void {
    // Speed Chart
    this.register({
      type: 'speed-chart',
      name: 'Speed Chart',
      description: 'Vehicle speed over time with impact annotations',
      icon: '📈',
      component: SpeedChart,
      defaultSize: { w: 6, h: 3 },
      minSize: { w: 4, h: 2 },
      maxSize: { w: 12, h: 6 },
      defaultSettings: {
        showGrid: true,
        showLegend: true,
        showAnnotations: true,
      },
      category: 'analysis',
    });

    // Force Vector Widget
    this.register({
      type: 'force-vector',
      name: 'Force Vectors',
      description: '3D visualization of impact force vectors',
      icon: '🎯',
      component: ForceVectorWidget,
      defaultSize: { w: 6, h: 4 },
      minSize: { w: 4, h: 3 },
      maxSize: { w: 12, h: 8 },
      defaultSettings: {
        show3D: true,
        showAxes: true,
        showGrid: true,
      },
      category: 'visualization',
    });

    // Impact Analysis Widget
    this.register({
      type: 'impact-analysis',
      name: 'Impact Analysis',
      description: 'Detailed impact point analysis with severity mapping',
      icon: '💥',
      component: ImpactAnalysisWidget,
      defaultSize: { w: 6, h: 4 },
      minSize: { w: 4, h: 3 },
      defaultSettings: {
        showSeverity: true,
        showStatistics: true,
      },
      category: 'analysis',
    });

    // Energy Flow Widget
    this.register({
      type: 'energy-flow',
      name: 'Energy Flow',
      description: 'Sankey diagram of energy transfer between components',
      icon: '⚡',
      component: EnergyFlowWidget,
      defaultSize: { w: 8, h: 4 },
      minSize: { w: 6, h: 3 },
      defaultSettings: {
        showLabels: true,
        showStatistics: true,
      },
      category: 'analysis',
    });

    // Trajectory Widget
    this.register({
      type: 'trajectory',
      name: 'Trajectory',
      description: '2D/3D vehicle trajectory visualization',
      icon: '🛣️',
      component: TrajectoryWidget,
      defaultSize: { w: 6, h: 4 },
      minSize: { w: 4, h: 3 },
      defaultSettings: {
        viewMode: '2d',
        showPath: true,
        showVelocity: true,
      },
      category: 'visualization',
    });

    // Damage Heatmap
    this.register({
      type: 'damage-heatmap',
      name: 'Damage Heatmap',
      description: 'Vehicle damage severity heatmap',
      icon: '🔥',
      component: DamageHeatmap,
      defaultSize: { w: 6, h: 4 },
      minSize: { w: 4, h: 3 },
      defaultSettings: {
        showLegend: true,
        showStatistics: true,
      },
      category: 'analysis',
    });

    // Timeline Widget
    this.register({
      type: 'timeline',
      name: 'Event Timeline',
      description: 'Interactive timeline with playback controls',
      icon: '⏱️',
      component: TimelineWidget,
      defaultSize: { w: 12, h: 3 },
      minSize: { w: 6, h: 2 },
      defaultSettings: {
        showControls: true,
        autoPlay: false,
        playbackSpeed: 1,
      },
      category: 'visualization',
    });

    // Statistics Card
    this.register({
      type: 'statistics',
      name: 'Statistics',
      description: 'KPI cards with trends and metrics',
      icon: '📊',
      component: StatisticsCard,
      defaultSize: { w: 4, h: 2 },
      minSize: { w: 3, h: 2 },
      defaultSettings: {
        showTrends: true,
        showChanges: true,
      },
      category: 'summary',
    });

    // Comparison Widget
    this.register({
      type: 'comparison',
      name: 'Comparison',
      description: 'Before/after comparison with slider',
      icon: '⚖️',
      component: ComparisonWidget,
      defaultSize: { w: 6, h: 4 },
      minSize: { w: 4, h: 3 },
      defaultSettings: {
        viewMode: 'slider',
        showDifferences: true,
      },
      category: 'analysis',
    });

    // Data Table
    this.register({
      type: 'data-table',
      name: 'Data Table',
      description: 'Sortable, filterable data table',
      icon: '📋',
      component: DataTable,
      defaultSize: { w: 8, h: 4 },
      minSize: { w: 6, h: 3 },
      defaultSettings: {
        pageSize: 10,
        showFilters: true,
        showPagination: true,
      },
      category: 'data',
    });

    // Report Summary
    this.register({
      type: 'report-summary',
      name: 'Report Summary',
      description: 'Auto-generated executive summary',
      icon: '📄',
      component: ReportSummary,
      defaultSize: { w: 6, h: 5 },
      minSize: { w: 4, h: 4 },
      defaultSettings: {
        showFindings: true,
        showSeverity: true,
      },
      category: 'summary',
    });
  }

  /**
   * Register a widget
   */
  register(entry: WidgetRegistryEntry): void {
    this.registry.set(entry.type, entry);
  }

  /**
   * Unregister a widget
   */
  unregister(type: WidgetType): boolean {
    return this.registry.delete(type);
  }

  /**
   * Get a widget definition
   */
  getWidget(type: WidgetType): WidgetRegistryEntry | undefined {
    return this.registry.get(type);
  }

  /**
   * Get all registered widgets
   */
  getAllWidgets(): WidgetRegistryEntry[] {
    return Array.from(this.registry.values());
  }

  /**
   * Get widgets by category
   */
  getWidgetsByCategory(
    category: 'analysis' | 'visualization' | 'data' | 'summary'
  ): WidgetRegistryEntry[] {
    return Array.from(this.registry.values()).filter(
      (w) => w.category === category
    );
  }

  /**
   * Check if a widget type is registered
   */
  hasWidget(type: WidgetType): boolean {
    return this.registry.has(type);
  }

  /**
   * Get widget count
   */
  getWidgetCount(): number {
    return this.registry.size;
  }

  /**
   * Clear all widgets (use with caution)
   */
  clear(): void {
    this.registry.clear();
  }
}

// Export singleton instance
export const WidgetRegistry = new WidgetRegistryClass();
