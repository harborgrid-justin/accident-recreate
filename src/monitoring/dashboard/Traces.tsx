/**
 * AccuScene Enterprise v0.2.0
 * Traces Dashboard Panel
 */

import React, { useState, useEffect } from 'react';
import { globalCollector } from '../core/collector';
import { Span } from '../tracing/span';

interface TracesProps {
  refreshInterval: number;
}

export const Traces: React.FC<TracesProps> = ({ refreshInterval }) => {
  const [traces, setTraces] = useState<Span[]>([]);
  const [selectedTrace, setSelectedTrace] = useState<Span | null>(null);

  useEffect(() => {
    const updateTraces = () => {
      const collected = globalCollector.collect();
      setTraces(collected.traces.slice(-50)); // Last 50 traces
    };

    updateTraces();
    const interval = setInterval(updateTraces, refreshInterval);

    return () => clearInterval(interval);
  }, [refreshInterval]);

  const groupedTraces = traces.reduce((acc, trace) => {
    const traceId = trace.context.traceId;
    if (!acc[traceId]) {
      acc[traceId] = [];
    }
    acc[traceId].push(trace);
    return acc;
  }, {} as Record<string, Span[]>);

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'ok':
        return '#4caf50';
      case 'error':
        return '#f44336';
      default:
        return '#9e9e9e';
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={styles.title}>Distributed Traces ({Object.keys(groupedTraces).length})</h2>
      </div>

      <div style={styles.content}>
        <div style={styles.tracesList}>
          {Object.entries(groupedTraces).map(([traceId, spans]) => {
            const rootSpan = spans.find(s => !s.parentSpanId) || spans[0];

            return (
              <div
                key={traceId}
                style={styles.traceItem}
                onClick={() => setSelectedTrace(rootSpan)}
              >
                <div style={styles.traceHeader}>
                  <span style={styles.traceName}>{rootSpan.name}</span>
                  <span style={{
                    ...styles.traceStatus,
                    backgroundColor: getStatusColor(rootSpan.status)
                  }}>
                    {rootSpan.status}
                  </span>
                </div>
                <div style={styles.traceInfo}>
                  <span style={styles.traceDetail}>
                    Spans: {spans.length}
                  </span>
                  <span style={styles.traceDetail}>
                    Duration: {rootSpan.getDuration()}ms
                  </span>
                  <span style={styles.traceDetail}>
                    {new Date(rootSpan.startTime).toLocaleString()}
                  </span>
                </div>
                <div style={styles.traceId}>
                  Trace ID: {traceId.substring(0, 16)}...
                </div>
              </div>
            );
          })}

          {Object.keys(groupedTraces).length === 0 && (
            <div style={styles.emptyState}>
              <span style={styles.emptyIcon}>🔍</span>
              <p style={styles.emptyText}>No traces collected yet</p>
            </div>
          )}
        </div>

        {selectedTrace && (
          <div style={styles.traceDetails}>
            <h3 style={styles.detailsTitle}>Trace Details</h3>
            <div style={styles.detailsContent}>
              <div style={styles.detailGroup}>
                <div style={styles.detailLabel}>Name</div>
                <div style={styles.detailValue}>{selectedTrace.name}</div>
              </div>
              <div style={styles.detailGroup}>
                <div style={styles.detailLabel}>Trace ID</div>
                <div style={styles.detailValue}>{selectedTrace.context.traceId}</div>
              </div>
              <div style={styles.detailGroup}>
                <div style={styles.detailLabel}>Span ID</div>
                <div style={styles.detailValue}>{selectedTrace.context.spanId}</div>
              </div>
              <div style={styles.detailGroup}>
                <div style={styles.detailLabel}>Duration</div>
                <div style={styles.detailValue}>{selectedTrace.getDuration()}ms</div>
              </div>
              <div style={styles.detailGroup}>
                <div style={styles.detailLabel}>Status</div>
                <div style={{
                  ...styles.detailValue,
                  color: getStatusColor(selectedTrace.status)
                }}>
                  {selectedTrace.status}
                </div>
              </div>

              {Object.keys(selectedTrace.attributes).length > 0 && (
                <div style={styles.detailGroup}>
                  <div style={styles.detailLabel}>Attributes</div>
                  <div style={styles.attributesList}>
                    {Object.entries(selectedTrace.attributes).map(([key, value]) => (
                      <div key={key} style={styles.attributeItem}>
                        <span style={styles.attributeKey}>{key}:</span>
                        <span style={styles.attributeValue}>{String(value)}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {selectedTrace.events.length > 0 && (
                <div style={styles.detailGroup}>
                  <div style={styles.detailLabel}>Events ({selectedTrace.events.length})</div>
                  <div style={styles.eventsList}>
                    {selectedTrace.events.map((event, idx) => (
                      <div key={idx} style={styles.eventItem}>
                        <div style={styles.eventName}>{event.name}</div>
                        <div style={styles.eventTime}>
                          {new Date(event.timestamp).toLocaleTimeString()}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
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
    marginBottom: '20px'
  },
  title: {
    margin: 0,
    fontSize: '24px',
    fontWeight: 500,
    color: '#333'
  },
  content: {
    display: 'grid',
    gridTemplateColumns: selectedTrace => selectedTrace ? '1fr 1fr' : '1fr',
    gap: '20px'
  },
  tracesList: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '20px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    maxHeight: '70vh',
    overflowY: 'auto'
  },
  traceItem: {
    padding: '15px',
    marginBottom: '10px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px',
    cursor: 'pointer',
    transition: 'background-color 0.2s'
  },
  traceHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '10px'
  },
  traceName: {
    fontSize: '16px',
    fontWeight: 500,
    color: '#333'
  },
  traceStatus: {
    padding: '4px 8px',
    borderRadius: '4px',
    color: 'white',
    fontSize: '12px',
    fontWeight: 500,
    textTransform: 'uppercase'
  },
  traceInfo: {
    display: 'flex',
    gap: '15px',
    marginBottom: '8px'
  },
  traceDetail: {
    fontSize: '13px',
    color: '#666'
  },
  traceId: {
    fontSize: '12px',
    color: '#999',
    fontFamily: 'monospace'
  },
  emptyState: {
    textAlign: 'center',
    padding: '60px 20px',
    color: '#999'
  },
  emptyIcon: {
    fontSize: '48px',
    display: 'block',
    marginBottom: '15px'
  },
  emptyText: {
    margin: 0,
    fontSize: '16px'
  },
  traceDetails: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '20px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    maxHeight: '70vh',
    overflowY: 'auto'
  },
  detailsTitle: {
    margin: '0 0 20px 0',
    fontSize: '18px',
    fontWeight: 500,
    color: '#333'
  },
  detailsContent: {
    display: 'flex',
    flexDirection: 'column',
    gap: '15px'
  },
  detailGroup: {
    paddingBottom: '15px',
    borderBottom: '1px solid #e0e0e0'
  },
  detailLabel: {
    fontSize: '12px',
    color: '#666',
    marginBottom: '5px',
    fontWeight: 500
  },
  detailValue: {
    fontSize: '14px',
    color: '#333',
    fontFamily: 'monospace',
    wordBreak: 'break-all'
  },
  attributesList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '5px',
    marginTop: '8px'
  },
  attributeItem: {
    fontSize: '13px',
    fontFamily: 'monospace'
  },
  attributeKey: {
    color: '#666',
    marginRight: '8px'
  },
  attributeValue: {
    color: '#333'
  },
  eventsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    marginTop: '8px'
  },
  eventItem: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '8px',
    backgroundColor: '#f5f5f5',
    borderRadius: '4px'
  },
  eventName: {
    fontSize: '13px',
    fontWeight: 500,
    color: '#333'
  },
  eventTime: {
    fontSize: '12px',
    color: '#666'
  }
};
