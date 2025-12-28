/**
 * CaseTable Component - Table displaying cases
 */

import React from 'react';
import { useNavigate } from 'react-router-dom';
import { CaseData } from '../../services/api';
import Button from '../Common/Button';

export interface CaseTableProps {
  cases: CaseData[];
  onEdit?: (caseData: CaseData) => void;
  onDelete?: (id: string) => void;
  onStatusChange?: (id: string, status: string) => void;
  isLoading?: boolean;
}

export const CaseTable: React.FC<CaseTableProps> = ({
  cases,
  onEdit,
  onDelete,
  onStatusChange,
  isLoading = false,
}) => {
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

  if (isLoading) {
    return <div className="table-loading">Loading cases...</div>;
  }

  if (cases.length === 0) {
    return <div className="table-empty">No cases found</div>;
  }

  return (
    <div className="table-container">
      <table className="table">
        <thead>
          <tr>
            <th>Case Number</th>
            <th>Title</th>
            <th>Client</th>
            <th>Status</th>
            <th>Priority</th>
            <th>Assigned To</th>
            <th>Created</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {cases.map((caseData) => (
            <tr key={caseData.id} className="table-row">
              <td>
                <span className="table-case-number">{caseData.caseNumber}</span>
              </td>
              <td>
                <button
                  className="table-link"
                  onClick={() => navigate(`/cases/${caseData.id}`)}
                >
                  {caseData.title}
                </button>
              </td>
              <td>{caseData.clientName || 'N/A'}</td>
              <td>
                <span className={`table-badge ${getStatusClass(caseData.status)}`}>
                  {caseData.status?.replace('_', ' ')}
                </span>
              </td>
              <td>
                <span className={`table-badge ${getPriorityClass(caseData.priority)}`}>
                  {caseData.priority}
                </span>
              </td>
              <td>{caseData.assignedTo || 'Unassigned'}</td>
              <td>{formatDate(caseData.createdAt)}</td>
              <td>
                <div className="table-actions">
                  <Button
                    size="small"
                    variant="outline"
                    onClick={() => navigate(`/cases/${caseData.id}`)}
                  >
                    View
                  </Button>
                  {onEdit && (
                    <Button
                      size="small"
                      variant="outline"
                      onClick={() => onEdit(caseData)}
                    >
                      Edit
                    </Button>
                  )}
                  {onDelete && (
                    <Button
                      size="small"
                      variant="danger"
                      onClick={() => caseData.id && onDelete(caseData.id)}
                    >
                      Delete
                    </Button>
                  )}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default CaseTable;
