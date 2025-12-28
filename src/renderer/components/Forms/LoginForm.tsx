/**
 * LoginForm Component - User login form
 */

import React, { useState, FormEvent } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../../hooks/useAuth';
import Input from '../Common/Input';
import Button from '../Common/Button';

export const LoginForm: React.FC = () => {
  const navigate = useNavigate();
  const { login, error, isLoading } = useAuth();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [errors, setErrors] = useState<{ email?: string; password?: string }>({});

  const validate = (): boolean => {
    const newErrors: { email?: string; password?: string } = {};

    if (!email) {
      newErrors.email = 'Email is required';
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      newErrors.email = 'Invalid email format';
    }

    if (!password) {
      newErrors.password = 'Password is required';
    } else if (password.length < 8) {
      newErrors.password = 'Password must be at least 8 characters';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (!validate()) {
      return;
    }

    const success = await login(email, password);
    if (success) {
      navigate('/dashboard');
    }
  };

  return (
    <form className="login-form" onSubmit={handleSubmit}>
      <div className="login-form-header">
        <h1>Welcome to AccuScene</h1>
        <p>Sign in to your account</p>
      </div>

      {error && <div className="login-form-error">{error}</div>}

      <Input
        type="email"
        label="Email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        error={errors.email}
        placeholder="Enter your email"
        fullWidth
        required
        autoComplete="email"
      />

      <Input
        type="password"
        label="Password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        error={errors.password}
        placeholder="Enter your password"
        fullWidth
        required
        autoComplete="current-password"
      />

      <div className="login-form-actions">
        <Button type="submit" variant="primary" fullWidth loading={isLoading}>
          Sign In
        </Button>
      </div>

      <div className="login-form-footer">
        <a href="#" className="login-form-link">
          Forgot password?
        </a>
      </div>
    </form>
  );
};

export default LoginForm;
