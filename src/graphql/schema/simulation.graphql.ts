/**
 * Simulation GraphQL Schema
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { gql } from 'graphql-tag';

export const simulationTypeDefs = gql`
  """
  Represents a physics simulation of an accident
  """
  type Simulation @key(fields: "id") {
    id: ID!
    caseId: ID!
    case: Case
    sceneId: ID!
    scene: Scene
    name: String!
    description: String
    status: SimulationStatus!
    progress: Float!
    parameters: SimulationParameters!
    results: SimulationResults
    startedAt: DateTime
    completedAt: DateTime
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  """
  Simulation status enumeration
  """
  enum SimulationStatus {
    PENDING
    RUNNING
    COMPLETED
    FAILED
    CANCELLED
  }

  """
  Simulation parameters
  """
  type SimulationParameters {
    timeStep: Float!
    duration: Float!
    iterations: Int!
    physicsEngine: PhysicsEngine!
    environmentalFactors: EnvironmentalFactors!
  }

  """
  Physics engine enumeration
  """
  enum PhysicsEngine {
    BASIC
    ADVANCED
    REAL_TIME
  }

  """
  Environmental factors affecting simulation
  """
  type EnvironmentalFactors {
    roadFriction: Float!
    airDensity: Float!
    gravity: Float!
    windSpeed: Float
    windDirection: Float
  }

  """
  Simulation results
  """
  type SimulationResults {
    frames: [SimulationFrame!]!
    metrics: SimulationMetrics!
    summary: String!
  }

  """
  Single frame in a simulation
  """
  type SimulationFrame {
    timestamp: Float!
    vehicleStates: [VehicleState!]!
  }

  """
  Vehicle state at a specific time
  """
  type VehicleState {
    vehicleId: ID!
    position: Position!
    velocity: Velocity!
    acceleration: Acceleration!
    rotation: Float!
  }

  """
  Velocity vector
  """
  type Velocity {
    x: Float!
    y: Float!
    z: Float
    magnitude: Float!
  }

  """
  Acceleration vector
  """
  type Acceleration {
    x: Float!
    y: Float!
    z: Float
    magnitude: Float!
  }

  """
  Simulation metrics
  """
  type SimulationMetrics {
    maxSpeed: Float!
    impactForce: Float!
    energyDissipation: Float!
    collisionTime: Float!
    deltaV: Float!
  }

  """
  Input type for creating a simulation
  """
  input CreateSimulationInput {
    caseId: ID!
    sceneId: ID!
    name: String!
    description: String
    parameters: SimulationParametersInput!
  }

  """
  Input type for simulation parameters
  """
  input SimulationParametersInput {
    timeStep: Float! = 0.016
    duration: Float! = 10.0
    iterations: Int! = 100
    physicsEngine: PhysicsEngine! = ADVANCED
    environmentalFactors: EnvironmentalFactorsInput
  }

  """
  Input type for environmental factors
  """
  input EnvironmentalFactorsInput {
    roadFriction: Float! = 0.7
    airDensity: Float! = 1.225
    gravity: Float! = 9.81
    windSpeed: Float
    windDirection: Float
  }

  """
  Input type for updating simulation
  """
  input UpdateSimulationInput {
    name: String
    description: String
    parameters: SimulationParametersInput
  }

  extend type Query {
    """
    Get a single simulation by ID
    """
    simulation(id: ID!): Simulation @auth(requires: [CASE_READ])

    """
    Get all simulations for a case
    """
    simulationsByCase(caseId: ID!): [Simulation!]! @auth(requires: [CASE_READ])

    """
    Get all simulations for a scene
    """
    simulationsByScene(sceneId: ID!): [Simulation!]! @auth(requires: [CASE_READ])

    """
    Get simulation frame by timestamp
    """
    simulationFrame(simulationId: ID!, timestamp: Float!): SimulationFrame @auth(requires: [CASE_READ])

    """
    Get simulation frames range
    """
    simulationFrames(
      simulationId: ID!
      startTime: Float
      endTime: Float
    ): [SimulationFrame!]! @auth(requires: [CASE_READ])
  }

  extend type Mutation {
    """
    Create a new simulation
    """
    createSimulation(input: CreateSimulationInput!): Simulation! @auth(requires: [SIMULATION_RUN])

    """
    Update simulation parameters
    """
    updateSimulation(id: ID!, input: UpdateSimulationInput!): Simulation! @auth(requires: [SIMULATION_RUN])

    """
    Delete a simulation
    """
    deleteSimulation(id: ID!): Boolean! @auth(requires: [SIMULATION_RUN])

    """
    Start a simulation
    """
    startSimulation(id: ID!): Simulation! @auth(requires: [SIMULATION_RUN])

    """
    Cancel a running simulation
    """
    cancelSimulation(id: ID!): Simulation! @auth(requires: [SIMULATION_RUN])

    """
    Re-run a simulation with same parameters
    """
    rerunSimulation(id: ID!): Simulation! @auth(requires: [SIMULATION_RUN])

    """
    Export simulation results
    """
    exportSimulation(id: ID!, format: ExportFormat!): String! @auth(requires: [CASE_READ])
  }

  """
  Export format enumeration
  """
  enum ExportFormat {
    JSON
    CSV
    VIDEO
  }

  extend type Subscription {
    """
    Subscribe to simulation progress updates
    """
    simulationProgress(simulationId: ID!): SimulationProgressPayload! @auth(requires: [SIMULATION_RUN])

    """
    Subscribe to simulation completion
    """
    simulationCompleted(caseId: ID): Simulation! @auth(requires: [SIMULATION_RUN])
  }

  """
  Simulation progress subscription payload
  """
  type SimulationProgressPayload {
    simulation: Simulation!
    progress: Float!
    status: SimulationStatus!
    currentFrame: Int
    totalFrames: Int
  }
`;
