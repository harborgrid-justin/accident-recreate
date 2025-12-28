/**
 * VehicleTable Component - Table displaying vehicles
 */

import React from 'react';
import { VehicleData } from '../../services/api';
import Button from '../Common/Button';

export interface VehicleTableProps {
  vehicles: VehicleData[];
  onEdit?: (vehicle: VehicleData) => void;
  onDelete?: (id: string) => void;
  isLoading?: boolean;
}

export const VehicleTable: React.FC<VehicleTableProps> = ({
  vehicles,
  onEdit,
  onDelete,
  isLoading = false,
}) => {
  const getDamageSeverityClass = (severity: string = '') => {
    const severityMap: Record<string, string> = {
      none: 'damage-none',
      minor: 'damage-minor',
      moderate: 'damage-moderate',
      severe: 'damage-severe',
      totaled: 'damage-totaled',
    };
    return severityMap[severity] || 'damage-default';
  };

  if (isLoading) {
    return <div className="table-loading">Loading vehicles...</div>;
  }

  if (vehicles.length === 0) {
    return <div className="table-empty">No vehicles added yet</div>;
  }

  return (
    <div className="table-container">
      <table className="table">
        <thead>
          <tr>
            <th>Vehicle #</th>
            <th>Type</th>
            <th>Make/Model</th>
            <th>Year</th>
            <th>License Plate</th>
            <th>Driver</th>
            <th>Speed</th>
            <th>Damage</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {vehicles.map((vehicle) => (
            <tr key={vehicle.id} className="table-row">
              <td>
                <strong>Vehicle {vehicle.vehicleNumber}</strong>
              </td>
              <td>{vehicle.type}</td>
              <td>
                {vehicle.make} {vehicle.model}
              </td>
              <td>{vehicle.year || 'N/A'}</td>
              <td>{vehicle.licensePlate || 'N/A'}</td>
              <td>{vehicle.driverName}</td>
              <td>{vehicle.speed ? `${vehicle.speed} MPH` : 'N/A'}</td>
              <td>
                <span className={`table-badge ${getDamageSeverityClass(vehicle.damageSeverity)}`}>
                  {vehicle.damageSeverity}
                </span>
              </td>
              <td>
                <div className="table-actions">
                  {onEdit && (
                    <Button
                      size="small"
                      variant="outline"
                      onClick={() => onEdit(vehicle)}
                    >
                      Edit
                    </Button>
                  )}
                  {onDelete && vehicle.id && (
                    <Button
                      size="small"
                      variant="danger"
                      onClick={() => onDelete(vehicle.id!)}
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

export default VehicleTable;
