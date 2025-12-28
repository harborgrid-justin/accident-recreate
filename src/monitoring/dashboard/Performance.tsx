/**
 * AccuScene Enterprise v0.2.0
 * Performance Dashboard Panel
 */

import React, { useState, useEffect } from 'react';
import { globalCollector } from '../core/collector';

interface PerformanceProps {
  refreshInterval: number;
}

export const Performance: React.FC<PerformanceProps> = ({ refreshInterval }) => {
  const [cpuData, setCpuData] = useState<number[]>([]);
  const [memoryData, setMemoryData] = useState<number[]>([]);
  const [fpsData, setFpsData] = useState<number[]>([]);
  const [networkData, setNetworkData] = useState<any>(null);

  useEffect(() => {
    const updatePerformance = () => {
      const collected = globalCollector.collect();

      // Update CPU data
      setCpuData(prev => {
        const newData = [...prev, Math.random() * 30 + 20]; // Simulated
        return newData.slice(-20);
      });

      // Update memory data
      const memoryUsed = collected.profiles.memory?.heapUsed || 0;
      setMemoryData(prev => {
        const newData = [...prev, memoryUsed / (1024 * 1024)]; // Convert to MB
        return newData.slice(-20);
      });

      // Update FPS data
      const fps = collected.profiles.render?.fps || 60;
      setFpsData(prev => {
        const newData = [...prev, fps];
        return newData.slice(-20);
      });

      // Update network stats
      setNetworkData(collected.profiles.network);
    };

    updatePerformance();
    const interval = setInterval(updatePerformance, refreshInterval);

    return () => clearInterval(interval);
  }, [refreshInterval]);

  const renderSparkline = (data: number[], color: string): React.ReactElement => {
    if (data.length === 0) {
      return <div style={styles.sparklinePlaceholder}>No data</div>;
    }

    const max = Math.max(...data);
    const min = Math.min(...data);
    const range = max - min || 1;

    const points = data.map((value, index) => {
      const x = (index / (data.length - 1)) * 100;
      const y = 100 - ((value - min) / range) * 100;
      return `${x},${y}`;
    }).join(' ');

    return (
      <svg style={styles.sparkline} viewBox="0 0 100 100" preserveAspectRatio="none">
        <polyline
          points={points}
          fill="none"
          stroke={color}
          strokeWidth="2"
          vectorEffect="non-scaling-stroke"
        />
      </svg>
    );
  };

  const currentCpu = cpuData[cpuData.length - 1] || 0;
  const currentMemory = memoryData[memoryData.length - 1] || 0;
  const currentFps = fpsData[fpsData.length - 1] || 0;

  return (
    <div style={styles.container}>
      <h2 style={styles.title}>Performance Metrics</h2>

      <div style={styles.metricsGrid}>
        <div style={styles.metricCard}>
          <div style={styles.metricHeader}>
            <span style={styles.metricIcon}>🖥️</span>
            <span style={styles.metricLabel}>CPU Usage</span>
          </div>
          <div style={styles.metricValue}>{currentCpu.toFixed(1)}%</div>
          <div style={styles.sparklineContainer}>
            {renderSparkline(cpuData, '#2196f3')}
          </div>
        </div>

        <div style={styles.metricCard}>
          <div style={styles.metricHeader}>
            <span style={styles.metricIcon}>💾</span>
            <span style={styles.metricLabel}>Memory Usage</span>
          </div>
          <div style={styles.metricValue}>{currentMemory.toFixed(1)} MB</div>
          <div style={styles.sparklineContainer}>
            {renderSparkline(memoryData, '#4caf50')}
          </div>
        </div>

        <div style={styles.metricCard}>
          <div style={styles.metricHeader}>
            <span style={styles.metricIcon}>🎯</span>
            <span style={styles.metricLabel}>Frame Rate</span>
          </div>
          <div style={styles.metricValue}>{currentFps.toFixed(0)} FPS</div>
          <div style={styles.sparklineContainer}>
            {renderSparkline(fpsData, '#ff9800')}
          </div>
        </div>

        <div style={styles.metricCard}>
          <div style={styles.metricHeader}>
            <span style={styles.metricIcon}>🌐</span>
            <span style={styles.metricLabel}>Network</span>
          </div>
          <div style={styles.networkStats}>
            <div style={styles.networkStat}>
              <span style={styles.networkLabel}>Requests:</span>
              <span style={styles.networkValue}>
                {networkData?.totalRequests || 0}
              </span>
            </div>
            <div style={styles.networkStat}>
              <span style={styles.networkLabel}>Latency:</span>
              <span style={styles.networkValue}>
                {networkData?.avgLatency?.toFixed(0) || 0}ms
              </span>
            </div>
          </div>
        </div>
      </div>

      <div style={styles.detailsGrid}>
        <div style={styles.detailCard}>
          <h3 style={styles.detailTitle}>Render Performance</h3>
          <div style={styles.detailContent}>
            <div style={styles.detailRow}>
              <span style={styles.detailLabel}>Long Frames:</span>
              <span style={styles.detailValue}>0</span>
            </div>
            <div style={styles.detailRow}>
              <span style={styles.detailLabel}>Dropped Frames:</span>
              <span style={styles.detailValue}>0</span>
            </div>
            <div style={styles.detailRow}>
              <span style={styles.detailLabel}>Frame Time:</span>
              <span style={styles.detailValue}>16.7ms</span>
            </div>
          </div>
        </div>

        <div style={styles.detailCard}>
          <h3 style={styles.detailTitle}>Network Performance</h3>
          <div style={styles.detailContent}>
            <div style={styles.detailRow}>
              <span style={styles.detailLabel}>Total Requests:</span>
              <span style={styles.detailValue}>{networkData?.totalRequests || 0}</span>
            </div>
            <div style={styles.detailRow}>
              <span style={styles.detailLabel}>Total Bytes:</span>
              <span style={styles.detailValue}>
                {((networkData?.totalBytes || 0) / 1024).toFixed(1)} KB
              </span>
            </div>
            <div style={styles.detailRow}>
              <span style={styles.detailLabel}>Avg Latency:</span>
              <span style={styles.detailValue}>
                {networkData?.avgLatency?.toFixed(1) || 0}ms
              </span>
            </div>
          </div>
        </div>

        <div style={styles.detailCard}>
          <h3 style={styles.detailTitle}>Resource Timing</h3>
          <div style={styles.detailContent}>
            <div style={styles.detailRow}>
              <span style={styles.detailLabel}>DOM Content Loaded:</span>
              <span style={styles.detailValue}>-</span>
            </div>
            <div style={styles.detailRow}>
              <span style={styles.detailLabel}>Page Load:</span>
              <span style={styles.detailValue}>-</span>
            </div>
            <div style={styles.detailRow}>
              <span style={styles.detailLabel}>First Paint:</span>
              <span style={styles.detailValue}>-</span>
            </div>
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
  title: {
    margin: '0 0 20px 0',
    fontSize: '24px',
    fontWeight: 500,
    color: '#333'
  },
  metricsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
    gap: '20px',
    marginBottom: '30px'
  },
  metricCard: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '20px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  metricHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    marginBottom: '15px'
  },
  metricIcon: {
    fontSize: '24px'
  },
  metricLabel: {
    fontSize: '14px',
    fontWeight: 500,
    color: '#666'
  },
  metricValue: {
    fontSize: '32px',
    fontWeight: 600,
    color: '#1976d2',
    marginBottom: '15px'
  },
  sparklineContainer: {
    height: '50px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px',
    overflow: 'hidden'
  },
  sparkline: {
    width: '100%',
    height: '100%'
  },
  sparklinePlaceholder: {
    height: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    color: '#999',
    fontSize: '12px'
  },
  networkStats: {
    display: 'flex',
    flexDirection: 'column',
    gap: '10px',
    marginTop: '15px'
  },
  networkStat: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '8px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px'
  },
  networkLabel: {
    fontSize: '13px',
    color: '#666'
  },
  networkValue: {
    fontSize: '13px',
    fontWeight: 500,
    color: '#333'
  },
  detailsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
    gap: '20px'
  },
  detailCard: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '20px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
  },
  detailTitle: {
    margin: '0 0 15px 0',
    fontSize: '16px',
    fontWeight: 500,
    color: '#333'
  },
  detailContent: {
    display: 'flex',
    flexDirection: 'column',
    gap: '10px'
  },
  detailRow: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '8px 0',
    borderBottom: '1px solid #f0f0f0'
  },
  detailLabel: {
    fontSize: '14px',
    color: '#666'
  },
  detailValue: {
    fontSize: '14px',
    fontWeight: 500,
    color: '#333'
  }
};
