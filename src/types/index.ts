/**
 * AccuScene Enterprise Accident Recreation Platform
 * Core Type Definitions
 *
 * This file contains all shared interfaces and types used across the platform.
 * All modules MUST import types from this file to ensure consistency.
 */

// ============================================================================
// USER & AUTHENTICATION TYPES
// ============================================================================

export enum UserRole {
  ADMIN = 'ADMIN',
  INVESTIGATOR = 'INVESTIGATOR',
  VIEWER = 'VIEWER',
  CLIENT = 'CLIENT'
}

export enum UserStatus {
  ACTIVE = 'ACTIVE',
  INACTIVE = 'INACTIVE',
  SUSPENDED = 'SUSPENDED',
  PENDING = 'PENDING'
}

export interface User {
  id: string;
  email: string;
  username: string;
  passwordHash: string;
  firstName: string;
  lastName: string;
  role: UserRole;
  status: UserStatus;
  organization?: string;
  licenseNumber?: string;
  phone?: string;
  createdAt: Date;
  updatedAt: Date;
  lastLoginAt?: Date;
  preferences: UserPreferences;
}

export interface UserPreferences {
  theme: 'light' | 'dark' | 'auto';
  measurementSystem: 'metric' | 'imperial';
  language: string;
  notifications: NotificationSettings;
  defaultDiagramView: DiagramViewMode;
}

export interface NotificationSettings {
  email: boolean;
  inApp: boolean;
  caseUpdates: boolean;
  reportReady: boolean;
  systemAlerts: boolean;
}

// ============================================================================
// CASE MANAGEMENT TYPES
// ============================================================================

export enum CaseStatus {
  DRAFT = 'DRAFT',
  IN_PROGRESS = 'IN_PROGRESS',
  UNDER_REVIEW = 'UNDER_REVIEW',
  COMPLETED = 'COMPLETED',
  ARCHIVED = 'ARCHIVED',
  CLOSED = 'CLOSED'
}

export enum CasePriority {
  LOW = 'LOW',
  MEDIUM = 'MEDIUM',
  HIGH = 'HIGH',
  CRITICAL = 'CRITICAL'
}

export interface Case {
  id: string;
  caseNumber: string;
  title: string;
  description: string;
  status: CaseStatus;
  priority: CasePriority;
  investigatorId: string;
  assignedUsers: string[];
  clientId?: string;
  clientName?: string;
  incidentDate: Date;
  reportDate: Date;
  location: Location;
  createdAt: Date;
  updatedAt: Date;
  completedAt?: Date;
  metadata: CaseMetadata;
  tags: string[];
}

export interface CaseMetadata {
  policeReportNumber?: string;
  insuranceClaimNumber?: string;
  courtCaseNumber?: string;
  jurisdiction?: string;
  respondingOfficers?: string[];
  estimatedDamages?: number;
  injurySeverity?: 'none' | 'minor' | 'moderate' | 'severe' | 'fatal';
  vehiclesInvolved: number;
  pedestriansInvolved: number;
  cyclistsInvolved: number;
}

export interface Location {
  address: string;
  city: string;
  state: string;
  zipCode: string;
  country: string;
  latitude?: number;
  longitude?: number;
  intersection?: string;
  roadType?: string;
}

// ============================================================================
// ACCIDENT & DIAGRAM TYPES
// ============================================================================

export enum AccidentType {
  REAR_END = 'REAR_END',
  HEAD_ON = 'HEAD_ON',
  SIDE_IMPACT = 'SIDE_IMPACT',
  ANGLE = 'ANGLE',
  SIDESWIPE = 'SIDESWIPE',
  ROLLOVER = 'ROLLOVER',
  PEDESTRIAN = 'PEDESTRIAN',
  FIXED_OBJECT = 'FIXED_OBJECT',
  MULTIPLE_VEHICLE = 'MULTIPLE_VEHICLE',
  OTHER = 'OTHER'
}

export interface Accident {
  id: string;
  caseId: string;
  type: AccidentType;
  timestamp: Date;
  location: Location;
  weather: Weather;
  roadConditions: RoadCondition;
  vehicles: Vehicle[];
  diagram: AccidentDiagram;
  timeline: AccidentTimeline[];
  physics: PhysicsSimulation;
  createdAt: Date;
  updatedAt: Date;
}

export interface AccidentDiagram {
  id: string;
  accidentId: string;
  version: number;
  elements: DiagramElement[];
  scale: number; // pixels per meter
  dimensions: DiagramDimensions;
  viewSettings: DiagramViewSettings;
  layers: DiagramLayer[];
  createdAt: Date;
  updatedAt: Date;
}

export interface DiagramDimensions {
  width: number;
  height: number;
  realWorldWidth: number; // in meters
  realWorldHeight: number; // in meters
}

export interface DiagramViewSettings {
  gridEnabled: boolean;
  gridSize: number;
  snapToGrid: boolean;
  showMeasurements: boolean;
  showVelocityVectors: boolean;
  showTrajectories: boolean;
  showCollisionPoints: boolean;
  showTimestamps: boolean;
}

export enum DiagramViewMode {
  TOP_DOWN = 'TOP_DOWN',
  PERSPECTIVE = 'PERSPECTIVE',
  THREE_D = '3D'
}

export enum DiagramElementType {
  VEHICLE = 'VEHICLE',
  PEDESTRIAN = 'PEDESTRIAN',
  CYCLIST = 'CYCLIST',
  ROAD = 'ROAD',
  LANE_MARKING = 'LANE_MARKING',
  TRAFFIC_SIGNAL = 'TRAFFIC_SIGNAL',
  SIGN = 'SIGN',
  DEBRIS = 'DEBRIS',
  SKID_MARK = 'SKID_MARK',
  COLLISION_POINT = 'COLLISION_POINT',
  MEASUREMENT = 'MEASUREMENT',
  ANNOTATION = 'ANNOTATION',
  FIXED_OBJECT = 'FIXED_OBJECT'
}

export interface DiagramElement {
  id: string;
  type: DiagramElementType;
  position: Position;
  rotation: number; // degrees
  scale: number;
  zIndex: number;
  layerId: string;
  properties: ElementProperties;
  metadata: ElementMetadata;
  locked: boolean;
  visible: boolean;
}

export interface Position {
  x: number; // in diagram coordinates (pixels)
  y: number;
  realWorldX?: number; // in meters
  realWorldY?: number;
}

export interface ElementProperties {
  color?: string;
  strokeWidth?: number;
  opacity?: number;
  label?: string;
  vehicleId?: string;
  length?: number; // meters
  width?: number; // meters
  icon?: string;
  customData?: Record<string, any>;
}

export interface ElementMetadata {
  createdBy: string;
  createdAt: Date;
  updatedBy: string;
  updatedAt: Date;
  notes?: string;
}

export interface DiagramLayer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  opacity: number;
  order: number;
}

// ============================================================================
// VEHICLE TYPES
// ============================================================================

export enum VehicleType {
  SEDAN = 'SEDAN',
  SUV = 'SUV',
  TRUCK = 'TRUCK',
  MOTORCYCLE = 'MOTORCYCLE',
  BUS = 'BUS',
  BICYCLE = 'BICYCLE',
  PEDESTRIAN = 'PEDESTRIAN',
  COMMERCIAL = 'COMMERCIAL',
  EMERGENCY = 'EMERGENCY',
  OTHER = 'OTHER'
}

export enum VehicleCondition {
  EXCELLENT = 'EXCELLENT',
  GOOD = 'GOOD',
  FAIR = 'FAIR',
  POOR = 'POOR',
  UNKNOWN = 'UNKNOWN'
}

export interface Vehicle {
  id: string;
  accidentId: string;
  caseId: string;
  identifier: string; // e.g., "Vehicle A", "V1"
  type: VehicleType;
  make: string;
  model: string;
  year: number;
  color: string;
  vin?: string;
  licensePlate?: string;
  state?: string;
  owner: PersonInfo;
  driver: PersonInfo;
  passengers: PersonInfo[];
  dimensions: VehicleDimensions;
  specifications: VehicleSpecifications;
  condition: VehicleCondition;
  damage: VehicleDamage[];
  preImpactState: VehicleState;
  atImpactState: VehicleState;
  postImpactState: VehicleState;
  insuranceInfo?: InsuranceInfo;
  createdAt: Date;
  updatedAt: Date;
}

export interface PersonInfo {
  name: string;
  age?: number;
  gender?: 'male' | 'female' | 'other' | 'unknown';
  contactInfo?: ContactInfo;
  licenseNumber?: string;
  licenseState?: string;
  injuries?: Injury[];
  seatbeltUsed?: boolean;
  airbagDeployed?: boolean;
}

export interface ContactInfo {
  phone?: string;
  email?: string;
  address?: string;
}

export interface Injury {
  description: string;
  severity: 'minor' | 'moderate' | 'severe' | 'critical' | 'fatal';
  bodyPart: string;
  treatment?: string;
  hospitalName?: string;
}

export interface VehicleDimensions {
  length: number; // meters
  width: number; // meters
  height: number; // meters
  wheelbase: number; // meters
  weight: number; // kg
  groundClearance?: number; // meters
}

export interface VehicleSpecifications {
  engineType?: string;
  horsePower?: number;
  transmission?: string;
  drivetrain?: 'FWD' | 'RWD' | 'AWD' | '4WD';
  fuelType?: string;
  safetyFeatures?: string[];
}

export interface VehicleDamage {
  id: string;
  area: VehicleDamageArea;
  severity: DamageSeverity;
  description: string;
  estimatedCost?: number;
  photos?: string[];
  measurements?: DamageMeasurements;
}

export enum VehicleDamageArea {
  FRONT = 'FRONT',
  FRONT_LEFT = 'FRONT_LEFT',
  FRONT_RIGHT = 'FRONT_RIGHT',
  LEFT_SIDE = 'LEFT_SIDE',
  RIGHT_SIDE = 'RIGHT_SIDE',
  REAR = 'REAR',
  REAR_LEFT = 'REAR_LEFT',
  REAR_RIGHT = 'REAR_RIGHT',
  ROOF = 'ROOF',
  UNDERCARRIAGE = 'UNDERCARRIAGE',
  INTERIOR = 'INTERIOR'
}

export enum DamageSeverity {
  NONE = 'NONE',
  MINOR = 'MINOR',
  MODERATE = 'MODERATE',
  SEVERE = 'SEVERE',
  TOTAL_LOSS = 'TOTAL_LOSS'
}

export interface DamageMeasurements {
  depth?: number; // cm
  width?: number; // cm
  height?: number; // cm
  crushedArea?: number; // cm²
}

export interface VehicleState {
  timestamp: Date;
  position: Position;
  velocity: Velocity;
  acceleration: Acceleration;
  heading: number; // degrees
  lane?: number;
  signalStatus?: SignalStatus;
  brakingStatus?: BrakingStatus;
  steeringAngle?: number; // degrees
}

export interface Velocity {
  magnitude: number; // m/s
  angle: number; // degrees (0 = east, 90 = north)
  x: number; // m/s
  y: number; // m/s
}

export interface Acceleration {
  magnitude: number; // m/s²
  angle: number; // degrees
  x: number; // m/s²
  y: number; // m/s²
}

export enum SignalStatus {
  NONE = 'NONE',
  LEFT = 'LEFT',
  RIGHT = 'RIGHT',
  HAZARD = 'HAZARD'
}

export enum BrakingStatus {
  NONE = 'NONE',
  LIGHT = 'LIGHT',
  MODERATE = 'MODERATE',
  HEAVY = 'HEAVY',
  ABS_ENGAGED = 'ABS_ENGAGED'
}

// ============================================================================
// WEATHER & ROAD CONDITIONS
// ============================================================================

export interface Weather {
  condition: WeatherCondition;
  temperature: number; // Celsius
  visibility: number; // meters
  precipitation: PrecipitationType;
  windSpeed: number; // km/h
  windDirection?: number; // degrees
  humidity?: number; // percentage
  timeOfDay: TimeOfDay;
  lighting: LightingCondition;
}

export enum WeatherCondition {
  CLEAR = 'CLEAR',
  CLOUDY = 'CLOUDY',
  RAIN = 'RAIN',
  HEAVY_RAIN = 'HEAVY_RAIN',
  SNOW = 'SNOW',
  SLEET = 'SLEET',
  FOG = 'FOG',
  HAIL = 'HAIL',
  WIND = 'WIND'
}

export enum PrecipitationType {
  NONE = 'NONE',
  LIGHT = 'LIGHT',
  MODERATE = 'MODERATE',
  HEAVY = 'HEAVY'
}

export enum TimeOfDay {
  DAWN = 'DAWN',
  MORNING = 'MORNING',
  AFTERNOON = 'AFTERNOON',
  DUSK = 'DUSK',
  NIGHT = 'NIGHT'
}

export enum LightingCondition {
  DAYLIGHT = 'DAYLIGHT',
  DUSK_DAWN = 'DUSK_DAWN',
  DARK_STREET_LIGHTS = 'DARK_STREET_LIGHTS',
  DARK_NO_LIGHTS = 'DARK_NO_LIGHTS'
}

export interface RoadCondition {
  surface: RoadSurface;
  condition: RoadConditionType;
  friction: number; // coefficient (0-1)
  grade: number; // percentage
  curve: RoadCurve;
  speedLimit: number; // km/h
  lanes: number;
  laneWidth: number; // meters
  shoulderWidth: number; // meters
  trafficControl: TrafficControl[];
  defects?: RoadDefect[];
}

export enum RoadSurface {
  ASPHALT = 'ASPHALT',
  CONCRETE = 'CONCRETE',
  GRAVEL = 'GRAVEL',
  DIRT = 'DIRT',
  BRICK = 'BRICK',
  OTHER = 'OTHER'
}

export enum RoadConditionType {
  DRY = 'DRY',
  WET = 'WET',
  ICY = 'ICY',
  SNOWY = 'SNOWY',
  MUDDY = 'MUDDY',
  DEBRIS = 'DEBRIS',
  DAMAGED = 'DAMAGED'
}

export enum RoadCurve {
  STRAIGHT = 'STRAIGHT',
  GENTLE = 'GENTLE',
  MODERATE = 'MODERATE',
  SHARP = 'SHARP'
}

export enum TrafficControl {
  NONE = 'NONE',
  STOP_SIGN = 'STOP_SIGN',
  YIELD_SIGN = 'YIELD_SIGN',
  TRAFFIC_SIGNAL = 'TRAFFIC_SIGNAL',
  ROUNDABOUT = 'ROUNDABOUT',
  RAILROAD_CROSSING = 'RAILROAD_CROSSING'
}

export interface RoadDefect {
  type: 'pothole' | 'crack' | 'uneven' | 'debris' | 'other';
  location: Position;
  severity: 'minor' | 'moderate' | 'severe';
  description: string;
}

// ============================================================================
// PHYSICS SIMULATION TYPES
// ============================================================================

export interface PhysicsSimulation {
  id: string;
  accidentId: string;
  algorithm: PhysicsAlgorithm;
  parameters: SimulationParameters;
  results: SimulationResults;
  iterations: number;
  convergence: number;
  validated: boolean;
  validationNotes?: string;
  createdAt: Date;
  updatedAt: Date;
}

export enum PhysicsAlgorithm {
  IMPULSE_MOMENTUM = 'IMPULSE_MOMENTUM',
  ENERGY_CONSERVATION = 'ENERGY_CONSERVATION',
  TRAJECTORY_ANALYSIS = 'TRAJECTORY_ANALYSIS',
  COMBINED = 'COMBINED'
}

export interface SimulationParameters {
  timeStep: number; // seconds
  duration: number; // seconds
  gravity: number; // m/s²
  airResistance: boolean;
  rollingResistance: boolean;
  environmentalFactors: boolean;
  coefficientOfRestitution?: number;
}

export interface SimulationResults {
  collisionPoints: CollisionPoint[];
  deltaV: DeltaVResults[];
  energyDissipation: EnergyDissipation;
  trajectories: Trajectory[];
  impactForces: ImpactForce[];
  confidence: number; // 0-1
  uncertaintyFactors: string[];
}

export interface CollisionPoint {
  id: string;
  position: Position;
  timestamp: number; // seconds from accident start
  vehiclesInvolved: string[];
  impactAngle: number; // degrees
  impactSpeed: number; // m/s
  principalDirectionOfForce: number; // degrees
  depth: number; // meters (crush depth)
}

export interface DeltaVResults {
  vehicleId: string;
  deltaV: number; // m/s
  direction: number; // degrees
  components: {
    x: number;
    y: number;
  };
}

export interface EnergyDissipation {
  totalEnergy: number; // Joules
  kineticEnergyLoss: number; // Joules
  deformationEnergy: number; // Joules
  frictionEnergy: number; // Joules
  otherEnergy: number; // Joules
}

export interface Trajectory {
  vehicleId: string;
  points: TrajectoryPoint[];
}

export interface TrajectoryPoint {
  timestamp: number; // seconds
  position: Position;
  velocity: Velocity;
  acceleration: Acceleration;
  rotation: number; // degrees
}

export interface ImpactForce {
  vehicleId: string;
  peakForce: number; // Newtons
  averageForce: number; // Newtons
  duration: number; // seconds
  impulse: number; // Newton-seconds
}

// ============================================================================
// TIMELINE TYPES
// ============================================================================

export interface AccidentTimeline {
  id: string;
  timestamp: number; // seconds from accident start (can be negative)
  eventType: TimelineEventType;
  description: string;
  involvedEntities: string[]; // vehicle IDs, person names, etc.
  evidence?: string[]; // Evidence IDs
  verified: boolean;
  confidence: number; // 0-1
}

export enum TimelineEventType {
  PRE_IMPACT = 'PRE_IMPACT',
  IMPACT = 'IMPACT',
  POST_IMPACT = 'POST_IMPACT',
  VEHICLE_MANEUVER = 'VEHICLE_MANEUVER',
  PEDESTRIAN_ACTION = 'PEDESTRIAN_ACTION',
  TRAFFIC_SIGNAL_CHANGE = 'TRAFFIC_SIGNAL_CHANGE',
  WITNESS_OBSERVATION = 'WITNESS_OBSERVATION',
  OTHER = 'OTHER'
}

// ============================================================================
// EVIDENCE & WITNESS TYPES
// ============================================================================

export interface Evidence {
  id: string;
  caseId: string;
  type: EvidenceType;
  title: string;
  description: string;
  source: string;
  collectedBy: string;
  collectedAt: Date;
  location?: Position;
  fileUrl?: string;
  fileName?: string;
  fileSize?: number;
  mimeType?: string;
  metadata: EvidenceMetadata;
  chainOfCustody: CustodyRecord[];
  verified: boolean;
  tags: string[];
  createdAt: Date;
  updatedAt: Date;
}

export enum EvidenceType {
  PHOTO = 'PHOTO',
  VIDEO = 'VIDEO',
  DOCUMENT = 'DOCUMENT',
  AUDIO = 'AUDIO',
  PHYSICAL = 'PHYSICAL',
  SKID_MARK = 'SKID_MARK',
  DEBRIS = 'DEBRIS',
  DAMAGE_ASSESSMENT = 'DAMAGE_ASSESSMENT',
  WITNESS_STATEMENT = 'WITNESS_STATEMENT',
  POLICE_REPORT = 'POLICE_REPORT',
  MEDICAL_RECORD = 'MEDICAL_RECORD',
  OTHER = 'OTHER'
}

export interface EvidenceMetadata {
  timestamp?: Date;
  cameraModel?: string;
  gpsCoordinates?: { lat: number; lng: number };
  weatherAtCapture?: string;
  lightingAtCapture?: string;
  photographer?: string;
  notes?: string;
}

export interface CustodyRecord {
  timestamp: Date;
  transferredFrom: string;
  transferredTo: string;
  purpose: string;
  location: string;
  signature?: string;
}

export interface Witness {
  id: string;
  caseId: string;
  name: string;
  contactInfo: ContactInfo;
  role: WitnessRole;
  statement: string;
  statementDate: Date;
  location: string;
  viewpoint: WitnessViewpoint;
  reliability: WitnessReliability;
  interviewed: boolean;
  interviewedBy?: string;
  interviewDate?: Date;
  recordingUrl?: string;
  notes?: string;
  createdAt: Date;
  updatedAt: Date;
}

export enum WitnessRole {
  EYEWITNESS = 'EYEWITNESS',
  DRIVER = 'DRIVER',
  PASSENGER = 'PASSENGER',
  PEDESTRIAN = 'PEDESTRIAN',
  FIRST_RESPONDER = 'FIRST_RESPONDER',
  EXPERT = 'EXPERT',
  OTHER = 'OTHER'
}

export interface WitnessViewpoint {
  position?: Position;
  distance?: number; // meters from accident
  angleOfView?: number; // degrees
  obstructions?: string[];
  visualAcuity?: string;
}

export enum WitnessReliability {
  HIGH = 'HIGH',
  MEDIUM = 'MEDIUM',
  LOW = 'LOW',
  UNKNOWN = 'UNKNOWN'
}

// ============================================================================
// REPORT TYPES
// ============================================================================

export interface Report {
  id: string;
  caseId: string;
  type: ReportType;
  title: string;
  version: number;
  status: ReportStatus;
  author: string;
  reviewer?: string;
  template: string;
  content: ReportContent;
  sections: ReportSection[];
  attachments: ReportAttachment[];
  generatedAt: Date;
  reviewedAt?: Date;
  publishedAt?: Date;
  expiresAt?: Date;
  watermark?: string;
  confidential: boolean;
  distribution: string[];
  createdAt: Date;
  updatedAt: Date;
}

export enum ReportType {
  PRELIMINARY = 'PRELIMINARY',
  COMPREHENSIVE = 'COMPREHENSIVE',
  TECHNICAL = 'TECHNICAL',
  INSURANCE = 'INSURANCE',
  LEGAL = 'LEGAL',
  EXPERT_WITNESS = 'EXPERT_WITNESS',
  SUMMARY = 'SUMMARY'
}

export enum ReportStatus {
  DRAFT = 'DRAFT',
  IN_REVIEW = 'IN_REVIEW',
  APPROVED = 'APPROVED',
  PUBLISHED = 'PUBLISHED',
  ARCHIVED = 'ARCHIVED',
  REJECTED = 'REJECTED'
}

export interface ReportContent {
  executiveSummary?: string;
  introduction?: string;
  methodology?: string;
  findings: string;
  analysis: string;
  conclusions: string;
  recommendations?: string;
  limitations?: string;
  references?: string[];
  appendices?: string[];
}

export interface ReportSection {
  id: string;
  title: string;
  order: number;
  content: string;
  diagrams: string[];
  tables: ReportTable[];
  charts: ReportChart[];
}

export interface ReportTable {
  id: string;
  caption: string;
  headers: string[];
  rows: string[][];
}

export interface ReportChart {
  id: string;
  type: 'bar' | 'line' | 'pie' | 'scatter' | 'timeline';
  caption: string;
  data: any;
}

export interface ReportAttachment {
  id: string;
  name: string;
  type: string;
  url: string;
  size: number;
  description?: string;
}

// ============================================================================
// INSURANCE TYPES
// ============================================================================

export interface InsuranceInfo {
  carrier: string;
  policyNumber: string;
  policyHolder: string;
  effectiveDate: Date;
  expirationDate: Date;
  coverage: InsuranceCoverage;
  agentName?: string;
  agentPhone?: string;
  claimNumber?: string;
  claimStatus?: ClaimStatus;
}

export interface InsuranceCoverage {
  liabilityLimit: number;
  collisionDeductible?: number;
  comprehensiveDeductible?: number;
  medicalPayments?: number;
  uninsuredMotorist?: number;
  underinsuredMotorist?: number;
}

export enum ClaimStatus {
  NOT_FILED = 'NOT_FILED',
  FILED = 'FILED',
  UNDER_INVESTIGATION = 'UNDER_INVESTIGATION',
  PENDING = 'PENDING',
  APPROVED = 'APPROVED',
  DENIED = 'DENIED',
  SETTLED = 'SETTLED',
  LITIGATION = 'LITIGATION'
}

export interface InsuranceClaim {
  id: string;
  caseId: string;
  claimNumber: string;
  carrier: string;
  claimant: string;
  claimType: ClaimType;
  filedDate: Date;
  status: ClaimStatus;
  amountRequested: number;
  amountOffered?: number;
  amountSettled?: number;
  settlementDate?: Date;
  adjuster: string;
  adjusterContact: ContactInfo;
  notes: string;
  documents: string[];
  createdAt: Date;
  updatedAt: Date;
}

export enum ClaimType {
  LIABILITY = 'LIABILITY',
  COLLISION = 'COLLISION',
  COMPREHENSIVE = 'COMPREHENSIVE',
  MEDICAL = 'MEDICAL',
  UNINSURED_MOTORIST = 'UNINSURED_MOTORIST',
  PROPERTY_DAMAGE = 'PROPERTY_DAMAGE',
  PERSONAL_INJURY = 'PERSONAL_INJURY'
}

// ============================================================================
// EXPORT & IMPORT TYPES
// ============================================================================

export interface ExportData {
  version: string;
  exportedAt: Date;
  exportedBy: string;
  cases: Case[];
  accidents: Accident[];
  vehicles: Vehicle[];
  evidence: Evidence[];
  witnesses: Witness[];
  reports: Report[];
  metadata: ExportMetadata;
}

export interface ExportMetadata {
  applicationVersion: string;
  databaseVersion: string;
  exportFormat: 'json' | 'xml' | 'pdf' | 'csv';
  compressed: boolean;
  encrypted: boolean;
  checksum?: string;
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  field: string;
  message: string;
  code: string;
}

export interface ValidationWarning {
  field: string;
  message: string;
  code: string;
}

export interface PaginationParams {
  page: number;
  limit: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export interface SearchParams {
  query: string;
  filters?: Record<string, any>;
  pagination?: PaginationParams;
}

export interface AuditLog {
  id: string;
  entityType: string;
  entityId: string;
  action: AuditAction;
  userId: string;
  username: string;
  timestamp: Date;
  changes?: ChangeRecord[];
  ipAddress?: string;
  userAgent?: string;
}

export enum AuditAction {
  CREATE = 'CREATE',
  READ = 'READ',
  UPDATE = 'UPDATE',
  DELETE = 'DELETE',
  EXPORT = 'EXPORT',
  IMPORT = 'IMPORT',
  LOGIN = 'LOGIN',
  LOGOUT = 'LOGOUT',
  FAILED_LOGIN = 'FAILED_LOGIN'
}

export interface ChangeRecord {
  field: string;
  oldValue: any;
  newValue: any;
}

// ============================================================================
// ENTERPRISE v0.2.0 TYPE RE-EXPORTS
// ============================================================================

/**
 * GraphQL Federation API Types (Agent 6)
 */
export type * from '../graphql/types';

/**
 * Real-time Collaboration Types (Agent 7)
 */
export type * from '../collaboration/types';

/**
 * Advanced UI Component Types (Agent 8)
 */
export type * from '../renderer/components/advanced/types';

/**
 * Plugin Architecture Types (Agent 9)
 */
export type * from '../plugins/types';

/**
 * Performance Monitoring Types (Agent 10)
 */
export type * from '../monitoring/types';
