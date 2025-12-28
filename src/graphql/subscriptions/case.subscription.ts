/**
 * Case Subscriptions
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { withFilter } from 'graphql-subscriptions';
import { GraphQLContext, CaseUpdatePayload, Case } from '../types';

/**
 * Case update subscription topics
 */
export const CASE_TOPICS = {
  CASE_UPDATED: 'CASE_UPDATED',
  CASE_CREATED: 'CASE_CREATED',
  CASE_DELETED: 'CASE_DELETED',
} as const;

/**
 * Subscribe to case updates
 * Filters by caseId if provided
 */
export const caseUpdatedSubscription = {
  subscribe: withFilter(
    (_: unknown, __: unknown, context: GraphQLContext) => {
      return context.pubsub.asyncIterator([CASE_TOPICS.CASE_UPDATED]);
    },
    (payload: { caseUpdated: CaseUpdatePayload }, variables: { caseId?: string }) => {
      // If caseId is specified, only send updates for that case
      if (variables.caseId) {
        return payload.caseUpdated.case.id === variables.caseId;
      }
      // Otherwise, send all case updates
      return true;
    }
  ),
};

/**
 * Subscribe to new case creation
 * Filters by organizationId for the current user
 */
export const caseCreatedSubscription = {
  subscribe: withFilter(
    (_: unknown, __: unknown, context: GraphQLContext) => {
      return context.pubsub.asyncIterator([CASE_TOPICS.CASE_CREATED]);
    },
    (payload: { caseCreated: Case }, _: unknown, context: GraphQLContext) => {
      // Only send notifications for cases in the user's organization
      if (!context.user) {
        return false;
      }
      return payload.caseCreated.organizationId === context.user.organizationId;
    }
  ),
};

/**
 * Subscribe to case status changes
 */
export const caseStatusChangedSubscription = {
  subscribe: withFilter(
    (_: unknown, __: unknown, context: GraphQLContext) => {
      return context.pubsub.asyncIterator([CASE_TOPICS.CASE_UPDATED]);
    },
    (
      payload: { caseUpdated: CaseUpdatePayload },
      variables: { caseId?: string; status?: string }
    ) => {
      const { caseUpdated } = payload;

      // Filter by caseId if provided
      if (variables.caseId && caseUpdated.case.id !== variables.caseId) {
        return false;
      }

      // Filter by status if provided
      if (variables.status && caseUpdated.case.status !== variables.status) {
        return false;
      }

      return true;
    }
  ),
};

/**
 * Helper function to publish case update
 */
export async function publishCaseUpdate(
  context: GraphQLContext,
  payload: CaseUpdatePayload
): Promise<void> {
  await context.pubsub.publish(CASE_TOPICS.CASE_UPDATED, { caseUpdated: payload });

  // Also publish to specific case topic
  await context.pubsub.publish(`${CASE_TOPICS.CASE_UPDATED}_${payload.case.id}`, {
    caseUpdated: payload,
  });
}

/**
 * Helper function to publish case creation
 */
export async function publishCaseCreated(
  context: GraphQLContext,
  caseData: Case
): Promise<void> {
  await context.pubsub.publish(CASE_TOPICS.CASE_CREATED, { caseCreated: caseData });
}
