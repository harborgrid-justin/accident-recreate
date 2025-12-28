/**
 * CaseDetailPage - Detailed view of a single case
 */

import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { api, CaseData, AccidentData, VehicleData } from '../services/api';
import { useUIStore } from '../store/uiStore';
import Button from '../components/Common/Button';
import Loading from '../components/Common/Loading';
import VehicleTable from '../components/Tables/VehicleTable';
import VehicleModal from '../components/Modals/VehicleModal';
import ConfirmModal from '../components/Modals/ConfirmModal';

export const CaseDetailPage: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { showNotification, setLoading } = useUIStore();

  const [caseData, setCaseData] = useState<CaseData | null>(null);
  const [accident, setAccident] = useState<AccidentData | null>(null);
  const [vehicles, setVehicles] = useState<VehicleData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showVehicleModal, setShowVehicleModal] = useState(false);
  const [editingVehicle, setEditingVehicle] = useState<VehicleData | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<{ id: string; type: string } | null>(null);

  useEffect(() => {
    if (id) {
      loadCaseData(id);
    }
  }, [id]);

  const loadCaseData = async (caseId: string) => {
    setIsLoading(true);
    setError(null);

    try {
      const [caseResponse, accidentResponse] = await Promise.all([
        api.getCase(caseId),
        api.getAccident(caseId),
      ]);

      if (caseResponse.success && caseResponse.data) {
        setCaseData(caseResponse.data);
      } else {
        setError(caseResponse.error || 'Failed to load case');
      }

      if (accidentResponse.success && accidentResponse.data) {
        setAccident(accidentResponse.data);
        if (accidentResponse.data.id) {
          loadVehicles(accidentResponse.data.id);
        }
      }
    } catch (err) {
      setError('Failed to load case data');
    } finally {
      setIsLoading(false);
    }
  };

  const loadVehicles = async (accidentId: string) => {
    const response = await api.getVehicles(accidentId);
    if (response.success && response.data) {
      setVehicles(response.data);
    }
  };

  const handleAddVehicle = async (vehicleData: VehicleData) => {
    setLoading(true);
    const response = await api.createVehicle(vehicleData);
    setLoading(false);

    if (response.success && response.data) {
      showNotification('success', 'Vehicle added successfully');
      setVehicles([...vehicles, response.data]);
      setShowVehicleModal(false);
    } else {
      showNotification('error', response.error || 'Failed to add vehicle');
    }
  };

  const handleEditVehicle = async (vehicleData: VehicleData) => {
    if (!editingVehicle?.id) return;

    setLoading(true);
    const response = await api.updateVehicle(editingVehicle.id, vehicleData);
    setLoading(false);

    if (response.success && response.data) {
      showNotification('success', 'Vehicle updated successfully');
      setVehicles(vehicles.map((v) => (v.id === editingVehicle.id ? response.data! : v)));
      setEditingVehicle(null);
      setShowVehicleModal(false);
    } else {
      showNotification('error', response.error || 'Failed to update vehicle');
    }
  };

  const handleDeleteVehicle = async () => {
    if (!deleteConfirm || deleteConfirm.type !== 'vehicle') return;

    setLoading(true);
    const response = await api.deleteVehicle(deleteConfirm.id);
    setLoading(false);

    if (response.success) {
      showNotification('success', 'Vehicle deleted successfully');
      setVehicles(vehicles.filter((v) => v.id !== deleteConfirm.id));
      setDeleteConfirm(null);
    } else {
      showNotification('error', response.error || 'Failed to delete vehicle');
    }
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return 'N/A';
    const date = new Date(dateString);
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  if (isLoading) {
    return (
      <div className="page">
        <Loading fullScreen message="Loading case details..." />
      </div>
    );
  }

  if (error || !caseData) {
    return (
      <div className="page">
        <div className="page-error">
          <h2>Error Loading Case</h2>
          <p>{error || 'Case not found'}</p>
          <Button onClick={() => navigate('/cases')}>Back to Cases</Button>
        </div>
      </div>
    );
  }

  return (
    <div className="page case-detail-page">
      <div className="page-header">
        <div className="page-title-section">
          <div className="page-breadcrumb">
            <button className="breadcrumb-link" onClick={() => navigate('/cases')}>
              Cases
            </button>
            <span className="breadcrumb-separator">/</span>
            <span>{caseData.caseNumber}</span>
          </div>
          <h1 className="page-title">{caseData.title}</h1>
          <div className="page-meta">
            <span className={`badge status-${caseData.status}`}>{caseData.status}</span>
            <span className={`badge priority-${caseData.priority}`}>{caseData.priority}</span>
          </div>
        </div>
        <div className="page-actions">
          <Button variant="outline" onClick={() => navigate(`/cases/${id}/edit`)}>
            Edit Case
          </Button>
          <Button variant="primary" onClick={() => navigate(`/editor/${id}`)}>
            Open Editor
          </Button>
          <Button variant="secondary" onClick={() => navigate(`/reports?caseId=${id}`)}>
            Generate Report
          </Button>
        </div>
      </div>

      <div className="case-detail-content">
        <div className="case-detail-section">
          <h2>Case Information</h2>
          <div className="case-detail-grid">
            <div className="case-detail-item">
              <label>Case Number</label>
              <span>{caseData.caseNumber}</span>
            </div>
            <div className="case-detail-item">
              <label>Status</label>
              <span className={`badge status-${caseData.status}`}>{caseData.status}</span>
            </div>
            <div className="case-detail-item">
              <label>Priority</label>
              <span className={`badge priority-${caseData.priority}`}>{caseData.priority}</span>
            </div>
            <div className="case-detail-item">
              <label>Created</label>
              <span>{formatDate(caseData.createdAt)}</span>
            </div>
            <div className="case-detail-item">
              <label>Last Updated</label>
              <span>{formatDate(caseData.updatedAt)}</span>
            </div>
            <div className="case-detail-item">
              <label>Assigned To</label>
              <span>{caseData.assignedTo || 'Unassigned'}</span>
            </div>
          </div>
          {caseData.description && (
            <div className="case-detail-description">
              <label>Description</label>
              <p>{caseData.description}</p>
            </div>
          )}
        </div>

        <div className="case-detail-section">
          <h2>Client Information</h2>
          <div className="case-detail-grid">
            <div className="case-detail-item">
              <label>Client Name</label>
              <span>{caseData.clientName || 'N/A'}</span>
            </div>
            <div className="case-detail-item">
              <label>Phone</label>
              <span>{caseData.clientPhone || 'N/A'}</span>
            </div>
            <div className="case-detail-item">
              <label>Email</label>
              <span>{caseData.clientEmail || 'N/A'}</span>
            </div>
          </div>
        </div>

        {accident && (
          <div className="case-detail-section">
            <h2>Accident Details</h2>
            <div className="case-detail-grid">
              <div className="case-detail-item">
                <label>Date & Time</label>
                <span>{formatDate(accident.dateTime)}</span>
              </div>
              <div className="case-detail-item">
                <label>Location</label>
                <span>{accident.location}</span>
              </div>
              <div className="case-detail-item">
                <label>Weather</label>
                <span>{accident.weather}</span>
              </div>
              <div className="case-detail-item">
                <label>Road Conditions</label>
                <span>{accident.roadConditions}</span>
              </div>
              <div className="case-detail-item">
                <label>Light Conditions</label>
                <span>{accident.lightConditions}</span>
              </div>
              <div className="case-detail-item">
                <label>Injuries</label>
                <span>{accident.injuries || 0}</span>
              </div>
              <div className="case-detail-item">
                <label>Fatalities</label>
                <span>{accident.fatalities || 0}</span>
              </div>
              <div className="case-detail-item">
                <label>Police Report #</label>
                <span>{accident.policeReportNumber || 'N/A'}</span>
              </div>
            </div>
            {accident.description && (
              <div className="case-detail-description">
                <label>Description</label>
                <p>{accident.description}</p>
              </div>
            )}
          </div>
        )}

        <div className="case-detail-section">
          <div className="case-detail-section-header">
            <h2>Vehicles ({vehicles.length})</h2>
            <Button
              variant="primary"
              size="small"
              onClick={() => {
                setEditingVehicle(null);
                setShowVehicleModal(true);
              }}
            >
              + Add Vehicle
            </Button>
          </div>
          <VehicleTable
            vehicles={vehicles}
            onEdit={(vehicle) => {
              setEditingVehicle(vehicle);
              setShowVehicleModal(true);
            }}
            onDelete={(vehicleId) => setDeleteConfirm({ id: vehicleId, type: 'vehicle' })}
          />
        </div>

        {caseData.notes && (
          <div className="case-detail-section">
            <h2>Notes</h2>
            <div className="case-detail-notes">
              <p>{caseData.notes}</p>
            </div>
          </div>
        )}
      </div>

      {showVehicleModal && accident?.id && (
        <VehicleModal
          vehicle={editingVehicle || undefined}
          accidentId={accident.id}
          vehicleNumber={vehicles.length + 1}
          onSubmit={editingVehicle ? handleEditVehicle : handleAddVehicle}
          onClose={() => {
            setShowVehicleModal(false);
            setEditingVehicle(null);
          }}
        />
      )}

      {deleteConfirm && (
        <ConfirmModal
          title={`Delete ${deleteConfirm.type}`}
          message={`Are you sure you want to delete this ${deleteConfirm.type}? This action cannot be undone.`}
          confirmText="Delete"
          variant="danger"
          onConfirm={handleDeleteVehicle}
          onCancel={() => setDeleteConfirm(null)}
        />
      )}
    </div>
  );
};

export default CaseDetailPage;
