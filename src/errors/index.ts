/**
 * AccuScene Enterprise Error Handling Infrastructure
 *
 * Comprehensive error handling with:
 * - Unified error types across TypeScript and Rust
 * - Error boundaries for React components
 * - Error reporting and logging
 * - Recovery strategies
 * - Retry mechanisms
 * - Error metrics and analytics
 *
 * @example
 * ```tsx
 * import { AccuSceneError, ErrorBoundary, useErrorHandler } from '@/errors';
 *
 * // In a component
 * function MyComponent() {
 *   const { handleError } = useErrorHandler();
 *
 *   const loadData = async () => {
 *     try {
 *       const data = await fetchData();
 *     } catch (error) {
 *       handleError(error, 'Failed to load data');
 *     }
 *   };
 * }
 *
 * // Wrap with error boundary
 * <ErrorBoundary>
 *   <MyComponent />
 * </ErrorBoundary>
 * ```
 */

// Core types
export {
  AccuSceneError,
  ErrorCode,
  ErrorSeverity,
  RecoveryAction,
  ErrorContext,
  isAccuSceneError,
  toAccuSceneError,
} from './types';

// Error boundary components
export {
  ErrorBoundary,
  AsyncErrorBoundary,
  withErrorBoundary,
} from './ErrorBoundary';

// Error fallback components
export {
  ErrorFallback,
  InlineErrorFallback,
} from './ErrorFallback';

// Error reporter
export {
  ErrorReporter,
  type ErrorReport,
} from './ErrorReporter';

// Hooks
export {
  useErrorHandler,
  useAsyncError,
  useRetry,
  useErrorRecovery,
  useErrorBoundary,
  useErrorMetrics,
} from './hooks/useErrorHandler';

// Utilities
export {
  withErrorHandling,
  withErrorHandlingSync,
  ensure,
  ensureValid,
  ensureFound,
  retry,
  withTimeout,
  catchError,
  aggregateErrors,
  isErrorCode,
  isErrorSeverity,
  filterErrorsByCode,
  filterErrorsBySeverity,
  groupErrorsByCode,
  groupErrorsBySeverity,
  getErrorDeduplicationKey,
  deduplicateErrors,
  tryCatch,
  tryCatchAsync,
  unwrap,
  unwrapOr,
  CircuitBreaker,
} from './utils/errorUtils';
