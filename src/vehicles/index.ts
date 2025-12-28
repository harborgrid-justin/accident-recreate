/**
 * Vehicle System - Main Export
 * AccuScene Enterprise - Accident Recreation Platform
 */

// Vehicle Types
export {
  VehicleType,
  VehicleDimensions,
  VehicleWeightSpec,
  VehicleTypeSpec,
  DEFAULT_VEHICLE_SPECS,
  getDefaultSpecsForType,
  getAllVehicleTypes,
  getVehicleTypeName,
} from './VehicleTypes';

// Vehicle Specifications Database
export {
  VehicleSpec,
  searchVehicleSpecs,
  getVehicleSpec,
  getAllMakes,
  getModelsForMake,
  getYearsForMakeModel,
  createCustomVehicleSpec,
} from './VehicleSpecs';

// Damage Model
export {
  DamageZone,
  DamageSeverity,
  DamageZoneDetail,
  VehicleDamage,
  DAMAGE_SEVERITY_DESCRIPTIONS,
  ZONE_COMPONENTS,
  getDamageSeverityDescription,
  createDamageZoneDetail,
  createVehicleDamage,
  addDamageZone,
  removeDamageZone,
  calculateOverallSeverity,
  addPhotoToDamageZone,
  removePhotoFromDamageZone,
  updateDrivability,
  addFluidLeak,
  getDamageSummary,
  isTotalLoss,
} from './DamageModel';

// Occupant Information
export {
  OccupantPosition,
  InjurySeverity,
  InjuryType,
  TransportDestination,
  InjuryDetail,
  Occupant,
  INJURY_SEVERITY_DESCRIPTIONS,
  POSITION_NAMES,
  getInjurySeverityDescription,
  getPositionName,
  createOccupant,
  createInjuryDetail,
  addInjury,
  removeInjury,
  calculateOverallInjurySeverity,
  updateSafetyEquipment,
  updateMedicalTransport,
  getInjurySummary,
  getSafetyEquipmentSummary,
  validateOccupant,
  getOccupantsSummary,
} from './OccupantInfo';

// Vehicle Physics
export {
  PHYSICS_CONSTANTS,
  RoadCondition,
  getFrictionCoefficient,
  calculateMass,
  calculateMomentum,
  calculateKineticEnergy,
  calculateBrakingDistance,
  calculateSpeedFromSkidMarks,
  calculateImpactSpeedFromDamage,
  calculateDeltaV,
  calculateFollowingDistance,
  calculateLateralAcceleration,
  calculateRolloverSpeed,
  calculateTimeToCollision,
  mphToFps,
  fpsToMph,
  mphToKph,
  kphToMph,
  calculateAxleWeights,
  calculateAcceleration,
  calculateStoppingDistanceOnGrade,
} from './VehiclePhysics';

// Vehicle Renderer
export {
  VehicleRenderOptions,
  DEFAULT_RENDER_OPTIONS,
  renderVehicle,
  renderDamageOverlay,
  getVehicleBounds,
  isPointInVehicle,
} from './VehicleRenderer';

// Vehicle Service (Main API)
export {
  Vehicle,
  CreateVehicleData,
  UpdateVehicleData,
  createVehicle,
  getVehicle,
  updateVehicle,
  deleteVehicle,
  getVehiclesByAccident,
  addPhotoToVehicle,
  removePhotoFromVehicle,
  updateVehicleDamage,
  addOccupantToVehicle,
  updateOccupantInVehicle,
  removeOccupantFromVehicle,
  updateVehiclePosition,
  getVehicleSummary,
  validateVehicle,
  clearAllVehicles,
  getAllVehicles,
  exportVehicleData,
  importVehicleData,
} from './VehicleService';
