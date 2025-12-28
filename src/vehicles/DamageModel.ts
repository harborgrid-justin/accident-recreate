/**
 * Vehicle Damage Model
 * AccuScene Enterprise - Accident Recreation Platform
 */

export enum DamageZone {
  FRONT = 'FRONT',
  FRONT_LEFT = 'FRONT_LEFT',
  FRONT_RIGHT = 'FRONT_RIGHT',
  LEFT = 'LEFT',
  RIGHT = 'RIGHT',
  REAR = 'REAR',
  REAR_LEFT = 'REAR_LEFT',
  REAR_RIGHT = 'REAR_RIGHT',
  ROOF = 'ROOF',
  UNDERCARRIAGE = 'UNDERCARRIAGE',
}

export enum DamageSeverity {
  NONE = 0,
  MINOR = 1,
  MODERATE = 2,
  SEVERE = 3,
  MAJOR = 4,
  CATASTROPHIC = 5,
}

export interface DamageZoneDetail {
  zone: DamageZone;
  severity: DamageSeverity;
  description: string;
  extent: number; // 0-100 percentage of zone affected
  depth: number; // 0-100 depth of damage (0=surface, 100=structural)
  components: string[]; // Affected components (e.g., "bumper", "hood", "fender")
  photoIds: string[]; // References to uploaded photos
  estimatedCost?: number; // Repair cost estimate in dollars
  notes?: string;
}

export interface VehicleDamage {
  vehicleId: string;
  overallSeverity: DamageSeverity;
  zones: DamageZoneDetail[];
  isDrivable: boolean;
  isAirbagsDeployed: boolean;
  fluidLeaks: string[]; // e.g., "oil", "coolant", "brake fluid"
  totalEstimatedCost?: number;
  notes?: string;
  createdAt: Date;
  updatedAt: Date;
}

/**
 * Damage severity descriptions
 */
export const DAMAGE_SEVERITY_DESCRIPTIONS: Record<DamageSeverity, string> = {
  [DamageSeverity.NONE]: 'No visible damage',
  [DamageSeverity.MINOR]: 'Minor scratches, scuffs, or small dents',
  [DamageSeverity.MODERATE]: 'Significant dents, broken lights, or body panel damage',
  [DamageSeverity.SEVERE]: 'Crushed body panels, frame damage, or major component failure',
  [DamageSeverity.MAJOR]: 'Extensive structural damage, multiple systems compromised',
  [DamageSeverity.CATASTROPHIC]: 'Total loss, severe structural collapse, fire damage',
};

/**
 * Get damage severity description
 */
export function getDamageSeverityDescription(severity: DamageSeverity): string {
  return DAMAGE_SEVERITY_DESCRIPTIONS[severity];
}

/**
 * Common component names by zone
 */
export const ZONE_COMPONENTS: Record<DamageZone, string[]> = {
  [DamageZone.FRONT]: [
    'Front Bumper',
    'Grille',
    'Hood',
    'Headlights',
    'Radiator',
    'Engine',
    'Front Frame Rails',
    'Windshield',
  ],
  [DamageZone.FRONT_LEFT]: [
    'Left Front Fender',
    'Left Headlight',
    'Left Front Wheel',
    'Left Front Door',
    'Left Front Quarter Panel',
  ],
  [DamageZone.FRONT_RIGHT]: [
    'Right Front Fender',
    'Right Headlight',
    'Right Front Wheel',
    'Right Front Door',
    'Right Front Quarter Panel',
  ],
  [DamageZone.LEFT]: [
    'Left Doors',
    'Left Side Panels',
    'Left Side Mirrors',
    'Left Windows',
    'Left Rocker Panel',
  ],
  [DamageZone.RIGHT]: [
    'Right Doors',
    'Right Side Panels',
    'Right Side Mirrors',
    'Right Windows',
    'Right Rocker Panel',
  ],
  [DamageZone.REAR]: [
    'Rear Bumper',
    'Trunk/Tailgate',
    'Tail Lights',
    'Rear Window',
    'Rear Frame Rails',
    'Exhaust System',
  ],
  [DamageZone.REAR_LEFT]: [
    'Left Rear Fender',
    'Left Tail Light',
    'Left Rear Wheel',
    'Left Rear Door',
    'Left Rear Quarter Panel',
  ],
  [DamageZone.REAR_RIGHT]: [
    'Right Rear Fender',
    'Right Tail Light',
    'Right Rear Wheel',
    'Right Rear Door',
    'Right Rear Quarter Panel',
  ],
  [DamageZone.ROOF]: [
    'Roof Panel',
    'Sunroof',
    'Roof Rails',
    'A-Pillar',
    'B-Pillar',
    'C-Pillar',
  ],
  [DamageZone.UNDERCARRIAGE]: [
    'Frame',
    'Suspension',
    'Transmission',
    'Fuel Tank',
    'Exhaust System',
    'Drive Shaft',
  ],
};

/**
 * Create new damage zone detail
 */
export function createDamageZoneDetail(
  zone: DamageZone,
  severity: DamageSeverity,
  description: string = '',
  extent: number = 50,
  depth: number = 30
): DamageZoneDetail {
  return {
    zone,
    severity,
    description,
    extent: Math.max(0, Math.min(100, extent)),
    depth: Math.max(0, Math.min(100, depth)),
    components: [],
    photoIds: [],
  };
}

/**
 * Create new vehicle damage record
 */
export function createVehicleDamage(vehicleId: string): VehicleDamage {
  return {
    vehicleId,
    overallSeverity: DamageSeverity.NONE,
    zones: [],
    isDrivable: true,
    isAirbagsDeployed: false,
    fluidLeaks: [],
    createdAt: new Date(),
    updatedAt: new Date(),
  };
}

/**
 * Add damage zone to vehicle damage
 */
export function addDamageZone(
  damage: VehicleDamage,
  zoneDetail: DamageZoneDetail
): VehicleDamage {
  const existingIndex = damage.zones.findIndex(z => z.zone === zoneDetail.zone);

  const zones =
    existingIndex >= 0
      ? damage.zones.map((z, i) => (i === existingIndex ? zoneDetail : z))
      : [...damage.zones, zoneDetail];

  const overallSeverity = calculateOverallSeverity(zones);

  return {
    ...damage,
    zones,
    overallSeverity,
    updatedAt: new Date(),
  };
}

/**
 * Remove damage zone from vehicle damage
 */
export function removeDamageZone(
  damage: VehicleDamage,
  zone: DamageZone
): VehicleDamage {
  const zones = damage.zones.filter(z => z.zone !== zone);
  const overallSeverity = calculateOverallSeverity(zones);

  return {
    ...damage,
    zones,
    overallSeverity,
    updatedAt: new Date(),
  };
}

/**
 * Calculate overall damage severity from all zones
 */
export function calculateOverallSeverity(zones: DamageZoneDetail[]): DamageSeverity {
  if (zones.length === 0) {
    return DamageSeverity.NONE;
  }

  // Get the maximum severity
  const maxSeverity = Math.max(...zones.map(z => z.severity));

  // Count zones with significant damage
  const significantZones = zones.filter(z => z.severity >= DamageSeverity.MODERATE);

  // If multiple zones have significant damage, bump up severity
  if (significantZones.length >= 3 && maxSeverity < DamageSeverity.CATASTROPHIC) {
    return Math.min(maxSeverity + 1, DamageSeverity.CATASTROPHIC) as DamageSeverity;
  }

  return maxSeverity as DamageSeverity;
}

/**
 * Add photo to damage zone
 */
export function addPhotoToDamageZone(
  damage: VehicleDamage,
  zone: DamageZone,
  photoId: string
): VehicleDamage {
  const zones = damage.zones.map(z => {
    if (z.zone === zone) {
      return {
        ...z,
        photoIds: [...z.photoIds, photoId],
      };
    }
    return z;
  });

  return {
    ...damage,
    zones,
    updatedAt: new Date(),
  };
}

/**
 * Remove photo from damage zone
 */
export function removePhotoFromDamageZone(
  damage: VehicleDamage,
  zone: DamageZone,
  photoId: string
): VehicleDamage {
  const zones = damage.zones.map(z => {
    if (z.zone === zone) {
      return {
        ...z,
        photoIds: z.photoIds.filter(id => id !== photoId),
      };
    }
    return z;
  });

  return {
    ...damage,
    zones,
    updatedAt: new Date(),
  };
}

/**
 * Update drivability assessment
 */
export function updateDrivability(
  damage: VehicleDamage,
  isDrivable: boolean,
  reason?: string
): VehicleDamage {
  return {
    ...damage,
    isDrivable,
    notes: reason ? `Drivability: ${reason}` : damage.notes,
    updatedAt: new Date(),
  };
}

/**
 * Add fluid leak
 */
export function addFluidLeak(damage: VehicleDamage, fluid: string): VehicleDamage {
  if (damage.fluidLeaks.includes(fluid)) {
    return damage;
  }

  return {
    ...damage,
    fluidLeaks: [...damage.fluidLeaks, fluid],
    updatedAt: new Date(),
  };
}

/**
 * Get damage summary text
 */
export function getDamageSummary(damage: VehicleDamage): string {
  if (damage.zones.length === 0) {
    return 'No damage reported';
  }

  const zoneNames = damage.zones
    .map(z => z.zone.toLowerCase().replace(/_/g, ' '))
    .join(', ');

  const severity = getDamageSeverityDescription(damage.overallSeverity);
  const drivable = damage.isDrivable ? 'Vehicle is drivable' : 'Vehicle is not drivable';
  const airbags = damage.isAirbagsDeployed ? 'Airbags deployed' : 'Airbags not deployed';

  return `${severity} damage to ${zoneNames}. ${drivable}. ${airbags}.`;
}

/**
 * Estimate if vehicle is total loss
 */
export function isTotalLoss(
  damage: VehicleDamage,
  vehicleValue: number,
  repairCostThreshold: number = 0.75
): boolean {
  if (damage.overallSeverity >= DamageSeverity.CATASTROPHIC) {
    return true;
  }

  if (damage.totalEstimatedCost && vehicleValue) {
    return damage.totalEstimatedCost / vehicleValue >= repairCostThreshold;
  }

  // Heuristic: multiple zones with severe+ damage likely total loss
  const severeZones = damage.zones.filter(
    z => z.severity >= DamageSeverity.SEVERE
  ).length;

  return severeZones >= 3;
}
