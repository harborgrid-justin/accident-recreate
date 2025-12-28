/**
 * DashboardPage - Main dashboard with overview and statistics
 */

import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import { api, CaseData } from '../services/api';
import StatCard from '../components/Cards/StatCard';
import CaseCard from '../components/Cards/CaseCard';
import Loading from '../components/Common/Loading';

interface DashboardStats {
  totalCases: number;
  activeCases: number;
  completedCases: number;
  recentCases: CaseData[];
}

export const DashboardPage: React.FC = () => {
  const navigate = useNavigate();
  const { user } = useAuth();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadDashboardData();
  }, []);

  const loadDashboardData = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await api.getStats();
      if (response.success && response.data) {
        setStats(response.data);
      } else {
        setError(response.error || 'Failed to load dashboard data');
      }
    } catch (err) {
      setError('Failed to load dashboard data');
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="page">
        <Loading fullScreen message="Loading dashboard..." />
      </div>
    );
  }

  if (error) {
    return (
      <div className="page">
        <div className="page-error">
          <h2>Error Loading Dashboard</h2>
          <p>{error}</p>
          <button className="btn btn-primary" onClick={loadDashboardData}>
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="page dashboard-page">
      <div className="page-header">
        <div className="page-title-section">
          <h1 className="page-title">Dashboard</h1>
          <p className="page-subtitle">Welcome back, {user?.fullName || user?.email}</p>
        </div>
        <div className="page-actions">
          <button className="btn btn-primary" onClick={() => navigate('/cases/new')}>
            + New Case
          </button>
        </div>
      </div>

      <div className="dashboard-stats">
        <StatCard
          title="Total Cases"
          value={stats?.totalCases || 0}
          icon="📊"
          color="blue"
          onClick={() => navigate('/cases')}
        />
        <StatCard
          title="Active Cases"
          value={stats?.activeCases || 0}
          icon="🔄"
          color="orange"
          onClick={() => navigate('/cases?status=active')}
        />
        <StatCard
          title="Completed Cases"
          value={stats?.completedCases || 0}
          icon="✓"
          color="green"
          onClick={() => navigate('/cases?status=completed')}
        />
        <StatCard
          title="Pending Review"
          value={stats ? stats.totalCases - stats.activeCases - stats.completedCases : 0}
          icon="⏳"
          color="purple"
          onClick={() => navigate('/cases?status=under_review')}
        />
      </div>

      <div className="dashboard-recent">
        <div className="dashboard-section-header">
          <h2>Recent Cases</h2>
          <button className="btn btn-outline" onClick={() => navigate('/cases')}>
            View All
          </button>
        </div>

        {stats?.recentCases && stats.recentCases.length > 0 ? (
          <div className="dashboard-cases-grid">
            {stats.recentCases.map((caseData) => (
              <CaseCard key={caseData.id} caseData={caseData} />
            ))}
          </div>
        ) : (
          <div className="dashboard-empty">
            <p>No recent cases</p>
            <button className="btn btn-primary" onClick={() => navigate('/cases/new')}>
              Create Your First Case
            </button>
          </div>
        )}
      </div>

      <div className="dashboard-quick-actions">
        <h2>Quick Actions</h2>
        <div className="dashboard-actions-grid">
          <div className="dashboard-action-card" onClick={() => navigate('/cases/new')}>
            <div className="dashboard-action-icon">+</div>
            <div className="dashboard-action-label">New Case</div>
          </div>
          <div className="dashboard-action-card" onClick={() => navigate('/cases')}>
            <div className="dashboard-action-icon">📁</div>
            <div className="dashboard-action-label">Browse Cases</div>
          </div>
          <div className="dashboard-action-card" onClick={() => navigate('/reports')}>
            <div className="dashboard-action-icon">📄</div>
            <div className="dashboard-action-label">Generate Report</div>
          </div>
          <div className="dashboard-action-card" onClick={() => navigate('/settings')}>
            <div className="dashboard-action-icon">⚙️</div>
            <div className="dashboard-action-label">Settings</div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DashboardPage;
