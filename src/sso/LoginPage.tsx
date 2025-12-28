/**
 * SSO Login Page Component
 * Displays available SSO providers and handles login initiation
 */

import React, { useState, useEffect } from 'react';
import { useAuth } from './AuthProvider';
import { SSOProvider, SSOProviderType } from './types';
import AuthService from './services/AuthService';

export interface LoginPageProps {
  onSuccess?: () => void;
  redirectTo?: string;
  providers?: SSOProvider[];
  showLDAP?: boolean;
}

export const LoginPage: React.FC<LoginPageProps> = ({
  onSuccess,
  redirectTo = '/',
  providers: providedProviders,
  showLDAP = true,
}) => {
  const { login, loginWithLDAP, isLoading, error } = useAuth();
  const [providers, setProviders] = useState<SSOProvider[]>(providedProviders || []);
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
  const [ldapUsername, setLdapUsername] = useState('');
  const [ldapPassword, setLdapPassword] = useState('');
  const [showLdapForm, setShowLdapForm] = useState(false);

  useEffect(() => {
    if (!providedProviders) {
      loadProviders();
    }
  }, [providedProviders]);

  const loadProviders = async () => {
    try {
      // In production, fetch from API
      const mockProviders: SSOProvider[] = [
        {
          name: 'google',
          type: SSOProviderType.OIDC,
          display_name: 'Google',
          icon_url: 'https://www.google.com/favicon.ico',
          enabled: true,
        },
        {
          name: 'okta',
          type: SSOProviderType.OIDC,
          display_name: 'Okta',
          icon_url: '/icons/okta.svg',
          enabled: true,
        },
        {
          name: 'azure-ad',
          type: SSOProviderType.OIDC,
          display_name: 'Microsoft Azure AD',
          icon_url: '/icons/microsoft.svg',
          enabled: true,
        },
        {
          name: 'ldap',
          type: SSOProviderType.LDAP,
          display_name: 'LDAP / Active Directory',
          icon_url: '/icons/ldap.svg',
          enabled: showLDAP,
        },
      ];
      setProviders(mockProviders.filter((p) => p.enabled));
    } catch (err) {
      console.error('Failed to load providers:', err);
    }
  };

  const handleProviderLogin = async (providerName: string) => {
    setSelectedProvider(providerName);
    const provider = providers.find((p) => p.name === providerName);

    if (provider?.type === SSOProviderType.LDAP) {
      setShowLdapForm(true);
      return;
    }

    try {
      await login(providerName, window.location.origin + '/auth/callback?redirect=' + redirectTo);
    } catch (err) {
      console.error('Login failed:', err);
      setSelectedProvider(null);
    }
  };

  const handleLdapSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!selectedProvider) return;

    try {
      await loginWithLDAP(selectedProvider, ldapUsername, ldapPassword);
      if (onSuccess) {
        onSuccess();
      } else {
        window.location.href = redirectTo;
      }
    } catch (err) {
      console.error('LDAP login failed:', err);
    }
  };

  const getProviderIcon = (provider: SSOProvider) => {
    if (provider.icon_url) {
      return (
        <img
          src={provider.icon_url}
          alt={provider.display_name}
          className="w-6 h-6"
        />
      );
    }

    // Default icon based on type
    return (
      <div className="w-6 h-6 bg-blue-500 rounded-full flex items-center justify-center text-white text-xs font-bold">
        {provider.display_name.charAt(0)}
      </div>
    );
  };

  if (showLdapForm) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8">
          <div className="text-center mb-6">
            <h1 className="text-3xl font-bold text-gray-900">
              AccuScene Enterprise
            </h1>
            <p className="text-gray-600 mt-2">LDAP Authentication</p>
          </div>

          <form onSubmit={handleLdapSubmit} className="space-y-4">
            <div>
              <label
                htmlFor="username"
                className="block text-sm font-medium text-gray-700 mb-1"
              >
                Username
              </label>
              <input
                id="username"
                type="text"
                value={ldapUsername}
                onChange={(e) => setLdapUsername(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="Enter your username"
                required
                autoFocus
              />
            </div>

            <div>
              <label
                htmlFor="password"
                className="block text-sm font-medium text-gray-700 mb-1"
              >
                Password
              </label>
              <input
                id="password"
                type="password"
                value={ldapPassword}
                onChange={(e) => setLdapPassword(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="Enter your password"
                required
              />
            </div>

            {error && (
              <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md text-sm">
                {error}
              </div>
            )}

            <div className="flex gap-3">
              <button
                type="button"
                onClick={() => {
                  setShowLdapForm(false);
                  setSelectedProvider(null);
                  setLdapUsername('');
                  setLdapPassword('');
                }}
                className="flex-1 px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 transition-colors"
              >
                Back
              </button>
              <button
                type="submit"
                disabled={isLoading}
                className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? 'Signing in...' : 'Sign In'}
              </button>
            </div>
          </form>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100">
      <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900">
            AccuScene Enterprise
          </h1>
          <p className="text-gray-600 mt-2">Sign in to continue</p>
        </div>

        {error && (
          <div className="mb-6 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md text-sm">
            {error}
          </div>
        )}

        <div className="space-y-3">
          {providers.map((provider) => (
            <button
              key={provider.name}
              onClick={() => handleProviderLogin(provider.name)}
              disabled={isLoading && selectedProvider === provider.name}
              className="w-full flex items-center justify-center gap-3 px-4 py-3 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {getProviderIcon(provider)}
              <span className="text-gray-700 font-medium">
                {isLoading && selectedProvider === provider.name
                  ? 'Connecting...'
                  : `Continue with ${provider.display_name}`}
              </span>
            </button>
          ))}
        </div>

        <div className="mt-8 text-center text-sm text-gray-500">
          <p>
            By signing in, you agree to our Terms of Service and Privacy
            Policy.
          </p>
        </div>
      </div>
    </div>
  );
};

export default LoginPage;
