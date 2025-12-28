/**
 * Vehicle GraphQL Schema
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { gql } from 'graphql-tag';

export const vehicleTypeDefs = gql`
  """
  Represents a vehicle involved in an accident
  """
  type Vehicle @key(fields: "id") {
    id: ID!
    caseId: ID!
    case: Case
    make: String!
    model: String!
    year: Int!
    vin: String
    color: String!
    type: VehicleType!
    role: VehicleRole!
    damage: DamageAssessment
    occupants: [Occupant!]!
    specifications: VehicleSpecifications
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  """
  Vehicle type enumeration
  """
  enum VehicleType {
    SEDAN
    SUV
    TRUCK
    MOTORCYCLE
    BUS
    COMMERCIAL
  }

  """
  Vehicle role in the accident
  """
  enum VehicleRole {
    PRIMARY
    SECONDARY
    WITNESS
  }

  """
  Damage assessment for a vehicle
  """
  type DamageAssessment {
    severity: DamageSeverity!
    zones: [DamageZone!]!
    estimatedCost: Float!
    photos: [String!]!
    description: String!
  }

  """
  Damage severity enumeration
  """
  enum DamageSeverity {
    MINOR
    MODERATE
    SEVERE
    TOTAL_LOSS
  }

  """
  Specific damage zone on a vehicle
  """
  type DamageZone {
    location: String!
    type: String!
    severity: DamageSeverity!
    measurements: Measurements
  }

  """
  Measurements for damage zones
  """
  type Measurements {
    width: Float!
    height: Float!
    depth: Float!
    unit: MeasurementUnit!
  }

  """
  Measurement unit enumeration
  """
  enum MeasurementUnit {
    INCHES
    CENTIMETERS
    FEET
    METERS
  }

  """
  Vehicle occupant information
  """
  type Occupant {
    id: ID!
    position: OccupantPosition!
    name: String
    age: Int
    injuries: [Injury!]
    restraintUsed: Boolean!
    ejected: Boolean!
  }

  """
  Occupant position enumeration
  """
  enum OccupantPosition {
    DRIVER
    FRONT_PASSENGER
    REAR_LEFT
    REAR_CENTER
    REAR_RIGHT
  }

  """
  Injury information
  """
  type Injury {
    type: String!
    severity: InjurySeverity!
    bodyPart: String!
    description: String!
  }

  """
  Injury severity enumeration
  """
  enum InjurySeverity {
    NONE
    MINOR
    MODERATE
    SEVERE
    FATAL
  }

  """
  Vehicle specifications
  """
  type VehicleSpecifications {
    length: Float!
    width: Float!
    height: Float!
    weight: Float!
    wheelbase: Float!
    engine: EngineSpecifications
  }

  """
  Engine specifications
  """
  type EngineSpecifications {
    type: String!
    displacement: Float!
    horsepower: Float!
    torque: Float!
  }

  """
  Input type for creating a vehicle
  """
  input CreateVehicleInput {
    caseId: ID!
    make: String!
    model: String!
    year: Int!
    vin: String
    color: String!
    type: VehicleType!
    role: VehicleRole!
  }

  """
  Input type for updating a vehicle
  """
  input UpdateVehicleInput {
    make: String
    model: String
    year: Int
    vin: String
    color: String
    type: VehicleType
    role: VehicleRole
  }

  """
  Input type for damage assessment
  """
  input DamageAssessmentInput {
    severity: DamageSeverity!
    zones: [DamageZoneInput!]!
    estimatedCost: Float!
    photos: [String!]
    description: String!
  }

  """
  Input type for damage zone
  """
  input DamageZoneInput {
    location: String!
    type: String!
    severity: DamageSeverity!
    measurements: MeasurementsInput
  }

  """
  Input type for measurements
  """
  input MeasurementsInput {
    width: Float!
    height: Float!
    depth: Float!
    unit: MeasurementUnit!
  }

  """
  Input type for occupant
  """
  input OccupantInput {
    position: OccupantPosition!
    name: String
    age: Int
    injuries: [InjuryInput!]
    restraintUsed: Boolean!
    ejected: Boolean!
  }

  """
  Input type for injury
  """
  input InjuryInput {
    type: String!
    severity: InjurySeverity!
    bodyPart: String!
    description: String!
  }

  extend type Query {
    """
    Get a single vehicle by ID
    """
    vehicle(id: ID!): Vehicle @auth(requires: [CASE_READ])

    """
    Get all vehicles for a case
    """
    vehiclesByCase(caseId: ID!): [Vehicle!]! @auth(requires: [CASE_READ])

    """
    Search vehicles by VIN
    """
    vehicleByVIN(vin: String!): Vehicle @auth(requires: [CASE_READ])
  }

  extend type Mutation {
    """
    Create a new vehicle
    """
    createVehicle(input: CreateVehicleInput!): Vehicle! @auth(requires: [CASE_UPDATE])

    """
    Update an existing vehicle
    """
    updateVehicle(id: ID!, input: UpdateVehicleInput!): Vehicle! @auth(requires: [CASE_UPDATE])

    """
    Delete a vehicle
    """
    deleteVehicle(id: ID!): Boolean! @auth(requires: [CASE_UPDATE])

    """
    Add damage assessment to a vehicle
    """
    addDamageAssessment(vehicleId: ID!, input: DamageAssessmentInput!): Vehicle! @auth(requires: [CASE_UPDATE])

    """
    Add occupant to a vehicle
    """
    addOccupant(vehicleId: ID!, input: OccupantInput!): Vehicle! @auth(requires: [CASE_UPDATE])

    """
    Remove occupant from a vehicle
    """
    removeOccupant(vehicleId: ID!, occupantId: ID!): Vehicle! @auth(requires: [CASE_UPDATE])

    """
    Upload vehicle photo
    """
    uploadVehiclePhoto(vehicleId: ID!, photo: Upload!): String! @auth(requires: [CASE_UPDATE])
  }
`;
