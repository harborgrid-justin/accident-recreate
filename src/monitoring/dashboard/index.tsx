/**
 * AccuScene Enterprise v0.2.0
 * Monitoring Dashboard Main Component
 *
 * Real-time performance monitoring dashboard
 */

import React, { useState, useEffect } from 'react';
import { Overview } from './Overview';
import { Metrics } from './Metrics';
import { Traces } from './Traces';
import { Alerts } from './Alerts';
import { Performance } from './Performance';
import { globalCollector } from '../core/collector';
import { globalAggregator } from '../core/aggregator';
import { AlertRulesEngine } from '../alerting/rules';

interface DashboardProps {
  refreshInterval?: number;
}

export const MonitoringDashboard: React.FC<DashboardProps> = ({ refreshInterval = 5000 }) => {
  const [activeTab, setActiveTab] = useState<'overview' | 'metrics' | 'traces' | 'alerts' | 'performance'>('overview');
  const [isCollecting, setIsCollecting] = useState(false);

  useEffect(() => {
    // Start collection when dashboard mounts
    globalCollector.startCollection();
    setIsCollecting(true);

    return () => {
      // Stop collection when dashboard unmounts
      globalCollector.stopCollection();
      setIsCollecting(false);
    };
  }, []);

  const tabs = [
    { id: 'overview' as const, label: 'Overview', icon: '📊' },
    { id: 'metrics' as const, label: 'Metrics', icon: '📈' },
    { id: 'traces' as const, label: 'Traces', icon: '🔍' },
    { id: 'alerts' as const, label: 'Alerts', icon: '🚨' },
    { id: 'performance' as const, label: 'Performance', icon: '⚡' }
  ];

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h1 style={styles.title}>AccuScene Performance Monitoring</h1>
        <div style={styles.status}>
          <span style={{
            ...styles.statusDot,
            backgroundColor: isCollecting ? '#4caf50' : '#f44336'
          }} />
          <span style={styles.statusText}>
            {isCollecting ? 'Collecting' : 'Stopped'}
          </span>
        </div>
      </div>

      <div style={styles.tabs}>
        {tabs.map(tab => (
          <button
            key={tab.id}
            style={{
              ...styles.tab,
              ...(activeTab === tab.id ? styles.activeTab : {})
            }}
            onClick={() => setActiveTab(tab.id)}
          >
            <span style={styles.tabIcon}>{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      <div style={styles.content}>
        {activeTab === 'overview' && <Overview refreshInterval={refreshInterval} />}
        {activeTab === 'metrics' && <Metrics refreshInterval={refreshInterval} />}
        {activeTab === 'traces' && <Traces refreshInterval={refreshInterval} />}
        {activeTab === 'alerts' && <Alerts refreshInterval={refreshInterval} />}
        {activeTab === 'performance' && <Performance refreshInterval={refreshInterval} />}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    width: '100%',
    height: '100vh',
    backgroundColor: '#f5f5f5',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    display: 'flex',
    flexDirection: 'column'
  },
  header: {
    backgroundColor: '#1976d2',
    color: 'white',
    padding: '20px 30px',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  title: {
    margin: 0,
    fontSize: '24px',
    fontWeight: 500
  },
  status: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px'
  },
  statusDot: {
    width: '10px',
    height: '10px',
    borderRadius: '50%',
    animation: 'pulse 2s infinite'
  },
  statusText: {
    fontSize: '14px',
    fontWeight: 500
  },
  tabs: {
    backgroundColor: 'white',
    borderBottom: '1px solid #e0e0e0',
    display: 'flex',
    gap: '4px',
    padding: '0 20px'
  },
  tab: {
    padding: '16px 24px',
    border: 'none',
    backgroundColor: 'transparent',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 500,
    color: '#666',
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    borderBottom: '3px solid transparent',
    transition: 'all 0.3s ease'
  },
  activeTab: {
    color: '#1976d2',
    borderBottomColor: '#1976d2'
  },
  tabIcon: {
    fontSize: '18px'
  },
  content: {
    flex: 1,
    overflow: 'auto',
    padding: '20px'
  }
};

export default MonitoringDashboard;
