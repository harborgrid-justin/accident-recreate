/**
 * GraphQL TypeScript Type Definitions
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { Request, Response } from 'express';
import DataLoader from 'dataloader';

// ============================================================================
// Context Types
// ============================================================================

export interface GraphQLContext {
  req: Request;
  res: Response;
  user?: AuthenticatedUser;
  dataloaders: DataLoaders;
  pubsub: PubSubEngine;
  requestId: string;
  startTime: number;
}

export interface AuthenticatedUser {
  id: string;
  email: string;
  role: UserRole;
  organizationId: string;
  permissions: Permission[];
}

export enum UserRole {
  ADMIN = 'ADMIN',
  INVESTIGATOR = 'INVESTIGATOR',
  ANALYST = 'ANALYST',
  VIEWER = 'VIEWER',
}

export enum Permission {
  CASE_CREATE = 'CASE_CREATE',
  CASE_READ = 'CASE_READ',
  CASE_UPDATE = 'CASE_UPDATE',
  CASE_DELETE = 'CASE_DELETE',
  SIMULATION_RUN = 'SIMULATION_RUN',
  REPORT_GENERATE = 'REPORT_GENERATE',
  USER_MANAGE = 'USER_MANAGE',
}

// ============================================================================
// DataLoader Types
// ============================================================================

export interface DataLoaders {
  caseLoader: DataLoader<string, Case>;
  vehicleLoader: DataLoader<string, Vehicle>;
  sceneLoader: DataLoader<string, Scene>;
  userLoader: DataLoader<string, User>;
  reportLoader: DataLoader<string, Report>;
  simulationLoader: DataLoader<string, Simulation>;
}

// ============================================================================
// Domain Entity Types
// ============================================================================

export interface Case {
  id: string;
  caseNumber: string;
  title: string;
  description?: string;
  status: CaseStatus;
  priority: CasePriority;
  incidentDate: Date;
  location: Location;
  weather: WeatherCondition;
  investigatorId: string;
  organizationId: string;
  vehicles: Vehicle[];
  scenes: Scene[];
  simulations: Simulation[];
  reports: Report[];
  metadata: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
  deletedAt?: Date;
}

export enum CaseStatus {
  DRAFT = 'DRAFT',
  ACTIVE = 'ACTIVE',
  REVIEW = 'REVIEW',
  COMPLETED = 'COMPLETED',
  ARCHIVED = 'ARCHIVED',
}

export enum CasePriority {
  LOW = 'LOW',
  MEDIUM = 'MEDIUM',
  HIGH = 'HIGH',
  CRITICAL = 'CRITICAL',
}

export interface Location {
  address: string;
  city: string;
  state: string;
  zipCode: string;
  country: string;
  coordinates: Coordinates;
}

export interface Coordinates {
  latitude: number;
  longitude: number;
}

export interface WeatherCondition {
  temperature: number;
  humidity: number;
  windSpeed: number;
  windDirection: number;
  visibility: number;
  precipitation: number;
  conditions: string;
}

export interface Vehicle {
  id: string;
  caseId: string;
  make: string;
  model: string;
  year: number;
  vin?: string;
  color: string;
  type: VehicleType;
  role: VehicleRole;
  damage: DamageAssessment;
  occupants: Occupant[];
  specifications: VehicleSpecifications;
  createdAt: Date;
  updatedAt: Date;
}

export enum VehicleType {
  SEDAN = 'SEDAN',
  SUV = 'SUV',
  TRUCK = 'TRUCK',
  MOTORCYCLE = 'MOTORCYCLE',
  BUS = 'BUS',
  COMMERCIAL = 'COMMERCIAL',
}

export enum VehicleRole {
  PRIMARY = 'PRIMARY',
  SECONDARY = 'SECONDARY',
  WITNESS = 'WITNESS',
}

export interface DamageAssessment {
  severity: DamageSeverity;
  zones: DamageZone[];
  estimatedCost: number;
  photos: string[];
  description: string;
}

export enum DamageSeverity {
  MINOR = 'MINOR',
  MODERATE = 'MODERATE',
  SEVERE = 'SEVERE',
  TOTAL_LOSS = 'TOTAL_LOSS',
}

export interface DamageZone {
  location: string;
  type: string;
  severity: DamageSeverity;
  measurements?: Measurements;
}

export interface Measurements {
  width: number;
  height: number;
  depth: number;
  unit: MeasurementUnit;
}

export enum MeasurementUnit {
  INCHES = 'INCHES',
  CENTIMETERS = 'CENTIMETERS',
  FEET = 'FEET',
  METERS = 'METERS',
}

export interface Occupant {
  id: string;
  position: OccupantPosition;
  name?: string;
  age?: number;
  injuries?: Injury[];
  restraintUsed: boolean;
  ejected: boolean;
}

export enum OccupantPosition {
  DRIVER = 'DRIVER',
  FRONT_PASSENGER = 'FRONT_PASSENGER',
  REAR_LEFT = 'REAR_LEFT',
  REAR_CENTER = 'REAR_CENTER',
  REAR_RIGHT = 'REAR_RIGHT',
}

export interface Injury {
  type: string;
  severity: InjurySeverity;
  bodyPart: string;
  description: string;
}

export enum InjurySeverity {
  NONE = 'NONE',
  MINOR = 'MINOR',
  MODERATE = 'MODERATE',
  SEVERE = 'SEVERE',
  FATAL = 'FATAL',
}

export interface VehicleSpecifications {
  length: number;
  width: number;
  height: number;
  weight: number;
  wheelbase: number;
  engine: EngineSpecifications;
}

export interface EngineSpecifications {
  type: string;
  displacement: number;
  horsepower: number;
  torque: number;
}

export interface Scene {
  id: string;
  caseId: string;
  name: string;
  description?: string;
  type: SceneType;
  elements: SceneElement[];
  dimensions: SceneDimensions;
  viewState: ViewState;
  metadata: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
}

export enum SceneType {
  INTERSECTION = 'INTERSECTION',
  HIGHWAY = 'HIGHWAY',
  PARKING_LOT = 'PARKING_LOT',
  RESIDENTIAL = 'RESIDENTIAL',
  CUSTOM = 'CUSTOM',
}

export interface SceneElement {
  id: string;
  type: ElementType;
  position: Position;
  rotation: number;
  properties: Record<string, unknown>;
}

export enum ElementType {
  VEHICLE = 'VEHICLE',
  ROAD = 'ROAD',
  TRAFFIC_SIGN = 'TRAFFIC_SIGN',
  TRAFFIC_LIGHT = 'TRAFFIC_LIGHT',
  BUILDING = 'BUILDING',
  TREE = 'TREE',
  MARKER = 'MARKER',
  MEASUREMENT = 'MEASUREMENT',
}

export interface Position {
  x: number;
  y: number;
  z?: number;
}

export interface SceneDimensions {
  width: number;
  height: number;
  scale: number;
  unit: MeasurementUnit;
}

export interface ViewState {
  zoom: number;
  center: Position;
  rotation: number;
}

export interface Simulation {
  id: string;
  caseId: string;
  sceneId: string;
  name: string;
  description?: string;
  status: SimulationStatus;
  progress: number;
  parameters: SimulationParameters;
  results?: SimulationResults;
  startedAt?: Date;
  completedAt?: Date;
  createdAt: Date;
  updatedAt: Date;
}

export enum SimulationStatus {
  PENDING = 'PENDING',
  RUNNING = 'RUNNING',
  COMPLETED = 'COMPLETED',
  FAILED = 'FAILED',
  CANCELLED = 'CANCELLED',
}

export interface SimulationParameters {
  timeStep: number;
  duration: number;
  iterations: number;
  physicsEngine: PhysicsEngine;
  environmentalFactors: EnvironmentalFactors;
}

export enum PhysicsEngine {
  BASIC = 'BASIC',
  ADVANCED = 'ADVANCED',
  REAL_TIME = 'REAL_TIME',
}

export interface EnvironmentalFactors {
  roadFriction: number;
  airDensity: number;
  gravity: number;
  windSpeed?: number;
  windDirection?: number;
}

export interface SimulationResults {
  frames: SimulationFrame[];
  metrics: SimulationMetrics;
  summary: string;
}

export interface SimulationFrame {
  timestamp: number;
  vehicleStates: VehicleState[];
}

export interface VehicleState {
  vehicleId: string;
  position: Position;
  velocity: Velocity;
  acceleration: Acceleration;
  rotation: number;
}

export interface Velocity {
  x: number;
  y: number;
  z?: number;
  magnitude: number;
}

export interface Acceleration {
  x: number;
  y: number;
  z?: number;
  magnitude: number;
}

export interface SimulationMetrics {
  maxSpeed: number;
  impactForce: number;
  energyDissipation: number;
  collisionTime: number;
  deltaV: number;
}

export interface Report {
  id: string;
  caseId: string;
  type: ReportType;
  format: ReportFormat;
  status: ReportStatus;
  title: string;
  content?: string;
  fileUrl?: string;
  generatedBy: string;
  generatedAt?: Date;
  createdAt: Date;
  updatedAt: Date;
}

export enum ReportType {
  PRELIMINARY = 'PRELIMINARY',
  TECHNICAL = 'TECHNICAL',
  EXECUTIVE = 'EXECUTIVE',
  EXPERT_WITNESS = 'EXPERT_WITNESS',
  INSURANCE = 'INSURANCE',
}

export enum ReportFormat {
  PDF = 'PDF',
  DOCX = 'DOCX',
  HTML = 'HTML',
  JSON = 'JSON',
}

export enum ReportStatus {
  DRAFT = 'DRAFT',
  GENERATING = 'GENERATING',
  COMPLETED = 'COMPLETED',
  FAILED = 'FAILED',
}

export interface User {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  role: UserRole;
  organizationId: string;
  permissions: Permission[];
  avatar?: string;
  preferences: UserPreferences;
  lastLoginAt?: Date;
  createdAt: Date;
  updatedAt: Date;
}

export interface UserPreferences {
  theme: Theme;
  notifications: NotificationPreferences;
  defaultMeasurementUnit: MeasurementUnit;
  locale: string;
  timezone: string;
}

export enum Theme {
  LIGHT = 'LIGHT',
  DARK = 'DARK',
  AUTO = 'AUTO',
}

export interface NotificationPreferences {
  email: boolean;
  push: boolean;
  inApp: boolean;
}

// ============================================================================
// Input Types
// ============================================================================

export interface CreateCaseInput {
  title: string;
  description?: string;
  incidentDate: Date;
  location: LocationInput;
  weather?: WeatherConditionInput;
  priority?: CasePriority;
}

export interface LocationInput {
  address: string;
  city: string;
  state: string;
  zipCode: string;
  country: string;
  coordinates?: CoordinatesInput;
}

export interface CoordinatesInput {
  latitude: number;
  longitude: number;
}

export interface WeatherConditionInput {
  temperature: number;
  humidity: number;
  windSpeed: number;
  windDirection: number;
  visibility: number;
  precipitation: number;
  conditions: string;
}

export interface UpdateCaseInput {
  title?: string;
  description?: string;
  status?: CaseStatus;
  priority?: CasePriority;
  incidentDate?: Date;
  location?: LocationInput;
  weather?: WeatherConditionInput;
}

export interface CreateVehicleInput {
  caseId: string;
  make: string;
  model: string;
  year: number;
  vin?: string;
  color: string;
  type: VehicleType;
  role: VehicleRole;
}

export interface CreateSimulationInput {
  caseId: string;
  sceneId: string;
  name: string;
  description?: string;
  parameters: SimulationParametersInput;
}

export interface SimulationParametersInput {
  timeStep: number;
  duration: number;
  iterations: number;
  physicsEngine: PhysicsEngine;
  environmentalFactors?: EnvironmentalFactorsInput;
}

export interface EnvironmentalFactorsInput {
  roadFriction: number;
  airDensity: number;
  gravity: number;
  windSpeed?: number;
  windDirection?: number;
}

export interface GenerateReportInput {
  caseId: string;
  type: ReportType;
  format: ReportFormat;
  options?: ReportOptions;
}

export interface ReportOptions {
  includeSimulations: boolean;
  includePhotos: boolean;
  includeMetrics: boolean;
  customSections?: string[];
}

// ============================================================================
// Pagination Types
// ============================================================================

export interface PaginationInput {
  page?: number;
  limit?: number;
  sortBy?: string;
  sortOrder?: SortOrder;
}

export enum SortOrder {
  ASC = 'ASC',
  DESC = 'DESC',
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
  hasNextPage: boolean;
  hasPreviousPage: boolean;
}

// ============================================================================
// Filter Types
// ============================================================================

export interface CaseFilter {
  status?: CaseStatus[];
  priority?: CasePriority[];
  investigatorId?: string;
  organizationId?: string;
  dateFrom?: Date;
  dateTo?: Date;
  search?: string;
}

// ============================================================================
// Subscription Types
// ============================================================================

export interface PubSubEngine {
  publish(trigger: string, payload: unknown): Promise<void>;
  subscribe(trigger: string, onMessage: Function): Promise<number>;
  unsubscribe(subId: number): void;
}

export interface CaseUpdatePayload {
  case: Case;
  mutation: MutationType;
  userId: string;
}

export enum MutationType {
  CREATED = 'CREATED',
  UPDATED = 'UPDATED',
  DELETED = 'DELETED',
}

export interface SimulationProgressPayload {
  simulation: Simulation;
  progress: number;
  status: SimulationStatus;
  currentFrame?: number;
  totalFrames?: number;
}

// ============================================================================
// Error Types
// ============================================================================

export interface GraphQLError {
  message: string;
  code: ErrorCode;
  path?: string[];
  extensions?: Record<string, unknown>;
}

export enum ErrorCode {
  UNAUTHENTICATED = 'UNAUTHENTICATED',
  UNAUTHORIZED = 'UNAUTHORIZED',
  BAD_REQUEST = 'BAD_REQUEST',
  NOT_FOUND = 'NOT_FOUND',
  CONFLICT = 'CONFLICT',
  INTERNAL_ERROR = 'INTERNAL_ERROR',
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
}

// ============================================================================
// Directive Types
// ============================================================================

export interface AuthDirectiveArgs {
  requires: Permission[];
}

export interface RateLimitDirectiveArgs {
  max: number;
  window: number;
}

export interface ValidateDirectiveArgs {
  schema: string;
}

// ============================================================================
// Federation Types
// ============================================================================

export interface ReferenceResolver<T> {
  __resolveReference(reference: { __typename: string; id: string }): Promise<T | null>;
}
