/**
 * AccuScene Enterprise Accident Recreation Platform
 * Event Bus Type Definitions
 *
 * This file defines the event system used for inter-module communication.
 * All modules should use the EventBus for decoupled messaging.
 */

import {
  Case,
  Accident,
  Vehicle,
  DiagramElement,
  Report,
  Evidence,
  Witness,
  User,
  PhysicsSimulation,
} from './index';

// ============================================================================
// EVENT BUS TYPES
// ============================================================================

export type EventHandler<T = any> = (data: T) => void | Promise<void>;

export interface EventSubscription {
  id: string;
  event: EventType;
  handler: EventHandler;
  priority: number;
  once: boolean;
}

export interface EventBusInterface {
  subscribe<T>(event: EventType, handler: EventHandler<T>, priority?: number): string;
  subscribeOnce<T>(event: EventType, handler: EventHandler<T>, priority?: number): string;
  unsubscribe(subscriptionId: string): void;
  publish<T>(event: EventType, data: T): Promise<void>;
  clear(): void;
  getSubscriptions(event?: EventType): EventSubscription[];
}

// ============================================================================
// EVENT TYPES ENUM
// ============================================================================

export enum EventType {
  // Case Events
  CASE_CREATED = 'CASE_CREATED',
  CASE_UPDATED = 'CASE_UPDATED',
  CASE_DELETED = 'CASE_DELETED',
  CASE_STATUS_CHANGED = 'CASE_STATUS_CHANGED',
  CASE_ASSIGNED = 'CASE_ASSIGNED',
  CASE_UNASSIGNED = 'CASE_UNASSIGNED',

  // Accident Events
  ACCIDENT_CREATED = 'ACCIDENT_CREATED',
  ACCIDENT_UPDATED = 'ACCIDENT_UPDATED',
  ACCIDENT_DELETED = 'ACCIDENT_DELETED',

  // Diagram Events
  DIAGRAM_CREATED = 'DIAGRAM_CREATED',
  DIAGRAM_UPDATED = 'DIAGRAM_UPDATED',
  DIAGRAM_SAVED = 'DIAGRAM_SAVED',
  DIAGRAM_ELEMENT_ADDED = 'DIAGRAM_ELEMENT_ADDED',
  DIAGRAM_ELEMENT_UPDATED = 'DIAGRAM_ELEMENT_UPDATED',
  DIAGRAM_ELEMENT_DELETED = 'DIAGRAM_ELEMENT_DELETED',
  DIAGRAM_ELEMENT_SELECTED = 'DIAGRAM_ELEMENT_SELECTED',
  DIAGRAM_ELEMENT_DESELECTED = 'DIAGRAM_ELEMENT_DESELECTED',
  DIAGRAM_CLEARED = 'DIAGRAM_CLEARED',
  DIAGRAM_LAYER_CHANGED = 'DIAGRAM_LAYER_CHANGED',
  DIAGRAM_VIEW_CHANGED = 'DIAGRAM_VIEW_CHANGED',
  DIAGRAM_ZOOM_CHANGED = 'DIAGRAM_ZOOM_CHANGED',

  // Vehicle Events
  VEHICLE_CREATED = 'VEHICLE_CREATED',
  VEHICLE_UPDATED = 'VEHICLE_UPDATED',
  VEHICLE_DELETED = 'VEHICLE_DELETED',
  VEHICLE_DAMAGE_ADDED = 'VEHICLE_DAMAGE_ADDED',
  VEHICLE_STATE_UPDATED = 'VEHICLE_STATE_UPDATED',

  // Physics Events
  SIMULATION_STARTED = 'SIMULATION_STARTED',
  SIMULATION_COMPLETED = 'SIMULATION_COMPLETED',
  SIMULATION_FAILED = 'SIMULATION_FAILED',
  SIMULATION_PROGRESS = 'SIMULATION_PROGRESS',
  COLLISION_DETECTED = 'COLLISION_DETECTED',
  TRAJECTORY_CALCULATED = 'TRAJECTORY_CALCULATED',

  // Evidence Events
  EVIDENCE_ADDED = 'EVIDENCE_ADDED',
  EVIDENCE_UPDATED = 'EVIDENCE_UPDATED',
  EVIDENCE_DELETED = 'EVIDENCE_DELETED',
  EVIDENCE_VERIFIED = 'EVIDENCE_VERIFIED',

  // Witness Events
  WITNESS_ADDED = 'WITNESS_ADDED',
  WITNESS_UPDATED = 'WITNESS_UPDATED',
  WITNESS_DELETED = 'WITNESS_DELETED',
  WITNESS_INTERVIEWED = 'WITNESS_INTERVIEWED',

  // Report Events
  REPORT_GENERATION_STARTED = 'REPORT_GENERATION_STARTED',
  REPORT_GENERATION_COMPLETED = 'REPORT_GENERATION_COMPLETED',
  REPORT_GENERATION_FAILED = 'REPORT_GENERATION_FAILED',
  REPORT_UPDATED = 'REPORT_UPDATED',
  REPORT_APPROVED = 'REPORT_APPROVED',
  REPORT_PUBLISHED = 'REPORT_PUBLISHED',
  REPORT_EXPORTED = 'REPORT_EXPORTED',

  // User Events
  USER_LOGGED_IN = 'USER_LOGGED_IN',
  USER_LOGGED_OUT = 'USER_LOGGED_OUT',
  USER_CREATED = 'USER_CREATED',
  USER_UPDATED = 'USER_UPDATED',
  USER_DELETED = 'USER_DELETED',
  USER_ROLE_CHANGED = 'USER_ROLE_CHANGED',
  USER_STATUS_CHANGED = 'USER_STATUS_CHANGED',

  // System Events
  SYSTEM_ERROR = 'SYSTEM_ERROR',
  SYSTEM_WARNING = 'SYSTEM_WARNING',
  SYSTEM_INFO = 'SYSTEM_INFO',
  DATABASE_CONNECTED = 'DATABASE_CONNECTED',
  DATABASE_DISCONNECTED = 'DATABASE_DISCONNECTED',
  DATABASE_ERROR = 'DATABASE_ERROR',
  FILE_UPLOADED = 'FILE_UPLOADED',
  FILE_DELETED = 'FILE_DELETED',
  EXPORT_STARTED = 'EXPORT_STARTED',
  EXPORT_COMPLETED = 'EXPORT_COMPLETED',
  IMPORT_STARTED = 'IMPORT_STARTED',
  IMPORT_COMPLETED = 'IMPORT_COMPLETED',

  // UI Events
  NOTIFICATION_SHOW = 'NOTIFICATION_SHOW',
  NOTIFICATION_HIDE = 'NOTIFICATION_HIDE',
  MODAL_OPEN = 'MODAL_OPEN',
  MODAL_CLOSE = 'MODAL_CLOSE',
  LOADING_START = 'LOADING_START',
  LOADING_END = 'LOADING_END',
  THEME_CHANGED = 'THEME_CHANGED',
  LANGUAGE_CHANGED = 'LANGUAGE_CHANGED',

  // Validation Events
  VALIDATION_STARTED = 'VALIDATION_STARTED',
  VALIDATION_COMPLETED = 'VALIDATION_COMPLETED',
  VALIDATION_FAILED = 'VALIDATION_FAILED',

  // Collaboration Events
  USER_JOINED_SESSION = 'USER_JOINED_SESSION',
  USER_LEFT_SESSION = 'USER_LEFT_SESSION',
  CURSOR_MOVED = 'CURSOR_MOVED',
  SELECTION_CHANGED = 'SELECTION_CHANGED',
  COMMENT_ADDED = 'COMMENT_ADDED',
  COMMENT_UPDATED = 'COMMENT_UPDATED',
  COMMENT_DELETED = 'COMMENT_DELETED',
}

// ============================================================================
// EVENT DATA PAYLOADS
// ============================================================================

// Case Event Payloads
export interface CaseCreatedEvent {
  case: Case;
  createdBy: string;
  timestamp: Date;
}

export interface CaseUpdatedEvent {
  caseId: string;
  case: Partial<Case>;
  updatedBy: string;
  changes: ChangeDetails[];
  timestamp: Date;
}

export interface CaseDeletedEvent {
  caseId: string;
  deletedBy: string;
  timestamp: Date;
}

export interface CaseStatusChangedEvent {
  caseId: string;
  oldStatus: string;
  newStatus: string;
  changedBy: string;
  timestamp: Date;
}

export interface CaseAssignedEvent {
  caseId: string;
  userId: string;
  assignedBy: string;
  timestamp: Date;
}

// Accident Event Payloads
export interface AccidentCreatedEvent {
  accident: Accident;
  caseId: string;
  createdBy: string;
  timestamp: Date;
}

export interface AccidentUpdatedEvent {
  accidentId: string;
  accident: Partial<Accident>;
  updatedBy: string;
  changes: ChangeDetails[];
  timestamp: Date;
}

// Diagram Event Payloads
export interface DiagramCreatedEvent {
  diagramId: string;
  accidentId: string;
  createdBy: string;
  timestamp: Date;
}

export interface DiagramUpdatedEvent {
  diagramId: string;
  changes: ChangeDetails[];
  updatedBy: string;
  timestamp: Date;
}

export interface DiagramSavedEvent {
  diagramId: string;
  version: number;
  savedBy: string;
  timestamp: Date;
}

export interface DiagramElementAddedEvent {
  diagramId: string;
  element: DiagramElement;
  addedBy: string;
  timestamp: Date;
}

export interface DiagramElementUpdatedEvent {
  diagramId: string;
  elementId: string;
  element: Partial<DiagramElement>;
  changes: ChangeDetails[];
  updatedBy: string;
  timestamp: Date;
}

export interface DiagramElementDeletedEvent {
  diagramId: string;
  elementId: string;
  deletedBy: string;
  timestamp: Date;
}

export interface DiagramElementSelectedEvent {
  diagramId: string;
  elementIds: string[];
  selectedBy: string;
  timestamp: Date;
}

export interface DiagramViewChangedEvent {
  diagramId: string;
  viewMode: string;
  zoom: number;
  pan: { x: number; y: number };
  changedBy: string;
  timestamp: Date;
}

// Vehicle Event Payloads
export interface VehicleCreatedEvent {
  vehicle: Vehicle;
  accidentId: string;
  createdBy: string;
  timestamp: Date;
}

export interface VehicleUpdatedEvent {
  vehicleId: string;
  vehicle: Partial<Vehicle>;
  changes: ChangeDetails[];
  updatedBy: string;
  timestamp: Date;
}

export interface VehicleDamageAddedEvent {
  vehicleId: string;
  damage: any;
  addedBy: string;
  timestamp: Date;
}

// Physics Event Payloads
export interface SimulationStartedEvent {
  simulationId: string;
  accidentId: string;
  parameters: any;
  startedBy: string;
  timestamp: Date;
}

export interface SimulationCompletedEvent {
  simulationId: string;
  accidentId: string;
  results: PhysicsSimulation;
  duration: number;
  timestamp: Date;
}

export interface SimulationFailedEvent {
  simulationId: string;
  accidentId: string;
  error: string;
  timestamp: Date;
}

export interface SimulationProgressEvent {
  simulationId: string;
  progress: number; // 0-100
  currentIteration: number;
  totalIterations: number;
  timestamp: Date;
}

export interface CollisionDetectedEvent {
  simulationId: string;
  collisionPoint: any;
  vehiclesInvolved: string[];
  timestamp: Date;
}

// Evidence Event Payloads
export interface EvidenceAddedEvent {
  evidence: Evidence;
  caseId: string;
  addedBy: string;
  timestamp: Date;
}

export interface EvidenceUpdatedEvent {
  evidenceId: string;
  evidence: Partial<Evidence>;
  changes: ChangeDetails[];
  updatedBy: string;
  timestamp: Date;
}

export interface EvidenceVerifiedEvent {
  evidenceId: string;
  verifiedBy: string;
  timestamp: Date;
}

// Witness Event Payloads
export interface WitnessAddedEvent {
  witness: Witness;
  caseId: string;
  addedBy: string;
  timestamp: Date;
}

export interface WitnessInterviewedEvent {
  witnessId: string;
  interviewedBy: string;
  interviewDate: Date;
  timestamp: Date;
}

// Report Event Payloads
export interface ReportGenerationStartedEvent {
  reportId: string;
  caseId: string;
  reportType: string;
  requestedBy: string;
  timestamp: Date;
}

export interface ReportGenerationCompletedEvent {
  reportId: string;
  report: Report;
  duration: number;
  timestamp: Date;
}

export interface ReportGenerationFailedEvent {
  reportId: string;
  caseId: string;
  error: string;
  timestamp: Date;
}

export interface ReportApprovedEvent {
  reportId: string;
  approvedBy: string;
  timestamp: Date;
}

export interface ReportPublishedEvent {
  reportId: string;
  publishedBy: string;
  recipients: string[];
  timestamp: Date;
}

// User Event Payloads
export interface UserLoggedInEvent {
  userId: string;
  username: string;
  ipAddress: string;
  timestamp: Date;
}

export interface UserLoggedOutEvent {
  userId: string;
  username: string;
  timestamp: Date;
}

export interface UserCreatedEvent {
  user: User;
  createdBy: string;
  timestamp: Date;
}

export interface UserRoleChangedEvent {
  userId: string;
  oldRole: string;
  newRole: string;
  changedBy: string;
  timestamp: Date;
}

// System Event Payloads
export interface SystemErrorEvent {
  error: Error;
  context: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  timestamp: Date;
}

export interface SystemWarningEvent {
  message: string;
  context: string;
  timestamp: Date;
}

export interface SystemInfoEvent {
  message: string;
  context: string;
  timestamp: Date;
}

export interface DatabaseErrorEvent {
  operation: string;
  error: Error;
  timestamp: Date;
}

export interface FileUploadedEvent {
  fileId: string;
  fileName: string;
  fileSize: number;
  mimeType: string;
  uploadedBy: string;
  timestamp: Date;
}

export interface ExportStartedEvent {
  exportId: string;
  exportType: string;
  entityIds: string[];
  requestedBy: string;
  timestamp: Date;
}

export interface ExportCompletedEvent {
  exportId: string;
  exportType: string;
  fileUrl: string;
  fileSize: number;
  duration: number;
  timestamp: Date;
}

// UI Event Payloads
export interface NotificationShowEvent {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  message: string;
  duration?: number;
  actions?: NotificationAction[];
  timestamp: Date;
}

export interface NotificationAction {
  label: string;
  action: () => void;
}

export interface ModalOpenEvent {
  modalId: string;
  modalType: string;
  data?: any;
  timestamp: Date;
}

export interface LoadingEvent {
  loadingId: string;
  message?: string;
  timestamp: Date;
}

export interface ThemeChangedEvent {
  theme: 'light' | 'dark' | 'auto';
  changedBy: string;
  timestamp: Date;
}

// Validation Event Payloads
export interface ValidationStartedEvent {
  entityType: string;
  entityId: string;
  timestamp: Date;
}

export interface ValidationCompletedEvent {
  entityType: string;
  entityId: string;
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  timestamp: Date;
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

// Collaboration Event Payloads
export interface UserJoinedSessionEvent {
  userId: string;
  username: string;
  sessionId: string;
  timestamp: Date;
}

export interface UserLeftSessionEvent {
  userId: string;
  username: string;
  sessionId: string;
  timestamp: Date;
}

export interface CursorMovedEvent {
  userId: string;
  position: { x: number; y: number };
  diagramId: string;
  timestamp: Date;
}

export interface CommentAddedEvent {
  commentId: string;
  entityType: string;
  entityId: string;
  content: string;
  authorId: string;
  timestamp: Date;
}

// ============================================================================
// HELPER TYPES
// ============================================================================

export interface ChangeDetails {
  field: string;
  oldValue: any;
  newValue: any;
  changedAt: Date;
}

// ============================================================================
// EVENT PAYLOAD MAPPING
// ============================================================================

export interface EventPayloadMap {
  // Case Events
  [EventType.CASE_CREATED]: CaseCreatedEvent;
  [EventType.CASE_UPDATED]: CaseUpdatedEvent;
  [EventType.CASE_DELETED]: CaseDeletedEvent;
  [EventType.CASE_STATUS_CHANGED]: CaseStatusChangedEvent;
  [EventType.CASE_ASSIGNED]: CaseAssignedEvent;

  // Accident Events
  [EventType.ACCIDENT_CREATED]: AccidentCreatedEvent;
  [EventType.ACCIDENT_UPDATED]: AccidentUpdatedEvent;

  // Diagram Events
  [EventType.DIAGRAM_CREATED]: DiagramCreatedEvent;
  [EventType.DIAGRAM_UPDATED]: DiagramUpdatedEvent;
  [EventType.DIAGRAM_SAVED]: DiagramSavedEvent;
  [EventType.DIAGRAM_ELEMENT_ADDED]: DiagramElementAddedEvent;
  [EventType.DIAGRAM_ELEMENT_UPDATED]: DiagramElementUpdatedEvent;
  [EventType.DIAGRAM_ELEMENT_DELETED]: DiagramElementDeletedEvent;
  [EventType.DIAGRAM_ELEMENT_SELECTED]: DiagramElementSelectedEvent;
  [EventType.DIAGRAM_VIEW_CHANGED]: DiagramViewChangedEvent;

  // Vehicle Events
  [EventType.VEHICLE_CREATED]: VehicleCreatedEvent;
  [EventType.VEHICLE_UPDATED]: VehicleUpdatedEvent;
  [EventType.VEHICLE_DAMAGE_ADDED]: VehicleDamageAddedEvent;

  // Physics Events
  [EventType.SIMULATION_STARTED]: SimulationStartedEvent;
  [EventType.SIMULATION_COMPLETED]: SimulationCompletedEvent;
  [EventType.SIMULATION_FAILED]: SimulationFailedEvent;
  [EventType.SIMULATION_PROGRESS]: SimulationProgressEvent;
  [EventType.COLLISION_DETECTED]: CollisionDetectedEvent;

  // Evidence Events
  [EventType.EVIDENCE_ADDED]: EvidenceAddedEvent;
  [EventType.EVIDENCE_UPDATED]: EvidenceUpdatedEvent;
  [EventType.EVIDENCE_VERIFIED]: EvidenceVerifiedEvent;

  // Witness Events
  [EventType.WITNESS_ADDED]: WitnessAddedEvent;
  [EventType.WITNESS_INTERVIEWED]: WitnessInterviewedEvent;

  // Report Events
  [EventType.REPORT_GENERATION_STARTED]: ReportGenerationStartedEvent;
  [EventType.REPORT_GENERATION_COMPLETED]: ReportGenerationCompletedEvent;
  [EventType.REPORT_GENERATION_FAILED]: ReportGenerationFailedEvent;
  [EventType.REPORT_APPROVED]: ReportApprovedEvent;
  [EventType.REPORT_PUBLISHED]: ReportPublishedEvent;

  // User Events
  [EventType.USER_LOGGED_IN]: UserLoggedInEvent;
  [EventType.USER_LOGGED_OUT]: UserLoggedOutEvent;
  [EventType.USER_CREATED]: UserCreatedEvent;
  [EventType.USER_ROLE_CHANGED]: UserRoleChangedEvent;

  // System Events
  [EventType.SYSTEM_ERROR]: SystemErrorEvent;
  [EventType.SYSTEM_WARNING]: SystemWarningEvent;
  [EventType.SYSTEM_INFO]: SystemInfoEvent;
  [EventType.DATABASE_ERROR]: DatabaseErrorEvent;
  [EventType.FILE_UPLOADED]: FileUploadedEvent;
  [EventType.EXPORT_STARTED]: ExportStartedEvent;
  [EventType.EXPORT_COMPLETED]: ExportCompletedEvent;

  // UI Events
  [EventType.NOTIFICATION_SHOW]: NotificationShowEvent;
  [EventType.MODAL_OPEN]: ModalOpenEvent;
  [EventType.LOADING_START]: LoadingEvent;
  [EventType.LOADING_END]: LoadingEvent;
  [EventType.THEME_CHANGED]: ThemeChangedEvent;

  // Validation Events
  [EventType.VALIDATION_STARTED]: ValidationStartedEvent;
  [EventType.VALIDATION_COMPLETED]: ValidationCompletedEvent;

  // Collaboration Events
  [EventType.USER_JOINED_SESSION]: UserJoinedSessionEvent;
  [EventType.USER_LEFT_SESSION]: UserLeftSessionEvent;
  [EventType.CURSOR_MOVED]: CursorMovedEvent;
  [EventType.COMMENT_ADDED]: CommentAddedEvent;
}
