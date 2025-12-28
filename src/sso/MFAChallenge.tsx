/**
 * MFA Challenge Component
 * Handles TOTP and WebAuthn verification during login
 */

import React, { useState, useEffect, useRef } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useAuth } from './AuthProvider';
import { MFAMethod } from './types';

export interface MFAChallengeProps {
  onSuccess?: () => void;
  onCancel?: () => void;
  defaultRedirect?: string;
}

export const MFAChallenge: React.FC<MFAChallengeProps> = ({
  onSuccess,
  onCancel,
  defaultRedirect = '/dashboard',
}) => {
  const navigate = useNavigate();
  const location = useLocation();
  const { verifyTOTP, getMFAConfig } = useAuth();

  const [mfaToken, setMfaToken] = useState<string | null>(null);
  const [availableMethods, setAvailableMethods] = useState<MFAMethod[]>([]);
  const [selectedMethod, setSelectedMethod] = useState<MFAMethod | null>(null);
  const [totpCode, setTotpCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const codeInputRefs = useRef<(HTMLInputElement | null)[]>([]);

  useEffect(() => {
    // Get MFA token from location state
    const token = (location.state as any)?.mfa_token;
    if (!token) {
      navigate('/auth/login');
      return;
    }

    setMfaToken(token);
    loadMFAMethods();
  }, [location]);

  const loadMFAMethods = async () => {
    try {
      const config = await getMFAConfig();
      setAvailableMethods(config.enabled_methods);

      // Auto-select if only one method available
      if (config.enabled_methods.length === 1) {
        setSelectedMethod(config.enabled_methods[0]);
      }
    } catch (err) {
      console.error('Failed to load MFA methods:', err);
      setError('Failed to load authentication methods');
    }
  };

  const handleTOTPVerify = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!mfaToken || totpCode.length !== 6) {
      setError('Please enter a 6-digit code');
      return;
    }

    try {
      setIsLoading(true);
      setError(null);

      await verifyTOTP(totpCode, mfaToken);

      if (onSuccess) {
        onSuccess();
      } else {
        navigate(defaultRedirect);
      }
    } catch (err) {
      setError('Invalid verification code. Please try again.');
      setTotpCode('');
      codeInputRefs.current[0]?.focus();
    } finally {
      setIsLoading(false);
    }
  };

  const handleCodeChange = (index: number, value: string) => {
    if (!/^\d*$/.test(value)) return;

    const newCode = totpCode.split('');
    newCode[index] = value;
    setTotpCode(newCode.join(''));

    // Auto-focus next input
    if (value && index < 5) {
      codeInputRefs.current[index + 1]?.focus();
    }
  };

  const handleKeyDown = (index: number, e: React.KeyboardEvent) => {
    if (e.key === 'Backspace' && !totpCode[index] && index > 0) {
      codeInputRefs.current[index - 1]?.focus();
    }
  };

  const handleCancel = () => {
    if (onCancel) {
      onCancel();
    } else {
      navigate('/auth/login');
    }
  };

  if (!selectedMethod && availableMethods.length > 1) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8">
          <div className="text-center mb-6">
            <h2 className="text-2xl font-bold text-gray-900">
              Choose Verification Method
            </h2>
            <p className="text-gray-600 mt-2">
              Select how you would like to verify your identity
            </p>
          </div>

          <div className="space-y-3">
            {availableMethods.includes(MFAMethod.TOTP) && (
              <button
                onClick={() => setSelectedMethod(MFAMethod.TOTP)}
                className="w-full flex items-center gap-3 px-4 py-3 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
              >
                <svg
                  className="w-6 h-6 text-gray-700"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z"
                  />
                </svg>
                <div className="text-left">
                  <div className="font-medium text-gray-900">
                    Authenticator App
                  </div>
                  <div className="text-sm text-gray-600">
                    Use a 6-digit code from your authenticator app
                  </div>
                </div>
              </button>
            )}

            {availableMethods.includes(MFAMethod.WEBAUTHN) && (
              <button
                onClick={() => setSelectedMethod(MFAMethod.WEBAUTHN)}
                className="w-full flex items-center gap-3 px-4 py-3 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
              >
                <svg
                  className="w-6 h-6 text-gray-700"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"
                  />
                </svg>
                <div className="text-left">
                  <div className="font-medium text-gray-900">
                    Security Key
                  </div>
                  <div className="text-sm text-gray-600">
                    Use your hardware security key or biometrics
                  </div>
                </div>
              </button>
            )}
          </div>

          <button
            onClick={handleCancel}
            className="w-full mt-4 px-4 py-2 text-gray-700 hover:bg-gray-50 transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    );
  }

  if (selectedMethod === MFAMethod.TOTP) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8">
          <div className="text-center mb-6">
            <div className="inline-flex items-center justify-center w-12 h-12 bg-blue-100 rounded-full mb-4">
              <svg
                className="w-6 h-6 text-blue-600"
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
            <h2 className="text-2xl font-bold text-gray-900">
              Enter Verification Code
            </h2>
            <p className="text-gray-600 mt-2">
              Enter the 6-digit code from your authenticator app
            </p>
          </div>

          <form onSubmit={handleTOTPVerify}>
            <div className="flex justify-center gap-2 mb-6">
              {[0, 1, 2, 3, 4, 5].map((index) => (
                <input
                  key={index}
                  ref={(el) => (codeInputRefs.current[index] = el)}
                  type="text"
                  inputMode="numeric"
                  maxLength={1}
                  value={totpCode[index] || ''}
                  onChange={(e) => handleCodeChange(index, e.target.value)}
                  onKeyDown={(e) => handleKeyDown(index, e)}
                  className="w-12 h-14 text-center text-2xl font-bold border-2 border-gray-300 rounded-md focus:border-blue-500 focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  autoFocus={index === 0}
                />
              ))}
            </div>

            {error && (
              <div className="mb-4 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md text-sm">
                {error}
              </div>
            )}

            <div className="flex gap-3">
              <button
                type="button"
                onClick={handleCancel}
                className="flex-1 px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={isLoading || totpCode.length !== 6}
                className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? 'Verifying...' : 'Verify'}
              </button>
            </div>
          </form>
        </div>
      </div>
    );
  }

  if (selectedMethod === MFAMethod.WEBAUTHN) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8 text-center">
          <div className="inline-flex items-center justify-center w-12 h-12 bg-blue-100 rounded-full mb-4">
            <svg
              className="w-6 h-6 text-blue-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"
              />
            </svg>
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">
            Security Key Required
          </h2>
          <p className="text-gray-600 mb-6">
            Please insert your security key or use biometric authentication
          </p>
          <button
            onClick={handleCancel}
            className="w-full px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    );
  }

  return null;
};

export default MFAChallenge;
