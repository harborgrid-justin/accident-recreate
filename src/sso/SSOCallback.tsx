/**
 * SSO Callback Handler Component
 * Handles OAuth/OIDC callback after successful authentication
 */

import React, { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAuth } from './AuthProvider';
import AuthService from './services/AuthService';
import { SSOError } from './types';

export interface SSOCallbackProps {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
  defaultRedirect?: string;
}

export const SSOCallback: React.FC<SSOCallbackProps> = ({
  onSuccess,
  onError,
  defaultRedirect = '/dashboard',
}) => {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { refreshSession } = useAuth();
  const [status, setStatus] = useState<'processing' | 'success' | 'error' | 'mfa_required'>(
    'processing'
  );
  const [error, setError] = useState<string | null>(null);
  const [mfaToken, setMfaToken] = useState<string | null>(null);

  useEffect(() => {
    handleCallback();
  }, []);

  const handleCallback = async () => {
    try {
      // Extract callback parameters
      const code = searchParams.get('code');
      const state = searchParams.get('state');
      const error = searchParams.get('error');
      const errorDescription = searchParams.get('error_description');
      const redirect = searchParams.get('redirect') || defaultRedirect;

      // Check for OAuth errors
      if (error) {
        throw new SSOError(
          errorDescription || error,
          'AUTHENTICATION_FAILED'
        );
      }

      // Validate required parameters
      if (!code || !state) {
        throw new SSOError(
          'Missing required callback parameters',
          'AUTHENTICATION_FAILED'
        );
      }

      // Handle the callback
      const result = await AuthService.handleCallback({
        code,
        state,
      });

      // Check if MFA is required
      if (result.mfa_required) {
        setStatus('mfa_required');
        setMfaToken(result.mfa_token || null);
        // Redirect to MFA challenge page
        navigate('/auth/mfa', {
          state: { mfa_token: result.mfa_token },
        });
        return;
      }

      // Success!
      setStatus('success');

      if (onSuccess) {
        onSuccess();
      } else {
        // Redirect to destination
        setTimeout(() => {
          navigate(redirect);
        }, 1000);
      }
    } catch (err) {
      console.error('Callback error:', err);
      const errorMessage =
        err instanceof SSOError
          ? err.message
          : err instanceof Error
          ? err.message
          : 'Authentication failed';

      setError(errorMessage);
      setStatus('error');

      if (onError) {
        onError(err instanceof Error ? err : new Error(errorMessage));
      }
    }
  };

  const handleRetry = () => {
    navigate('/auth/login');
  };

  if (status === 'processing') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8 text-center">
          <div className="mb-4">
            <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">
            Completing sign-in...
          </h2>
          <p className="text-gray-600">
            Please wait while we verify your credentials.
          </p>
        </div>
      </div>
    );
  }

  if (status === 'success') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8 text-center">
          <div className="mb-4">
            <div className="inline-flex items-center justify-center w-12 h-12 bg-green-100 rounded-full">
              <svg
                className="w-6 h-6 text-green-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M5 13l4 4L19 7"
                />
              </svg>
            </div>
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">
            Sign-in successful!
          </h2>
          <p className="text-gray-600">Redirecting you now...</p>
        </div>
      </div>
    );
  }

  if (status === 'mfa_required') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8 text-center">
          <div className="mb-4">
            <div className="inline-flex items-center justify-center w-12 h-12 bg-yellow-100 rounded-full">
              <svg
                className="w-6 h-6 text-yellow-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                />
              </svg>
            </div>
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">
            Multi-Factor Authentication Required
          </h2>
          <p className="text-gray-600">Redirecting to verification...</p>
        </div>
      </div>
    );
  }

  // Error state
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100">
      <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8">
        <div className="text-center mb-6">
          <div className="inline-flex items-center justify-center w-12 h-12 bg-red-100 rounded-full mb-4">
            <svg
              className="w-6 h-6 text-red-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">
            Sign-in Failed
          </h2>
          <p className="text-gray-600">{error || 'An error occurred during authentication'}</p>
        </div>

        <button
          onClick={handleRetry}
          className="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
        >
          Try Again
        </button>
      </div>
    </div>
  );
};

export default SSOCallback;
