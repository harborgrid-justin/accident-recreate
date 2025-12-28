/**
 * AccuScene Enterprise v0.2.0
 * Metrics Dashboard Panel
 */

import React, { useState, useEffect } from 'react';
import { globalCollector } from '../core/collector';
import { globalAggregator } from '../core/aggregator';

interface MetricsProps {
  refreshInterval: number;
}

export const Metrics: React.FC<MetricsProps> = ({ refreshInterval }) => {
  const [metricNames, setMetricNames] = useState<string[]>([]);
  const [selectedMetric, setSelectedMetric] = useState<string>('');
  const [metricData, setMetricData] = useState<any>(null);

  useEffect(() => {
    const updateMetrics = () => {
      const names = globalCollector.getMetricNames();
      setMetricNames(names);

      if (!selectedMetric && names.length > 0) {
        setSelectedMetric(names[0]);
      }

      if (selectedMetric) {
        const metric = globalCollector.getMetric(selectedMetric);
        const now = Date.now();
        const hourAgo = now - 3600000;
        const snapshot = globalAggregator.getSnapshot(selectedMetric, hourAgo, now);

        setMetricData({
          metric,
          snapshot
        });
      }
    };

    updateMetrics();
    const interval = setInterval(updateMetrics, refreshInterval);

    return () => clearInterval(interval);
  }, [refreshInterval, selectedMetric]);

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={styles.title}>Metrics ({metricNames.length})</h2>
        <select
          style={styles.select}
          value={selectedMetric}
          onChange={(e) => setSelectedMetric(e.target.value)}
        >
          {metricNames.map(name => (
            <option key={name} value={name}>{name}</option>
          ))}
        </select>
      </div>

      {metricData && (
        <div style={styles.content}>
          <div style={styles.metricInfo}>
            <div style={styles.infoCard}>
              <div style={styles.infoLabel}>Type</div>
              <div style={styles.infoValue}>{metricData.metric.metadata.type}</div>
            </div>
            <div style={styles.infoCard}>
              <div style={styles.infoLabel}>Description</div>
              <div style={styles.infoValue}>{metricData.metric.metadata.help}</div>
            </div>
            {metricData.metric.metadata.unit && (
              <div style={styles.infoCard}>
                <div style={styles.infoLabel}>Unit</div>
                <div style={styles.infoValue}>{metricData.metric.metadata.unit}</div>
              </div>
            )}
          </div>

          <div style={styles.statsGrid}>
            <div style={styles.statCard}>
              <div style={styles.statLabel}>Count</div>
              <div style={styles.statValue}>{metricData.snapshot.count}</div>
            </div>
            <div style={styles.statCard}>
              <div style={styles.statLabel}>Average</div>
              <div style={styles.statValue}>{metricData.snapshot.avg.toFixed(2)}</div>
            </div>
            <div style={styles.statCard}>
              <div style={styles.statLabel}>Min</div>
              <div style={styles.statValue}>{metricData.snapshot.min.toFixed(2)}</div>
            </div>
            <div style={styles.statCard}>
              <div style={styles.statLabel}>Max</div>
              <div style={styles.statValue}>{metricData.snapshot.max.toFixed(2)}</div>
            </div>
            {metricData.snapshot.p95 !== undefined && (
              <div style={styles.statCard}>
                <div style={styles.statLabel}>P95</div>
                <div style={styles.statValue}>{metricData.snapshot.p95.toFixed(2)}</div>
              </div>
            )}
            {metricData.snapshot.p99 !== undefined && (
              <div style={styles.statCard}>
                <div style={styles.statLabel}>P99</div>
                <div style={styles.statValue}>{metricData.snapshot.p99.toFixed(2)}</div>
              </div>
            )}
          </div>
        </div>
      )}

      <div style={styles.metricsList}>
        <h3 style={styles.listTitle}>All Metrics</h3>
        <div style={styles.listContent}>
          {metricNames.map(name => (
            <div
              key={name}
              style={{
                ...styles.metricItem,
                ...(name === selectedMetric ? styles.selectedMetric : {})
              }}
              onClick={() => setSelectedMetric(name)}
            >
              <span style={styles.metricName}>{name}</span>
              <span style={styles.metricBadge}>
                {globalCollector.getMetric(name)?.metadata.type}
              </span>
            </div>
          ))}
        </div>
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
  select: {
    padding: '8px 12px',
    fontSize: '14px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    backgroundColor: 'white',
    cursor: 'pointer'
  },
  content: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '20px',
    marginBottom: '20px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  metricInfo: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '15px',
    marginBottom: '20px'
  },
  infoCard: {
    padding: '12px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px'
  },
  infoLabel: {
    fontSize: '12px',
    color: '#666',
    marginBottom: '5px'
  },
  infoValue: {
    fontSize: '14px',
    fontWeight: 500,
    color: '#333'
  },
  statsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
    gap: '15px'
  },
  statCard: {
    textAlign: 'center',
    padding: '15px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px'
  },
  statLabel: {
    fontSize: '12px',
    color: '#666',
    marginBottom: '5px'
  },
  statValue: {
    fontSize: '24px',
    fontWeight: 600,
    color: '#1976d2'
  },
  metricsList: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '20px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  listTitle: {
    margin: '0 0 15px 0',
    fontSize: '18px',
    fontWeight: 500,
    color: '#333'
  },
  listContent: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    maxHeight: '400px',
    overflowY: 'auto'
  },
  metricItem: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px',
    cursor: 'pointer',
    transition: 'background-color 0.2s'
  },
  selectedMetric: {
    backgroundColor: '#e3f2fd',
    borderLeft: '3px solid #1976d2'
  },
  metricName: {
    fontSize: '14px',
    color: '#333',
    fontFamily: 'monospace'
  },
  metricBadge: {
    padding: '4px 8px',
    fontSize: '11px',
    backgroundColor: 'white',
    borderRadius: '4px',
    color: '#666',
    textTransform: 'uppercase'
  }
};
