/**
 * AccuScene Enterprise - Module Type Declarations v0.2.5
 * Type declarations for external modules and assets
 * Updated: 2025-12-28
 */

// ============================================================================
// ASSET MODULES
// ============================================================================

/**
 * Image imports
 */
declare module '*.png' {
  const content: string;
  export default content;
}

declare module '*.jpg' {
  const content: string;
  export default content;
}

declare module '*.jpeg' {
  const content: string;
  export default content;
}

declare module '*.gif' {
  const content: string;
  export default content;
}

declare module '*.svg' {
  const content: string;
  export default content;
}

declare module '*.webp' {
  const content: string;
  export default content;
}

declare module '*.ico' {
  const content: string;
  export default content;
}

/**
 * Style modules
 */
declare module '*.css' {
  const content: { [className: string]: string };
  export default content;
}

declare module '*.scss' {
  const content: { [className: string]: string };
  export default content;
}

declare module '*.sass' {
  const content: { [className: string]: string };
  export default content;
}

declare module '*.less' {
  const content: { [className: string]: string };
  export default content;
}

declare module '*.module.css' {
  const content: { [className: string]: string };
  export default content;
}

declare module '*.module.scss' {
  const content: { [className: string]: string };
  export default content;
}

/**
 * Font modules
 */
declare module '*.woff' {
  const content: string;
  export default content;
}

declare module '*.woff2' {
  const content: string;
  export default content;
}

declare module '*.ttf' {
  const content: string;
  export default content;
}

declare module '*.otf' {
  const content: string;
  export default content;
}

declare module '*.eot' {
  const content: string;
  export default content;
}

/**
 * Data formats
 */
declare module '*.json' {
  const content: { [key: string]: unknown };
  export default content;
}

declare module '*.json5' {
  const content: { [key: string]: unknown };
  export default content;
}

declare module '*.yaml' {
  const content: { [key: string]: unknown };
  export default content;
}

declare module '*.yml' {
  const content: { [key: string]: unknown };
  export default content;
}

declare module '*.toml' {
  const content: { [key: string]: unknown };
  export default content;
}

declare module '*.xml' {
  const content: string;
  export default content;
}

/**
 * 3D model formats
 */
declare module '*.gltf' {
  const content: string;
  export default content;
}

declare module '*.glb' {
  const content: string;
  export default content;
}

declare module '*.obj' {
  const content: string;
  export default content;
}

declare module '*.fbx' {
  const content: string;
  export default content;
}

declare module '*.dae' {
  const content: string;
  export default content;
}

declare module '*.stl' {
  const content: string;
  export default content;
}

/**
 * Video/Audio
 */
declare module '*.mp4' {
  const content: string;
  export default content;
}

declare module '*.webm' {
  const content: string;
  export default content;
}

declare module '*.ogg' {
  const content: string;
  export default content;
}

declare module '*.mp3' {
  const content: string;
  export default content;
}

declare module '*.wav' {
  const content: string;
  export default content;
}

/**
 * Markdown
 */
declare module '*.md' {
  const content: string;
  export default content;
}

declare module '*.mdx' {
  const content: string;
  export default content;
}

/**
 * Text files
 */
declare module '*.txt' {
  const content: string;
  export default content;
}

/**
 * WebAssembly
 */
declare module '*.wasm' {
  const content: WebAssembly.Module;
  export default content;
}

// ============================================================================
// NATIVE NODE MODULES
// ============================================================================

/**
 * AccuScene Rust Core FFI Module
 */
declare module 'accuscene-core' {
  export interface RustCore {
    version: string;

    // Scene management
    loadScene(path: string): Promise<Scene>;
    saveScene(path: string, scene: Scene): Promise<void>;

    // Physics simulation
    runSimulation(params: SimulationParams): Promise<SimulationResult>;
    calculatePhysics(data: PhysicsData): Promise<PhysicsResult>;

    // Compression
    compress(data: Uint8Array): Promise<Uint8Array>;
    decompress(data: Uint8Array): Promise<Uint8Array>;

    // Cryptography
    encrypt(data: Uint8Array, key: string): Promise<Uint8Array>;
    decrypt(data: Uint8Array, key: string): Promise<Uint8Array>;

    // Telemetry
    recordEvent(event: TelemetryEvent): void;
    getMetrics(): Promise<Metrics>;
  }

  const core: RustCore;
  export default core;
}

// ============================================================================
// ELECTRON MODULES
// ============================================================================

/**
 * Electron main process
 */
declare module 'electron' {
  export * from 'electron/main';
}

/**
 * Electron renderer process
 */
declare module 'electron/renderer' {
  export const ipcRenderer: {
    send(channel: string, ...args: unknown[]): void;
    on(channel: string, listener: (event: unknown, ...args: unknown[]) => void): void;
    once(channel: string, listener: (event: unknown, ...args: unknown[]) => void): void;
    removeListener(channel: string, listener: (event: unknown, ...args: unknown[]) => void): void;
    removeAllListeners(channel: string): void;
    invoke<T = unknown>(channel: string, ...args: unknown[]): Promise<T>;
  };
}

// ============================================================================
// EXTERNAL LIBRARY AUGMENTATIONS
// ============================================================================

/**
 * Better SQLite3 module extensions
 */
declare module 'better-sqlite3' {
  import { Database as BetterSqlite3Database } from 'better-sqlite3';

  export interface Database extends BetterSqlite3Database {
    // Extended methods for AccuScene
    executeTransaction<T>(fn: () => T): T;
    backup(destination: string): Promise<void>;
    optimize(): void;
  }

  export default Database;
}

/**
 * Socket.IO extensions
 */
declare module 'socket.io' {
  import { Server as SocketIOServer } from 'socket.io';

  export interface Server extends SocketIOServer {
    // Extended methods for AccuScene real-time features
    broadcastToScene(sceneId: string, event: string, data: unknown): void;
    broadcastToProject(projectId: string, event: string, data: unknown): void;
  }

  export default Server;
}

/**
 * React Three Fiber extensions
 */
declare module '@react-three/fiber' {
  import { ThreeElements } from '@react-three/fiber';

  export interface ThreeElements {
    // Custom AccuScene 3D elements
    accuSceneVehicle: {
      vehicleId: string;
      position: [number, number, number];
      rotation: [number, number, number];
      model: string;
    };
    accuSceneRoad: {
      points: [number, number, number][];
      width: number;
      texture: string;
    };
  }
}

/**
 * Konva extensions for diagram editor
 */
declare module 'konva' {
  import Konva from 'konva';

  namespace Konva {
    // Custom AccuScene shapes
    class VehicleShape extends Konva.Group {
      constructor(config: VehicleShapeConfig);
    }

    class RoadShape extends Konva.Group {
      constructor(config: RoadShapeConfig);
    }

    interface VehicleShapeConfig extends Konva.GroupConfig {
      vehicleType: string;
      heading: number;
      speed: number;
    }

    interface RoadShapeConfig extends Konva.GroupConfig {
      lanes: number;
      laneWidth: number;
    }
  }

  export = Konva;
}

/**
 * Winston logger extensions
 */
declare module 'winston' {
  import * as winston from 'winston';

  // Extend Winston logger with AccuScene-specific methods
  export interface Logger extends winston.Logger {
    logSceneEvent(sceneId: string, event: string, data?: unknown): void;
    logUserAction(userId: string, action: string, data?: unknown): void;
    logPerformance(operation: string, duration: number, metadata?: unknown): void;
  }
}

// ============================================================================
// CUSTOM PLUGIN MODULES
// ============================================================================

/**
 * AccuScene plugins
 */
declare module '@accuscene/plugins' {
  export interface Plugin {
    name: string;
    version: string;
    initialize(): Promise<void>;
    dispose(): Promise<void>;
  }

  export interface PluginManager {
    register(plugin: Plugin): Promise<void>;
    unregister(pluginName: string): Promise<void>;
    getPlugin(pluginName: string): Plugin | undefined;
    listPlugins(): Plugin[];
  }

  export const pluginManager: PluginManager;
}

/**
 * AccuScene themes
 */
declare module '@accuscene/themes' {
  export interface Theme {
    name: string;
    colors: {
      primary: string;
      secondary: string;
      background: string;
      foreground: string;
      accent: string;
      error: string;
      warning: string;
      success: string;
      info: string;
    };
    fonts: {
      heading: string;
      body: string;
      monospace: string;
    };
    spacing: {
      xs: number;
      sm: number;
      md: number;
      lg: number;
      xl: number;
    };
    borderRadius: {
      sm: number;
      md: number;
      lg: number;
    };
  }

  export const defaultTheme: Theme;
  export const darkTheme: Theme;
  export const lightTheme: Theme;
}

// ============================================================================
// WORKER MODULES
// ============================================================================

/**
 * Web Worker module
 */
declare module 'worker-loader!*' {
  class WebpackWorker extends Worker {
    constructor();
  }

  export default WebpackWorker;
}

/**
 * Shared Worker module
 */
declare module 'shared-worker-loader!*' {
  class WebpackSharedWorker extends SharedWorker {
    constructor();
  }

  export default WebpackSharedWorker;
}

// ============================================================================
// PROTOCOL BUFFER MODULES
// ============================================================================

/**
 * Protocol buffers
 */
declare module '*.proto' {
  export const fileDescriptor: unknown;
  export default fileDescriptor;
}

// ============================================================================
// SHADER MODULES
// ============================================================================

/**
 * GLSL shaders
 */
declare module '*.glsl' {
  const content: string;
  export default content;
}

declare module '*.vert' {
  const content: string;
  export default content;
}

declare module '*.frag' {
  const content: string;
  export default content;
}

// ============================================================================
// CUSTOM FILE FORMATS
// ============================================================================

/**
 * AccuScene custom file formats
 */
declare module '*.ascene' {
  const content: {
    version: string;
    scene: unknown;
  };
  export default content;
}

declare module '*.aproject' {
  const content: {
    version: string;
    project: unknown;
  };
  export default content;
}

declare module '*.areport' {
  const content: {
    version: string;
    report: unknown;
  };
  export default content;
}

// ============================================================================
// INLINE MODULES
// ============================================================================

/**
 * Inline worker
 */
declare module 'inline-worker:*' {
  class InlineWorker extends Worker {
    constructor();
  }
  export default InlineWorker;
}

/**
 * Raw file content
 */
declare module 'raw-loader!*' {
  const content: string;
  export default content;
}

/**
 * URL loader
 */
declare module 'url-loader!*' {
  const content: string;
  export default content;
}

// ============================================================================
// LEGACY MODULE COMPATIBILITY
// ============================================================================

/**
 * CommonJS modules without type definitions
 */
declare module '*' {
  const content: unknown;
  export default content;
}
