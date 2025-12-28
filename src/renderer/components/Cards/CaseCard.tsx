/**
 * CaseCard Component - Card displaying case summary
 */

import React from 'react';
import { useNavigate } from 'react-router-dom';
import { CaseData } from '../../services/api';

export interface CaseCardProps {
  caseData: CaseData;
  onClick?: () => void;
}

export const CaseCard: React.FC<CaseCardProps> = ({ caseData, onClick }) => {
  const navigate = useNavigate();

  const getStatusClass = (status: string = '') => {
    const statusMap: Record<string, string> = {
      draft: 'status-draft',
      active: 'status-active',
      under_review: 'status-review',
      pending_approval: 'status-pending',
      completed: 'status-completed',
      archived: 'status-archived',
      closed: 'status-closed',
    };
    return statusMap[status] || 'status-default';
  };

  const getPriorityClass = (priority: string = '') => {
    const priorityMap: Record<string, string> = {
      low: 'priority-low',
      medium: 'priority-medium',
      high: 'priority-high',
      critical: 'priority-critical',
    };
    return priorityMap[priority] || 'priority-default';
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return 'N/A';
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  };

  const handleClick = () => {
    if (onClick) {
      onClick();
    } else if (caseData.id) {
      navigate(`/cases/${caseData.id}`);
    }
  };

  return (
    <div className="case-card" onClick={handleClick}>
      <div className="case-card-header">
        <div className="case-card-number">{caseData.caseNumber}</div>
        <div className="case-card-badges">
          <span className={`case-card-badge ${getPriorityClass(caseData.priority)}`}>
            {caseData.priority}
          </span>
          <span className={`case-card-badge ${getStatusClass(caseData.status)}`}>
            {caseData.status?.replace('_', ' ')}
          </span>
        </div>
      </div>

      <div className="case-card-body">
        <h3 className="case-card-title">{caseData.title}</h3>
        {caseData.description && (
          <p className="case-card-description">
            {caseData.description.length > 100
              ? `${caseData.description.substring(0, 100)}...`
              : caseData.description}
          </p>
        )}
      </div>

      <div className="case-card-footer">
        <div className="case-card-info">
          <div className="case-card-info-item">
            <span className="case-card-label">Client:</span>
            <span className="case-card-value">{caseData.clientName || 'N/A'}</span>
          </div>
          <div className="case-card-info-item">
            <span className="case-card-label">Assigned:</span>
            <span className="case-card-value">{caseData.assignedTo || 'Unassigned'}</span>
          </div>
          <div className="case-card-info-item">
            <span className="case-card-label">Created:</span>
            <span className="case-card-value">{formatDate(caseData.createdAt)}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CaseCard;
