/**
 * AccuScene Enterprise - Type Guard Utilities v0.2.5
 * Runtime type checking and validation functions
 * Updated: 2025-12-28
 */

import {
  Scene,
  Vehicle,
  Project,
  User,
  Organization,
  Report,
  SimulationState,
  VehicleType,
  ProjectStatus,
  UserRole,
} from '@types/enterprise';

// ============================================================================
// PRIMITIVE TYPE GUARDS
// ============================================================================

/**
 * Type guard for string
 */
export function isString(value: unknown): value is string {
  return typeof value === 'string';
}

/**
 * Type guard for number (excluding NaN)
 */
export function isNumber(value: unknown): value is number {
  return typeof value === 'number' && !Number.isNaN(value);
}

/**
 * Type guard for boolean
 */
export function isBoolean(value: unknown): value is boolean {
  return typeof value === 'boolean';
}

/**
 * Type guard for null
 */
export function isNull(value: unknown): value is null {
  return value === null;
}

/**
 * Type guard for undefined
 */
export function isUndefined(value: unknown): value is undefined {
  return value === undefined;
}

/**
 * Type guard for null or undefined
 */
export function isNullish(value: unknown): value is null | undefined {
  return value === null || value === undefined;
}

/**
 * Type guard for non-null and non-undefined
 */
export function isDefined<T>(value: T | null | undefined): value is T {
  return value !== null && value !== undefined;
}

/**
 * Type guard for function
 */
export function isFunction(value: unknown): value is (...args: unknown[]) => unknown {
  return typeof value === 'function';
}

/**
 * Type guard for object (excluding null)
 */
export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

/**
 * Type guard for plain object (not array, date, etc.)
 */
export function isPlainObject(value: unknown): value is Record<string, unknown> {
  if (!isObject(value)) {
    return false;
  }
  const prototype = Object.getPrototypeOf(value);
  return prototype === null || prototype === Object.prototype;
}

/**
 * Type guard for array
 */
export function isArray<T = unknown>(value: unknown): value is T[] {
  return Array.isArray(value);
}

/**
 * Type guard for array of specific type
 */
export function isArrayOf<T>(
  value: unknown,
  guard: (item: unknown) => item is T
): value is T[] {
  return isArray(value) && value.every(guard);
}

/**
 * Type guard for Date
 */
export function isDate(value: unknown): value is Date {
  return value instanceof Date && !isNaN(value.getTime());
}

/**
 * Type guard for RegExp
 */
export function isRegExp(value: unknown): value is RegExp {
  return value instanceof RegExp;
}

/**
 * Type guard for Error
 */
export function isError(value: unknown): value is Error {
  return value instanceof Error;
}

/**
 * Type guard for Promise
 */
export function isPromise<T = unknown>(value: unknown): value is Promise<T> {
  return value instanceof Promise || (
    isObject(value) &&
    isFunction((value as { then?: unknown }).then)
  );
}

// ============================================================================
// NUMERIC TYPE GUARDS
// ============================================================================

/**
 * Type guard for integer
 */
export function isInteger(value: unknown): value is number {
  return isNumber(value) && Number.isInteger(value);
}

/**
 * Type guard for positive number
 */
export function isPositive(value: unknown): value is number {
  return isNumber(value) && value > 0;
}

/**
 * Type guard for non-negative number
 */
export function isNonNegative(value: unknown): value is number {
  return isNumber(value) && value >= 0;
}

/**
 * Type guard for number in range
 */
export function isInRange(value: unknown, min: number, max: number): value is number {
  return isNumber(value) && value >= min && value <= max;
}

/**
 * Type guard for finite number
 */
export function isFiniteNumber(value: unknown): value is number {
  return isNumber(value) && Number.isFinite(value);
}

// ============================================================================
// STRING TYPE GUARDS
// ============================================================================

/**
 * Type guard for non-empty string
 */
export function isNonEmptyString(value: unknown): value is string {
  return isString(value) && value.trim().length > 0;
}

/**
 * Type guard for UUID format
 */
export function isUUID(value: unknown): value is string {
  if (!isString(value)) {
    return false;
  }
  const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  return uuidRegex.test(value);
}

/**
 * Type guard for email format
 */
export function isEmail(value: unknown): value is string {
  if (!isString(value)) {
    return false;
  }
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(value);
}

/**
 * Type guard for URL format
 */
export function isURL(value: unknown): value is string {
  if (!isString(value)) {
    return false;
  }
  try {
    new URL(value);
    return true;
  } catch {
    return false;
  }
}

/**
 * Type guard for ISO date string
 */
export function isISODateString(value: unknown): value is string {
  if (!isString(value)) {
    return false;
  }
  const date = new Date(value);
  return isDate(date);
}

/**
 * Type guard for hex color
 */
export function isHexColor(value: unknown): value is string {
  if (!isString(value)) {
    return false;
  }
  const hexRegex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i;
  return hexRegex.test(value);
}

// ============================================================================
// STRUCTURE TYPE GUARDS
// ============================================================================

/**
 * Type guard for object with specific property
 */
export function hasProperty<K extends string>(
  value: unknown,
  key: K
): value is Record<K, unknown> {
  return isObject(value) && key in value;
}

/**
 * Type guard for object with multiple properties
 */
export function hasProperties<K extends string>(
  value: unknown,
  keys: K[]
): value is Record<K, unknown> {
  return isObject(value) && keys.every(key => key in value);
}

/**
 * Type guard for object with specific property type
 */
export function hasPropertyOfType<K extends string, T>(
  value: unknown,
  key: K,
  guard: (value: unknown) => value is T
): value is Record<K, T> {
  return hasProperty(value, key) && guard(value[key]);
}

// ============================================================================
// GEOMETRIC TYPE GUARDS
// ============================================================================

/**
 * Type guard for Point2D
 */
export function isPoint2D(value: unknown): value is { x: number; y: number } {
  return (
    isObject(value) &&
    hasPropertyOfType(value, 'x', isNumber) &&
    hasPropertyOfType(value, 'y', isNumber)
  );
}

/**
 * Type guard for Point3D
 */
export function isPoint3D(
  value: unknown
): value is { x: number; y: number; z: number } {
  return (
    isPoint2D(value) &&
    hasPropertyOfType(value, 'z', isNumber)
  );
}

/**
 * Type guard for Vector3D
 */
export function isVector3D(value: unknown): value is { x: number; y: number; z: number } {
  return isPoint3D(value); // Same structure as Point3D
}

/**
 * Type guard for Dimensions2D
 */
export function isDimensions2D(
  value: unknown
): value is { width: number; height: number } {
  return (
    isObject(value) &&
    hasPropertyOfType(value, 'width', isNonNegative) &&
    hasPropertyOfType(value, 'height', isNonNegative)
  );
}

/**
 * Type guard for Dimensions3D
 */
export function isDimensions3D(
  value: unknown
): value is { width: number; height: number; depth: number } {
  return (
    isDimensions2D(value) &&
    hasPropertyOfType(value, 'depth', isNonNegative)
  );
}

// ============================================================================
// ENTERPRISE TYPE GUARDS
// ============================================================================

/**
 * Type guard for VehicleType enum
 */
export function isVehicleType(value: unknown): value is VehicleType {
  const validTypes: VehicleType[] = [
    VehicleType.SEDAN,
    VehicleType.SUV,
    VehicleType.TRUCK,
    VehicleType.MOTORCYCLE,
    VehicleType.BICYCLE,
    VehicleType.PEDESTRIAN,
    VehicleType.BUS,
    VehicleType.TRAILER,
    VehicleType.EMERGENCY,
  ];
  return validTypes.includes(value as VehicleType);
}

/**
 * Type guard for UserRole enum
 */
export function isUserRole(value: unknown): value is UserRole {
  const validRoles: UserRole[] = [
    UserRole.SUPER_ADMIN,
    UserRole.ADMIN,
    UserRole.MANAGER,
    UserRole.INVESTIGATOR,
    UserRole.ANALYST,
    UserRole.VIEWER,
    UserRole.GUEST,
  ];
  return validRoles.includes(value as UserRole);
}

/**
 * Type guard for ProjectStatus enum
 */
export function isProjectStatus(value: unknown): value is ProjectStatus {
  const validStatuses: ProjectStatus[] = [
    ProjectStatus.DRAFT,
    ProjectStatus.ACTIVE,
    ProjectStatus.REVIEW,
    ProjectStatus.COMPLETED,
    ProjectStatus.ARCHIVED,
    ProjectStatus.DELETED,
  ];
  return validStatuses.includes(value as ProjectStatus);
}

/**
 * Type guard for Vehicle (basic validation)
 */
export function isVehicle(value: unknown): value is Vehicle {
  return (
    isObject(value) &&
    hasPropertyOfType(value, 'id', isUUID) &&
    hasPropertyOfType(value, 'type', isVehicleType) &&
    hasPropertyOfType(value, 'position', isPoint3D) &&
    hasPropertyOfType(value, 'rotation', isVector3D) &&
    hasPropertyOfType(value, 'velocity', isVector3D)
  );
}

/**
 * Type guard for Scene (basic validation)
 */
export function isScene(value: unknown): value is Scene {
  return (
    isObject(value) &&
    hasPropertyOfType(value, 'id', isUUID) &&
    hasPropertyOfType(value, 'projectId', isUUID) &&
    hasPropertyOfType(value, 'name', isNonEmptyString) &&
    hasProperty(value, 'vehicles') &&
    isArrayOf(value.vehicles, isVehicle)
  );
}

/**
 * Type guard for User (basic validation)
 */
export function isUser(value: unknown): value is User {
  return (
    isObject(value) &&
    hasPropertyOfType(value, 'id', isUUID) &&
    hasPropertyOfType(value, 'email', isEmail) &&
    hasPropertyOfType(value, 'username', isNonEmptyString) &&
    hasPropertyOfType(value, 'role', isUserRole) &&
    hasPropertyOfType(value, 'isActive', isBoolean)
  );
}

/**
 * Type guard for Project (basic validation)
 */
export function isProject(value: unknown): value is Project {
  return (
    isObject(value) &&
    hasPropertyOfType(value, 'id', isUUID) &&
    hasPropertyOfType(value, 'organizationId', isUUID) &&
    hasPropertyOfType(value, 'name', isNonEmptyString) &&
    hasPropertyOfType(value, 'status', isProjectStatus)
  );
}

/**
 * Type guard for Organization (basic validation)
 */
export function isOrganization(value: unknown): value is Organization {
  return (
    isObject(value) &&
    hasPropertyOfType(value, 'id', isUUID) &&
    hasPropertyOfType(value, 'name', isNonEmptyString) &&
    hasPropertyOfType(value, 'slug', isNonEmptyString)
  );
}

/**
 * Type guard for Report (basic validation)
 */
export function isReport(value: unknown): value is Report {
  return (
    isObject(value) &&
    hasPropertyOfType(value, 'id', isUUID) &&
    hasPropertyOfType(value, 'projectId', isUUID) &&
    hasPropertyOfType(value, 'name', isNonEmptyString)
  );
}

// ============================================================================
// API RESPONSE TYPE GUARDS
// ============================================================================

/**
 * Type guard for successful API response
 */
export function isSuccessResponse<T>(
  value: unknown,
  dataGuard?: (data: unknown) => data is T
): value is { success: true; data: T } {
  if (!isObject(value) || !hasPropertyOfType(value, 'success', isBoolean)) {
    return false;
  }

  if (value.success !== true) {
    return false;
  }

  if (dataGuard && hasProperty(value, 'data')) {
    return dataGuard(value.data);
  }

  return hasProperty(value, 'data');
}

/**
 * Type guard for error API response
 */
export function isErrorResponse(
  value: unknown
): value is { success: false; error: { code: string; message: string } } {
  return (
    isObject(value) &&
    hasPropertyOfType(value, 'success', isBoolean) &&
    value.success === false &&
    hasProperty(value, 'error') &&
    isObject(value.error) &&
    hasPropertyOfType(value.error, 'code', isString) &&
    hasPropertyOfType(value.error, 'message', isString)
  );
}

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

/**
 * Validate and narrow type or throw error
 */
export function validate<T>(
  value: unknown,
  guard: (value: unknown) => value is T,
  errorMessage?: string
): T {
  if (!guard(value)) {
    throw new TypeError(errorMessage ?? 'Validation failed');
  }
  return value;
}

/**
 * Create a type guard from a predicate
 */
export function createTypeGuard<T>(
  predicate: (value: unknown) => boolean
): (value: unknown) => value is T {
  return (value: unknown): value is T => predicate(value);
}

/**
 * Combine multiple type guards with AND logic
 */
export function andGuards<T>(
  ...guards: Array<(value: unknown) => value is T>
): (value: unknown) => value is T {
  return (value: unknown): value is T => guards.every(guard => guard(value));
}

/**
 * Combine multiple type guards with OR logic
 */
export function orGuards<T>(
  ...guards: Array<(value: unknown) => value is T>
): (value: unknown) => value is T {
  return (value: unknown): value is T => guards.some(guard => guard(value));
}

/**
 * Negate a type guard
 */
export function notGuard<T>(
  guard: (value: unknown) => value is T
): (value: unknown) => value is Exclude<unknown, T> {
  return (value: unknown): value is Exclude<unknown, T> => !guard(value);
}

// ============================================================================
// EXPORTS
// ============================================================================

export default {
  // Primitives
  isString,
  isNumber,
  isBoolean,
  isNull,
  isUndefined,
  isNullish,
  isDefined,
  isFunction,
  isObject,
  isPlainObject,
  isArray,
  isArrayOf,
  isDate,
  isRegExp,
  isError,
  isPromise,

  // Numeric
  isInteger,
  isPositive,
  isNonNegative,
  isInRange,
  isFiniteNumber,

  // String
  isNonEmptyString,
  isUUID,
  isEmail,
  isURL,
  isISODateString,
  isHexColor,

  // Structure
  hasProperty,
  hasProperties,
  hasPropertyOfType,

  // Geometric
  isPoint2D,
  isPoint3D,
  isVector3D,
  isDimensions2D,
  isDimensions3D,

  // Enterprise
  isVehicleType,
  isUserRole,
  isProjectStatus,
  isVehicle,
  isScene,
  isUser,
  isProject,
  isOrganization,
  isReport,

  // API
  isSuccessResponse,
  isErrorResponse,

  // Helpers
  validate,
  createTypeGuard,
  andGuards,
  orGuards,
  notGuard,
};
