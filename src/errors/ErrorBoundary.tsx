/**
 * React Error Boundary for catching and handling errors in component tree
 */

import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AccuSceneError, ErrorCode, toAccuSceneError } from './types';
import { ErrorReporter } from './ErrorReporter';
import { ErrorFallback } from './ErrorFallback';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: (error: AccuSceneError, reset: () => void) => ReactNode;
  onError?: (error: AccuSceneError, errorInfo: ErrorInfo) => void;
  resetKeys?: Array<string | number>;
  isolate?: boolean;
}

interface ErrorBoundaryState {
  error: AccuSceneError | null;
  errorInfo: ErrorInfo | null;
}

/**
 * ErrorBoundary component for handling React errors
 *
 * @example
 * ```tsx
 * <ErrorBoundary onError={handleError}>
 *   <MyComponent />
 * </ErrorBoundary>
 * ```
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    // Convert error to AccuSceneError
    const accuSceneError = toAccuSceneError(error);

    return {
      error: accuSceneError,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    const accuSceneError = toAccuSceneError(error).withContext(
      'Error caught by ErrorBoundary',
      {
        componentStack: errorInfo.componentStack,
      }
    );

    // Report error
    ErrorReporter.reportError(accuSceneError, {
      componentStack: errorInfo.componentStack,
    });

    // Call custom error handler if provided
    if (this.props.onError) {
      this.props.onError(accuSceneError, errorInfo);
    }

    this.setState({
      error: accuSceneError,
      errorInfo,
    });
  }

  componentDidUpdate(prevProps: ErrorBoundaryProps): void {
    const { error } = this.state;
    const { resetKeys } = this.props;

    if (error && resetKeys) {
      const hasResetKeyChanged = resetKeys.some(
        (key, index) => prevProps.resetKeys?.[index] !== key
      );

      if (hasResetKeyChanged) {
        this.reset();
      }
    }
  }

  reset = (): void => {
    this.setState({
      error: null,
      errorInfo: null,
    });
  };

  render(): ReactNode {
    const { error } = this.state;
    const { children, fallback, isolate } = this.props;

    if (error) {
      if (fallback) {
        return fallback(error, this.reset);
      }

      return <ErrorFallback error={error} resetError={this.reset} isolate={isolate} />;
    }

    return children;
  }
}

/**
 * Hook-based error boundary wrapper
 */
interface WithErrorBoundaryOptions {
  fallback?: (error: AccuSceneError, reset: () => void) => ReactNode;
  onError?: (error: AccuSceneError, errorInfo: ErrorInfo) => void;
  resetKeys?: Array<string | number>;
}

export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  options?: WithErrorBoundaryOptions
): React.FC<P> {
  const WrappedComponent: React.FC<P> = (props) => (
    <ErrorBoundary {...options}>
      <Component {...props} />
    </ErrorBoundary>
  );

  WrappedComponent.displayName = `withErrorBoundary(${
    Component.displayName || Component.name || 'Component'
  })`;

  return WrappedComponent;
}

/**
 * Async error boundary for handling async errors
 * Since error boundaries don't catch async errors, we need a wrapper
 */
export class AsyncErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      error: null,
      errorInfo: null,
    };

    // Set up global error handler for uncaught promise rejections
    if (typeof window !== 'undefined') {
      window.addEventListener('unhandledrejection', this.handlePromiseRejection);
    }
  }

  componentWillUnmount(): void {
    if (typeof window !== 'undefined') {
      window.removeEventListener('unhandledrejection', this.handlePromiseRejection);
    }
  }

  handlePromiseRejection = (event: PromiseRejectionEvent): void => {
    event.preventDefault();

    const error = toAccuSceneError(event.reason).withContext('Unhandled promise rejection');

    ErrorReporter.reportError(error);

    this.setState({
      error,
      errorInfo: {
        componentStack: '',
      },
    });

    if (this.props.onError) {
      this.props.onError(error, {
        componentStack: '',
      });
    }
  };

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return {
      error: toAccuSceneError(error),
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    const accuSceneError = toAccuSceneError(error).withContext(
      'Error caught by AsyncErrorBoundary'
    );

    ErrorReporter.reportError(accuSceneError, {
      componentStack: errorInfo.componentStack,
    });

    if (this.props.onError) {
      this.props.onError(accuSceneError, errorInfo);
    }

    this.setState({
      error: accuSceneError,
      errorInfo,
    });
  }

  reset = (): void => {
    this.setState({
      error: null,
      errorInfo: null,
    });
  };

  render(): ReactNode {
    const { error } = this.state;
    const { children, fallback } = this.props;

    if (error) {
      if (fallback) {
        return fallback(error, this.reset);
      }

      return <ErrorFallback error={error} resetError={this.reset} />;
    }

    return children;
  }
}
