/**
 * Error utility functions
 */

import { AccuSceneError, ErrorCode, ErrorSeverity, toAccuSceneError } from '../types';

/**
 * Wraps an async function with error handling
 */
export function withErrorHandling<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  context?: string
): T {
  return (async (...args: Parameters<T>): Promise<ReturnType<T>> => {
    try {
      return await fn(...args);
    } catch (error) {
      const accuSceneError = toAccuSceneError(error);
      const errorWithContext = context
        ? accuSceneError.withContext(context)
        : accuSceneError;
      throw errorWithContext;
    }
  }) as T;
}

/**
 * Wraps a sync function with error handling
 */
export function withErrorHandlingSync<T extends (...args: any[]) => any>(
  fn: T,
  context?: string
): T {
  return ((...args: Parameters<T>): ReturnType<T> => {
    try {
      return fn(...args);
    } catch (error) {
      const accuSceneError = toAccuSceneError(error);
      const errorWithContext = context
        ? accuSceneError.withContext(context)
        : accuSceneError;
      throw errorWithContext;
    }
  }) as T;
}

/**
 * Ensures a condition is true, otherwise throws an error
 */
export function ensure(
  condition: boolean,
  code: ErrorCode,
  message: string,
  details?: string
): asserts condition {
  if (!condition) {
    throw new AccuSceneError({ code, message, details });
  }
}

/**
 * Creates a validation error if condition is false
 */
export function ensureValid(
  condition: boolean,
  message: string,
  details?: string
): asserts condition {
  ensure(condition, ErrorCode.VALIDATION, message, details);
}

/**
 * Creates a not found error if value is null or undefined
 */
export function ensureFound<T>(
  value: T | null | undefined,
  message: string
): asserts value is T {
  if (value == null) {
    throw AccuSceneError.notFound(message);
  }
}

/**
 * Retries an async operation with exponential backoff
 */
export async function retry<T>(
  fn: () => Promise<T>,
  options: {
    maxAttempts?: number;
    initialDelay?: number;
    maxDelay?: number;
    backoffMultiplier?: number;
    shouldRetry?: (error: AccuSceneError) => boolean;
  } = {}
): Promise<T> {
  const {
    maxAttempts = 3,
    initialDelay = 100,
    maxDelay = 30000,
    backoffMultiplier = 2,
    shouldRetry = (err) => err.code === ErrorCode.NETWORK || err.code === ErrorCode.TIMEOUT,
  } = options;

  let attempt = 0;
  let lastError: AccuSceneError | null = null;

  while (attempt < maxAttempts) {
    try {
      return await fn();
    } catch (error) {
      const accuSceneError = toAccuSceneError(error);
      lastError = accuSceneError;

      if (!shouldRetry(accuSceneError)) {
        throw accuSceneError;
      }

      attempt++;

      if (attempt < maxAttempts) {
        const delay = Math.min(
          initialDelay * Math.pow(backoffMultiplier, attempt),
          maxDelay
        );
        await sleep(delay);
      }
    }
  }

  throw lastError?.withContext(`Failed after ${maxAttempts} attempts`) ??
    AccuSceneError.internal('Retry failed');
}

/**
 * Sleep utility
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Wraps a promise with a timeout
 */
export async function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  message = 'Operation timed out'
): Promise<T> {
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => {
      reject(AccuSceneError.timeout(message));
    }, timeoutMs);
  });

  return Promise.race([promise, timeoutPromise]);
}

/**
 * Catches and converts errors to AccuSceneError
 */
export function catchError<T>(
  promise: Promise<T>,
  context?: string
): Promise<T | AccuSceneError> {
  return promise.catch((error) => {
    const accuSceneError = toAccuSceneError(error);
    return context ? accuSceneError.withContext(context) : accuSceneError;
  });
}

/**
 * Aggregates multiple errors into one
 */
export function aggregateErrors(
  errors: Array<Error | AccuSceneError>,
  message = 'Multiple errors occurred'
): AccuSceneError {
  const accuSceneErrors = errors.map(toAccuSceneError);

  const details = accuSceneErrors.map((err, i) => `${i + 1}. ${err.message}`).join('\n');

  const highestSeverity = accuSceneErrors.reduce(
    (max, err) => (err.severity > max ? err.severity : max),
    ErrorSeverity.LOW
  );

  return new AccuSceneError({
    code: ErrorCode.INTERNAL,
    message,
    details,
    severity: highestSeverity,
  });
}

/**
 * Checks if an error is of a specific code
 */
export function isErrorCode(error: unknown, code: ErrorCode): boolean {
  if (!(error instanceof AccuSceneError)) {
    return false;
  }
  return error.code === code;
}

/**
 * Checks if an error is of a specific severity or higher
 */
export function isErrorSeverity(
  error: unknown,
  minSeverity: ErrorSeverity
): boolean {
  if (!(error instanceof AccuSceneError)) {
    return false;
  }
  return error.severity >= minSeverity;
}

/**
 * Filters errors by code
 */
export function filterErrorsByCode(
  errors: AccuSceneError[],
  code: ErrorCode
): AccuSceneError[] {
  return errors.filter((err) => err.code === code);
}

/**
 * Filters errors by severity
 */
export function filterErrorsBySeverity(
  errors: AccuSceneError[],
  minSeverity: ErrorSeverity
): AccuSceneError[] {
  return errors.filter((err) => err.severity >= minSeverity);
}

/**
 * Groups errors by code
 */
export function groupErrorsByCode(
  errors: AccuSceneError[]
): Map<ErrorCode, AccuSceneError[]> {
  const groups = new Map<ErrorCode, AccuSceneError[]>();

  errors.forEach((err) => {
    const existing = groups.get(err.code) || [];
    existing.push(err);
    groups.set(err.code, existing);
  });

  return groups;
}

/**
 * Groups errors by severity
 */
export function groupErrorsBySeverity(
  errors: AccuSceneError[]
): Map<ErrorSeverity, AccuSceneError[]> {
  const groups = new Map<ErrorSeverity, AccuSceneError[]>();

  errors.forEach((err) => {
    const existing = groups.get(err.severity) || [];
    existing.push(err);
    groups.set(err.severity, existing);
  });

  return groups;
}

/**
 * Creates a deduplication key for an error
 */
export function getErrorDeduplicationKey(error: AccuSceneError): string {
  return `${error.code}:${error.message}`;
}

/**
 * Deduplicates errors based on code and message
 */
export function deduplicateErrors(errors: AccuSceneError[]): AccuSceneError[] {
  const seen = new Set<string>();
  return errors.filter((err) => {
    const key = getErrorDeduplicationKey(err);
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

/**
 * Safely executes a function and returns result or error
 */
export function tryCatch<T>(
  fn: () => T
): { success: true; data: T } | { success: false; error: AccuSceneError } {
  try {
    return { success: true, data: fn() };
  } catch (error) {
    return { success: false, error: toAccuSceneError(error) };
  }
}

/**
 * Safely executes an async function and returns result or error
 */
export async function tryCatchAsync<T>(
  fn: () => Promise<T>
): Promise<{ success: true; data: T } | { success: false; error: AccuSceneError }> {
  try {
    return { success: true, data: await fn() };
  } catch (error) {
    return { success: false, error: toAccuSceneError(error) };
  }
}

/**
 * Unwraps a result, throwing the error if unsuccessful
 */
export function unwrap<T>(
  result: { success: true; data: T } | { success: false; error: AccuSceneError }
): T {
  if (result.success) {
    return result.data;
  }
  throw result.error;
}

/**
 * Unwraps a result with a default value if unsuccessful
 */
export function unwrapOr<T>(
  result: { success: true; data: T } | { success: false; error: AccuSceneError },
  defaultValue: T
): T {
  return result.success ? result.data : defaultValue;
}

/**
 * Creates a circuit breaker for error-prone operations
 */
export class CircuitBreaker<T extends (...args: any[]) => Promise<any>> {
  private failureCount = 0;
  private successCount = 0;
  private isOpen = false;
  private lastFailureTime: number | null = null;

  constructor(
    private fn: T,
    private options: {
      failureThreshold?: number;
      successThreshold?: number;
      timeout?: number;
    } = {}
  ) {
    this.options = {
      failureThreshold: 5,
      successThreshold: 2,
      timeout: 60000,
      ...options,
    };
  }

  async execute(...args: Parameters<T>): Promise<ReturnType<T>> {
    if (this.isOpen) {
      if (
        this.lastFailureTime &&
        Date.now() - this.lastFailureTime > this.options.timeout!
      ) {
        this.isOpen = false;
        this.failureCount = 0;
        this.successCount = 0;
      } else {
        throw AccuSceneError.unavailable('Circuit breaker is open');
      }
    }

    try {
      const result = await this.fn(...args);
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess(): void {
    this.successCount++;
    if (this.successCount >= this.options.successThreshold!) {
      this.failureCount = 0;
      this.isOpen = false;
    }
  }

  private onFailure(): void {
    this.failureCount++;
    this.successCount = 0;
    this.lastFailureTime = Date.now();

    if (this.failureCount >= this.options.failureThreshold!) {
      this.isOpen = true;
    }
  }

  reset(): void {
    this.failureCount = 0;
    this.successCount = 0;
    this.isOpen = false;
    this.lastFailureTime = null;
  }

  getState(): {
    isOpen: boolean;
    failureCount: number;
    successCount: number;
  } {
    return {
      isOpen: this.isOpen,
      failureCount: this.failureCount,
      successCount: this.successCount,
    };
  }
}
