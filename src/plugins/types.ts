/**
 * AccuScene Enterprise v0.2.0 - Plugin Architecture Types
 * Comprehensive type definitions for the extensible plugin system
 */

import { ReactNode, ComponentType } from 'react';

// ============================================================================
// Core Plugin Types
// ============================================================================

export type PluginId = string;
export type PluginVersion = string;
export type PluginNamespace = string;

export enum PluginState {
  UNLOADED = 'unloaded',
  LOADING = 'loading',
  LOADED = 'loaded',
  INITIALIZING = 'initializing',
  ACTIVE = 'active',
  PAUSED = 'paused',
  ERROR = 'error',
  UNLOADING = 'unloading',
}

export enum PluginType {
  CORE = 'core',
  BUILTIN = 'builtin',
  EXTENSION = 'extension',
  THIRD_PARTY = 'third_party',
}

export interface PluginMetadata {
  id: PluginId;
  name: string;
  version: PluginVersion;
  description: string;
  author: string;
  license: string;
  homepage?: string;
  repository?: string;
  keywords?: string[];
  type: PluginType;
  category?: string;
  icon?: string;
}

export interface PluginDependency {
  id: PluginId;
  version: string;
  optional?: boolean;
}

export interface PluginManifest extends PluginMetadata {
  main: string;
  dependencies?: PluginDependency[];
  peerDependencies?: PluginDependency[];
  permissions?: PluginPermission[];
  capabilities?: PluginCapability[];
  exports?: Record<string, string>;
  contributes?: PluginContributions;
  engines?: {
    accuscene?: string;
    node?: string;
  };
}

// ============================================================================
// Plugin API Types
// ============================================================================

export interface PluginContext {
  readonly id: PluginId;
  readonly version: PluginVersion;
  readonly manifest: PluginManifest;
  readonly services: PluginServices;
  readonly storage: PluginStorage;
  readonly events: PluginEventEmitter;
  readonly ui: PluginUI;
  readonly commands: PluginCommands;
  readonly logger: PluginLogger;
}

export interface PluginServices {
  get<T = any>(serviceId: string): T | undefined;
  register<T = any>(serviceId: string, service: T): void;
  unregister(serviceId: string): void;
  has(serviceId: string): boolean;
}

export interface PluginStorage {
  get<T = any>(key: string): Promise<T | undefined>;
  set<T = any>(key: string, value: T): Promise<void>;
  delete(key: string): Promise<void>;
  clear(): Promise<void>;
  keys(): Promise<string[]>;
}

export interface PluginEventEmitter {
  on<T = any>(event: string, handler: EventHandler<T>): Disposable;
  once<T = any>(event: string, handler: EventHandler<T>): Disposable;
  off<T = any>(event: string, handler: EventHandler<T>): void;
  emit<T = any>(event: string, data: T): void;
}

export interface PluginUI {
  registerToolbar(contribution: ToolbarContribution): Disposable;
  registerPanel(contribution: PanelContribution): Disposable;
  registerMenuItem(contribution: MenuContribution): Disposable;
  registerContextMenu(contribution: ContextMenuContribution): Disposable;
  showNotification(notification: Notification): void;
  showDialog(dialog: Dialog): Promise<DialogResult>;
}

export interface PluginCommands {
  register(command: Command): Disposable;
  execute(commandId: string, ...args: any[]): Promise<any>;
  has(commandId: string): boolean;
}

export interface PluginLogger {
  debug(message: string, ...args: any[]): void;
  info(message: string, ...args: any[]): void;
  warn(message: string, ...args: any[]): void;
  error(message: string, error?: Error, ...args: any[]): void;
}

// ============================================================================
// Plugin Lifecycle Types
// ============================================================================

export interface PluginLifecycle {
  activate?(context: PluginContext): Promise<void> | void;
  deactivate?(context: PluginContext): Promise<void> | void;
  onStateChange?(state: PluginState, context: PluginContext): void;
}

export interface Plugin extends PluginLifecycle {
  readonly manifest: PluginManifest;
}

export type PluginFactory = (context: PluginContext) => Plugin | Promise<Plugin>;

export interface LifecycleHook {
  type: LifecycleHookType;
  handler: LifecycleHookHandler;
  priority?: number;
}

export enum LifecycleHookType {
  PRE_LOAD = 'pre_load',
  POST_LOAD = 'post_load',
  PRE_ACTIVATE = 'pre_activate',
  POST_ACTIVATE = 'post_activate',
  PRE_DEACTIVATE = 'pre_deactivate',
  POST_DEACTIVATE = 'post_deactivate',
  PRE_UNLOAD = 'pre_unload',
  POST_UNLOAD = 'post_unload',
}

export type LifecycleHookHandler = (
  pluginId: PluginId,
  context?: PluginContext
) => Promise<void> | void;

// ============================================================================
// Extension Point Types
// ============================================================================

export interface PluginContributions {
  toolbars?: ToolbarContribution[];
  panels?: PanelContribution[];
  menus?: MenuContribution[];
  contextMenus?: ContextMenuContribution[];
  commands?: Command[];
  exporters?: ExporterContribution[];
  importers?: ImporterContribution[];
  tools?: ToolContribution[];
}

export interface ToolbarContribution {
  id: string;
  location: ToolbarLocation;
  items: ToolbarItem[];
  priority?: number;
}

export enum ToolbarLocation {
  TOP = 'top',
  LEFT = 'left',
  RIGHT = 'right',
  BOTTOM = 'bottom',
}

export interface ToolbarItem {
  id: string;
  type: 'button' | 'separator' | 'dropdown' | 'custom';
  label?: string;
  icon?: string;
  tooltip?: string;
  command?: string;
  component?: ComponentType<any>;
  when?: string; // Condition expression
}

export interface PanelContribution {
  id: string;
  title: string;
  icon?: string;
  location: PanelLocation;
  component: ComponentType<PanelProps>;
  priority?: number;
  when?: string;
}

export enum PanelLocation {
  LEFT_SIDEBAR = 'left_sidebar',
  RIGHT_SIDEBAR = 'right_sidebar',
  BOTTOM_PANEL = 'bottom_panel',
  FLOATING = 'floating',
}

export interface PanelProps {
  active: boolean;
  context: PluginContext;
}

export interface MenuContribution {
  id: string;
  location: MenuLocation;
  items: MenuItem[];
  priority?: number;
}

export enum MenuLocation {
  MAIN = 'main',
  FILE = 'file',
  EDIT = 'edit',
  VIEW = 'view',
  TOOLS = 'tools',
  HELP = 'help',
}

export interface MenuItem {
  id: string;
  type: 'item' | 'separator' | 'submenu';
  label?: string;
  icon?: string;
  command?: string;
  submenu?: MenuItem[];
  when?: string;
  shortcut?: string;
}

export interface ContextMenuContribution {
  id: string;
  context: ContextMenuContext;
  items: MenuItem[];
  priority?: number;
  when?: string;
}

export enum ContextMenuContext {
  SCENE = 'scene',
  OBJECT = 'object',
  TIMELINE = 'timeline',
  CANVAS = 'canvas',
  EDITOR = 'editor',
}

export interface ExporterContribution {
  id: string;
  name: string;
  extensions: string[];
  mimeTypes: string[];
  export: (data: any, options?: ExportOptions) => Promise<Blob>;
}

export interface ImporterContribution {
  id: string;
  name: string;
  extensions: string[];
  mimeTypes: string[];
  import: (file: File, options?: ImportOptions) => Promise<any>;
}

export interface ToolContribution {
  id: string;
  name: string;
  icon: string;
  category: string;
  component: ComponentType<ToolProps>;
  cursor?: string;
}

export interface ToolProps {
  active: boolean;
  context: PluginContext;
  sceneData: any;
}

export interface ExportOptions {
  format?: string;
  quality?: number;
  [key: string]: any;
}

export interface ImportOptions {
  merge?: boolean;
  [key: string]: any;
}

// ============================================================================
// Security Types
// ============================================================================

export enum PluginPermission {
  READ_STORAGE = 'storage:read',
  WRITE_STORAGE = 'storage:write',
  READ_FILE = 'file:read',
  WRITE_FILE = 'file:write',
  NETWORK = 'network',
  EXECUTE_COMMAND = 'command:execute',
  REGISTER_COMMAND = 'command:register',
  UI_MODIFY = 'ui:modify',
  SCENE_READ = 'scene:read',
  SCENE_WRITE = 'scene:write',
  CLIPBOARD = 'clipboard',
  NOTIFICATIONS = 'notifications',
}

export enum PluginCapability {
  HOT_RELOAD = 'hot_reload',
  BACKGROUND_TASK = 'background_task',
  WORKER_THREAD = 'worker_thread',
  NATIVE_MODULE = 'native_module',
  WEBGL = 'webgl',
  WEBGPU = 'webgpu',
  WEB_WORKER = 'web_worker',
}

export interface SecurityPolicy {
  permissions: PluginPermission[];
  capabilities: PluginCapability[];
  isolated: boolean;
  sandbox: boolean;
  trustedOrigins?: string[];
}

export interface PermissionRequest {
  pluginId: PluginId;
  permission: PluginPermission;
  reason: string;
}

export interface PermissionResult {
  granted: boolean;
  remember?: boolean;
}

// ============================================================================
// Store Types
// ============================================================================

export interface PluginStoreEntry {
  manifest: PluginManifest;
  downloads: number;
  rating: number;
  reviews: number;
  verified: boolean;
  featured: boolean;
  screenshots?: string[];
  changelog?: PluginChangelog[];
}

export interface PluginChangelog {
  version: string;
  date: string;
  changes: string[];
}

export interface PluginInstallOptions {
  version?: string;
  source?: 'marketplace' | 'local' | 'url';
  autoActivate?: boolean;
}

export interface PluginUpdateInfo {
  pluginId: PluginId;
  currentVersion: string;
  latestVersion: string;
  changelog: PluginChangelog;
  breaking: boolean;
}

// ============================================================================
// Manager Types
// ============================================================================

export interface PluginManagerConfig {
  pluginDirectory: string;
  autoLoad?: boolean;
  autoActivate?: boolean;
  hotReload?: boolean;
  maxPlugins?: number;
  timeout?: number;
}

export interface PluginRegistry {
  register(plugin: Plugin): void;
  unregister(pluginId: PluginId): void;
  get(pluginId: PluginId): Plugin | undefined;
  getAll(): Plugin[];
  has(pluginId: PluginId): boolean;
  clear(): void;
}

export interface PluginLoader {
  load(path: string): Promise<Plugin>;
  unload(pluginId: PluginId): Promise<void>;
  reload(pluginId: PluginId): Promise<void>;
}

export interface PluginValidator {
  validate(manifest: PluginManifest): ValidationResult;
  validatePermissions(permissions: PluginPermission[]): ValidationResult;
  validateDependencies(dependencies: PluginDependency[]): ValidationResult;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  code: string;
  message: string;
  field?: string;
}

export interface ValidationWarning {
  code: string;
  message: string;
  field?: string;
}

// ============================================================================
// Event Types
// ============================================================================

export type EventHandler<T = any> = (data: T) => void | Promise<void>;

export interface PluginEvent<T = any> {
  type: string;
  pluginId: PluginId;
  timestamp: number;
  data: T;
}

export enum SystemEvent {
  PLUGIN_LOADED = 'plugin:loaded',
  PLUGIN_ACTIVATED = 'plugin:activated',
  PLUGIN_DEACTIVATED = 'plugin:deactivated',
  PLUGIN_UNLOADED = 'plugin:unloaded',
  PLUGIN_ERROR = 'plugin:error',
  PLUGIN_UPDATED = 'plugin:updated',
}

// ============================================================================
// UI Types
// ============================================================================

export interface Notification {
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  duration?: number;
  actions?: NotificationAction[];
}

export interface NotificationAction {
  label: string;
  callback: () => void;
}

export interface Dialog {
  type: 'info' | 'warning' | 'error' | 'confirm' | 'prompt';
  title: string;
  message: string;
  defaultValue?: string;
  buttons?: DialogButton[];
}

export interface DialogButton {
  label: string;
  value: any;
  primary?: boolean;
}

export interface DialogResult {
  confirmed: boolean;
  value?: any;
}

export interface Command {
  id: string;
  title: string;
  category?: string;
  icon?: string;
  handler: CommandHandler;
  when?: string;
}

export type CommandHandler = (...args: any[]) => any | Promise<any>;

// ============================================================================
// Utility Types
// ============================================================================

export interface Disposable {
  dispose(): void;
}

export class DisposableStore implements Disposable {
  private disposables: Disposable[] = [];

  add(disposable: Disposable): void {
    this.disposables.push(disposable);
  }

  dispose(): void {
    this.disposables.forEach(d => d.dispose());
    this.disposables = [];
  }
}

export type AsyncFunction<T = any> = (...args: any[]) => Promise<T>;
export type MaybePromise<T> = T | Promise<T>;
