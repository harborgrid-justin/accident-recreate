/**
 * AccuScene Enterprise v0.3.0 - Report Summary Widget
 * Auto-generated executive summary with key findings
 */

import React, { useMemo } from 'react';
import { WidgetProps, AnalyticsData } from '../types';

interface Finding {
  severity: 'critical' | 'high' | 'medium' | 'low';
  category: string;
  title: string;
  description: string;
  value?: string;
  icon: string;
}

const ReportSummary: React.FC<WidgetProps<AnalyticsData>> = ({
  config,
  data,
}) => {
  // Generate findings from data
  const findings = useMemo((): Finding[] => {
    if (!data) return [];

    const results: Finding[] = [];

    // Analyze impacts
    if (data.impacts && data.impacts.length > 0) {
      const criticalImpacts = data.impacts.filter((i) => i.severity > 0.7);
      if (criticalImpacts.length > 0) {
        results.push({
          severity: 'critical',
          category: 'Impact Analysis',
          title: 'Critical Impact Events Detected',
          description: `${criticalImpacts.length} impact event(s) with severity above 70% were detected. These require immediate attention and detailed analysis.`,
          value: `${criticalImpacts.length} critical impact${criticalImpacts.length > 1 ? 's' : ''}`,
          icon: '⚠️',
        });
      }

      const totalEnergy = data.impacts.reduce((sum, i) => sum + i.energy, 0);
      results.push({
        severity: totalEnergy > 100000 ? 'high' : 'medium',
        category: 'Energy Transfer',
        title: 'Total Impact Energy',
        description: `Total energy dissipated during impact events was ${totalEnergy.toFixed(0)} Joules. ${totalEnergy > 100000 ? 'This indicates a high-energy collision requiring comprehensive structural analysis.' : 'Energy levels are within moderate range.'}`,
        value: `${(totalEnergy / 1000).toFixed(1)} kJ`,
        icon: '⚡',
      });
    }

    // Analyze vehicles
    if (data.vehicles && data.vehicles.length > 0) {
      data.vehicles.forEach((vehicle) => {
        if (vehicle.damageProfile && vehicle.damageProfile.length > 0) {
          const criticalDamage = vehicle.damageProfile.filter(
            (d) => d.severity > 0.7
          );
          if (criticalDamage.length > 0) {
            results.push({
              severity: 'high',
              category: 'Damage Assessment',
              title: `${vehicle.name} - Critical Damage`,
              description: `Vehicle sustained ${criticalDamage.length} critical damage point(s). Total damage area: ${vehicle.damageProfile.reduce((sum, d) => sum + d.area, 0).toFixed(2)} m².`,
              value: `${criticalDamage.length} point(s)`,
              icon: '🚗',
            });
          }
        }

        // Speed analysis
        if (vehicle.speed && vehicle.speed.length > 0) {
          const maxSpeed = Math.max(...vehicle.speed.map((s) => s.value));
          if (maxSpeed > 60) {
            results.push({
              severity: maxSpeed > 80 ? 'high' : 'medium',
              category: 'Speed Analysis',
              title: `${vehicle.name} - High Speed Detected`,
              description: `Vehicle reached a maximum speed of ${maxSpeed.toFixed(1)} mph. ${maxSpeed > 80 ? 'This exceeds typical safety thresholds and contributed significantly to impact severity.' : 'Speed was a contributing factor to the incident.'}`,
              value: `${maxSpeed.toFixed(1)} mph`,
              icon: '🏎️',
            });
          }
        }
      });
    }

    // Analyze force vectors
    if (data.forceVectors && data.forceVectors.length > 0) {
      const maxForce = Math.max(...data.forceVectors.map((f) => f.magnitude));
      if (maxForce > 30000) {
        results.push({
          severity: maxForce > 50000 ? 'critical' : 'high',
          category: 'Force Analysis',
          title: 'Extreme Force Detected',
          description: `Maximum force magnitude recorded was ${maxForce.toFixed(0)} Newtons. Forces of this magnitude can cause severe structural deformation and injury.`,
          value: `${(maxForce / 1000).toFixed(1)} kN`,
          icon: '💪',
        });
      }
    }

    // Timeline analysis
    if (data.timeline && data.timeline.length > 0) {
      const criticalEvents = data.timeline.filter(
        (e) => e.severity === 'critical' || e.severity === 'high'
      );
      if (criticalEvents.length > 0) {
        results.push({
          severity: 'medium',
          category: 'Event Sequence',
          title: 'Critical Event Sequence',
          description: `${criticalEvents.length} critical or high-severity event(s) occurred during the incident timeline. Review the timeline widget for detailed chronological analysis.`,
          value: `${criticalEvents.length} event(s)`,
          icon: '📅',
        });
      }
    }

    // Sort by severity
    const severityOrder = { critical: 0, high: 1, medium: 2, low: 3 };
    return results.sort(
      (a, b) => severityOrder[a.severity] - severityOrder[b.severity]
    );
  }, [data]);

  // Generate executive summary
  const executiveSummary = useMemo(() => {
    if (!data) return '';

    const vehicleCount = data.vehicles?.length || 0;
    const impactCount = data.impacts?.length || 0;
    const criticalFindings = findings.filter(
      (f) => f.severity === 'critical' || f.severity === 'high'
    ).length;

    return `Analysis of ${vehicleCount} vehicle${vehicleCount > 1 ? 's' : ''} involved in ${impactCount} impact event${impactCount > 1 ? 's' : ''}. ${criticalFindings > 0 ? `${criticalFindings} critical or high-severity finding${criticalFindings > 1 ? 's' : ''} identified requiring immediate attention.` : 'No critical issues identified.'} Detailed metrics and visualizations are available in the dashboard widgets.`;
  }, [data, findings]);

  // Get severity color
  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return '#ef4444';
      case 'high':
        return '#f59e0b';
      case 'medium':
        return '#3b82f6';
      case 'low':
        return '#10b981';
      default:
        return '#64748b';
    }
  };

  const getSeverityLabel = (severity: string) => {
    return severity.charAt(0).toUpperCase() + severity.slice(1);
  };

  if (!data) {
    return (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#94a3b8',
        }}
      >
        No data available for report generation
      </div>
    );
  }

  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        gap: '1rem',
        overflow: 'auto',
      }}
    >
      {/* Header */}
      <div
        style={{
          backgroundColor: '#1e293b',
          padding: '1rem',
          borderRadius: '4px',
          border: '1px solid #334155',
        }}
      >
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: '0.5rem',
          }}
        >
          <h3
            style={{
              margin: 0,
              fontSize: '1.25rem',
              fontWeight: 600,
              color: '#f1f5f9',
            }}
          >
            Executive Summary
          </h3>
          <div
            style={{
              fontSize: '0.75rem',
              color: '#94a3b8',
            }}
          >
            Generated: {new Date().toLocaleString()}
          </div>
        </div>
        <p
          style={{
            margin: 0,
            fontSize: '0.875rem',
            color: '#cbd5e1',
            lineHeight: 1.6,
          }}
        >
          {executiveSummary}
        </p>
      </div>

      {/* Findings */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        {findings.length > 0 ? (
          findings.map((finding, index) => (
            <div
              key={index}
              style={{
                backgroundColor: '#1e293b',
                padding: '1rem',
                borderRadius: '4px',
                border: '1px solid #334155',
                borderLeft: `4px solid ${getSeverityColor(finding.severity)}`,
              }}
            >
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'flex-start',
                  marginBottom: '0.5rem',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                  <span style={{ fontSize: '1.5rem' }}>{finding.icon}</span>
                  <div>
                    <div
                      style={{
                        fontSize: '0.75rem',
                        color: '#94a3b8',
                        marginBottom: '0.25rem',
                      }}
                    >
                      {finding.category}
                    </div>
                    <div
                      style={{
                        fontSize: '1rem',
                        fontWeight: 600,
                        color: '#f1f5f9',
                      }}
                    >
                      {finding.title}
                    </div>
                  </div>
                </div>
                <div
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'flex-end',
                  }}
                >
                  <div
                    style={{
                      padding: '0.25rem 0.75rem',
                      backgroundColor: getSeverityColor(finding.severity) + '20',
                      color: getSeverityColor(finding.severity),
                      borderRadius: '4px',
                      fontSize: '0.75rem',
                      fontWeight: 600,
                      marginBottom: '0.25rem',
                    }}
                  >
                    {getSeverityLabel(finding.severity)}
                  </div>
                  {finding.value && (
                    <div
                      style={{
                        fontSize: '0.875rem',
                        fontWeight: 600,
                        color: '#f1f5f9',
                      }}
                    >
                      {finding.value}
                    </div>
                  )}
                </div>
              </div>
              <p
                style={{
                  margin: 0,
                  fontSize: '0.875rem',
                  color: '#cbd5e1',
                  lineHeight: 1.5,
                }}
              >
                {finding.description}
              </p>
            </div>
          ))
        ) : (
          <div
            style={{
              backgroundColor: '#1e293b',
              padding: '2rem',
              borderRadius: '4px',
              border: '1px solid #334155',
              textAlign: 'center',
              color: '#10b981',
            }}
          >
            <div style={{ fontSize: '3rem', marginBottom: '0.5rem' }}>✓</div>
            <div style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '0.5rem' }}>
              No Critical Issues
            </div>
            <div style={{ fontSize: '0.875rem', color: '#94a3b8' }}>
              All parameters are within normal ranges
            </div>
          </div>
        )}
      </div>

      {/* Footer */}
      <div
        style={{
          backgroundColor: '#1e293b',
          padding: '0.75rem 1rem',
          borderRadius: '4px',
          border: '1px solid #334155',
          fontSize: '0.75rem',
          color: '#94a3b8',
          textAlign: 'center',
        }}
      >
        This is an automated summary. For complete analysis, review all dashboard
        widgets and export a full report.
      </div>
    </div>
  );
};

export default ReportSummary;
