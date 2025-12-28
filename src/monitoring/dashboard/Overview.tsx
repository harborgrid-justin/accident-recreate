/**
 * AccuScene Enterprise v0.2.0
 * Overview Dashboard Panel
 */

import React, { useState, useEffect } from 'react';
import { globalCollector } from '../core/collector';
import { globalHealthRegistry } from '../health/checks';

interface OverviewProps {
  refreshInterval: number;
}

export const Overview: React.FC<OverviewProps> = ({ refreshInterval }) => {
  const [metrics, setMetrics] = useState<any>({});
  const [health, setHealth] = useState<any>(null);
  const [uptime, setUptime] = useState(0);

  useEffect(() => {
    const startTime = Date.now();
    const updateData = async () => {
      // Collect current metrics
      const collected = globalCollector.collect();

      // Get health status
      const healthReport = await globalHealthRegistry.runAll();

      // Calculate uptime
      const currentUptime = (Date.now() - startTime) / 1000;

      setMetrics({
        totalMetrics: globalCollector.getMetricsCount(),
        totalTraces: collected.traces.length,
        memoryUsage: collected.profiles.memory?.heapUsed || 0,
        cpuUsage: 0
      });

      setHealth(healthReport);
      setUptime(currentUptime);
    };

    updateData();
    const interval = setInterval(updateData, refreshInterval);

    return () => clearInterval(interval);
  }, [refreshInterval]);

  const formatBytes = (bytes: number): string => {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(2)} ${units[unitIndex]}`;
  };

  const formatUptime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    return `${hours}h ${minutes}m ${secs}s`;
  };

  const getHealthColor = (status: string): string => {
    switch (status) {
      case 'healthy':
        return '#4caf50';
      case 'degraded':
        return '#ff9800';
      case 'unhealthy':
        return '#f44336';
      default:
        return '#9e9e9e';
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.grid}>
        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <span style={styles.cardIcon}>📊</span>
            <h3 style={styles.cardTitle}>System Health</h3>
          </div>
          <div style={styles.cardContent}>
            {health && (
              <div style={styles.healthStatus}>
                <div style={{
                  ...styles.healthBadge,
                  backgroundColor: getHealthColor(health.status)
                }}>
                  {health.status.toUpperCase()}
                </div>
                <div style={styles.healthChecks}>
                  {Object.entries(health.checks).map(([name, result]: [string, any]) => (
                    <div key={name} style={styles.healthCheckItem}>
                      <span style={{
                        ...styles.healthCheckDot,
                        backgroundColor: getHealthColor(result.status)
                      }} />
                      <span style={styles.healthCheckName}>{name}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>

        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <span style={styles.cardIcon}>⏱️</span>
            <h3 style={styles.cardTitle}>Uptime</h3>
          </div>
          <div style={styles.cardContent}>
            <div style={styles.statValue}>{formatUptime(uptime)}</div>
            <div style={styles.statLabel}>Session Duration</div>
          </div>
        </div>

        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <span style={styles.cardIcon}>💾</span>
            <h3 style={styles.cardTitle}>Memory Usage</h3>
          </div>
          <div style={styles.cardContent}>
            <div style={styles.statValue}>{formatBytes(metrics.memoryUsage)}</div>
            <div style={styles.statLabel}>Heap Used</div>
          </div>
        </div>

        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <span style={styles.cardIcon}>📈</span>
            <h3 style={styles.cardTitle}>Metrics Collected</h3>
          </div>
          <div style={styles.cardContent}>
            <div style={styles.statValue}>{metrics.totalMetrics}</div>
            <div style={styles.statLabel}>Active Metrics</div>
          </div>
        </div>

        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <span style={styles.cardIcon}>🔍</span>
            <h3 style={styles.cardTitle}>Traces</h3>
          </div>
          <div style={styles.cardContent}>
            <div style={styles.statValue}>{metrics.totalTraces}</div>
            <div style={styles.statLabel}>Spans Collected</div>
          </div>
        </div>

        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <span style={styles.cardIcon}>⚡</span>
            <h3 style={styles.cardTitle}>Performance</h3>
          </div>
          <div style={styles.cardContent}>
            <div style={styles.performanceGrid}>
              <div style={styles.performanceItem}>
                <div style={styles.performanceLabel}>FPS</div>
                <div style={styles.performanceValue}>60</div>
              </div>
              <div style={styles.performanceItem}>
                <div style={styles.performanceLabel}>Latency</div>
                <div style={styles.performanceValue}>12ms</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div style={styles.recentActivity}>
        <h3 style={styles.sectionTitle}>Recent Activity</h3>
        <div style={styles.activityList}>
          <div style={styles.activityItem}>
            <span style={styles.activityTime}>{new Date().toLocaleTimeString()}</span>
            <span style={styles.activityMessage}>Monitoring system started</span>
          </div>
          <div style={styles.activityItem}>
            <span style={styles.activityTime}>{new Date().toLocaleTimeString()}</span>
            <span style={styles.activityMessage}>Metrics collection active</span>
          </div>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    width: '100%'
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
    gap: '20px',
    marginBottom: '30px'
  },
  card: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '20px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  cardHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    marginBottom: '15px'
  },
  cardIcon: {
    fontSize: '24px'
  },
  cardTitle: {
    margin: 0,
    fontSize: '16px',
    fontWeight: 500,
    color: '#333'
  },
  cardContent: {
    minHeight: '80px'
  },
  healthStatus: {
    display: 'flex',
    flexDirection: 'column',
    gap: '15px'
  },
  healthBadge: {
    display: 'inline-block',
    padding: '6px 12px',
    borderRadius: '4px',
    color: 'white',
    fontWeight: 500,
    fontSize: '12px',
    alignSelf: 'flex-start'
  },
  healthChecks: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px'
  },
  healthCheckItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '14px'
  },
  healthCheckDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%'
  },
  healthCheckName: {
    color: '#666'
  },
  statValue: {
    fontSize: '32px',
    fontWeight: 600,
    color: '#1976d2',
    marginBottom: '8px'
  },
  statLabel: {
    fontSize: '14px',
    color: '#666'
  },
  performanceGrid: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '15px'
  },
  performanceItem: {
    textAlign: 'center'
  },
  performanceLabel: {
    fontSize: '12px',
    color: '#666',
    marginBottom: '5px'
  },
  performanceValue: {
    fontSize: '24px',
    fontWeight: 600,
    color: '#1976d2'
  },
  recentActivity: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '20px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  sectionTitle: {
    margin: '0 0 15px 0',
    fontSize: '18px',
    fontWeight: 500,
    color: '#333'
  },
  activityList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '10px'
  },
  activityItem: {
    display: 'flex',
    gap: '15px',
    padding: '10px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px',
    fontSize: '14px'
  },
  activityTime: {
    color: '#666',
    fontWeight: 500,
    minWidth: '100px'
  },
  activityMessage: {
    color: '#333'
  }
};
