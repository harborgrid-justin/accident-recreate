/**
 * AccuScene Enterprise v0.2.0
 * Alerts Dashboard Panel
 */

import React, { useState, useEffect } from 'react';
import { AlertRulesEngine } from '../alerting/rules';
import { Alert, AlertState, AlertSeverity } from '../types';

interface AlertsProps {
  refreshInterval: number;
}

export const Alerts: React.FC<AlertsProps> = ({ refreshInterval }) => {
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [filter, setFilter] = useState<'all' | 'firing' | 'resolved'>('all');

  useEffect(() => {
    const updateAlerts = () => {
      // In a real implementation, this would fetch from AlertRulesEngine
      // For now, we'll show an empty state
      setAlerts([]);
    };

    updateAlerts();
    const interval = setInterval(updateAlerts, refreshInterval);

    return () => clearInterval(interval);
  }, [refreshInterval]);

  const getSeverityColor = (severity: AlertSeverity): string => {
    switch (severity) {
      case AlertSeverity.INFO:
        return '#2196f3';
      case AlertSeverity.WARNING:
        return '#ff9800';
      case AlertSeverity.ERROR:
        return '#f44336';
      case AlertSeverity.CRITICAL:
        return '#d32f2f';
      default:
        return '#9e9e9e';
    }
  };

  const getStateColor = (state: AlertState): string => {
    switch (state) {
      case AlertState.FIRING:
        return '#f44336';
      case AlertState.PENDING:
        return '#ff9800';
      case AlertState.RESOLVED:
        return '#4caf50';
      default:
        return '#9e9e9e';
    }
  };

  const filteredAlerts = alerts.filter(alert => {
    if (filter === 'all') return true;
    if (filter === 'firing') return alert.state === AlertState.FIRING;
    if (filter === 'resolved') return alert.state === AlertState.RESOLVED;
    return true;
  });

  const firingCount = alerts.filter(a => a.state === AlertState.FIRING).length;
  const pendingCount = alerts.filter(a => a.state === AlertState.PENDING).length;
  const resolvedCount = alerts.filter(a => a.state === AlertState.RESOLVED).length;

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={styles.title}>Alerts</h2>
        <div style={styles.filters}>
          <button
            style={{
              ...styles.filterButton,
              ...(filter === 'all' ? styles.activeFilter : {})
            }}
            onClick={() => setFilter('all')}
          >
            All ({alerts.length})
          </button>
          <button
            style={{
              ...styles.filterButton,
              ...(filter === 'firing' ? styles.activeFilter : {})
            }}
            onClick={() => setFilter('firing')}
          >
            Firing ({firingCount})
          </button>
          <button
            style={{
              ...styles.filterButton,
              ...(filter === 'resolved' ? styles.activeFilter : {})
            }}
            onClick={() => setFilter('resolved')}
          >
            Resolved ({resolvedCount})
          </button>
        </div>
      </div>

      <div style={styles.stats}>
        <div style={styles.statCard}>
          <div style={styles.statIcon}>🔥</div>
          <div style={styles.statContent}>
            <div style={styles.statValue}>{firingCount}</div>
            <div style={styles.statLabel}>Firing</div>
          </div>
        </div>
        <div style={styles.statCard}>
          <div style={styles.statIcon}>⏳</div>
          <div style={styles.statContent}>
            <div style={styles.statValue}>{pendingCount}</div>
            <div style={styles.statLabel}>Pending</div>
          </div>
        </div>
        <div style={styles.statCard}>
          <div style={styles.statIcon}>✓</div>
          <div style={styles.statContent}>
            <div style={styles.statValue}>{resolvedCount}</div>
            <div style={styles.statLabel}>Resolved</div>
          </div>
        </div>
      </div>

      <div style={styles.alertsList}>
        {filteredAlerts.length > 0 ? (
          filteredAlerts.map(alert => (
            <div key={alert.id} style={styles.alertItem}>
              <div style={styles.alertHeader}>
                <div style={styles.alertTitleRow}>
                  <span style={{
                    ...styles.severityBadge,
                    backgroundColor: getSeverityColor(alert.rule.severity)
                  }}>
                    {alert.rule.severity}
                  </span>
                  <span style={styles.alertName}>{alert.rule.name}</span>
                </div>
                <span style={{
                  ...styles.stateBadge,
                  backgroundColor: getStateColor(alert.state)
                }}>
                  {alert.state}
                </span>
              </div>

              <div style={styles.alertDescription}>
                {alert.rule.description}
              </div>

              <div style={styles.alertMetrics}>
                <div style={styles.alertMetric}>
                  <span style={styles.metricLabel}>Value:</span>
                  <span style={styles.metricValue}>{alert.value.toFixed(2)}</span>
                </div>
                <div style={styles.alertMetric}>
                  <span style={styles.metricLabel}>Threshold:</span>
                  <span style={styles.metricValue}>
                    {alert.rule.condition.operator} {alert.rule.condition.threshold}
                  </span>
                </div>
                <div style={styles.alertMetric}>
                  <span style={styles.metricLabel}>Started:</span>
                  <span style={styles.metricValue}>
                    {new Date(alert.startsAt).toLocaleString()}
                  </span>
                </div>
                {alert.endsAt && (
                  <div style={styles.alertMetric}>
                    <span style={styles.metricLabel}>Ended:</span>
                    <span style={styles.metricValue}>
                      {new Date(alert.endsAt).toLocaleString()}
                    </span>
                  </div>
                )}
              </div>

              {Object.keys(alert.annotations).length > 0 && (
                <div style={styles.alertAnnotations}>
                  {Object.entries(alert.annotations).map(([key, value]) => (
                    <div key={key} style={styles.annotation}>
                      <strong>{key}:</strong> {value}
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))
        ) : (
          <div style={styles.emptyState}>
            <span style={styles.emptyIcon}>✓</span>
            <p style={styles.emptyText}>
              {filter === 'all' ? 'No alerts' : `No ${filter} alerts`}
            </p>
            <p style={styles.emptySubtext}>
              System is operating normally
            </p>
          </div>
        )}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    width: '100%'
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '20px'
  },
  title: {
    margin: 0,
    fontSize: '24px',
    fontWeight: 500,
    color: '#333'
  },
  filters: {
    display: 'flex',
    gap: '8px'
  },
  filterButton: {
    padding: '8px 16px',
    fontSize: '14px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    backgroundColor: 'white',
    cursor: 'pointer',
    transition: 'all 0.2s'
  },
  activeFilter: {
    backgroundColor: '#1976d2',
    color: 'white',
    borderColor: '#1976d2'
  },
  stats: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '15px',
    marginBottom: '20px'
  },
  statCard: {
    display: 'flex',
    alignItems: 'center',
    gap: '15px',
    backgroundColor: 'white',
    padding: '20px',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  statIcon: {
    fontSize: '32px'
  },
  statContent: {
    flex: 1
  },
  statValue: {
    fontSize: '28px',
    fontWeight: 600,
    color: '#1976d2',
    marginBottom: '4px'
  },
  statLabel: {
    fontSize: '14px',
    color: '#666'
  },
  alertsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '15px'
  },
  alertItem: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '20px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  alertHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: '12px'
  },
  alertTitleRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px'
  },
  severityBadge: {
    padding: '4px 8px',
    borderRadius: '4px',
    color: 'white',
    fontSize: '11px',
    fontWeight: 500,
    textTransform: 'uppercase'
  },
  alertName: {
    fontSize: '18px',
    fontWeight: 500,
    color: '#333'
  },
  stateBadge: {
    padding: '6px 12px',
    borderRadius: '4px',
    color: 'white',
    fontSize: '12px',
    fontWeight: 500,
    textTransform: 'uppercase'
  },
  alertDescription: {
    fontSize: '14px',
    color: '#666',
    marginBottom: '15px'
  },
  alertMetrics: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '12px',
    marginBottom: '15px'
  },
  alertMetric: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '8px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px'
  },
  metricLabel: {
    fontSize: '13px',
    color: '#666'
  },
  metricValue: {
    fontSize: '13px',
    fontWeight: 500,
    color: '#333',
    fontFamily: 'monospace'
  },
  alertAnnotations: {
    padding: '12px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px',
    fontSize: '13px'
  },
  annotation: {
    marginBottom: '5px',
    color: '#666'
  },
  emptyState: {
    textAlign: 'center',
    padding: '60px 20px',
    backgroundColor: 'white',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  emptyIcon: {
    fontSize: '64px',
    display: 'block',
    marginBottom: '15px',
    color: '#4caf50'
  },
  emptyText: {
    margin: '0 0 8px 0',
    fontSize: '18px',
    fontWeight: 500,
    color: '#333'
  },
  emptySubtext: {
    margin: 0,
    fontSize: '14px',
    color: '#666'
  }
};
