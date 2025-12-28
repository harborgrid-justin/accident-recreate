/**
 * Header Component - Top navigation bar
 */

import React from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../../hooks/useAuth';
import { useUIStore } from '../../store/uiStore';

export const Header: React.FC = () => {
  const navigate = useNavigate();
  const { user, logout } = useAuth();
  const { theme, toggleTheme, toggleSidebar } = useUIStore();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <header className="header">
      <div className="header-left">
        <button className="header-menu-btn" onClick={toggleSidebar} aria-label="Toggle sidebar">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z" />
          </svg>
        </button>
        <div className="header-logo" onClick={() => navigate('/dashboard')}>
          <span className="header-logo-text">AccuScene Enterprise</span>
        </div>
      </div>

      <div className="header-center">
        <nav className="header-nav">
          <button className="header-nav-item" onClick={() => navigate('/dashboard')}>
            Dashboard
          </button>
          <button className="header-nav-item" onClick={() => navigate('/cases')}>
            Cases
          </button>
          <button className="header-nav-item" onClick={() => navigate('/reports')}>
            Reports
          </button>
        </nav>
      </div>

      <div className="header-right">
        <button
          className="header-icon-btn"
          onClick={toggleTheme}
          aria-label="Toggle theme"
          title={`Switch to ${theme === 'light' ? 'dark' : 'light'} mode`}
        >
          {theme === 'light' ? (
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
            </svg>
          ) : (
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <circle cx="12" cy="12" r="5" />
              <line x1="12" y1="1" x2="12" y2="3" />
              <line x1="12" y1="21" x2="12" y2="23" />
              <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
              <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
              <line x1="1" y1="12" x2="3" y2="12" />
              <line x1="21" y1="12" x2="23" y2="12" />
              <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
              <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
            </svg>
          )}
        </button>

        {user && (
          <div className="header-user">
            <div className="header-user-info">
              <span className="header-user-name">{user.fullName || user.email}</span>
              <span className="header-user-role">{user.role}</span>
            </div>
            <div className="header-user-menu">
              <button className="header-user-menu-item" onClick={() => navigate('/settings')}>
                Settings
              </button>
              <button className="header-user-menu-item" onClick={handleLogout}>
                Logout
              </button>
            </div>
          </div>
        )}
      </div>
    </header>
  );
};

export default Header;
