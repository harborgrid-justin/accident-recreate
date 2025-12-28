/**
 * Protected Route Component
 * Wraps routes that require authentication
 */

import React, { useEffect } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { useAuth } from '../AuthProvider';
import { ProtectedRouteProps } from '../types';

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({
  children,
  requireAuth = true,
  requireMFA = false,
  redirectTo = '/auth/login',
  fallback,
}) => {
  const { isAuthenticated, isLoading, session } = useAuth();
  const location = useLocation();

  // Show loading state
  if (isLoading) {
    if (fallback) {
      return <>{fallback}</>;
    }

    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>
          <p className="text-gray-600">Loading...</p>
        </div>
      </div>
    );
  }

  // Check authentication requirement
  if (requireAuth && !isAuthenticated) {
    return <Navigate to={redirectTo} state={{ from: location }} replace />;
  }

  // Check MFA requirement
  if (requireMFA && session && !session.mfa_verified) {
    return <Navigate to="/auth/mfa" state={{ from: location }} replace />;
  }

  return <>{children}</>;
};

export default ProtectedRoute;
