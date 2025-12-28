/**
 * Simulation Subscriptions
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { withFilter } from 'graphql-subscriptions';
import { GraphQLContext, SimulationProgressPayload, Simulation } from '../types';

/**
 * Simulation subscription topics
 */
export const SIMULATION_TOPICS = {
  SIMULATION_PROGRESS: 'SIMULATION_PROGRESS',
  SIMULATION_COMPLETED: 'SIMULATION_COMPLETED',
  SIMULATION_FAILED: 'SIMULATION_FAILED',
  SIMULATION_STARTED: 'SIMULATION_STARTED',
} as const;

/**
 * Subscribe to simulation progress updates
 * Real-time updates as simulation runs
 */
export const simulationProgressSubscription = {
  subscribe: withFilter(
    (_: unknown, { simulationId }: { simulationId: string }, context: GraphQLContext) => {
      return context.pubsub.asyncIterator([`${SIMULATION_TOPICS.SIMULATION_PROGRESS}_${simulationId}`]);
    },
    (
      payload: { simulationProgress: SimulationProgressPayload },
      variables: { simulationId: string }
    ) => {
      return payload.simulationProgress.simulation.id === variables.simulationId;
    }
  ),
};

/**
 * Subscribe to simulation completion
 * Fires when simulation finishes successfully
 */
export const simulationCompletedSubscription = {
  subscribe: withFilter(
    (_: unknown, __: unknown, context: GraphQLContext) => {
      return context.pubsub.asyncIterator([SIMULATION_TOPICS.SIMULATION_COMPLETED]);
    },
    (
      payload: { simulationCompleted: Simulation },
      variables: { caseId?: string },
      context: GraphQLContext
    ) => {
      // Filter by caseId if provided
      if (variables.caseId) {
        return payload.simulationCompleted.caseId === variables.caseId;
      }

      // Otherwise, only show simulations from user's organization
      if (!context.user) {
        return false;
      }

      // TODO: Check if simulation belongs to user's organization
      return true;
    }
  ),
};

/**
 * Subscribe to simulation failures
 */
export const simulationFailedSubscription = {
  subscribe: withFilter(
    (_: unknown, __: unknown, context: GraphQLContext) => {
      return context.pubsub.asyncIterator([SIMULATION_TOPICS.SIMULATION_FAILED]);
    },
    (
      payload: { simulationFailed: Simulation },
      variables: { caseId?: string }
    ) => {
      if (variables.caseId) {
        return payload.simulationFailed.caseId === variables.caseId;
      }
      return true;
    }
  ),
};

/**
 * Subscribe to simulation start events
 */
export const simulationStartedSubscription = {
  subscribe: withFilter(
    (_: unknown, __: unknown, context: GraphQLContext) => {
      return context.pubsub.asyncIterator([SIMULATION_TOPICS.SIMULATION_STARTED]);
    },
    (
      payload: { simulationStarted: Simulation },
      variables: { caseId?: string }
    ) => {
      if (variables.caseId) {
        return payload.simulationStarted.caseId === variables.caseId;
      }
      return true;
    }
  ),
};

/**
 * Helper function to publish simulation progress
 */
export async function publishSimulationProgress(
  context: GraphQLContext,
  payload: SimulationProgressPayload
): Promise<void> {
  await context.pubsub.publish(
    `${SIMULATION_TOPICS.SIMULATION_PROGRESS}_${payload.simulation.id}`,
    { simulationProgress: payload }
  );
}

/**
 * Helper function to publish simulation completion
 */
export async function publishSimulationCompleted(
  context: GraphQLContext,
  simulation: Simulation
): Promise<void> {
  await context.pubsub.publish(SIMULATION_TOPICS.SIMULATION_COMPLETED, {
    simulationCompleted: simulation,
  });

  // Also publish to case-specific topic
  await context.pubsub.publish(`${SIMULATION_TOPICS.SIMULATION_COMPLETED}_${simulation.caseId}`, {
    simulationCompleted: simulation,
  });
}

/**
 * Helper function to publish simulation failure
 */
export async function publishSimulationFailed(
  context: GraphQLContext,
  simulation: Simulation
): Promise<void> {
  await context.pubsub.publish(SIMULATION_TOPICS.SIMULATION_FAILED, {
    simulationFailed: simulation,
  });
}

/**
 * Helper function to publish simulation start
 */
export async function publishSimulationStarted(
  context: GraphQLContext,
  simulation: Simulation
): Promise<void> {
  await context.pubsub.publish(SIMULATION_TOPICS.SIMULATION_STARTED, {
    simulationStarted: simulation,
  });
}
