/**
 * React hooks for error handling
 */

import { useCallback, useState, useEffect } from 'react';
import { AccuSceneError, ErrorCode, RecoveryAction, toAccuSceneError } from '../types';
import { ErrorReporter } from '../ErrorReporter';

/**
 * Hook for handling errors in components
 */
export function useErrorHandler() {
  const [error, setError] = useState<AccuSceneError | null>(null);

  const handleError = useCallback((err: unknown, context?: string) => {
    const accuSceneError = toAccuSceneError(err);
    const errorWithContext = context
      ? accuSceneError.withContext(context)
      : accuSceneError;

    setError(errorWithContext);
    ErrorReporter.reportError(errorWithContext);
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    error,
    handleError,
    clearError,
    hasError: error !== null,
  };
}

/**
 * Hook for async error handling with automatic try-catch
 */
export function useAsyncError() {
  const [error, setError] = useState<AccuSceneError | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const execute = useCallback(
    async <T,>(
      asyncFn: () => Promise<T>,
      options?: {
        onSuccess?: (data: T) => void;
        onError?: (error: AccuSceneError) => void;
        context?: string;
      }
    ): Promise<T | null> => {
      setIsLoading(true);
      setError(null);

      try {
        const result = await asyncFn();
        options?.onSuccess?.(result);
        return result;
      } catch (err) {
        const accuSceneError = toAccuSceneError(err);
        const errorWithContext = options?.context
          ? accuSceneError.withContext(options.context)
          : accuSceneError;

        setError(errorWithContext);
        ErrorReporter.reportError(errorWithContext);
        options?.onError?.(errorWithContext);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    execute,
    error,
    isLoading,
    clearError,
    hasError: error !== null,
  };
}

/**
 * Hook for retry logic with exponential backoff
 */
export function useRetry<T>(
  asyncFn: () => Promise<T>,
  options: {
    maxAttempts?: number;
    initialDelay?: number;
    maxDelay?: number;
    backoffMultiplier?: number;
    shouldRetry?: (error: AccuSceneError) => boolean;
  } = {}
) {
  const {
    maxAttempts = 3,
    initialDelay = 100,
    maxDelay = 30000,
    backoffMultiplier = 2,
    shouldRetry = (err) => isTransientError(err.code),
  } = options;

  const [attempt, setAttempt] = useState(0);
  const [error, setError] = useState<AccuSceneError | null>(null);
  const [isRetrying, setIsRetrying] = useState(false);

  const calculateDelay = (attemptNumber: number): number => {
    const baseDelay = initialDelay * Math.pow(backoffMultiplier, attemptNumber);
    const jitter = Math.random() * 0.1 * baseDelay;
    return Math.min(baseDelay + jitter, maxDelay);
  };

  const execute = useCallback(async (): Promise<T | null> => {
    let currentAttempt = 0;
    let lastError: AccuSceneError | null = null;

    while (currentAttempt < maxAttempts) {
      try {
        setAttempt(currentAttempt + 1);
        const result = await asyncFn();
        setError(null);
        setIsRetrying(false);
        return result;
      } catch (err) {
        const accuSceneError = toAccuSceneError(err);
        lastError = accuSceneError;

        if (!shouldRetry(accuSceneError)) {
          setError(accuSceneError);
          ErrorReporter.reportError(accuSceneError);
          return null;
        }

        currentAttempt++;

        if (currentAttempt < maxAttempts) {
          setIsRetrying(true);
          const delay = calculateDelay(currentAttempt);
          await new Promise((resolve) => setTimeout(resolve, delay));
        }
      }
    }

    if (lastError) {
      const finalError = lastError.withContext(
        `Failed after ${maxAttempts} attempts`
      );
      setError(finalError);
      ErrorReporter.reportError(finalError);
    }

    setIsRetrying(false);
    return null;
  }, [asyncFn, maxAttempts, shouldRetry, initialDelay, maxDelay, backoffMultiplier]);

  return {
    execute,
    attempt,
    error,
    isRetrying,
    hasError: error !== null,
  };
}

/**
 * Hook for error recovery strategies
 */
export function useErrorRecovery(error: AccuSceneError | null) {
  const [recoveryAction, setRecoveryAction] = useState<RecoveryAction | null>(null);

  useEffect(() => {
    if (!error) {
      setRecoveryAction(null);
      return;
    }

    // Determine recovery action based on error
    const action = determineRecoveryAction(error);
    setRecoveryAction(action);
  }, [error]);

  const executeRecovery = useCallback(
    async (customAction?: RecoveryAction): Promise<boolean> => {
      if (!error) return false;

      const action = customAction || recoveryAction;
      if (!action) return false;

      try {
        switch (action) {
          case RecoveryAction.RETRY:
            // Caller should implement retry logic
            return true;

          case RecoveryAction.FALLBACK:
            // Caller should provide fallback data/behavior
            return true;

          case RecoveryAction.SKIP:
            // Continue without the failed operation
            return true;

          case RecoveryAction.USE_CACHE:
            // Caller should implement cache retrieval
            return true;

          case RecoveryAction.DEGRADE:
            // Reduce functionality
            return true;

          case RecoveryAction.USER_INTERVENTION:
          case RecoveryAction.ABORT:
          default:
            return false;
        }
      } catch (err) {
        ErrorReporter.reportError(toAccuSceneError(err));
        return false;
      }
    },
    [error, recoveryAction]
  );

  return {
    recoveryAction,
    executeRecovery,
    canRecover: recoveryAction !== null && error?.recoverable === true,
  };
}

/**
 * Hook for error boundary integration
 */
export function useErrorBoundary() {
  const [error, setError] = useState<AccuSceneError | null>(null);

  const throwError = useCallback((err: unknown) => {
    const accuSceneError = toAccuSceneError(err);
    setError(accuSceneError);
    throw accuSceneError;
  }, []);

  const reset = useCallback(() => {
    setError(null);
  }, []);

  return {
    error,
    throwError,
    reset,
    hasError: error !== null,
  };
}

/**
 * Hook for tracking error metrics
 */
export function useErrorMetrics() {
  const [metrics, setMetrics] = useState(() => ErrorReporter.getErrorStats());

  const refresh = useCallback(() => {
    setMetrics(ErrorReporter.getErrorStats());
  }, []);

  useEffect(() => {
    // Refresh metrics every 5 seconds
    const interval = setInterval(refresh, 5000);
    return () => clearInterval(interval);
  }, [refresh]);

  return {
    metrics,
    refresh,
    recentErrors: ErrorReporter.getRecentErrors(10),
  };
}

// Helper functions

function isTransientError(code: ErrorCode): boolean {
  return [
    ErrorCode.NETWORK,
    ErrorCode.TIMEOUT,
    ErrorCode.UNAVAILABLE,
    ErrorCode.RATE_LIMIT,
    ErrorCode.EXTERNAL_SERVICE,
  ].includes(code);
}

function determineRecoveryAction(error: AccuSceneError): RecoveryAction | null {
  if (!error.recoverable) {
    return RecoveryAction.ABORT;
  }

  switch (error.code) {
    case ErrorCode.NETWORK:
    case ErrorCode.TIMEOUT:
    case ErrorCode.UNAVAILABLE:
      return RecoveryAction.RETRY;

    case ErrorCode.NOT_FOUND:
      return RecoveryAction.SKIP;

    case ErrorCode.CACHE:
      return RecoveryAction.USE_CACHE;

    case ErrorCode.VALIDATION:
    case ErrorCode.AUTHENTICATION:
    case ErrorCode.AUTHORIZATION:
      return RecoveryAction.USER_INTERVENTION;

    case ErrorCode.RATE_LIMIT:
      return RecoveryAction.RETRY;

    default:
      return RecoveryAction.FALLBACK;
  }
}
