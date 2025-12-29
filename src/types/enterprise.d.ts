/**
 * AccuScene Enterprise - Enterprise Feature Type Declarations v0.2.5
 * Type declarations for enterprise features
 * Updated: 2025-12-28
 */

import {
  UUID,
  Timestamp,
  Email,
  FilePath,
  Point3D,
  Vector3D,
  PaginatedResponse,
  ApiResponse,
} from './global';

// ============================================================================
// USER MANAGEMENT
// ============================================================================

/**
 * User roles with hierarchical permissions
 */
export enum UserRole {
  SUPER_ADMIN = 'super_admin',
  ADMIN = 'admin',
  MANAGER = 'manager',
  INVESTIGATOR = 'investigator',
  ANALYST = 'analyst',
  VIEWER = 'viewer',
  GUEST = 'guest',
}

/**
 * User permissions
 */
export enum Permission {
  // Scene permissions
  SCENE_CREATE = 'scene.create',
  SCENE_READ = 'scene.read',
  SCENE_UPDATE = 'scene.update',
  SCENE_DELETE = 'scene.delete',
  SCENE_SHARE = 'scene.share',

  // Project permissions
  PROJECT_CREATE = 'project.create',
  PROJECT_READ = 'project.read',
  PROJECT_UPDATE = 'project.update',
  PROJECT_DELETE = 'project.delete',
  PROJECT_MANAGE = 'project.manage',

  // User management
  USER_CREATE = 'user.create',
  USER_READ = 'user.read',
  USER_UPDATE = 'user.update',
  USER_DELETE = 'user.delete',

  // Report permissions
  REPORT_CREATE = 'report.create',
  REPORT_READ = 'report.read',
  REPORT_UPDATE = 'report.update',
  REPORT_DELETE = 'report.delete',
  REPORT_EXPORT = 'report.export',
  REPORT_PUBLISH = 'report.publish',

  // System permissions
  SYSTEM_SETTINGS = 'system.settings',
  SYSTEM_AUDIT = 'system.audit',
  SYSTEM_BACKUP = 'system.backup',
}

/**
 * User entity
 */
export interface User {
  id: UUID;
  email: Email;
  username: string;
  firstName: string;
  lastName: string;
  role: UserRole;
  permissions: Permission[];
  organizationId: UUID;
  departmentId?: UUID;
  avatarUrl?: string;
  isActive: boolean;
  emailVerified: boolean;
  lastLoginAt?: Timestamp;
  createdAt: Timestamp;
  updatedAt: Timestamp;
  preferences: UserPreferences;
  metadata?: Record<string, unknown>;
}

/**
 * User preferences
 */
export interface UserPreferences {
  theme: 'light' | 'dark' | 'system';
  language: string;
  timezone: string;
  dateFormat: string;
  timeFormat: '12h' | '24h';
  measurementSystem: 'metric' | 'imperial';
  notifications: NotificationPreferences;
  accessibility: AccessibilityPreferences;
  editor: EditorPreferences;
}

/**
 * Notification preferences
 */
export interface NotificationPreferences {
  email: boolean;
  push: boolean;
  inApp: boolean;
  sceneUpdates: boolean;
  projectUpdates: boolean;
  mentions: boolean;
  comments: boolean;
  reports: boolean;
  digest: 'realtime' | 'hourly' | 'daily' | 'weekly' | 'never';
}

/**
 * Accessibility preferences
 */
export interface AccessibilityPreferences {
  highContrast: boolean;
  reducedMotion: boolean;
  screenReader: boolean;
  fontSize: 'small' | 'medium' | 'large' | 'xlarge';
  keyboardNavigation: boolean;
  colorBlindMode?: 'protanopia' | 'deuteranopia' | 'tritanopia';
}

/**
 * Editor preferences
 */
export interface EditorPreferences {
  autoSave: boolean;
  autoSaveInterval: number;
  gridSnap: boolean;
  gridSize: number;
  showGrid: boolean;
  showRulers: boolean;
  showGuides: boolean;
  defaultViewMode: '2d' | '3d';
  cameraSpeed: number;
  mouseWheelSensitivity: number;
}

// ============================================================================
// ORGANIZATION MANAGEMENT
// ============================================================================

/**
 * Organization entity
 */
export interface Organization {
  id: UUID;
  name: string;
  slug: string;
  description?: string;
  logoUrl?: string;
  website?: string;
  industry?: string;
  size?: 'small' | 'medium' | 'large' | 'enterprise';
  address?: Address;
  billing: BillingInfo;
  subscription: Subscription;
  settings: OrganizationSettings;
  createdAt: Timestamp;
  updatedAt: Timestamp;
  metadata?: Record<string, unknown>;
}

/**
 * Address
 */
export interface Address {
  street: string;
  city: string;
  state: string;
  postalCode: string;
  country: string;
}

/**
 * Billing information
 */
export interface BillingInfo {
  companyName: string;
  taxId?: string;
  email: Email;
  phone?: string;
  address: Address;
}

/**
 * Subscription
 */
export interface Subscription {
  plan: 'free' | 'starter' | 'professional' | 'enterprise';
  status: 'active' | 'trial' | 'past_due' | 'canceled' | 'paused';
  startDate: Timestamp;
  endDate?: Timestamp;
  trialEndsAt?: Timestamp;
  seats: number;
  usedSeats: number;
  features: string[];
  limits: SubscriptionLimits;
}

/**
 * Subscription limits
 */
export interface SubscriptionLimits {
  maxScenes: number;
  maxProjects: number;
  maxUsers: number;
  maxStorageGB: number;
  maxSimulationsPerMonth: number;
  maxReportsPerMonth: number;
  maxAPICallsPerDay: number;
}

/**
 * Organization settings
 */
export interface OrganizationSettings {
  allowSelfSignup: boolean;
  requireEmailVerification: boolean;
  require2FA: boolean;
  allowGuestAccess: boolean;
  defaultUserRole: UserRole;
  sessionTimeout: number;
  passwordPolicy: PasswordPolicy;
  sso: SSOSettings;
  auditLog: AuditLogSettings;
  backup: BackupSettings;
}

/**
 * Password policy
 */
export interface PasswordPolicy {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireNumbers: boolean;
  requireSpecialChars: boolean;
  expiryDays?: number;
  preventReuse: number;
}

/**
 * SSO settings
 */
export interface SSOSettings {
  enabled: boolean;
  provider?: 'saml' | 'oidc' | 'oauth2';
  domain?: string;
  clientId?: string;
  issuerUrl?: string;
  callbackUrl?: string;
  autoProvision: boolean;
}

/**
 * Audit log settings
 */
export interface AuditLogSettings {
  enabled: boolean;
  retentionDays: number;
  logLevel: 'all' | 'important' | 'security';
  exportFormat: 'json' | 'csv' | 'xml';
}

/**
 * Backup settings
 */
export interface BackupSettings {
  enabled: boolean;
  frequency: 'hourly' | 'daily' | 'weekly';
  retentionDays: number;
  destination: 's3' | 'azure' | 'gcp' | 'local';
  encryption: boolean;
}

// ============================================================================
// PROJECT AND SCENE MANAGEMENT
// ============================================================================

/**
 * Project entity
 */
export interface Project {
  id: UUID;
  organizationId: UUID;
  name: string;
  description?: string;
  caseNumber?: string;
  status: ProjectStatus;
  priority: ProjectPriority;
  type: ProjectType;
  assignedTo: UUID[];
  createdBy: UUID;
  tags: string[];
  customFields: Record<string, unknown>;
  scenes: UUID[];
  reports: UUID[];
  attachments: Attachment[];
  collaboration: CollaborationSettings;
  createdAt: Timestamp;
  updatedAt: Timestamp;
  dueDate?: Timestamp;
  completedAt?: Timestamp;
  metadata?: Record<string, unknown>;
}

/**
 * Project status
 */
export enum ProjectStatus {
  DRAFT = 'draft',
  ACTIVE = 'active',
  REVIEW = 'review',
  COMPLETED = 'completed',
  ARCHIVED = 'archived',
  DELETED = 'deleted',
}

/**
 * Project priority
 */
export enum ProjectPriority {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  URGENT = 'urgent',
}

/**
 * Project type
 */
export enum ProjectType {
  ACCIDENT_RECONSTRUCTION = 'accident_reconstruction',
  FORENSIC_ANALYSIS = 'forensic_analysis',
  TRAINING = 'training',
  RESEARCH = 'research',
  LITIGATION_SUPPORT = 'litigation_support',
  EXPERT_WITNESS = 'expert_witness',
}

/**
 * Attachment
 */
export interface Attachment {
  id: UUID;
  name: string;
  type: string;
  size: number;
  url: string;
  uploadedBy: UUID;
  uploadedAt: Timestamp;
  metadata?: Record<string, unknown>;
}

/**
 * Collaboration settings
 */
export interface CollaborationSettings {
  enabled: boolean;
  allowComments: boolean;
  allowAnnotations: boolean;
  allowRealTimeEditing: boolean;
  shareMode: 'private' | 'organization' | 'public';
  sharedWith: SharedUser[];
  inviteLink?: string;
  inviteLinkExpiresAt?: Timestamp;
}

/**
 * Shared user
 */
export interface SharedUser {
  userId: UUID;
  role: 'viewer' | 'editor' | 'admin';
  permissions: Permission[];
  addedAt: Timestamp;
}

/**
 * Scene entity
 */
export interface Scene {
  id: UUID;
  projectId: UUID;
  organizationId: UUID;
  name: string;
  description?: string;
  version: number;
  status: SceneStatus;
  type: SceneType;
  environment: SceneEnvironment;
  vehicles: Vehicle[];
  objects: SceneObject[];
  road: RoadData;
  weather: WeatherConditions;
  lighting: LightingConditions;
  camera: CameraState;
  simulation: SimulationData;
  annotations: Annotation[];
  measurements: Measurement[];
  timeline: TimelineEvent[];
  createdBy: UUID;
  lastModifiedBy: UUID;
  createdAt: Timestamp;
  updatedAt: Timestamp;
  metadata?: Record<string, unknown>;
}

/**
 * Scene status
 */
export enum SceneStatus {
  DRAFT = 'draft',
  IN_PROGRESS = 'in_progress',
  REVIEW = 'review',
  FINALIZED = 'finalized',
  ARCHIVED = 'archived',
}

/**
 * Scene type
 */
export enum SceneType {
  INTERSECTION = 'intersection',
  HIGHWAY = 'highway',
  PARKING_LOT = 'parking_lot',
  RESIDENTIAL = 'residential',
  CUSTOM = 'custom',
}

/**
 * Scene environment
 */
export interface SceneEnvironment {
  location?: GeographicLocation;
  terrain: 'flat' | 'hilly' | 'mountainous';
  surface: 'dry' | 'wet' | 'icy' | 'snowy';
  visibility: number;
  temperature: number;
  time: Timestamp;
}

/**
 * Geographic location
 */
export interface GeographicLocation {
  latitude: number;
  longitude: number;
  altitude: number;
  accuracy?: number;
  address?: string;
}

/**
 * Vehicle
 */
export interface Vehicle {
  id: UUID;
  name: string;
  type: VehicleType;
  make?: string;
  model?: string;
  year?: number;
  color?: string;
  position: Point3D;
  rotation: Vector3D;
  velocity: Vector3D;
  mass: number;
  dimensions: {
    length: number;
    width: number;
    height: number;
  };
  wheelbase: number;
  track: number;
  centerOfGravity: Point3D;
  dragCoefficient: number;
  frontalArea: number;
  tires: TireData;
  braking: BrakingData;
  damage?: DamageData;
  occupants?: Occupant[];
  metadata?: Record<string, unknown>;
}

/**
 * Vehicle type
 */
export enum VehicleType {
  SEDAN = 'sedan',
  SUV = 'suv',
  TRUCK = 'truck',
  MOTORCYCLE = 'motorcycle',
  BICYCLE = 'bicycle',
  PEDESTRIAN = 'pedestrian',
  BUS = 'bus',
  TRAILER = 'trailer',
  EMERGENCY = 'emergency',
}

/**
 * Tire data
 */
export interface TireData {
  width: number;
  aspectRatio: number;
  diameter: number;
  pressure: number;
  coefficient: number;
}

/**
 * Braking data
 */
export interface BrakingData {
  maxForce: number;
  distribution: number;
  abs: boolean;
  coefficient: number;
}

/**
 * Damage data
 */
export interface DamageData {
  severity: 'minor' | 'moderate' | 'severe' | 'total';
  location: string[];
  description?: string;
  photos?: string[];
}

/**
 * Occupant
 */
export interface Occupant {
  id: UUID;
  position: 'driver' | 'front_passenger' | 'rear_left' | 'rear_center' | 'rear_right';
  age?: number;
  gender?: 'male' | 'female' | 'other';
  height?: number;
  weight?: number;
  seatbelt: boolean;
  airbag: boolean;
  injuries?: InjuryData[];
}

/**
 * Injury data
 */
export interface InjuryData {
  type: string;
  severity: 'minor' | 'moderate' | 'serious' | 'critical' | 'fatal';
  bodyPart: string;
  description?: string;
}

/**
 * Scene object
 */
export interface SceneObject {
  id: UUID;
  type: ObjectType;
  name: string;
  position: Point3D;
  rotation: Vector3D;
  scale: Vector3D;
  geometry?: string;
  material?: MaterialData;
  physics?: PhysicsProperties;
  metadata?: Record<string, unknown>;
}

/**
 * Object type
 */
export enum ObjectType {
  TRAFFIC_SIGN = 'traffic_sign',
  TRAFFIC_LIGHT = 'traffic_light',
  BARRIER = 'barrier',
  TREE = 'tree',
  BUILDING = 'building',
  POLE = 'pole',
  DEBRIS = 'debris',
  SKID_MARK = 'skid_mark',
  CUSTOM = 'custom',
}

/**
 * Material data
 */
export interface MaterialData {
  color: string;
  texture?: string;
  roughness: number;
  metalness: number;
  opacity: number;
}

/**
 * Physics properties
 */
export interface PhysicsProperties {
  mass: number;
  friction: number;
  restitution: number;
  static: boolean;
}

/**
 * Road data
 */
export interface RoadData {
  segments: RoadSegment[];
  lanes: Lane[];
  markings: RoadMarking[];
  surfaces: RoadSurface[];
}

/**
 * Road segment
 */
export interface RoadSegment {
  id: UUID;
  points: Point3D[];
  width: number;
  grade: number;
  curvature: number;
}

/**
 * Lane
 */
export interface Lane {
  id: UUID;
  segmentId: UUID;
  index: number;
  width: number;
  type: 'driving' | 'parking' | 'bike' | 'shoulder';
  direction: 'forward' | 'backward' | 'bidirectional';
}

/**
 * Road marking
 */
export interface RoadMarking {
  id: UUID;
  type: 'solid' | 'dashed' | 'double' | 'crosswalk' | 'arrow';
  color: string;
  points: Point3D[];
}

/**
 * Road surface
 */
export interface RoadSurface {
  id: UUID;
  material: 'asphalt' | 'concrete' | 'gravel' | 'dirt';
  condition: 'good' | 'fair' | 'poor';
  friction: number;
}

/**
 * Weather conditions
 */
export interface WeatherConditions {
  type: 'clear' | 'cloudy' | 'rain' | 'snow' | 'fog';
  intensity: number;
  windSpeed: number;
  windDirection: number;
  precipitation: number;
}

/**
 * Lighting conditions
 */
export interface LightingConditions {
  type: 'day' | 'dusk' | 'night';
  ambient: number;
  streetLights: boolean;
  headlights: boolean;
  sun?: {
    position: Vector3D;
    intensity: number;
    color: string;
  };
}

/**
 * Camera state
 */
export interface CameraState {
  position: Point3D;
  target: Point3D;
  zoom: number;
  mode: '2d' | '3d' | 'first-person' | 'orbit';
}

/**
 * Simulation data
 */
export interface SimulationData {
  status: 'idle' | 'running' | 'paused' | 'completed' | 'error';
  startTime: number;
  endTime: number;
  currentTime: number;
  timestep: number;
  iterations: number;
  results?: SimulationResult[];
}

/**
 * Simulation result
 */
export interface SimulationResult {
  time: number;
  vehicles: VehicleState[];
  collisions?: CollisionData[];
  energy?: EnergyData;
}

/**
 * Vehicle state
 */
export interface VehicleState {
  vehicleId: UUID;
  position: Point3D;
  rotation: Vector3D;
  velocity: Vector3D;
  acceleration: Vector3D;
}

/**
 * Collision data
 */
export interface CollisionData {
  time: number;
  vehicle1Id: UUID;
  vehicle2Id: UUID;
  impactPoint: Point3D;
  impactForce: Vector3D;
  deltaV: number;
  principalDirection: number;
}

/**
 * Energy data
 */
export interface EnergyData {
  kinetic: number;
  potential: number;
  dissipated: number;
  total: number;
}

/**
 * Annotation
 */
export interface Annotation {
  id: UUID;
  type: 'text' | 'arrow' | 'measurement' | 'highlight';
  content: string;
  position: Point3D;
  style?: Record<string, unknown>;
  createdBy: UUID;
  createdAt: Timestamp;
}

/**
 * Measurement
 */
export interface Measurement {
  id: UUID;
  type: 'distance' | 'angle' | 'area' | 'velocity';
  value: number;
  unit: string;
  points: Point3D[];
  label?: string;
}

/**
 * Timeline event
 */
export interface TimelineEvent {
  id: UUID;
  time: number;
  type: string;
  description: string;
  actors: UUID[];
  data?: Record<string, unknown>;
}

// ============================================================================
// REPORTING
// ============================================================================

/**
 * Report entity
 */
export interface Report {
  id: UUID;
  projectId: UUID;
  sceneId?: UUID;
  name: string;
  type: ReportType;
  status: ReportStatus;
  template: string;
  sections: ReportSection[];
  createdBy: UUID;
  reviewedBy?: UUID;
  approvedBy?: UUID;
  createdAt: Timestamp;
  updatedAt: Timestamp;
  publishedAt?: Timestamp;
  metadata?: Record<string, unknown>;
}

/**
 * Report type
 */
export enum ReportType {
  ACCIDENT_RECONSTRUCTION = 'accident_reconstruction',
  EXPERT_OPINION = 'expert_opinion',
  TECHNICAL_ANALYSIS = 'technical_analysis',
  COURT_TESTIMONY = 'court_testimony',
  SUMMARY = 'summary',
}

/**
 * Report status
 */
export enum ReportStatus {
  DRAFT = 'draft',
  REVIEW = 'review',
  APPROVED = 'approved',
  PUBLISHED = 'published',
}

/**
 * Report section
 */
export interface ReportSection {
  id: UUID;
  type: 'text' | 'image' | 'table' | 'chart' | 'diagram' | 'calculation';
  title: string;
  content: unknown;
  order: number;
}

// ============================================================================
// EXPORT TYPES
// ============================================================================

export {};
