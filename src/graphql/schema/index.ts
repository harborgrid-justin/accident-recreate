/**
 * GraphQL Schema Composition
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { gql } from 'graphql-tag';
import { caseTypeDefs } from './case.graphql';
import { vehicleTypeDefs } from './vehicle.graphql';
import { sceneTypeDefs } from './scene.graphql';
import { simulationTypeDefs } from './simulation.graphql';
import { userTypeDefs } from './user.graphql';
import { reportTypeDefs } from './report.graphql';

// ============================================================================
// Base Schema - Common types and directives
// ============================================================================

const baseTypeDefs = gql`
  # Custom Scalars
  scalar DateTime
  scalar JSON
  scalar JSONObject
  scalar URL
  scalar Email
  scalar PositiveInt
  scalar PositiveFloat
  scalar Upload

  # Directives
  directive @auth(requires: [Permission!]!) on FIELD_DEFINITION | OBJECT
  directive @rateLimit(max: Int!, window: Int!) on FIELD_DEFINITION
  directive @validate(schema: String!) on INPUT_FIELD_DEFINITION | ARGUMENT_DEFINITION
  directive @key(fields: String!) on OBJECT | INTERFACE
  directive @extends on OBJECT | INTERFACE

  # Base Query type
  type Query {
    _service: _Service!
    _empty: String
  }

  # Base Mutation type
  type Mutation {
    _empty: String
  }

  # Base Subscription type
  type Subscription {
    _empty: String
  }

  # Federation service type
  type _Service {
    sdl: String!
  }

  # Pagination input type
  input PaginationInput {
    page: Int = 1
    limit: Int = 20
    sortBy: String
    sortOrder: SortOrder = DESC
  }

  # Sort order enumeration
  enum SortOrder {
    ASC
    DESC
  }

  # Generic error type
  type Error {
    message: String!
    code: String!
    path: [String!]
    extensions: JSONObject
  }

  # Health check type
  type HealthCheck {
    status: String!
    version: String!
    uptime: Float!
    timestamp: DateTime!
  }

  # Add health check query
  extend type Query {
    health: HealthCheck!
  }
`;

// ============================================================================
// Combine all type definitions
// ============================================================================

export const typeDefs = [
  baseTypeDefs,
  caseTypeDefs,
  vehicleTypeDefs,
  sceneTypeDefs,
  simulationTypeDefs,
  userTypeDefs,
  reportTypeDefs,
];

// ============================================================================
// Export individual type defs for testing
// ============================================================================

export {
  baseTypeDefs,
  caseTypeDefs,
  vehicleTypeDefs,
  sceneTypeDefs,
  simulationTypeDefs,
  userTypeDefs,
  reportTypeDefs,
};
