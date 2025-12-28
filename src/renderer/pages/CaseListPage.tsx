/**
 * CaseListPage - List all cases with filtering and search
 */

import React, { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useCases } from '../hooks/useCases';
import { useUIStore } from '../store/uiStore';
import CaseTable from '../components/Tables/CaseTable';
import Select from '../components/Common/Select';
import Input from '../components/Common/Input';
import Button from '../components/Common/Button';
import Loading from '../components/Common/Loading';
import ConfirmModal from '../components/Modals/ConfirmModal';

export const CaseListPage: React.FC = () => {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const { cases, isLoading, error, fetchCases, deleteCase, setFilters } = useCases();
  const { showNotification } = useUIStore();

  const [searchTerm, setSearchTerm] = useState(searchParams.get('search') || '');
  const [statusFilter, setStatusFilter] = useState(searchParams.get('status') || '');
  const [priorityFilter, setPriorityFilter] = useState(searchParams.get('priority') || '');
  const [deleteConfirm, setDeleteConfirm] = useState<{ id: string; title: string } | null>(null);

  useEffect(() => {
    loadCases();
  }, [statusFilter, priorityFilter]);

  const loadCases = () => {
    const filters: any = {};
    if (statusFilter) filters.status = statusFilter;
    if (priorityFilter) filters.priority = priorityFilter;
    if (searchTerm) filters.search = searchTerm;

    setFilters(filters);
    fetchCases(filters);
  };

  const handleSearch = () => {
    loadCases();
    updateSearchParams();
  };

  const updateSearchParams = () => {
    const params: any = {};
    if (searchTerm) params.search = searchTerm;
    if (statusFilter) params.status = statusFilter;
    if (priorityFilter) params.priority = priorityFilter;
    setSearchParams(params);
  };

  const handleDelete = async () => {
    if (!deleteConfirm) return;

    const success = await deleteCase(deleteConfirm.id);
    if (success) {
      showNotification('success', 'Case deleted successfully');
      setDeleteConfirm(null);
    } else {
      showNotification('error', 'Failed to delete case');
    }
  };

  const handleClearFilters = () => {
    setSearchTerm('');
    setStatusFilter('');
    setPriorityFilter('');
    setSearchParams({});
    setFilters({});
    fetchCases({});
  };

  return (
    <div className="page case-list-page">
      <div className="page-header">
        <div className="page-title-section">
          <h1 className="page-title">Cases</h1>
          <p className="page-subtitle">Manage all accident investigation cases</p>
        </div>
        <div className="page-actions">
          <Button variant="primary" onClick={() => navigate('/cases/new')}>
            + New Case
          </Button>
        </div>
      </div>

      <div className="case-list-filters">
        <div className="case-list-filters-row">
          <Input
            type="search"
            placeholder="Search cases..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
          />

          <Select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            options={[
              { value: '', label: 'All Statuses' },
              { value: 'draft', label: 'Draft' },
              { value: 'active', label: 'Active' },
              { value: 'under_review', label: 'Under Review' },
              { value: 'pending_approval', label: 'Pending Approval' },
              { value: 'completed', label: 'Completed' },
              { value: 'archived', label: 'Archived' },
              { value: 'closed', label: 'Closed' },
            ]}
          />

          <Select
            value={priorityFilter}
            onChange={(e) => setPriorityFilter(e.target.value)}
            options={[
              { value: '', label: 'All Priorities' },
              { value: 'low', label: 'Low' },
              { value: 'medium', label: 'Medium' },
              { value: 'high', label: 'High' },
              { value: 'critical', label: 'Critical' },
            ]}
          />

          <Button variant="primary" onClick={handleSearch}>
            Search
          </Button>

          {(searchTerm || statusFilter || priorityFilter) && (
            <Button variant="outline" onClick={handleClearFilters}>
              Clear Filters
            </Button>
          )}
        </div>
      </div>

      {error && (
        <div className="page-error">
          <p>{error}</p>
          <Button onClick={loadCases}>Retry</Button>
        </div>
      )}

      {isLoading ? (
        <Loading message="Loading cases..." />
      ) : (
        <>
          <div className="case-list-info">
            <p>Showing {cases.length} case(s)</p>
          </div>

          <CaseTable
            cases={cases}
            onEdit={(caseData) => navigate(`/cases/${caseData.id}/edit`)}
            onDelete={(id) => {
              const caseData = cases.find((c) => c.id === id);
              if (caseData) {
                setDeleteConfirm({ id, title: caseData.title });
              }
            }}
            isLoading={isLoading}
          />
        </>
      )}

      {deleteConfirm && (
        <ConfirmModal
          title="Delete Case"
          message={`Are you sure you want to delete "${deleteConfirm.title}"? This action cannot be undone.`}
          confirmText="Delete"
          cancelText="Cancel"
          variant="danger"
          onConfirm={handleDelete}
          onCancel={() => setDeleteConfirm(null)}
        />
      )}
    </div>
  );
};

export default CaseListPage;
