/**
 * AccuScene Enterprise - Assertion Utilities v0.2.5
 * Runtime assertion functions for defensive programming
 * Updated: 2025-12-28
 */

import {
  isString,
  isNumber,
  isBoolean,
  isObject,
  isArray,
  isNullish,
  isDefined,
  isNonEmptyString,
  isUUID,
  isEmail,
  isPositive,
  isNonNegative,
  isInRange,
  isPoint3D,
  isVector3D,
} from './typeGuards';

// ============================================================================
// ASSERTION ERROR
// ============================================================================

/**
 * Custom error class for assertion failures
 */
export class AssertionError extends Error {
  constructor(message: string, public readonly context?: Record<string, unknown>) {
    super(message);
    this.name = 'AssertionError';

    // Maintains proper stack trace for where error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, AssertionError);
    }
  }
}

// ============================================================================
// BASIC ASSERTIONS
// ============================================================================

/**
 * Assert that a condition is true
 */
export function assert(condition: boolean, message?: string): asserts condition {
  if (!condition) {
    throw new AssertionError(message ?? 'Assertion failed');
  }
}

/**
 * Assert that a value is defined (not null or undefined)
 */
export function assertDefined<T>(
  value: T | null | undefined,
  message?: string
): asserts value is T {
  if (isNullish(value)) {
    throw new AssertionError(message ?? 'Value must be defined', { value });
  }
}

/**
 * Assert that a value is not null
 */
export function assertNotNull<T>(
  value: T | null,
  message?: string
): asserts value is T {
  if (value === null) {
    throw new AssertionError(message ?? 'Value must not be null', { value });
  }
}

/**
 * Assert that a value is not undefined
 */
export function assertNotUndefined<T>(
  value: T | undefined,
  message?: string
): asserts value is T {
  if (value === undefined) {
    throw new AssertionError(message ?? 'Value must not be undefined', { value });
  }
}

/**
 * Assert that code path is unreachable
 */
export function assertNever(value: never, message?: string): never {
  throw new AssertionError(
    message ?? `Unexpected value: ${JSON.stringify(value)}`,
    { value }
  );
}

// ============================================================================
// TYPE ASSERTIONS
// ============================================================================

/**
 * Assert that a value is a string
 */
export function assertString(value: unknown, message?: string): asserts value is string {
  if (!isString(value)) {
    throw new AssertionError(message ?? 'Value must be a string', {
      value,
      type: typeof value,
    });
  }
}

/**
 * Assert that a value is a non-empty string
 */
export function assertNonEmptyString(
  value: unknown,
  message?: string
): asserts value is string {
  if (!isNonEmptyString(value)) {
    throw new AssertionError(message ?? 'Value must be a non-empty string', {
      value,
      type: typeof value,
    });
  }
}

/**
 * Assert that a value is a number
 */
export function assertNumber(value: unknown, message?: string): asserts value is number {
  if (!isNumber(value)) {
    throw new AssertionError(message ?? 'Value must be a number', {
      value,
      type: typeof value,
    });
  }
}

/**
 * Assert that a value is a positive number
 */
export function assertPositive(value: unknown, message?: string): asserts value is number {
  if (!isPositive(value)) {
    throw new AssertionError(message ?? 'Value must be a positive number', {
      value,
      type: typeof value,
    });
  }
}

/**
 * Assert that a value is a non-negative number
 */
export function assertNonNegative(
  value: unknown,
  message?: string
): asserts value is number {
  if (!isNonNegative(value)) {
    throw new AssertionError(message ?? 'Value must be a non-negative number', {
      value,
      type: typeof value,
    });
  }
}

/**
 * Assert that a value is a boolean
 */
export function assertBoolean(value: unknown, message?: string): asserts value is boolean {
  if (!isBoolean(value)) {
    throw new AssertionError(message ?? 'Value must be a boolean', {
      value,
      type: typeof value,
    });
  }
}

/**
 * Assert that a value is an object
 */
export function assertObject(
  value: unknown,
  message?: string
): asserts value is Record<string, unknown> {
  if (!isObject(value)) {
    throw new AssertionError(message ?? 'Value must be an object', {
      value,
      type: typeof value,
    });
  }
}

/**
 * Assert that a value is an array
 */
export function assertArray(value: unknown, message?: string): asserts value is unknown[] {
  if (!isArray(value)) {
    throw new AssertionError(message ?? 'Value must be an array', {
      value,
      type: typeof value,
    });
  }
}

// ============================================================================
// FORMAT ASSERTIONS
// ============================================================================

/**
 * Assert that a value is a valid UUID
 */
export function assertUUID(value: unknown, message?: string): asserts value is string {
  if (!isUUID(value)) {
    throw new AssertionError(message ?? 'Value must be a valid UUID', { value });
  }
}

/**
 * Assert that a value is a valid email
 */
export function assertEmail(value: unknown, message?: string): asserts value is string {
  if (!isEmail(value)) {
    throw new AssertionError(message ?? 'Value must be a valid email', { value });
  }
}

// ============================================================================
// RANGE ASSERTIONS
// ============================================================================

/**
 * Assert that a number is in a specific range
 */
export function assertInRange(
  value: number,
  min: number,
  max: number,
  message?: string
): void {
  if (!isInRange(value, min, max)) {
    throw new AssertionError(
      message ?? `Value must be between ${min} and ${max}`,
      { value, min, max }
    );
  }
}

/**
 * Assert that a number is greater than a minimum value
 */
export function assertGreaterThan(
  value: number,
  min: number,
  message?: string
): void {
  if (value <= min) {
    throw new AssertionError(
      message ?? `Value must be greater than ${min}`,
      { value, min }
    );
  }
}

/**
 * Assert that a number is greater than or equal to a minimum value
 */
export function assertGreaterThanOrEqual(
  value: number,
  min: number,
  message?: string
): void {
  if (value < min) {
    throw new AssertionError(
      message ?? `Value must be greater than or equal to ${min}`,
      { value, min }
    );
  }
}

/**
 * Assert that a number is less than a maximum value
 */
export function assertLessThan(
  value: number,
  max: number,
  message?: string
): void {
  if (value >= max) {
    throw new AssertionError(
      message ?? `Value must be less than ${max}`,
      { value, max }
    );
  }
}

/**
 * Assert that a number is less than or equal to a maximum value
 */
export function assertLessThanOrEqual(
  value: number,
  max: number,
  message?: string
): void {
  if (value > max) {
    throw new AssertionError(
      message ?? `Value must be less than or equal to ${max}`,
      { value, max }
    );
  }
}

// ============================================================================
// STRING ASSERTIONS
// ============================================================================

/**
 * Assert that a string has a minimum length
 */
export function assertMinLength(
  value: string,
  minLength: number,
  message?: string
): void {
  if (value.length < minLength) {
    throw new AssertionError(
      message ?? `String must be at least ${minLength} characters long`,
      { value, length: value.length, minLength }
    );
  }
}

/**
 * Assert that a string has a maximum length
 */
export function assertMaxLength(
  value: string,
  maxLength: number,
  message?: string
): void {
  if (value.length > maxLength) {
    throw new AssertionError(
      message ?? `String must be at most ${maxLength} characters long`,
      { value, length: value.length, maxLength }
    );
  }
}

/**
 * Assert that a string matches a pattern
 */
export function assertMatches(
  value: string,
  pattern: RegExp,
  message?: string
): void {
  if (!pattern.test(value)) {
    throw new AssertionError(
      message ?? `String must match pattern ${pattern}`,
      { value, pattern: pattern.toString() }
    );
  }
}

// ============================================================================
// ARRAY ASSERTIONS
// ============================================================================

/**
 * Assert that an array is not empty
 */
export function assertNonEmptyArray<T>(
  value: T[],
  message?: string
): asserts value is [T, ...T[]] {
  if (value.length === 0) {
    throw new AssertionError(message ?? 'Array must not be empty', { value });
  }
}

/**
 * Assert that an array has a minimum length
 */
export function assertMinArrayLength<T>(
  value: T[],
  minLength: number,
  message?: string
): void {
  if (value.length < minLength) {
    throw new AssertionError(
      message ?? `Array must have at least ${minLength} elements`,
      { length: value.length, minLength }
    );
  }
}

/**
 * Assert that an array has a maximum length
 */
export function assertMaxArrayLength<T>(
  value: T[],
  maxLength: number,
  message?: string
): void {
  if (value.length > maxLength) {
    throw new AssertionError(
      message ?? `Array must have at most ${maxLength} elements`,
      { length: value.length, maxLength }
    );
  }
}

/**
 * Assert that all elements in an array satisfy a predicate
 */
export function assertArrayElements<T>(
  value: unknown[],
  predicate: (item: unknown) => item is T,
  message?: string
): asserts value is T[] {
  const invalidIndex = value.findIndex(item => !predicate(item));
  if (invalidIndex !== -1) {
    throw new AssertionError(
      message ?? `Array element at index ${invalidIndex} failed validation`,
      { index: invalidIndex, value: value[invalidIndex] }
    );
  }
}

// ============================================================================
// OBJECT ASSERTIONS
// ============================================================================

/**
 * Assert that an object has a specific property
 */
export function assertHasProperty<K extends string>(
  value: object,
  key: K,
  message?: string
): asserts value is Record<K, unknown> {
  if (!(key in value)) {
    throw new AssertionError(
      message ?? `Object must have property '${key}'`,
      { value, key }
    );
  }
}

/**
 * Assert that an object has multiple properties
 */
export function assertHasProperties<K extends string>(
  value: object,
  keys: K[],
  message?: string
): asserts value is Record<K, unknown> {
  const missingKeys = keys.filter(key => !(key in value));
  if (missingKeys.length > 0) {
    throw new AssertionError(
      message ?? `Object must have properties: ${missingKeys.join(', ')}`,
      { value, missingKeys }
    );
  }
}

/**
 * Assert that an object has a property with a specific type
 */
export function assertPropertyType<K extends string, T>(
  value: object,
  key: K,
  guard: (v: unknown) => v is T,
  message?: string
): asserts value is Record<K, T> {
  assertHasProperty(value, key, message);
  const propertyValue = (value as Record<K, unknown>)[key];
  if (!guard(propertyValue)) {
    throw new AssertionError(
      message ?? `Property '${key}' has invalid type`,
      { value: propertyValue, key }
    );
  }
}

// ============================================================================
// GEOMETRIC ASSERTIONS
// ============================================================================

/**
 * Assert that a value is a valid Point3D
 */
export function assertPoint3D(
  value: unknown,
  message?: string
): asserts value is { x: number; y: number; z: number } {
  if (!isPoint3D(value)) {
    throw new AssertionError(message ?? 'Value must be a valid Point3D', { value });
  }
}

/**
 * Assert that a value is a valid Vector3D
 */
export function assertVector3D(
  value: unknown,
  message?: string
): asserts value is { x: number; y: number; z: number } {
  if (!isVector3D(value)) {
    throw new AssertionError(message ?? 'Value must be a valid Vector3D', { value });
  }
}

/**
 * Assert that coordinates are within valid bounds
 */
export function assertCoordinatesInBounds(
  x: number,
  y: number,
  z: number,
  bounds: {
    minX: number;
    maxX: number;
    minY: number;
    maxY: number;
    minZ: number;
    maxZ: number;
  },
  message?: string
): void {
  if (
    x < bounds.minX ||
    x > bounds.maxX ||
    y < bounds.minY ||
    y > bounds.maxY ||
    z < bounds.minZ ||
    z > bounds.maxZ
  ) {
    throw new AssertionError(
      message ?? 'Coordinates must be within bounds',
      { x, y, z, bounds }
    );
  }
}

// ============================================================================
// EQUALITY ASSERTIONS
// ============================================================================

/**
 * Assert that two values are equal
 */
export function assertEqual<T>(
  actual: T,
  expected: T,
  message?: string
): void {
  if (actual !== expected) {
    throw new AssertionError(
      message ?? 'Values must be equal',
      { actual, expected }
    );
  }
}

/**
 * Assert that two values are not equal
 */
export function assertNotEqual<T>(
  actual: T,
  unexpected: T,
  message?: string
): void {
  if (actual === unexpected) {
    throw new AssertionError(
      message ?? 'Values must not be equal',
      { actual, unexpected }
    );
  }
}

/**
 * Assert that two values are deeply equal
 */
export function assertDeepEqual<T>(
  actual: T,
  expected: T,
  message?: string
): void {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new AssertionError(
      message ?? 'Values must be deeply equal',
      { actual, expected }
    );
  }
}

// ============================================================================
// INCLUSION ASSERTIONS
// ============================================================================

/**
 * Assert that a value is included in an array
 */
export function assertIncludes<T>(
  array: T[],
  value: T,
  message?: string
): void {
  if (!array.includes(value)) {
    throw new AssertionError(
      message ?? 'Array must include value',
      { array, value }
    );
  }
}

/**
 * Assert that a value is one of the allowed values
 */
export function assertOneOf<T>(
  value: T,
  allowedValues: T[],
  message?: string
): void {
  if (!allowedValues.includes(value)) {
    throw new AssertionError(
      message ?? `Value must be one of: ${allowedValues.join(', ')}`,
      { value, allowedValues }
    );
  }
}

// ============================================================================
// CUSTOM PREDICATE ASSERTIONS
// ============================================================================

/**
 * Assert that a value satisfies a custom predicate
 */
export function assertSatisfies<T>(
  value: unknown,
  predicate: (v: unknown) => v is T,
  message?: string
): asserts value is T {
  if (!predicate(value)) {
    throw new AssertionError(message ?? 'Value does not satisfy predicate', { value });
  }
}

/**
 * Assert that all values in an array satisfy a predicate
 */
export function assertAllSatisfy<T>(
  values: unknown[],
  predicate: (v: unknown) => v is T,
  message?: string
): asserts values is T[] {
  const failedIndex = values.findIndex(v => !predicate(v));
  if (failedIndex !== -1) {
    throw new AssertionError(
      message ?? `Value at index ${failedIndex} does not satisfy predicate`,
      { index: failedIndex, value: values[failedIndex] }
    );
  }
}

/**
 * Assert that at least one value in an array satisfies a predicate
 */
export function assertAnySatisfy<T>(
  values: unknown[],
  predicate: (v: unknown) => v is T,
  message?: string
): void {
  if (!values.some(predicate)) {
    throw new AssertionError(
      message ?? 'At least one value must satisfy predicate',
      { values }
    );
  }
}

// ============================================================================
// CONDITIONAL ASSERTIONS
// ============================================================================

/**
 * Assert a condition only if another condition is true
 */
export function assertIf(
  condition: boolean,
  assertion: () => void,
  message?: string
): void {
  if (condition) {
    try {
      assertion();
    } catch (error) {
      if (error instanceof AssertionError) {
        throw error;
      }
      throw new AssertionError(
        message ?? 'Conditional assertion failed',
        { originalError: error }
      );
    }
  }
}

/**
 * Assert a condition unless another condition is true
 */
export function assertUnless(
  condition: boolean,
  assertion: () => void,
  message?: string
): void {
  assertIf(!condition, assertion, message);
}

// ============================================================================
// ASYNC ASSERTIONS
// ============================================================================

/**
 * Assert that a promise resolves
 */
export async function assertResolves<T>(
  promise: Promise<T>,
  message?: string
): Promise<T> {
  try {
    return await promise;
  } catch (error) {
    throw new AssertionError(
      message ?? 'Promise must resolve',
      { error }
    );
  }
}

/**
 * Assert that a promise rejects
 */
export async function assertRejects(
  promise: Promise<unknown>,
  message?: string
): Promise<void> {
  try {
    await promise;
    throw new AssertionError(message ?? 'Promise must reject');
  } catch (error) {
    if (error instanceof AssertionError) {
      throw error;
    }
    // Promise rejected as expected
  }
}

/**
 * Assert that an async function throws
 */
export async function assertThrowsAsync(
  fn: () => Promise<unknown>,
  message?: string
): Promise<void> {
  await assertRejects(fn(), message);
}

// ============================================================================
// PERFORMANCE ASSERTIONS
// ============================================================================

/**
 * Assert that a function completes within a time limit
 */
export async function assertCompletesWithin(
  fn: () => Promise<unknown> | unknown,
  maxDuration: number,
  message?: string
): Promise<void> {
  const start = performance.now();
  await fn();
  const duration = performance.now() - start;

  if (duration > maxDuration) {
    throw new AssertionError(
      message ?? `Function must complete within ${maxDuration}ms`,
      { duration, maxDuration }
    );
  }
}

// ============================================================================
// EXPORTS
// ============================================================================

export default {
  assert,
  assertDefined,
  assertNotNull,
  assertNotUndefined,
  assertNever,
  assertString,
  assertNonEmptyString,
  assertNumber,
  assertPositive,
  assertNonNegative,
  assertBoolean,
  assertObject,
  assertArray,
  assertUUID,
  assertEmail,
  assertInRange,
  assertGreaterThan,
  assertGreaterThanOrEqual,
  assertLessThan,
  assertLessThanOrEqual,
  assertMinLength,
  assertMaxLength,
  assertMatches,
  assertNonEmptyArray,
  assertMinArrayLength,
  assertMaxArrayLength,
  assertArrayElements,
  assertHasProperty,
  assertHasProperties,
  assertPropertyType,
  assertPoint3D,
  assertVector3D,
  assertCoordinatesInBounds,
  assertEqual,
  assertNotEqual,
  assertDeepEqual,
  assertIncludes,
  assertOneOf,
  assertSatisfies,
  assertAllSatisfy,
  assertAnySatisfy,
  assertIf,
  assertUnless,
  assertResolves,
  assertRejects,
  assertThrowsAsync,
  assertCompletesWithin,
  AssertionError,
};
