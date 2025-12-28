/**
 * Sidebar Component - Side navigation menu
 */

import React from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useUIStore } from '../../store/uiStore';

interface NavItem {
  path: string;
  label: string;
  icon: string;
}

const navItems: NavItem[] = [
  { path: '/dashboard', label: 'Dashboard', icon: '📊' },
  { path: '/cases', label: 'Cases', icon: '📁' },
  { path: '/reports', label: 'Reports', icon: '📄' },
  { path: '/settings', label: 'Settings', icon: '⚙️' },
];

export const Sidebar: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const { sidebarOpen } = useUIStore();

  if (!sidebarOpen) {
    return null;
  }

  return (
    <aside className="sidebar">
      <nav className="sidebar-nav">
        {navItems.map((item) => (
          <button
            key={item.path}
            className={`sidebar-nav-item ${
              location.pathname === item.path ? 'sidebar-nav-item-active' : ''
            }`}
            onClick={() => navigate(item.path)}
          >
            <span className="sidebar-nav-icon">{item.icon}</span>
            <span className="sidebar-nav-label">{item.label}</span>
          </button>
        ))}
      </nav>

      <div className="sidebar-footer">
        <div className="sidebar-version">
          <small>AccuScene v1.0.0</small>
        </div>
      </div>
    </aside>
  );
};

export default Sidebar;
