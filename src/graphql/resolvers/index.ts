/**
 * GraphQL Resolver Composition
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { customScalars } from '../scalars';
import {
  caseQueries,
  caseMutations,
  caseFieldResolvers,
  caseSubscriptions,
  caseReference,
} from './case.resolver';
import {
  vehicleQueries,
  vehicleMutations,
  vehicleFieldResolvers,
  vehicleReference,
} from './vehicle.resolver';
import {
  sceneQueries,
  sceneMutations,
  sceneFieldResolvers,
  sceneReference,
} from './scene.resolver';
import {
  simulationQueries,
  simulationMutations,
  simulationFieldResolvers,
  simulationSubscriptions,
  simulationReference,
} from './simulation.resolver';
import {
  userQueries,
  userMutations,
  userFieldResolvers,
  userReference,
} from './user.resolver';
import {
  reportQueries,
  reportMutations,
  reportFieldResolvers,
  reportSubscriptions,
  reportReference,
} from './report.resolver';
import { GraphQLContext } from '../types';

// ============================================================================
// Root Resolvers
// ============================================================================

export const resolvers = {
  // Custom Scalars
  ...customScalars,

  // Queries
  Query: {
    // Health check
    health: () => ({
      status: 'healthy',
      version: '0.2.0',
      uptime: process.uptime(),
      timestamp: new Date(),
    }),

    // Federation service
    _service: () => ({
      sdl: '', // This will be populated by Apollo Federation
    }),

    // Case queries
    ...caseQueries,

    // Vehicle queries
    ...vehicleQueries,

    // Scene queries
    ...sceneQueries,

    // Simulation queries
    ...simulationQueries,

    // User queries
    ...userQueries,

    // Report queries
    ...reportQueries,
  },

  // Mutations
  Mutation: {
    // Case mutations
    ...caseMutations,

    // Vehicle mutations
    ...vehicleMutations,

    // Scene mutations
    ...sceneMutations,

    // Simulation mutations
    ...simulationMutations,

    // User mutations
    ...userMutations,

    // Report mutations
    ...reportMutations,
  },

  // Subscriptions
  Subscription: {
    // Case subscriptions
    ...caseSubscriptions,

    // Simulation subscriptions
    ...simulationSubscriptions,

    // Report subscriptions
    ...reportSubscriptions,
  },

  // Type-specific field resolvers
  Case: {
    ...caseFieldResolvers,
    __resolveReference: caseReference.__resolveReference,
  },

  Vehicle: {
    ...vehicleFieldResolvers,
    __resolveReference: vehicleReference.__resolveReference,
  },

  Scene: {
    ...sceneFieldResolvers,
    __resolveReference: sceneReference.__resolveReference,
  },

  Simulation: {
    ...simulationFieldResolvers,
    __resolveReference: simulationReference.__resolveReference,
  },

  User: {
    ...userFieldResolvers,
    __resolveReference: userReference.__resolveReference,
  },

  Report: {
    ...reportFieldResolvers,
    __resolveReference: reportReference.__resolveReference,
  },
};

// ============================================================================
// Export individual resolvers for testing
// ============================================================================

export {
  caseQueries,
  caseMutations,
  caseSubscriptions,
  vehicleQueries,
  vehicleMutations,
  sceneQueries,
  sceneMutations,
  simulationQueries,
  simulationMutations,
  simulationSubscriptions,
  userQueries,
  userMutations,
  reportQueries,
  reportMutations,
  reportSubscriptions,
};
