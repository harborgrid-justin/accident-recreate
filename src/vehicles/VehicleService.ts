/**
 * Vehicle Service - Main CRUD Operations
 * AccuScene Enterprise - Accident Recreation Platform
 */

import { VehicleType, VehicleDimensions, VehicleWeightSpec, getDefaultSpecsForType } from './VehicleTypes';
import { getVehicleSpec, createCustomVehicleSpec } from './VehicleSpecs';
import { VehicleDamage, createVehicleDamage } from './DamageModel';
import { Occupant } from './OccupantInfo';

export interface Vehicle {
  id: string;
  accidentId: string;

  // Basic Information
  type: VehicleType;
  year?: number;
  make?: string;
  model?: string;
  color?: string;
  licensePlate?: string;
  vin?: string;

  // Specifications
  dimensions: VehicleDimensions;
  weight: VehicleWeightSpec;

  // Driver Information
  driverName?: string;
  driverLicenseNumber?: string;
  insuranceCompany?: string;
  insurancePolicyNumber?: string;

  // Position and Orientation (for recreation)
  position?: {
    x: number; // feet from origin
    y: number; // feet from origin
  };
  heading?: number; // degrees (0 = North)
  speed?: number; // mph at time of impact

  // Damage Information
  damage?: VehicleDamage;

  // Occupants
  occupants: Occupant[];

  // Metadata
  notes?: string;
  photoIds: string[];
  createdAt: Date;
  updatedAt: Date;
}

/**
 * In-memory vehicle storage (replace with database in production)
 */
class VehicleStore {
  private vehicles: Map<string, Vehicle> = new Map();
  private accidentIndex: Map<string, Set<string>> = new Map();

  save(vehicle: Vehicle): void {
    this.vehicles.set(vehicle.id, vehicle);

    // Update accident index
    if (!this.accidentIndex.has(vehicle.accidentId)) {
      this.accidentIndex.set(vehicle.accidentId, new Set());
    }
    this.accidentIndex.get(vehicle.accidentId)!.add(vehicle.id);
  }

  get(id: string): Vehicle | null {
    return this.vehicles.get(id) || null;
  }

  delete(id: string): boolean {
    const vehicle = this.vehicles.get(id);
    if (!vehicle) {
      return false;
    }

    // Remove from accident index
    const accidentVehicles = this.accidentIndex.get(vehicle.accidentId);
    if (accidentVehicles) {
      accidentVehicles.delete(id);
      if (accidentVehicles.size === 0) {
        this.accidentIndex.delete(vehicle.accidentId);
      }
    }

    return this.vehicles.delete(id);
  }

  getByAccidentId(accidentId: string): Vehicle[] {
    const vehicleIds = this.accidentIndex.get(accidentId);
    if (!vehicleIds) {
      return [];
    }

    return Array.from(vehicleIds)
      .map(id => this.vehicles.get(id))
      .filter((v): v is Vehicle => v !== undefined);
  }

  clear(): void {
    this.vehicles.clear();
    this.accidentIndex.clear();
  }

  getAll(): Vehicle[] {
    return Array.from(this.vehicles.values());
  }
}

const store = new VehicleStore();

/**
 * Create vehicle data
 */
export interface CreateVehicleData {
  type: VehicleType;
  year?: number;
  make?: string;
  model?: string;
  color?: string;
  licensePlate?: string;
  vin?: string;
  driverName?: string;
  driverLicenseNumber?: string;
  insuranceCompany?: string;
  insurancePolicyNumber?: string;
  position?: { x: number; y: number };
  heading?: number;
  speed?: number;
  notes?: string;
  dimensions?: Partial<VehicleDimensions>;
  weight?: Partial<VehicleWeightSpec>;
}

/**
 * Update vehicle data
 */
export type UpdateVehicleData = Partial<CreateVehicleData>;

/**
 * Create a new vehicle
 */
export function createVehicle(accidentId: string, data: CreateVehicleData): Vehicle {
  const id = `veh_${Date.now()}_${Math.random().toString(36).substring(7)}`;

  // Get specifications
  let dimensions: VehicleDimensions;
  let weight: VehicleWeightSpec;

  if (data.year && data.make && data.model) {
    // Try to find exact specifications
    const spec = getVehicleSpec(data.year, data.make, data.model);
    if (spec) {
      dimensions = { ...spec.dimensions, ...data.dimensions };
      weight = { ...spec.weight, ...data.weight };
    } else {
      // Create custom spec
      const customSpec = createCustomVehicleSpec(
        data.year,
        data.make,
        data.model,
        data.type,
        data.dimensions,
        data.weight
      );
      dimensions = customSpec.dimensions;
      weight = customSpec.weight;
    }
  } else {
    // Use default specs for type
    const defaultSpec = getDefaultSpecsForType(data.type);
    dimensions = { ...defaultSpec.dimensions, ...data.dimensions };
    weight = { ...defaultSpec.weight, ...data.weight };
  }

  const vehicle: Vehicle = {
    id,
    accidentId,
    type: data.type,
    year: data.year,
    make: data.make,
    model: data.model,
    color: data.color,
    licensePlate: data.licensePlate,
    vin: data.vin,
    dimensions,
    weight,
    driverName: data.driverName,
    driverLicenseNumber: data.driverLicenseNumber,
    insuranceCompany: data.insuranceCompany,
    insurancePolicyNumber: data.insurancePolicyNumber,
    position: data.position,
    heading: data.heading,
    speed: data.speed,
    damage: createVehicleDamage(id),
    occupants: [],
    notes: data.notes,
    photoIds: [],
    createdAt: new Date(),
    updatedAt: new Date(),
  };

  store.save(vehicle);
  return vehicle;
}

/**
 * Get vehicle by ID
 */
export function getVehicle(vehicleId: string): Vehicle | null {
  return store.get(vehicleId);
}

/**
 * Update vehicle
 */
export function updateVehicle(vehicleId: string, updates: UpdateVehicleData): Vehicle | null {
  const vehicle = store.get(vehicleId);
  if (!vehicle) {
    return null;
  }

  // Update dimensions if provided
  let dimensions = vehicle.dimensions;
  if (updates.dimensions) {
    dimensions = { ...dimensions, ...updates.dimensions };
  }

  // Update weight if provided
  let weight = vehicle.weight;
  if (updates.weight) {
    weight = { ...weight, ...updates.weight };
  }

  // If year/make/model changed, try to get updated specs
  if (
    (updates.year && updates.year !== vehicle.year) ||
    (updates.make && updates.make !== vehicle.make) ||
    (updates.model && updates.model !== vehicle.model)
  ) {
    const year = updates.year || vehicle.year;
    const make = updates.make || vehicle.make;
    const model = updates.model || vehicle.model;

    if (year && make && model) {
      const spec = getVehicleSpec(year, make, model);
      if (spec) {
        dimensions = { ...spec.dimensions, ...updates.dimensions };
        weight = { ...spec.weight, ...updates.weight };
      }
    }
  }

  const updatedVehicle: Vehicle = {
    ...vehicle,
    type: updates.type ?? vehicle.type,
    year: updates.year ?? vehicle.year,
    make: updates.make ?? vehicle.make,
    model: updates.model ?? vehicle.model,
    color: updates.color ?? vehicle.color,
    licensePlate: updates.licensePlate ?? vehicle.licensePlate,
    vin: updates.vin ?? vehicle.vin,
    dimensions,
    weight,
    driverName: updates.driverName ?? vehicle.driverName,
    driverLicenseNumber: updates.driverLicenseNumber ?? vehicle.driverLicenseNumber,
    insuranceCompany: updates.insuranceCompany ?? vehicle.insuranceCompany,
    insurancePolicyNumber: updates.insurancePolicyNumber ?? vehicle.insurancePolicyNumber,
    position: updates.position ?? vehicle.position,
    heading: updates.heading ?? vehicle.heading,
    speed: updates.speed ?? vehicle.speed,
    notes: updates.notes ?? vehicle.notes,
    updatedAt: new Date(),
  };

  store.save(updatedVehicle);
  return updatedVehicle;
}

/**
 * Delete vehicle
 */
export function deleteVehicle(vehicleId: string): boolean {
  return store.delete(vehicleId);
}

/**
 * Get all vehicles for an accident
 */
export function getVehiclesByAccident(accidentId: string): Vehicle[] {
  return store.getByAccidentId(accidentId);
}

/**
 * Add photo to vehicle
 */
export function addPhotoToVehicle(vehicleId: string, photoId: string): Vehicle | null {
  const vehicle = store.get(vehicleId);
  if (!vehicle) {
    return null;
  }

  const updatedVehicle: Vehicle = {
    ...vehicle,
    photoIds: [...vehicle.photoIds, photoId],
    updatedAt: new Date(),
  };

  store.save(updatedVehicle);
  return updatedVehicle;
}

/**
 * Remove photo from vehicle
 */
export function removePhotoFromVehicle(vehicleId: string, photoId: string): Vehicle | null {
  const vehicle = store.get(vehicleId);
  if (!vehicle) {
    return null;
  }

  const updatedVehicle: Vehicle = {
    ...vehicle,
    photoIds: vehicle.photoIds.filter(id => id !== photoId),
    updatedAt: new Date(),
  };

  store.save(updatedVehicle);
  return updatedVehicle;
}

/**
 * Update vehicle damage
 */
export function updateVehicleDamage(vehicleId: string, damage: VehicleDamage): Vehicle | null {
  const vehicle = store.get(vehicleId);
  if (!vehicle) {
    return null;
  }

  const updatedVehicle: Vehicle = {
    ...vehicle,
    damage,
    updatedAt: new Date(),
  };

  store.save(updatedVehicle);
  return updatedVehicle;
}

/**
 * Add occupant to vehicle
 */
export function addOccupantToVehicle(vehicleId: string, occupant: Occupant): Vehicle | null {
  const vehicle = store.get(vehicleId);
  if (!vehicle) {
    return null;
  }

  const updatedVehicle: Vehicle = {
    ...vehicle,
    occupants: [...vehicle.occupants, occupant],
    updatedAt: new Date(),
  };

  store.save(updatedVehicle);
  return updatedVehicle;
}

/**
 * Update occupant in vehicle
 */
export function updateOccupantInVehicle(
  vehicleId: string,
  occupantId: string,
  occupant: Occupant
): Vehicle | null {
  const vehicle = store.get(vehicleId);
  if (!vehicle) {
    return null;
  }

  const updatedVehicle: Vehicle = {
    ...vehicle,
    occupants: vehicle.occupants.map(o => (o.id === occupantId ? occupant : o)),
    updatedAt: new Date(),
  };

  store.save(updatedVehicle);
  return updatedVehicle;
}

/**
 * Remove occupant from vehicle
 */
export function removeOccupantFromVehicle(vehicleId: string, occupantId: string): Vehicle | null {
  const vehicle = store.get(vehicleId);
  if (!vehicle) {
    return null;
  }

  const updatedVehicle: Vehicle = {
    ...vehicle,
    occupants: vehicle.occupants.filter(o => o.id !== occupantId),
    updatedAt: new Date(),
  };

  store.save(updatedVehicle);
  return updatedVehicle;
}

/**
 * Update vehicle position
 */
export function updateVehiclePosition(
  vehicleId: string,
  x: number,
  y: number,
  heading?: number
): Vehicle | null {
  const vehicle = store.get(vehicleId);
  if (!vehicle) {
    return null;
  }

  const updatedVehicle: Vehicle = {
    ...vehicle,
    position: { x, y },
    heading: heading ?? vehicle.heading,
    updatedAt: new Date(),
  };

  store.save(updatedVehicle);
  return updatedVehicle;
}

/**
 * Get vehicle summary
 */
export function getVehicleSummary(vehicle: Vehicle): string {
  const parts: string[] = [];

  if (vehicle.year || vehicle.make || vehicle.model) {
    const yearMakeModel = [vehicle.year, vehicle.make, vehicle.model]
      .filter(Boolean)
      .join(' ');
    parts.push(yearMakeModel);
  } else {
    parts.push(vehicle.type);
  }

  if (vehicle.color) {
    parts.push(vehicle.color);
  }

  if (vehicle.licensePlate) {
    parts.push(`(${vehicle.licensePlate})`);
  }

  return parts.join(' ');
}

/**
 * Validate vehicle data
 */
export function validateVehicle(vehicle: Vehicle): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  if (!vehicle.accidentId) {
    errors.push('Accident ID is required');
  }

  if (!vehicle.type) {
    errors.push('Vehicle type is required');
  }

  if (vehicle.year && (vehicle.year < 1900 || vehicle.year > new Date().getFullYear() + 1)) {
    errors.push('Invalid vehicle year');
  }

  if (vehicle.speed !== undefined && vehicle.speed < 0) {
    errors.push('Speed cannot be negative');
  }

  if (vehicle.heading !== undefined && (vehicle.heading < 0 || vehicle.heading >= 360)) {
    errors.push('Heading must be between 0 and 359 degrees');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Clear all vehicles (for testing)
 */
export function clearAllVehicles(): void {
  store.clear();
}

/**
 * Get all vehicles (for testing/debugging)
 */
export function getAllVehicles(): Vehicle[] {
  return store.getAll();
}

/**
 * Export vehicle data as JSON
 */
export function exportVehicleData(vehicleId: string): string {
  const vehicle = store.get(vehicleId);
  if (!vehicle) {
    throw new Error(`Vehicle ${vehicleId} not found`);
  }

  return JSON.stringify(vehicle, null, 2);
}

/**
 * Import vehicle data from JSON
 */
export function importVehicleData(jsonData: string): Vehicle {
  const vehicle = JSON.parse(jsonData) as Vehicle;

  // Convert date strings to Date objects
  vehicle.createdAt = new Date(vehicle.createdAt);
  vehicle.updatedAt = new Date(vehicle.updatedAt);

  if (vehicle.damage) {
    vehicle.damage.createdAt = new Date(vehicle.damage.createdAt);
    vehicle.damage.updatedAt = new Date(vehicle.damage.updatedAt);
  }

  vehicle.occupants.forEach(occupant => {
    occupant.createdAt = new Date(occupant.createdAt);
    occupant.updatedAt = new Date(occupant.updatedAt);
  });

  store.save(vehicle);
  return vehicle;
}
