/**
 * AccuScene Enterprise - Global Type Declarations v0.2.5
 * Global types, interfaces, and declarations available throughout the application
 * Updated: 2025-12-28
 */

// ============================================================================
// GLOBAL NAMESPACE EXTENSIONS
// ============================================================================

declare global {
  /**
   * Node.js process environment variables
   * Extend as needed for type safety
   */
  namespace NodeJS {
    interface ProcessEnv {
      readonly NODE_ENV: 'development' | 'production' | 'test';
      readonly PORT?: string;
      readonly HOST?: string;

      // Database
      readonly DB_HOST?: string;
      readonly DB_PORT?: string;
      readonly DB_NAME?: string;
      readonly DB_USER?: string;
      readonly DB_PASSWORD?: string;

      // Redis
      readonly REDIS_HOST?: string;
      readonly REDIS_PORT?: string;
      readonly REDIS_PASSWORD?: string;

      // JWT
      readonly JWT_SECRET?: string;
      readonly JWT_EXPIRES_IN?: string;

      // File paths
      readonly DATA_DIR?: string;
      readonly TEMP_DIR?: string;
      readonly LOG_DIR?: string;

      // Features
      readonly ENABLE_TELEMETRY?: string;
      readonly ENABLE_CLUSTERING?: string;
      readonly ENABLE_COMPRESSION?: string;

      // Rust backend
      readonly RUST_BACKEND_ENABLED?: string;
      readonly RUST_LOG?: string;
    }

    interface Global {
      // Global singleton instances
      __accuscene_core__?: unknown;
      __accuscene_db_pool__?: unknown;
      __accuscene_cache__?: unknown;
    }
  }

  /**
   * Window object extensions for renderer process
   */
  interface Window {
    // Electron IPC Bridge
    electron?: {
      ipcRenderer: {
        send: (channel: string, ...args: unknown[]) => void;
        on: (channel: string, listener: (...args: unknown[]) => void) => void;
        once: (channel: string, listener: (...args: unknown[]) => void) => void;
        removeListener: (channel: string, listener: (...args: unknown[]) => void) => void;
        removeAllListeners: (channel: string) => void;
        invoke: <T = unknown>(channel: string, ...args: unknown[]) => Promise<T>;
      };
      store: {
        get: <T = unknown>(key: string) => T;
        set: (key: string, value: unknown) => void;
        delete: (key: string) => void;
        clear: () => void;
      };
    };

    // AccuScene API Bridge
    accuscene?: {
      version: string;
      platform: string;
      arch: string;

      // Rust FFI Bridge
      rust: {
        available: boolean;
        version: string;
        loadScene: (path: string) => Promise<unknown>;
        saveScene: (path: string, data: unknown) => Promise<void>;
        runSimulation: (params: unknown) => Promise<unknown>;
        calculatePhysics: (data: unknown) => Promise<unknown>;
      };

      // File system operations
      fs: {
        readFile: (path: string) => Promise<string>;
        writeFile: (path: string, data: string) => Promise<void>;
        exists: (path: string) => Promise<boolean>;
        readDir: (path: string) => Promise<string[]>;
        createDir: (path: string) => Promise<void>;
        deleteFile: (path: string) => Promise<void>;
      };
    };

    // WebAssembly module cache
    __WASM_MODULES__?: Map<string, WebAssembly.Module>;

    // Development/Debug helpers
    __ACCUSCENE_DEBUG__?: boolean;
    __REACT_DEVTOOLS_GLOBAL_HOOK__?: unknown;
  }
}

// ============================================================================
// COMMON UTILITY TYPES
// ============================================================================

/**
 * Make all properties in T optional recursively
 */
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

/**
 * Make all properties in T required recursively
 */
export type DeepRequired<T> = {
  [P in keyof T]-?: T[P] extends object ? DeepRequired<T[P]> : T[P];
};

/**
 * Make all properties in T readonly recursively
 */
export type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P];
};

/**
 * Make all properties in T mutable (remove readonly)
 */
export type Mutable<T> = {
  -readonly [P in keyof T]: T[P];
};

/**
 * Make all properties in T nullable
 */
export type Nullable<T> = {
  [P in keyof T]: T[P] | null;
};

/**
 * Make all properties in T non-nullable
 */
export type NonNullable<T> = {
  [P in keyof T]: NonNullable<T[P]>;
};

/**
 * Extract keys from T where the value is of type U
 */
export type KeysOfType<T, U> = {
  [K in keyof T]: T[K] extends U ? K : never;
}[keyof T];

/**
 * Extract the promise type
 */
export type Awaited<T> = T extends Promise<infer U> ? U : T;

/**
 * Extract the array element type
 */
export type ArrayElement<T> = T extends (infer U)[] ? U : T;

/**
 * Ensure at least one property is defined
 */
export type AtLeastOne<T, Keys extends keyof T = keyof T> = Pick<T, Exclude<keyof T, Keys>> &
  {
    [K in Keys]-?: Required<Pick<T, K>> & Partial<Pick<T, Exclude<Keys, K>>>;
  }[Keys];

/**
 * Ensure exactly one property is defined
 */
export type ExactlyOne<T, Keys extends keyof T = keyof T> = Pick<T, Exclude<keyof T, Keys>> &
  {
    [K in Keys]: Required<Pick<T, K>> & Partial<Record<Exclude<Keys, K>, never>>;
  }[Keys];

/**
 * Make specific properties optional
 */
export type PartialBy<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

/**
 * Make specific properties required
 */
export type RequiredBy<T, K extends keyof T> = Omit<T, K> & Required<Pick<T, K>>;

/**
 * Type-safe object keys
 */
export type ObjectKeys<T> = keyof T;

/**
 * Type-safe object values
 */
export type ObjectValues<T> = T[keyof T];

/**
 * Type-safe object entries
 */
export type ObjectEntries<T> = {
  [K in keyof T]: [K, T[K]];
}[keyof T][];

/**
 * JSON serializable types
 */
export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonObject | JsonArray;
export type JsonObject = { [key: string]: JsonValue };
export type JsonArray = JsonValue[];

/**
 * Function type helpers
 */
export type AnyFunction = (...args: unknown[]) => unknown;
export type AsyncFunction<T = unknown> = (...args: unknown[]) => Promise<T>;
export type VoidFunction = () => void;
export type AsyncVoidFunction = () => Promise<void>;

/**
 * Constructor type
 */
export type Constructor<T = unknown> = new (...args: unknown[]) => T;

/**
 * Abstract constructor type
 */
export type AbstractConstructor<T = unknown> = abstract new (...args: unknown[]) => T;

/**
 * Class type
 */
export type Class<T = unknown> = Constructor<T> | AbstractConstructor<T>;

/**
 * Branded types for type safety
 */
export type Brand<T, B> = T & { __brand: B };

/**
 * Common branded types
 */
export type UUID = Brand<string, 'UUID'>;
export type Timestamp = Brand<number, 'Timestamp'>;
export type UnixTimestamp = Brand<number, 'UnixTimestamp'>;
export type ISODateString = Brand<string, 'ISODateString'>;
export type Email = Brand<string, 'Email'>;
export type URL = Brand<string, 'URL'>;
export type FilePath = Brand<string, 'FilePath'>;
export type Base64 = Brand<string, 'Base64'>;
export type JWT = Brand<string, 'JWT'>;

/**
 * Tuple helper types
 */
export type Head<T extends unknown[]> = T extends [infer H, ...unknown[]] ? H : never;
export type Tail<T extends unknown[]> = T extends [unknown, ...infer T] ? T : never;
export type Last<T extends unknown[]> = T extends [...unknown[], infer L] ? L : never;

/**
 * Result type for error handling
 */
export type Result<T, E = Error> =
  | { success: true; value: T }
  | { success: false; error: E };

/**
 * Option type for nullable values
 */
export type Option<T> = T | null | undefined;

/**
 * Predicate function type
 */
export type Predicate<T> = (value: T) => boolean;

/**
 * Comparator function type
 */
export type Comparator<T> = (a: T, b: T) => number;

/**
 * Mapper function type
 */
export type Mapper<T, U> = (value: T) => U;

/**
 * Reducer function type
 */
export type Reducer<T, U> = (accumulator: U, value: T) => U;

// ============================================================================
// ID TYPES
// ============================================================================

export type SceneId = Brand<string, 'SceneId'>;
export type VehicleId = Brand<string, 'VehicleId'>;
export type ObjectId = Brand<string, 'ObjectId'>;
export type UserId = Brand<string, 'UserId'>;
export type SessionId = Brand<string, 'SessionId'>;
export type ReportId = Brand<string, 'ReportId'>;
export type ProjectId = Brand<string, 'ProjectId'>;
export type OrganizationId = Brand<string, 'OrganizationId'>;

// ============================================================================
// COORDINATE AND DIMENSION TYPES
// ============================================================================

/**
 * 2D Point
 */
export interface Point2D {
  x: number;
  y: number;
}

/**
 * 3D Point
 */
export interface Point3D extends Point2D {
  z: number;
}

/**
 * 2D Vector
 */
export type Vector2D = Point2D;

/**
 * 3D Vector
 */
export type Vector3D = Point3D;

/**
 * 2D Dimensions
 */
export interface Dimensions2D {
  width: number;
  height: number;
}

/**
 * 3D Dimensions
 */
export interface Dimensions3D extends Dimensions2D {
  depth: number;
}

/**
 * 2D Bounding Box
 */
export interface BoundingBox2D {
  min: Point2D;
  max: Point2D;
}

/**
 * 3D Bounding Box
 */
export interface BoundingBox3D {
  min: Point3D;
  max: Point3D;
}

/**
 * RGB Color
 */
export interface RGBColor {
  r: number;
  g: number;
  b: number;
}

/**
 * RGBA Color
 */
export interface RGBAColor extends RGBColor {
  a: number;
}

/**
 * Hex color string
 */
export type HexColor = Brand<string, 'HexColor'>;

// ============================================================================
// TIMESTAMP TYPES
// ============================================================================

/**
 * Timestamp with timezone
 */
export interface TimestampWithZone {
  timestamp: number;
  timezone: string;
}

/**
 * Date range
 */
export interface DateRange {
  start: Date;
  end: Date;
}

/**
 * Timestamp range
 */
export interface TimestampRange {
  start: number;
  end: number;
}

// ============================================================================
// PAGINATION TYPES
// ============================================================================

/**
 * Pagination parameters
 */
export interface PaginationParams {
  page: number;
  pageSize: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

/**
 * Paginated response
 */
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasMore: boolean;
}

// ============================================================================
// API TYPES
// ============================================================================

/**
 * API Response wrapper
 */
export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: unknown;
  };
  meta?: {
    timestamp: number;
    requestId: string;
    version: string;
  };
}

/**
 * API Error
 */
export interface ApiError {
  code: string;
  message: string;
  statusCode?: number;
  details?: unknown;
}

// ============================================================================
// EXPORT MODULE
// ============================================================================

export {};
