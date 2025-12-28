/**
 * Main App Component - Root component with routing and providers
 */

import React, { useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuthStore } from './store/authStore';
import { CasesProvider } from './store/casesStore';
import { EditorProvider } from './store/editorStore';
import { UIProvider, useUIStore } from './store/uiStore';
import Header from './components/Layout/Header';
import Sidebar from './components/Layout/Sidebar';
import Footer from './components/Layout/Footer';
import Loading from './components/Common/Loading';
import LoginPage from './pages/LoginPage';
import DashboardPage from './pages/DashboardPage';
import CaseListPage from './pages/CaseListPage';
import CaseDetailPage from './pages/CaseDetailPage';
import EditorPage from './pages/EditorPage';
import ReportsPage from './pages/ReportsPage';
import SettingsPage from './pages/SettingsPage';
import './styles/global.css';
import './styles/variables.css';
import './styles/themes.css';

// Protected Route Component
const ProtectedRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { isAuthenticated, isLoading } = useAuthStore();

  if (isLoading) {
    return <Loading fullScreen message="Authenticating..." />;
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
};

// Layout Component
const Layout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { sidebarOpen } = useUIStore();

  return (
    <div className={`app-layout ${sidebarOpen ? 'sidebar-open' : 'sidebar-closed'}`}>
      <Header />
      <div className="app-body">
        <Sidebar />
        <main className="app-main">{children}</main>
      </div>
      <Footer />
    </div>
  );
};

// App Content Component (inside providers)
const AppContent: React.FC = () => {
  const { checkAuth } = useAuthStore();
  const { notifications, hideNotification, loading, loadingMessage } = useUIStore();

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  return (
    <>
      {/* Global Loading Overlay */}
      {loading && <Loading fullScreen message={loadingMessage} />}

      {/* Notifications */}
      <div className="notifications-container">
        {notifications.map((notification) => (
          <div
            key={notification.id}
            className={`notification notification-${notification.type}`}
            onClick={() => hideNotification(notification.id)}
          >
            <div className="notification-content">
              <span className="notification-message">{notification.message}</span>
              <button
                className="notification-close"
                onClick={() => hideNotification(notification.id)}
                aria-label="Close"
              >
                ×
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Routes */}
      <BrowserRouter>
        <Routes>
          {/* Public Routes */}
          <Route path="/login" element={<LoginPage />} />

          {/* Protected Routes */}
          <Route
            path="/dashboard"
            element={
              <ProtectedRoute>
                <Layout>
                  <DashboardPage />
                </Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/cases"
            element={
              <ProtectedRoute>
                <Layout>
                  <CaseListPage />
                </Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/cases/:id"
            element={
              <ProtectedRoute>
                <Layout>
                  <CaseDetailPage />
                </Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/editor/:id"
            element={
              <ProtectedRoute>
                <Layout>
                  <EditorPage />
                </Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/reports"
            element={
              <ProtectedRoute>
                <Layout>
                  <ReportsPage />
                </Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/settings"
            element={
              <ProtectedRoute>
                <Layout>
                  <SettingsPage />
                </Layout>
              </ProtectedRoute>
            }
          />

          {/* Redirect root to dashboard */}
          <Route path="/" element={<Navigate to="/dashboard" replace />} />

          {/* 404 - Redirect to dashboard */}
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Routes>
      </BrowserRouter>
    </>
  );
};

// Main App Component with Providers
const App: React.FC = () => {
  return (
    <UIProvider>
      <AuthProvider>
        <CasesProvider>
          <EditorProvider>
            <AppContent />
          </EditorProvider>
        </CasesProvider>
      </AuthProvider>
    </UIProvider>
  );
};

export default App;
