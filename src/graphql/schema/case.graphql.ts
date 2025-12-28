/**
 * Case GraphQL Schema
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { gql } from 'graphql-tag';

export const caseTypeDefs = gql`
  """
  Represents a case in the accident recreation system
  """
  type Case @key(fields: "id") {
    id: ID!
    caseNumber: String!
    title: String!
    description: String
    status: CaseStatus!
    priority: CasePriority!
    incidentDate: DateTime!
    location: Location!
    weather: WeatherCondition!
    investigatorId: ID!
    investigator: User
    organizationId: ID!
    vehicles: [Vehicle!]!
    scenes: [Scene!]!
    simulations: [Simulation!]!
    reports: [Report!]!
    metadata: JSONObject
    createdAt: DateTime!
    updatedAt: DateTime!
    deletedAt: DateTime
  }

  """
  Case status enumeration
  """
  enum CaseStatus {
    DRAFT
    ACTIVE
    REVIEW
    COMPLETED
    ARCHIVED
  }

  """
  Case priority enumeration
  """
  enum CasePriority {
    LOW
    MEDIUM
    HIGH
    CRITICAL
  }

  """
  Location information
  """
  type Location {
    address: String!
    city: String!
    state: String!
    zipCode: String!
    country: String!
    coordinates: Coordinates
  }

  """
  Geographic coordinates
  """
  type Coordinates {
    latitude: Float!
    longitude: Float!
  }

  """
  Weather conditions at the time of incident
  """
  type WeatherCondition {
    temperature: Float!
    humidity: Float!
    windSpeed: Float!
    windDirection: Float!
    visibility: Float!
    precipitation: Float!
    conditions: String!
  }

  """
  Input type for creating a new case
  """
  input CreateCaseInput {
    title: String!
    description: String
    incidentDate: DateTime!
    location: LocationInput!
    weather: WeatherConditionInput
    priority: CasePriority = MEDIUM
  }

  """
  Input type for updating a case
  """
  input UpdateCaseInput {
    title: String
    description: String
    status: CaseStatus
    priority: CasePriority
    incidentDate: DateTime
    location: LocationInput
    weather: WeatherConditionInput
  }

  """
  Input type for location
  """
  input LocationInput {
    address: String!
    city: String!
    state: String!
    zipCode: String!
    country: String!
    coordinates: CoordinatesInput
  }

  """
  Input type for coordinates
  """
  input CoordinatesInput {
    latitude: Float!
    longitude: Float!
  }

  """
  Input type for weather conditions
  """
  input WeatherConditionInput {
    temperature: Float!
    humidity: Float!
    windSpeed: Float!
    windDirection: Float!
    visibility: Float!
    precipitation: Float!
    conditions: String!
  }

  """
  Filter options for cases
  """
  input CaseFilter {
    status: [CaseStatus!]
    priority: [CasePriority!]
    investigatorId: ID
    organizationId: ID
    dateFrom: DateTime
    dateTo: DateTime
    search: String
  }

  """
  Paginated case results
  """
  type CasePage {
    items: [Case!]!
    total: Int!
    page: Int!
    limit: Int!
    totalPages: Int!
    hasNextPage: Boolean!
    hasPreviousPage: Boolean!
  }

  extend type Query {
    """
    Get a single case by ID
    """
    case(id: ID!): Case @auth(requires: [CASE_READ])

    """
    Get multiple cases with filtering and pagination
    """
    cases(
      filter: CaseFilter
      pagination: PaginationInput
    ): CasePage! @auth(requires: [CASE_READ]) @rateLimit(max: 100, window: 60)

    """
    Get cases assigned to the current user
    """
    myCases(
      pagination: PaginationInput
    ): CasePage! @auth(requires: [CASE_READ])

    """
    Search cases by text
    """
    searchCases(
      query: String!
      pagination: PaginationInput
    ): CasePage! @auth(requires: [CASE_READ])
  }

  extend type Mutation {
    """
    Create a new case
    """
    createCase(input: CreateCaseInput!): Case! @auth(requires: [CASE_CREATE])

    """
    Update an existing case
    """
    updateCase(id: ID!, input: UpdateCaseInput!): Case! @auth(requires: [CASE_UPDATE])

    """
    Delete a case (soft delete)
    """
    deleteCase(id: ID!): Boolean! @auth(requires: [CASE_DELETE])

    """
    Archive a case
    """
    archiveCase(id: ID!): Case! @auth(requires: [CASE_UPDATE])

    """
    Restore a deleted case
    """
    restoreCase(id: ID!): Case! @auth(requires: [CASE_UPDATE])

    """
    Assign a case to an investigator
    """
    assignCase(caseId: ID!, investigatorId: ID!): Case! @auth(requires: [CASE_UPDATE])

    """
    Change case status
    """
    changeCaseStatus(caseId: ID!, status: CaseStatus!): Case! @auth(requires: [CASE_UPDATE])

    """
    Duplicate a case
    """
    duplicateCase(id: ID!): Case! @auth(requires: [CASE_CREATE])
  }

  extend type Subscription {
    """
    Subscribe to case updates
    """
    caseUpdated(caseId: ID): CaseUpdatePayload! @auth(requires: [CASE_READ])

    """
    Subscribe to new cases
    """
    caseCreated: Case! @auth(requires: [CASE_READ])
  }

  """
  Case update subscription payload
  """
  type CaseUpdatePayload {
    case: Case!
    mutation: MutationType!
    userId: ID!
  }

  """
  Mutation type enumeration
  """
  enum MutationType {
    CREATED
    UPDATED
    DELETED
  }
`;
