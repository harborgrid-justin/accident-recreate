/**
 * Vehicle Specifications Database
 * AccuScene Enterprise - Accident Recreation Platform
 */

import { VehicleType, VehicleDimensions, VehicleWeightSpec } from './VehicleTypes';

export interface VehicleSpec {
  id: string;
  year: number;
  make: string;
  model: string;
  trim?: string;
  type: VehicleType;
  dimensions: VehicleDimensions;
  weight: VehicleWeightSpec;
  engineSize?: number; // liters
  horsePower?: number;
  fuelType?: 'GASOLINE' | 'DIESEL' | 'ELECTRIC' | 'HYBRID';
}

/**
 * Built-in vehicle specifications database
 */
const VEHICLE_SPECS_DATABASE: VehicleSpec[] = [
  // Toyota Models
  {
    id: 'toyota-camry-2020',
    year: 2020,
    make: 'Toyota',
    model: 'Camry',
    trim: 'LE',
    type: VehicleType.SEDAN,
    dimensions: {
      length: 16.0,
      width: 6.0,
      height: 4.7,
      wheelbase: 9.3,
      groundClearance: 5.5,
    },
    weight: {
      curb: 3310,
      gross: 4398,
      distribution: { front: 58, rear: 42 },
    },
    engineSize: 2.5,
    horsePower: 203,
    fuelType: 'GASOLINE',
  },
  {
    id: 'toyota-rav4-2021',
    year: 2021,
    make: 'Toyota',
    model: 'RAV4',
    type: VehicleType.SUV,
    dimensions: {
      length: 15.0,
      width: 6.0,
      height: 5.6,
      wheelbase: 8.7,
      groundClearance: 8.6,
    },
    weight: {
      curb: 3370,
      gross: 4750,
      distribution: { front: 56, rear: 44 },
    },
    engineSize: 2.5,
    horsePower: 203,
    fuelType: 'GASOLINE',
  },
  {
    id: 'toyota-tacoma-2022',
    year: 2022,
    make: 'Toyota',
    model: 'Tacoma',
    type: VehicleType.TRUCK,
    dimensions: {
      length: 17.6,
      width: 6.2,
      height: 5.9,
      wheelbase: 10.6,
      groundClearance: 9.4,
    },
    weight: {
      curb: 4425,
      gross: 6800,
      distribution: { front: 55, rear: 45 },
    },
    engineSize: 3.5,
    horsePower: 278,
    fuelType: 'GASOLINE',
  },

  // Honda Models
  {
    id: 'honda-accord-2021',
    year: 2021,
    make: 'Honda',
    model: 'Accord',
    type: VehicleType.SEDAN,
    dimensions: {
      length: 16.1,
      width: 6.0,
      height: 4.8,
      wheelbase: 9.0,
      groundClearance: 5.3,
    },
    weight: {
      curb: 3131,
      gross: 4387,
      distribution: { front: 60, rear: 40 },
    },
    engineSize: 1.5,
    horsePower: 192,
    fuelType: 'GASOLINE',
  },
  {
    id: 'honda-crv-2022',
    year: 2022,
    make: 'Honda',
    model: 'CR-V',
    type: VehicleType.SUV,
    dimensions: {
      length: 15.1,
      width: 6.0,
      height: 5.5,
      wheelbase: 8.7,
      groundClearance: 8.2,
    },
    weight: {
      curb: 3337,
      gross: 4662,
      distribution: { front: 59, rear: 41 },
    },
    engineSize: 1.5,
    horsePower: 190,
    fuelType: 'GASOLINE',
  },
  {
    id: 'honda-odyssey-2021',
    year: 2021,
    make: 'Honda',
    model: 'Odyssey',
    type: VehicleType.VAN,
    dimensions: {
      length: 16.9,
      width: 6.6,
      height: 5.7,
      wheelbase: 9.8,
      groundClearance: 4.5,
    },
    weight: {
      curb: 4593,
      gross: 6019,
      distribution: { front: 53, rear: 47 },
    },
    engineSize: 3.5,
    horsePower: 280,
    fuelType: 'GASOLINE',
  },

  // Ford Models
  {
    id: 'ford-f150-2021',
    year: 2021,
    make: 'Ford',
    model: 'F-150',
    type: VehicleType.TRUCK,
    dimensions: {
      length: 19.4,
      width: 6.7,
      height: 6.4,
      wheelbase: 11.6,
      groundClearance: 9.4,
    },
    weight: {
      curb: 4705,
      gross: 7050,
      distribution: { front: 58, rear: 42 },
    },
    engineSize: 3.5,
    horsePower: 400,
    fuelType: 'GASOLINE',
  },
  {
    id: 'ford-explorer-2022',
    year: 2022,
    make: 'Ford',
    model: 'Explorer',
    type: VehicleType.SUV,
    dimensions: {
      length: 16.5,
      width: 6.5,
      height: 5.8,
      wheelbase: 9.9,
      groundClearance: 8.0,
    },
    weight: {
      curb: 4345,
      gross: 6200,
      distribution: { front: 54, rear: 46 },
    },
    engineSize: 2.3,
    horsePower: 300,
    fuelType: 'GASOLINE',
  },
  {
    id: 'ford-fusion-2020',
    year: 2020,
    make: 'Ford',
    model: 'Fusion',
    type: VehicleType.SEDAN,
    dimensions: {
      length: 16.1,
      width: 6.0,
      height: 4.8,
      wheelbase: 9.1,
      groundClearance: 5.5,
    },
    weight: {
      curb: 3462,
      gross: 4662,
      distribution: { front: 59, rear: 41 },
    },
    engineSize: 2.5,
    horsePower: 175,
    fuelType: 'GASOLINE',
  },

  // Chevrolet Models
  {
    id: 'chevrolet-silverado-2021',
    year: 2021,
    make: 'Chevrolet',
    model: 'Silverado 1500',
    type: VehicleType.TRUCK,
    dimensions: {
      length: 19.3,
      width: 6.8,
      height: 6.4,
      wheelbase: 11.5,
      groundClearance: 8.9,
    },
    weight: {
      curb: 4520,
      gross: 7100,
      distribution: { front: 57, rear: 43 },
    },
    engineSize: 5.3,
    horsePower: 355,
    fuelType: 'GASOLINE',
  },
  {
    id: 'chevrolet-equinox-2022',
    year: 2022,
    make: 'Chevrolet',
    model: 'Equinox',
    type: VehicleType.SUV,
    dimensions: {
      length: 15.3,
      width: 6.0,
      height: 5.5,
      wheelbase: 8.9,
      groundClearance: 7.9,
    },
    weight: {
      curb: 3274,
      gross: 4750,
      distribution: { front: 60, rear: 40 },
    },
    engineSize: 1.5,
    horsePower: 170,
    fuelType: 'GASOLINE',
  },
  {
    id: 'chevrolet-malibu-2021',
    year: 2021,
    make: 'Chevrolet',
    model: 'Malibu',
    type: VehicleType.SEDAN,
    dimensions: {
      length: 16.3,
      width: 6.0,
      height: 4.8,
      wheelbase: 9.3,
      groundClearance: 5.0,
    },
    weight: {
      curb: 3221,
      gross: 4387,
      distribution: { front: 61, rear: 39 },
    },
    engineSize: 1.5,
    horsePower: 160,
    fuelType: 'GASOLINE',
  },

  // Tesla Models
  {
    id: 'tesla-model3-2022',
    year: 2022,
    make: 'Tesla',
    model: 'Model 3',
    type: VehicleType.SEDAN,
    dimensions: {
      length: 15.4,
      width: 6.1,
      height: 4.8,
      wheelbase: 9.5,
      groundClearance: 5.5,
    },
    weight: {
      curb: 3862,
      gross: 4960,
      distribution: { front: 47, rear: 53 },
    },
    horsePower: 283,
    fuelType: 'ELECTRIC',
  },
  {
    id: 'tesla-modely-2023',
    year: 2023,
    make: 'Tesla',
    model: 'Model Y',
    type: VehicleType.SUV,
    dimensions: {
      length: 15.6,
      width: 6.3,
      height: 5.4,
      wheelbase: 9.5,
      groundClearance: 6.6,
    },
    weight: {
      curb: 4416,
      gross: 5712,
      distribution: { front: 48, rear: 52 },
    },
    horsePower: 384,
    fuelType: 'ELECTRIC',
  },

  // Harley Davidson Motorcycles
  {
    id: 'harley-street750-2020',
    year: 2020,
    make: 'Harley-Davidson',
    model: 'Street 750',
    type: VehicleType.MOTORCYCLE,
    dimensions: {
      length: 7.5,
      width: 2.5,
      height: 3.8,
      wheelbase: 5.0,
      groundClearance: 5.9,
    },
    weight: {
      curb: 489,
      gross: 900,
      distribution: { front: 47, rear: 53 },
    },
    engineSize: 0.75,
    horsePower: 47,
    fuelType: 'GASOLINE',
  },
];

/**
 * Search for vehicle specifications
 */
export function searchVehicleSpecs(criteria: {
  year?: number;
  make?: string;
  model?: string;
  type?: VehicleType;
}): VehicleSpec[] {
  let results = [...VEHICLE_SPECS_DATABASE];

  if (criteria.year) {
    results = results.filter(spec => spec.year === criteria.year);
  }

  if (criteria.make) {
    const makeLower = criteria.make.toLowerCase();
    results = results.filter(spec => spec.make.toLowerCase().includes(makeLower));
  }

  if (criteria.model) {
    const modelLower = criteria.model.toLowerCase();
    results = results.filter(spec => spec.model.toLowerCase().includes(modelLower));
  }

  if (criteria.type) {
    results = results.filter(spec => spec.type === criteria.type);
  }

  return results;
}

/**
 * Get vehicle spec by exact match
 */
export function getVehicleSpec(year: number, make: string, model: string): VehicleSpec | null {
  const makeLower = make.toLowerCase();
  const modelLower = model.toLowerCase();

  const found = VEHICLE_SPECS_DATABASE.find(
    spec =>
      spec.year === year &&
      spec.make.toLowerCase() === makeLower &&
      spec.model.toLowerCase() === modelLower
  );

  return found || null;
}

/**
 * Get all unique makes in database
 */
export function getAllMakes(): string[] {
  const makes = new Set(VEHICLE_SPECS_DATABASE.map(spec => spec.make));
  return Array.from(makes).sort();
}

/**
 * Get all models for a specific make
 */
export function getModelsForMake(make: string): string[] {
  const makeLower = make.toLowerCase();
  const models = new Set(
    VEHICLE_SPECS_DATABASE
      .filter(spec => spec.make.toLowerCase() === makeLower)
      .map(spec => spec.model)
  );
  return Array.from(models).sort();
}

/**
 * Get all years for a specific make/model
 */
export function getYearsForMakeModel(make: string, model: string): number[] {
  const makeLower = make.toLowerCase();
  const modelLower = model.toLowerCase();

  const years = VEHICLE_SPECS_DATABASE
    .filter(
      spec =>
        spec.make.toLowerCase() === makeLower &&
        spec.model.toLowerCase() === modelLower
    )
    .map(spec => spec.year);

  return Array.from(new Set(years)).sort((a, b) => b - a);
}

/**
 * Create custom vehicle spec
 */
export function createCustomVehicleSpec(
  year: number,
  make: string,
  model: string,
  type: VehicleType,
  dimensions?: Partial<VehicleDimensions>,
  weight?: Partial<VehicleWeightSpec>
): VehicleSpec {
  const id = `${make.toLowerCase()}-${model.toLowerCase()}-${year}`.replace(/\s+/g, '-');

  // Get default specs for the type
  const defaultDimensions = {
    length: 15.0,
    width: 6.0,
    height: 5.0,
    wheelbase: 9.0,
    groundClearance: 6.0,
    ...dimensions,
  };

  const defaultWeight = {
    curb: 3500,
    gross: 4500,
    distribution: { front: 55, rear: 45 },
    ...weight,
  };

  return {
    id,
    year,
    make,
    model,
    type,
    dimensions: defaultDimensions,
    weight: defaultWeight,
  };
}
