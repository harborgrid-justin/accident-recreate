/**
 * Vehicle Types and Classifications
 * AccuScene Enterprise - Accident Recreation Platform
 */

export enum VehicleType {
  SEDAN = 'SEDAN',
  SUV = 'SUV',
  TRUCK = 'TRUCK',
  VAN = 'VAN',
  MOTORCYCLE = 'MOTORCYCLE',
  BICYCLE = 'BICYCLE',
  PEDESTRIAN = 'PEDESTRIAN',
  COMMERCIAL = 'COMMERCIAL',
}

export interface VehicleDimensions {
  length: number; // feet
  width: number; // feet
  height: number; // feet
  wheelbase: number; // feet
  groundClearance: number; // inches
}

export interface VehicleWeightSpec {
  curb: number; // pounds
  gross: number; // pounds
  distribution: {
    front: number; // percentage
    rear: number; // percentage
  };
}

export interface VehicleTypeSpec {
  type: VehicleType;
  dimensions: VehicleDimensions;
  weight: VehicleWeightSpec;
  passengerCapacity: number;
  description: string;
}

/**
 * Default specifications for each vehicle type
 */
export const DEFAULT_VEHICLE_SPECS: Record<VehicleType, VehicleTypeSpec> = {
  [VehicleType.SEDAN]: {
    type: VehicleType.SEDAN,
    dimensions: {
      length: 15.5,
      width: 6.0,
      height: 4.8,
      wheelbase: 9.0,
      groundClearance: 5.5,
    },
    weight: {
      curb: 3500,
      gross: 4600,
      distribution: {
        front: 55,
        rear: 45,
      },
    },
    passengerCapacity: 5,
    description: 'Standard 4-door sedan',
  },
  [VehicleType.SUV]: {
    type: VehicleType.SUV,
    dimensions: {
      length: 16.5,
      width: 6.5,
      height: 6.0,
      wheelbase: 9.5,
      groundClearance: 8.0,
    },
    weight: {
      curb: 4500,
      gross: 6000,
      distribution: {
        front: 52,
        rear: 48,
      },
    },
    passengerCapacity: 7,
    description: 'Sport Utility Vehicle',
  },
  [VehicleType.TRUCK]: {
    type: VehicleType.TRUCK,
    dimensions: {
      length: 19.0,
      width: 6.7,
      height: 6.3,
      wheelbase: 11.0,
      groundClearance: 9.0,
    },
    weight: {
      curb: 5000,
      gross: 7000,
      distribution: {
        front: 58,
        rear: 42,
      },
    },
    passengerCapacity: 5,
    description: 'Pickup truck',
  },
  [VehicleType.VAN]: {
    type: VehicleType.VAN,
    dimensions: {
      length: 17.0,
      width: 6.5,
      height: 6.5,
      wheelbase: 10.0,
      groundClearance: 6.0,
    },
    weight: {
      curb: 4200,
      gross: 6200,
      distribution: {
        front: 50,
        rear: 50,
      },
    },
    passengerCapacity: 8,
    description: 'Passenger van or minivan',
  },
  [VehicleType.MOTORCYCLE]: {
    type: VehicleType.MOTORCYCLE,
    dimensions: {
      length: 7.0,
      width: 2.5,
      height: 4.0,
      wheelbase: 4.5,
      groundClearance: 5.0,
    },
    weight: {
      curb: 500,
      gross: 900,
      distribution: {
        front: 48,
        rear: 52,
      },
    },
    passengerCapacity: 2,
    description: 'Motorcycle',
  },
  [VehicleType.BICYCLE]: {
    type: VehicleType.BICYCLE,
    dimensions: {
      length: 5.5,
      width: 1.5,
      height: 3.5,
      wheelbase: 3.5,
      groundClearance: 12.0,
    },
    weight: {
      curb: 25,
      gross: 250,
      distribution: {
        front: 45,
        rear: 55,
      },
    },
    passengerCapacity: 1,
    description: 'Bicycle',
  },
  [VehicleType.PEDESTRIAN]: {
    type: VehicleType.PEDESTRIAN,
    dimensions: {
      length: 1.5,
      width: 1.5,
      height: 5.5,
      wheelbase: 0,
      groundClearance: 0,
    },
    weight: {
      curb: 170,
      gross: 170,
      distribution: {
        front: 50,
        rear: 50,
      },
    },
    passengerCapacity: 1,
    description: 'Pedestrian',
  },
  [VehicleType.COMMERCIAL]: {
    type: VehicleType.COMMERCIAL,
    dimensions: {
      length: 25.0,
      width: 8.0,
      height: 10.0,
      wheelbase: 15.0,
      groundClearance: 12.0,
    },
    weight: {
      curb: 12000,
      gross: 26000,
      distribution: {
        front: 35,
        rear: 65,
      },
    },
    passengerCapacity: 2,
    description: 'Commercial truck or box truck',
  },
};

/**
 * Get default specifications for a vehicle type
 */
export function getDefaultSpecsForType(type: VehicleType): VehicleTypeSpec {
  return { ...DEFAULT_VEHICLE_SPECS[type] };
}

/**
 * Get all available vehicle types
 */
export function getAllVehicleTypes(): VehicleType[] {
  return Object.values(VehicleType);
}

/**
 * Get human-readable name for vehicle type
 */
export function getVehicleTypeName(type: VehicleType): string {
  const names: Record<VehicleType, string> = {
    [VehicleType.SEDAN]: 'Sedan',
    [VehicleType.SUV]: 'SUV',
    [VehicleType.TRUCK]: 'Pickup Truck',
    [VehicleType.VAN]: 'Van',
    [VehicleType.MOTORCYCLE]: 'Motorcycle',
    [VehicleType.BICYCLE]: 'Bicycle',
    [VehicleType.PEDESTRIAN]: 'Pedestrian',
    [VehicleType.COMMERCIAL]: 'Commercial Vehicle',
  };
  return names[type];
}
