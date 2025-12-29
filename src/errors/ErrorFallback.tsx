/**
 * Error Fallback UI component for displaying errors to users
 */

import React from 'react';
import { AccuSceneError, ErrorSeverity } from './types';
import { ErrorReporter } from './ErrorReporter';

interface ErrorFallbackProps {
  error: AccuSceneError;
  resetError: () => void;
  isolate?: boolean;
}

/**
 * Default error fallback component
 */
export const ErrorFallback: React.FC<ErrorFallbackProps> = ({
  error,
  resetError,
  isolate = false,
}) => {
  const getSeverityColor = (severity: ErrorSeverity): string => {
    switch (severity) {
      case ErrorSeverity.CRITICAL:
        return '#dc2626';
      case ErrorSeverity.HIGH:
        return '#ea580c';
      case ErrorSeverity.MEDIUM:
        return '#f59e0b';
      case ErrorSeverity.LOW:
        return '#10b981';
      case ErrorSeverity.WARNING:
        return '#eab308';
      case ErrorSeverity.INFO:
        return '#3b82f6';
      default:
        return '#6b7280';
    }
  };

  const getSeverityIcon = (severity: ErrorSeverity): string => {
    switch (severity) {
      case ErrorSeverity.CRITICAL:
      case ErrorSeverity.HIGH:
        return '⚠️';
      case ErrorSeverity.MEDIUM:
        return '⚡';
      case ErrorSeverity.LOW:
      case ErrorSeverity.WARNING:
        return '⚠️';
      case ErrorSeverity.INFO:
        return 'ℹ️';
      default:
        return '❌';
    }
  };

  const getUserFriendlyMessage = (): string => {
    switch (error.severity) {
      case ErrorSeverity.CRITICAL:
        return `A critical error occurred: ${error.message}. Please contact support immediately.`;
      case ErrorSeverity.HIGH:
        return `An error occurred: ${error.message}. Please try again or contact support if the problem persists.`;
      case ErrorSeverity.MEDIUM:
        return `${error.message}. Please try again.`;
      case ErrorSeverity.LOW:
      case ErrorSeverity.WARNING:
        return error.message;
      case ErrorSeverity.INFO:
        return `Note: ${error.message}`;
      default:
        return error.message;
    }
  };

  const handleReportError = (): void => {
    ErrorReporter.reportToSupport(error);
    alert('Error report sent to support team. Thank you!');
  };

  const handleCopyError = (): void => {
    const errorReport = ErrorReporter.formatForUser(error);
    navigator.clipboard.writeText(errorReport);
    alert('Error details copied to clipboard');
  };

  const containerStyle: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: isolate ? '200px' : '100vh',
    padding: '2rem',
    backgroundColor: '#f9fafb',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  };

  const cardStyle: React.CSSProperties = {
    maxWidth: '600px',
    width: '100%',
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '2rem',
    boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
  };

  const headerStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    marginBottom: '1.5rem',
    borderBottom: `3px solid ${getSeverityColor(error.severity)}`,
    paddingBottom: '1rem',
  };

  const iconStyle: React.CSSProperties = {
    fontSize: '2rem',
  };

  const titleStyle: React.CSSProperties = {
    fontSize: '1.5rem',
    fontWeight: 'bold',
    color: '#111827',
    margin: 0,
  };

  const messageStyle: React.CSSProperties = {
    fontSize: '1rem',
    color: '#374151',
    lineHeight: '1.5',
    marginBottom: '1rem',
  };

  const detailsStyle: React.CSSProperties = {
    backgroundColor: '#f3f4f6',
    padding: '1rem',
    borderRadius: '4px',
    marginBottom: '1rem',
    fontSize: '0.875rem',
    fontFamily: 'monospace',
    color: '#1f2937',
    overflowX: 'auto',
  };

  const buttonContainerStyle: React.CSSProperties = {
    display: 'flex',
    gap: '0.75rem',
    flexWrap: 'wrap',
    marginTop: '1.5rem',
  };

  const buttonStyle = (variant: 'primary' | 'secondary' | 'outline'): React.CSSProperties => ({
    padding: '0.625rem 1.25rem',
    borderRadius: '4px',
    border: variant === 'outline' ? '1px solid #d1d5db' : 'none',
    backgroundColor:
      variant === 'primary'
        ? '#2563eb'
        : variant === 'secondary'
        ? '#6b7280'
        : 'white',
    color: variant === 'outline' ? '#374151' : 'white',
    fontSize: '0.875rem',
    fontWeight: '500',
    cursor: 'pointer',
    transition: 'all 0.2s',
  });

  const metadataStyle: React.CSSProperties = {
    fontSize: '0.75rem',
    color: '#6b7280',
    marginTop: '1rem',
    paddingTop: '1rem',
    borderTop: '1px solid #e5e7eb',
  };

  return (
    <div style={containerStyle}>
      <div style={cardStyle}>
        <div style={headerStyle}>
          <span style={iconStyle}>{getSeverityIcon(error.severity)}</span>
          <div>
            <h1 style={titleStyle}>
              {error.severity === ErrorSeverity.CRITICAL ? 'Critical Error' : 'Error Occurred'}
            </h1>
            <div style={{ fontSize: '0.875rem', color: '#6b7280' }}>
              {error.code} • {error.severity}
            </div>
          </div>
        </div>

        <div style={messageStyle}>{getUserFriendlyMessage()}</div>

        {error.details && (
          <details>
            <summary style={{ cursor: 'pointer', marginBottom: '0.5rem', color: '#4b5563' }}>
              Technical Details
            </summary>
            <div style={detailsStyle}>{error.details}</div>
          </details>
        )}

        {error.context && (
          <div style={{ marginTop: '1rem' }}>
            <strong style={{ color: '#4b5563' }}>Context:</strong>
            <div style={{ ...detailsStyle, marginTop: '0.5rem' }}>{error.context.message}</div>
          </div>
        )}

        <div style={buttonContainerStyle}>
          {error.recoverable && (
            <button
              onClick={resetError}
              style={buttonStyle('primary')}
              onMouseOver={(e) => (e.currentTarget.style.backgroundColor = '#1d4ed8')}
              onMouseOut={(e) => (e.currentTarget.style.backgroundColor = '#2563eb')}
            >
              Try Again
            </button>
          )}

          <button
            onClick={handleCopyError}
            style={buttonStyle('outline')}
            onMouseOver={(e) => (e.currentTarget.style.backgroundColor = '#f9fafb')}
            onMouseOut={(e) => (e.currentTarget.style.backgroundColor = 'white')}
          >
            Copy Error Details
          </button>

          {(error.severity === ErrorSeverity.CRITICAL || error.severity === ErrorSeverity.HIGH) && (
            <button
              onClick={handleReportError}
              style={buttonStyle('secondary')}
              onMouseOver={(e) => (e.currentTarget.style.backgroundColor = '#4b5563')}
              onMouseOut={(e) => (e.currentTarget.style.backgroundColor = '#6b7280')}
            >
              Report to Support
            </button>
          )}

          {!isolate && (
            <button
              onClick={() => window.location.reload()}
              style={buttonStyle('outline')}
              onMouseOver={(e) => (e.currentTarget.style.backgroundColor = '#f9fafb')}
              onMouseOut={(e) => (e.currentTarget.style.backgroundColor = 'white')}
            >
              Reload Page
            </button>
          )}
        </div>

        <div style={metadataStyle}>
          <div>Error ID: {error.id}</div>
          <div>Timestamp: {error.timestamp.toLocaleString()}</div>
          {error.location && <div>Location: {error.location}</div>}
        </div>
      </div>
    </div>
  );
};

/**
 * Minimal error fallback for inline errors
 */
export const InlineErrorFallback: React.FC<{ error: AccuSceneError; resetError?: () => void }> = ({
  error,
  resetError,
}) => {
  const style: React.CSSProperties = {
    padding: '1rem',
    backgroundColor: '#fef2f2',
    border: '1px solid #fecaca',
    borderRadius: '4px',
    color: '#991b1b',
    fontSize: '0.875rem',
  };

  return (
    <div style={style}>
      <strong>{error.code}:</strong> {error.message}
      {resetError && (
        <button
          onClick={resetError}
          style={{
            marginLeft: '1rem',
            padding: '0.25rem 0.5rem',
            backgroundColor: '#dc2626',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.75rem',
          }}
        >
          Retry
        </button>
      )}
    </div>
  );
};
