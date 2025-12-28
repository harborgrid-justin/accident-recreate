/**
 * ReportsPage - Report generation and management
 */

import React, { useState, useEffect } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { api, CaseData } from '../services/api';
import { useUIStore } from '../store/uiStore';
import Button from '../components/Common/Button';
import Select from '../components/Common/Select';
import Loading from '../components/Common/Loading';

interface Report {
  id: string;
  caseId: string;
  type: string;
  generatedAt: string;
  fileName: string;
  fileUrl: string;
}

export const ReportsPage: React.FC = () => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { showNotification, setLoading } = useUIStore();

  const [cases, setCases] = useState<CaseData[]>([]);
  const [selectedCaseId, setSelectedCaseId] = useState(searchParams.get('caseId') || '');
  const [reportType, setReportType] = useState('comprehensive');
  const [reports, setReports] = useState<Report[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isLoadingCases, setIsLoadingCases] = useState(true);
  const [isLoadingReports, setIsLoadingReports] = useState(false);

  useEffect(() => {
    loadCases();
  }, []);

  useEffect(() => {
    if (selectedCaseId) {
      loadReports(selectedCaseId);
    }
  }, [selectedCaseId]);

  const loadCases = async () => {
    setIsLoadingCases(true);
    const response = await api.getCases();
    setIsLoadingCases(false);

    if (response.success && response.data) {
      setCases(response.data.cases);
      if (response.data.cases.length > 0 && !selectedCaseId) {
        setSelectedCaseId(response.data.cases[0]!.id || '');
      }
    }
  };

  const loadReports = async (caseId: string) => {
    setIsLoadingReports(true);
    const response = await api.getReports(caseId);
    setIsLoadingReports(false);

    if (response.success && response.data) {
      setReports(response.data);
    }
  };

  const handleGenerateReport = async () => {
    if (!selectedCaseId) {
      showNotification('error', 'Please select a case');
      return;
    }

    setIsGenerating(true);
    setLoading(true, 'Generating report...');

    const response = await api.generateReport(selectedCaseId, reportType);

    setIsGenerating(false);
    setLoading(false);

    if (response.success && response.data) {
      showNotification('success', 'Report generated successfully');
      loadReports(selectedCaseId);
    } else {
      showNotification('error', response.error || 'Failed to generate report');
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="page reports-page">
      <div className="page-header">
        <div className="page-title-section">
          <h1 className="page-title">Reports</h1>
          <p className="page-subtitle">Generate and manage accident reports</p>
        </div>
      </div>

      <div className="reports-content">
        <div className="reports-generator">
          <div className="reports-generator-card">
            <h2>Generate New Report</h2>

            {isLoadingCases ? (
              <Loading message="Loading cases..." />
            ) : (
              <>
                <div className="reports-form">
                  <Select
                    label="Select Case"
                    value={selectedCaseId}
                    onChange={(e) => setSelectedCaseId(e.target.value)}
                    options={[
                      { value: '', label: 'Choose a case...' },
                      ...cases.map((c) => ({
                        value: c.id || '',
                        label: `${c.caseNumber} - ${c.title}`,
                      })),
                    ]}
                    fullWidth
                    required
                  />

                  <Select
                    label="Report Type"
                    value={reportType}
                    onChange={(e) => setReportType(e.target.value)}
                    options={[
                      { value: 'comprehensive', label: 'Comprehensive Report' },
                      { value: 'summary', label: 'Executive Summary' },
                      { value: 'technical', label: 'Technical Analysis' },
                      { value: 'insurance', label: 'Insurance Report' },
                      { value: 'legal', label: 'Legal Report' },
                    ]}
                    fullWidth
                    required
                  />

                  <div className="reports-info">
                    <h4>Report Contents:</h4>
                    <ul>
                      {reportType === 'comprehensive' && (
                        <>
                          <li>Complete case overview</li>
                          <li>Detailed accident analysis</li>
                          <li>Vehicle information and damage assessment</li>
                          <li>Physics simulation results</li>
                          <li>Evidence documentation</li>
                          <li>Witness statements</li>
                          <li>Diagrams and photographs</li>
                          <li>Conclusions and recommendations</li>
                        </>
                      )}
                      {reportType === 'summary' && (
                        <>
                          <li>Executive summary</li>
                          <li>Key findings</li>
                          <li>Basic accident details</li>
                          <li>Conclusion</li>
                        </>
                      )}
                      {reportType === 'technical' && (
                        <>
                          <li>Physics analysis</li>
                          <li>Speed calculations</li>
                          <li>Trajectory analysis</li>
                          <li>Impact force calculations</li>
                          <li>Technical diagrams</li>
                        </>
                      )}
                      {reportType === 'insurance' && (
                        <>
                          <li>Claim information</li>
                          <li>Damage assessment</li>
                          <li>Liability analysis</li>
                          <li>Cost estimates</li>
                        </>
                      )}
                      {reportType === 'legal' && (
                        <>
                          <li>Legal summary</li>
                          <li>Evidence chain of custody</li>
                          <li>Expert opinions</li>
                          <li>Regulatory compliance</li>
                        </>
                      )}
                    </ul>
                  </div>

                  <Button
                    variant="primary"
                    fullWidth
                    onClick={handleGenerateReport}
                    loading={isGenerating}
                    disabled={!selectedCaseId}
                  >
                    Generate Report
                  </Button>
                </div>
              </>
            )}
          </div>
        </div>

        <div className="reports-history">
          <h2>Report History</h2>

          {!selectedCaseId ? (
            <div className="reports-empty">
              <p>Select a case to view report history</p>
            </div>
          ) : isLoadingReports ? (
            <Loading message="Loading reports..." />
          ) : reports.length === 0 ? (
            <div className="reports-empty">
              <p>No reports generated for this case yet</p>
            </div>
          ) : (
            <div className="reports-list">
              {reports.map((report) => (
                <div key={report.id} className="report-item">
                  <div className="report-item-info">
                    <h3>{report.type.charAt(0).toUpperCase() + report.type.slice(1)} Report</h3>
                    <p className="report-item-date">Generated: {formatDate(report.generatedAt)}</p>
                    <p className="report-item-file">{report.fileName}</p>
                  </div>
                  <div className="report-item-actions">
                    <Button
                      variant="outline"
                      size="small"
                      onClick={() => window.open(report.fileUrl, '_blank')}
                    >
                      View
                    </Button>
                    <Button variant="outline" size="small" onClick={() => {}}>
                      Download
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {selectedCaseId && (
        <div className="reports-actions">
          <Button variant="outline" onClick={() => navigate(`/cases/${selectedCaseId}`)}>
            View Case Details
          </Button>
          <Button variant="outline" onClick={() => navigate(`/editor/${selectedCaseId}`)}>
            Open Editor
          </Button>
        </div>
      )}
    </div>
  );
};

export default ReportsPage;
