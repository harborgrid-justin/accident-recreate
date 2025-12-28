/**
 * AccuScene Enterprise v0.2.0
 *
 * Central export point for all enterprise-grade features and modules
 * introduced in version 0.2.0.
 *
 * @module enterprise
 * @version 0.2.0
 */

// ============================================================================
// GraphQL Federation API System (Agent 6)
// ============================================================================

export * from '../graphql';
export { default as GraphQLServer } from '../graphql/server';
export { createGraphQLContext } from '../graphql/context';
export {
  createFederationGateway,
  federationConfig
} from '../graphql/federation';
export {
  createGraphQLPlayground,
  playgroundConfig
} from '../graphql/playground';

// ============================================================================
// Real-time Collaboration System (Agent 7)
// ============================================================================

export * from '../collaboration';
export { CollaborationServer } from '../collaboration/server';
export { CollaborationClient } from '../collaboration/client';

// CRDT Exports
export {
  LWWRegister,
  GCounter,
  PNCounter,
  ORSet,
  LWWMap,
  RGA
} from '../collaboration/crdt';

// Synchronization
export {
  VectorClock,
  MerkleTree,
  DifferentialSync,
  ConflictResolver
} from '../collaboration/sync';

// Presence & Awareness
export {
  PresenceTracker,
  CursorTracker,
  SelectionAwareness
} from '../collaboration/presence';
export { AwarenessState } from '../collaboration/awareness';

// Operations & Transformation
export {
  OperationalTransform,
  OperationComposer,
  OperationHistory
} from '../collaboration/operations';

// Room Management
export {
  RoomManager,
  RoomPermissions,
  RoomState
} from '../collaboration/room';

// Persistence
export {
  SnapshotManager,
  OperationJournal
} from '../collaboration/persistence';

// Transport
export {
  WebSocketTransport,
  WebRTCTransport
} from '../collaboration/transport';

// ============================================================================
// Advanced UI Components (Agent 8)
// ============================================================================

export * from '../renderer/components/advanced';

// 3D Scene Components
export {
  Scene3D,
  Camera3D,
  Lighting3D,
  Ground3D,
  Environment3D
} from '../renderer/components/advanced/Scene3D';

// 3D Vehicle Components
export {
  Vehicle3D,
  VehicleDamage,
  VehiclePhysics,
  VehicleTrajectory
} from '../renderer/components/advanced/Vehicle3D';

// 3D Simulation Components
export {
  Simulation3D,
  SimulationTimeline,
  SimulationControls,
  SimulationAnalysis
} from '../renderer/components/advanced/Simulation3D';

// AR Components
export {
  ARPreview,
  AROverlay,
  ARMarkers
} from '../renderer/components/advanced/AR';

// Chart Components
export {
  ForceChart,
  VelocityChart,
  EnergyChart,
  ImpactChart
} from '../renderer/components/advanced/Charts';

// Heatmap Components
export {
  Heatmap,
  DamageHeatmap,
  ForceHeatmap
} from '../renderer/components/advanced/Heatmap';

// Timeline Components
export {
  Timeline,
  TimelineEvents,
  TimelineKeyframes
} from '../renderer/components/advanced/Timeline';

// Toolbar Components
export {
  Toolbar,
  ToolbarTools,
  ToolbarModes
} from '../renderer/components/advanced/Toolbar';

// Panel Components
export {
  Panel,
  PropertiesPanel,
  LayersPanel,
  HistoryPanel
} from '../renderer/components/advanced/Panel';

// Hooks
export {
  useScene3D,
  useAnimation,
  useAR
} from '../renderer/components/advanced/hooks';

// ============================================================================
// Plugin Architecture System (Agent 9)
// ============================================================================

export * from '../plugins';

// Core Plugin System
export {
  PluginManager,
  PluginRegistry,
  PluginLoader,
  PluginValidator,
  PluginSandbox
} from '../plugins/core';

// Lifecycle Management
export {
  PluginHooks,
  PluginState,
  PluginEvents
} from '../plugins/lifecycle';

// Plugin API
export {
  PluginContext,
  PluginServices,
  PluginStorage,
  PluginEventEmitter,
  PluginUI,
  PluginCommands,
  PluginMenu
} from '../plugins/api';

// Extension Points
export {
  ToolbarExtension,
  PanelExtension,
  MenuExtension,
  ContextMenuExtension,
  ExporterExtension,
  ImporterExtension,
  ToolExtension
} from '../plugins/extension';

// Manifest Handling
export {
  PluginManifestSchema,
  PluginManifestParser,
  PluginManifestValidator
} from '../plugins/manifest';

// Security Framework
export {
  PluginPermissions,
  PluginCapabilities,
  PluginIsolation
} from '../plugins/security';

// Plugin Store
export {
  PluginMarketplace,
  PluginInstaller,
  PluginUpdater
} from '../plugins/store';

// Built-in Plugins
export {
  MeasurementsPlugin,
  AnnotationsPlugin,
  ExportsPlugin
} from '../plugins/builtin';

// ============================================================================
// Performance Monitoring System (Agent 10)
// ============================================================================

export * from '../monitoring';

// Core Monitoring
export {
  PerformanceMonitor,
  MetricsCollector,
  MetricsRegistry
} from '../monitoring/core';

// Metrics
export {
  MetricsStore,
  MetricTypes,
  MetricAggregator
} from '../monitoring/metrics';

// Tracing
export {
  TracingProvider,
  SpanCollector,
  TraceExporter
} from '../monitoring/tracing';

// Profiling
export {
  Profiler,
  CPUProfiler,
  MemoryProfiler
} from '../monitoring/profiling';

// Health Checks
export {
  HealthChecker,
  HealthStatus,
  HealthEndpoint
} from '../monitoring/health';

// Alerting
export {
  AlertManager,
  AlertRule,
  AlertChannel
} from '../monitoring/alerting';

// Integrations
export {
  PrometheusExporter,
  DatadogIntegration,
  NewRelicIntegration
} from '../monitoring/integrations';

// Dashboard
export {
  MonitoringDashboard,
  MetricsDashboard,
  PerformanceDashboard
} from '../monitoring/dashboard';

// ============================================================================
// Version Information
// ============================================================================

export const ENTERPRISE_VERSION = '0.2.0';
export const ENTERPRISE_FEATURES = [
  'GraphQL Federation API',
  'Real-time Collaboration (CRDT)',
  'Advanced UI Components (3D, AR)',
  'Plugin Architecture',
  'Performance Monitoring',
] as const;

/**
 * Initialize all enterprise features
 */
export interface EnterpriseConfig {
  graphql?: {
    enabled: boolean;
    port?: number;
    playground?: boolean;
  };
  collaboration?: {
    enabled: boolean;
    port?: number;
    maxRooms?: number;
  };
  plugins?: {
    enabled: boolean;
    autoload?: boolean;
    marketplace?: string;
  };
  monitoring?: {
    enabled: boolean;
    exporters?: string[];
  };
}

export const DEFAULT_ENTERPRISE_CONFIG: EnterpriseConfig = {
  graphql: {
    enabled: true,
    port: 4000,
    playground: true,
  },
  collaboration: {
    enabled: true,
    port: 3001,
    maxRooms: 100,
  },
  plugins: {
    enabled: true,
    autoload: true,
    marketplace: 'https://plugins.accuscene.com',
  },
  monitoring: {
    enabled: true,
    exporters: ['prometheus'],
  },
};

/**
 * Initialize enterprise features with configuration
 */
export async function initializeEnterprise(
  config: EnterpriseConfig = DEFAULT_ENTERPRISE_CONFIG
): Promise<void> {
  console.log(`🚀 Initializing AccuScene Enterprise v${ENTERPRISE_VERSION}`);

  if (config.graphql?.enabled) {
    console.log('  ✓ GraphQL Federation API');
  }

  if (config.collaboration?.enabled) {
    console.log('  ✓ Real-time Collaboration');
  }

  if (config.plugins?.enabled) {
    console.log('  ✓ Plugin Architecture');
  }

  if (config.monitoring?.enabled) {
    console.log('  ✓ Performance Monitoring');
  }

  console.log(`✅ AccuScene Enterprise v${ENTERPRISE_VERSION} initialized`);
}
