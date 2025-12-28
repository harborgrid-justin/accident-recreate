/**
 * VehicleModal Component - Modal for adding/editing vehicles
 */

import React from 'react';
import { VehicleData } from '../../services/api';
import VehicleForm from '../Forms/VehicleForm';

export interface VehicleModalProps {
  vehicle?: VehicleData;
  accidentId: string;
  vehicleNumber: number;
  onSubmit: (data: VehicleData) => Promise<void>;
  onClose: () => void;
  isLoading?: boolean;
}

export const VehicleModal: React.FC<VehicleModalProps> = ({
  vehicle,
  accidentId,
  vehicleNumber,
  onSubmit,
  onClose,
  isLoading = false,
}) => {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal vehicle-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{vehicle ? 'Edit Vehicle' : 'Add Vehicle'}</h2>
          <button className="modal-close" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>

        <div className="modal-body">
          <VehicleForm
            initialData={vehicle}
            accidentId={accidentId}
            vehicleNumber={vehicleNumber}
            onSubmit={onSubmit}
            onCancel={onClose}
            isLoading={isLoading}
          />
        </div>
      </div>
    </div>
  );
};

export default VehicleModal;
