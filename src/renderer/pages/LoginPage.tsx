/**
 * LoginPage - User authentication page
 */

import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import LoginForm from '../components/Forms/LoginForm';

export const LoginPage: React.FC = () => {
  const navigate = useNavigate();
  const { isAuthenticated } = useAuth();

  useEffect(() => {
    if (isAuthenticated) {
      navigate('/dashboard');
    }
  }, [isAuthenticated, navigate]);

  return (
    <div className="login-page">
      <div className="login-page-container">
        <div className="login-page-left">
          <div className="login-page-branding">
            <h1 className="login-page-brand-title">AccuScene Enterprise</h1>
            <p className="login-page-brand-subtitle">
              Professional Accident Recreation Platform
            </p>
            <div className="login-page-features">
              <div className="login-page-feature">
                <span className="login-page-feature-icon">✓</span>
                <span>Advanced Physics Simulation</span>
              </div>
              <div className="login-page-feature">
                <span className="login-page-feature-icon">✓</span>
                <span>Detailed Accident Analysis</span>
              </div>
              <div className="login-page-feature">
                <span className="login-page-feature-icon">✓</span>
                <span>Comprehensive Reporting</span>
              </div>
              <div className="login-page-feature">
                <span className="login-page-feature-icon">✓</span>
                <span>Evidence Management</span>
              </div>
            </div>
          </div>
        </div>

        <div className="login-page-right">
          <LoginForm />
        </div>
      </div>
    </div>
  );
};

export default LoginPage;
