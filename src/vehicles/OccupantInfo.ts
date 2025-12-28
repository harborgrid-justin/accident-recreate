/**
 * Vehicle Occupant Information
 * AccuScene Enterprise - Accident Recreation Platform
 */

export enum OccupantPosition {
  DRIVER = 'DRIVER',
  FRONT_PASSENGER = 'FRONT_PASSENGER',
  REAR_LEFT = 'REAR_LEFT',
  REAR_CENTER = 'REAR_CENTER',
  REAR_RIGHT = 'REAR_RIGHT',
  THIRD_ROW_LEFT = 'THIRD_ROW_LEFT',
  THIRD_ROW_CENTER = 'THIRD_ROW_CENTER',
  THIRD_ROW_RIGHT = 'THIRD_ROW_RIGHT',
}

export enum InjurySeverity {
  NONE = 'NONE',
  MINOR = 'MINOR',
  MODERATE = 'MODERATE',
  SERIOUS = 'SERIOUS',
  CRITICAL = 'CRITICAL',
  FATAL = 'FATAL',
}

export enum InjuryType {
  LACERATION = 'LACERATION',
  CONTUSION = 'CONTUSION',
  ABRASION = 'ABRASION',
  FRACTURE = 'FRACTURE',
  SPRAIN = 'SPRAIN',
  STRAIN = 'STRAIN',
  CONCUSSION = 'CONCUSSION',
  TRAUMATIC_BRAIN_INJURY = 'TRAUMATIC_BRAIN_INJURY',
  WHIPLASH = 'WHIPLASH',
  INTERNAL_INJURY = 'INTERNAL_INJURY',
  BURN = 'BURN',
  OTHER = 'OTHER',
}

export enum TransportDestination {
  HOSPITAL = 'HOSPITAL',
  TRAUMA_CENTER = 'TRAUMA_CENTER',
  CLINIC = 'CLINIC',
  REFUSED = 'REFUSED',
  DECEASED_ON_SCENE = 'DECEASED_ON_SCENE',
}

export interface InjuryDetail {
  type: InjuryType;
  location: string; // Body part (e.g., "head", "chest", "left arm")
  severity: InjurySeverity;
  description: string;
  treatedOnScene: boolean;
  requiresHospitalization: boolean;
}

export interface Occupant {
  id: string;
  vehicleId: string;
  position: OccupantPosition;

  // Personal Information
  name?: string;
  age?: number;
  gender?: 'MALE' | 'FEMALE' | 'OTHER' | 'UNKNOWN';

  // Safety Equipment
  seatbeltWorn: boolean;
  airbagDeployed: boolean;
  childSeat: boolean;
  childSeatType?: 'REAR_FACING' | 'FORWARD_FACING' | 'BOOSTER' | 'NONE';

  // Injury Information
  injuries: InjuryDetail[];
  overallInjurySeverity: InjurySeverity;

  // Medical Response
  receivedMedicalAttention: boolean;
  transportedBy?: 'AMBULANCE' | 'HELICOPTER' | 'PRIVATE' | 'NONE';
  transportDestination?: TransportDestination;
  hospitalName?: string;

  // Additional Information
  emsProviderName?: string;
  emsIncidentNumber?: string;
  notes?: string;

  createdAt: Date;
  updatedAt: Date;
}

/**
 * Injury severity descriptions
 */
export const INJURY_SEVERITY_DESCRIPTIONS: Record<InjurySeverity, string> = {
  [InjurySeverity.NONE]: 'No apparent injury',
  [InjurySeverity.MINOR]: 'Minor injuries not requiring hospitalization',
  [InjurySeverity.MODERATE]: 'Moderate injuries requiring medical evaluation',
  [InjurySeverity.SERIOUS]: 'Serious injuries requiring hospitalization',
  [InjurySeverity.CRITICAL]: 'Life-threatening injuries',
  [InjurySeverity.FATAL]: 'Fatal injuries',
};

/**
 * Get injury severity description
 */
export function getInjurySeverityDescription(severity: InjurySeverity): string {
  return INJURY_SEVERITY_DESCRIPTIONS[severity];
}

/**
 * Position display names
 */
export const POSITION_NAMES: Record<OccupantPosition, string> = {
  [OccupantPosition.DRIVER]: 'Driver',
  [OccupantPosition.FRONT_PASSENGER]: 'Front Passenger',
  [OccupantPosition.REAR_LEFT]: 'Rear Left Passenger',
  [OccupantPosition.REAR_CENTER]: 'Rear Center Passenger',
  [OccupantPosition.REAR_RIGHT]: 'Rear Right Passenger',
  [OccupantPosition.THIRD_ROW_LEFT]: 'Third Row Left Passenger',
  [OccupantPosition.THIRD_ROW_CENTER]: 'Third Row Center Passenger',
  [OccupantPosition.THIRD_ROW_RIGHT]: 'Third Row Right Passenger',
};

/**
 * Get position display name
 */
export function getPositionName(position: OccupantPosition): string {
  return POSITION_NAMES[position];
}

/**
 * Create new occupant
 */
export function createOccupant(
  vehicleId: string,
  position: OccupantPosition,
  seatbeltWorn: boolean = true
): Occupant {
  return {
    id: `occ_${Date.now()}_${Math.random().toString(36).substring(7)}`,
    vehicleId,
    position,
    seatbeltWorn,
    airbagDeployed: false,
    childSeat: false,
    injuries: [],
    overallInjurySeverity: InjurySeverity.NONE,
    receivedMedicalAttention: false,
    createdAt: new Date(),
    updatedAt: new Date(),
  };
}

/**
 * Create injury detail
 */
export function createInjuryDetail(
  type: InjuryType,
  location: string,
  severity: InjurySeverity,
  description: string = ''
): InjuryDetail {
  return {
    type,
    location,
    severity,
    description,
    treatedOnScene: false,
    requiresHospitalization: severity >= InjurySeverity.SERIOUS,
  };
}

/**
 * Add injury to occupant
 */
export function addInjury(occupant: Occupant, injury: InjuryDetail): Occupant {
  const injuries = [...occupant.injuries, injury];
  const overallInjurySeverity = calculateOverallInjurySeverity(injuries);

  return {
    ...occupant,
    injuries,
    overallInjurySeverity,
    receivedMedicalAttention: overallInjurySeverity !== InjurySeverity.NONE,
    updatedAt: new Date(),
  };
}

/**
 * Remove injury from occupant
 */
export function removeInjury(occupant: Occupant, injuryIndex: number): Occupant {
  const injuries = occupant.injuries.filter((_, index) => index !== injuryIndex);
  const overallInjurySeverity = calculateOverallInjurySeverity(injuries);

  return {
    ...occupant,
    injuries,
    overallInjurySeverity,
    receivedMedicalAttention: overallInjurySeverity !== InjurySeverity.NONE,
    updatedAt: new Date(),
  };
}

/**
 * Calculate overall injury severity
 */
export function calculateOverallInjurySeverity(injuries: InjuryDetail[]): InjurySeverity {
  if (injuries.length === 0) {
    return InjurySeverity.NONE;
  }

  const severityOrder = [
    InjurySeverity.NONE,
    InjurySeverity.MINOR,
    InjurySeverity.MODERATE,
    InjurySeverity.SERIOUS,
    InjurySeverity.CRITICAL,
    InjurySeverity.FATAL,
  ];

  // Find the highest severity
  let maxSeverity = InjurySeverity.NONE;
  let maxIndex = 0;

  for (const injury of injuries) {
    const index = severityOrder.indexOf(injury.severity);
    if (index > maxIndex) {
      maxIndex = index;
      maxSeverity = injury.severity;
    }
  }

  // If multiple serious injuries, might bump up severity
  const seriousCount = injuries.filter(
    i => severityOrder.indexOf(i.severity) >= severityOrder.indexOf(InjurySeverity.SERIOUS)
  ).length;

  if (seriousCount >= 2 && maxSeverity === InjurySeverity.SERIOUS) {
    return InjurySeverity.CRITICAL;
  }

  return maxSeverity;
}

/**
 * Update safety equipment status
 */
export function updateSafetyEquipment(
  occupant: Occupant,
  updates: {
    seatbeltWorn?: boolean;
    airbagDeployed?: boolean;
    childSeat?: boolean;
    childSeatType?: 'REAR_FACING' | 'FORWARD_FACING' | 'BOOSTER' | 'NONE';
  }
): Occupant {
  return {
    ...occupant,
    ...updates,
    updatedAt: new Date(),
  };
}

/**
 * Update medical transport information
 */
export function updateMedicalTransport(
  occupant: Occupant,
  transportedBy: 'AMBULANCE' | 'HELICOPTER' | 'PRIVATE' | 'NONE',
  destination?: TransportDestination,
  hospitalName?: string,
  emsProviderName?: string,
  emsIncidentNumber?: string
): Occupant {
  return {
    ...occupant,
    transportedBy,
    transportDestination: destination,
    hospitalName,
    emsProviderName,
    emsIncidentNumber,
    receivedMedicalAttention: transportedBy !== 'NONE',
    updatedAt: new Date(),
  };
}

/**
 * Get injury summary for occupant
 */
export function getInjurySummary(occupant: Occupant): string {
  if (occupant.injuries.length === 0) {
    return 'No injuries reported';
  }

  const severityDesc = getInjurySeverityDescription(occupant.overallInjurySeverity);
  const injuryCount = occupant.injuries.length;
  const injuryWord = injuryCount === 1 ? 'injury' : 'injuries';

  const locations = occupant.injuries
    .map(i => i.location)
    .filter((v, i, a) => a.indexOf(v) === i) // unique
    .join(', ');

  return `${severityDesc} - ${injuryCount} ${injuryWord} (${locations})`;
}

/**
 * Get safety equipment summary
 */
export function getSafetyEquipmentSummary(occupant: Occupant): string {
  const parts: string[] = [];

  parts.push(occupant.seatbeltWorn ? 'Seatbelt worn' : 'No seatbelt');

  if (occupant.airbagDeployed) {
    parts.push('Airbag deployed');
  }

  if (occupant.childSeat && occupant.childSeatType && occupant.childSeatType !== 'NONE') {
    const seatType = occupant.childSeatType.toLowerCase().replace(/_/g, ' ');
    parts.push(`Child seat (${seatType})`);
  }

  return parts.join(', ');
}

/**
 * Validate occupant data
 */
export function validateOccupant(occupant: Occupant): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  if (!occupant.vehicleId) {
    errors.push('Vehicle ID is required');
  }

  if (!occupant.position) {
    errors.push('Occupant position is required');
  }

  if (occupant.age !== undefined && (occupant.age < 0 || occupant.age > 120)) {
    errors.push('Age must be between 0 and 120');
  }

  if (occupant.childSeat && (!occupant.childSeatType || occupant.childSeatType === 'NONE')) {
    errors.push('Child seat type must be specified when child seat is used');
  }

  // Warning: Child without child seat
  if (occupant.age !== undefined && occupant.age < 8 && !occupant.childSeat) {
    errors.push('Warning: Child under 8 should use child seat');
  }

  // Fatal injury checks
  const hasFatalInjury = occupant.injuries.some(
    i => i.severity === InjurySeverity.FATAL
  );
  if (hasFatalInjury && occupant.transportDestination !== TransportDestination.DECEASED_ON_SCENE) {
    errors.push('Warning: Fatal injury should indicate deceased on scene');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Get all occupants summary
 */
export function getOccupantsSummary(occupants: Occupant[]): string {
  if (occupants.length === 0) {
    return 'No occupants';
  }

  const total = occupants.length;
  const injured = occupants.filter(o => o.overallInjurySeverity !== InjurySeverity.NONE).length;
  const fatal = occupants.filter(o => o.overallInjurySeverity === InjurySeverity.FATAL).length;
  const seatbelts = occupants.filter(o => o.seatbeltWorn).length;

  let summary = `${total} occupant${total !== 1 ? 's' : ''}`;

  if (injured > 0) {
    summary += `, ${injured} injured`;
  }

  if (fatal > 0) {
    summary += `, ${fatal} fatal${fatal !== 1 ? 'ities' : 'ity'}`;
  }

  summary += `, ${seatbelts}/${total} seatbelts worn`;

  return summary;
}
