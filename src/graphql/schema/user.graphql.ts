/**
 * User GraphQL Schema
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { gql } from 'graphql-tag';

export const userTypeDefs = gql`
  """
  Represents a user in the system
  """
  type User @key(fields: "id") {
    id: ID!
    email: Email!
    firstName: String!
    lastName: String!
    fullName: String!
    role: UserRole!
    organizationId: ID!
    organization: Organization
    permissions: [Permission!]!
    avatar: URL
    preferences: UserPreferences!
    lastLoginAt: DateTime
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  """
  User role enumeration
  """
  enum UserRole {
    ADMIN
    INVESTIGATOR
    ANALYST
    VIEWER
  }

  """
  Permission enumeration
  """
  enum Permission {
    CASE_CREATE
    CASE_READ
    CASE_UPDATE
    CASE_DELETE
    SIMULATION_RUN
    REPORT_GENERATE
    USER_MANAGE
  }

  """
  Organization information
  """
  type Organization {
    id: ID!
    name: String!
    type: OrganizationType!
    address: String
    phone: String
    email: Email
    website: URL
    licenseType: LicenseType!
    licenseExpiry: DateTime
    users: [User!]!
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  """
  Organization type enumeration
  """
  enum OrganizationType {
    LAW_ENFORCEMENT
    INSURANCE
    LEGAL
    CONSULTING
    ACADEMIC
  }

  """
  License type enumeration
  """
  enum LicenseType {
    TRIAL
    BASIC
    PROFESSIONAL
    ENTERPRISE
  }

  """
  User preferences
  """
  type UserPreferences {
    theme: Theme!
    notifications: NotificationPreferences!
    defaultMeasurementUnit: MeasurementUnit!
    locale: String!
    timezone: String!
  }

  """
  Theme enumeration
  """
  enum Theme {
    LIGHT
    DARK
    AUTO
  }

  """
  Notification preferences
  """
  type NotificationPreferences {
    email: Boolean!
    push: Boolean!
    inApp: Boolean!
  }

  """
  Authentication payload
  """
  type AuthPayload {
    token: String!
    refreshToken: String!
    user: User!
    expiresIn: Int!
  }

  """
  Input type for user registration
  """
  input RegisterInput {
    email: Email!
    password: String!
    firstName: String!
    lastName: String!
    organizationName: String!
    organizationType: OrganizationType!
  }

  """
  Input type for user login
  """
  input LoginInput {
    email: Email!
    password: String!
  }

  """
  Input type for updating user profile
  """
  input UpdateUserInput {
    firstName: String
    lastName: String
    avatar: URL
    preferences: UserPreferencesInput
  }

  """
  Input type for user preferences
  """
  input UserPreferencesInput {
    theme: Theme
    notifications: NotificationPreferencesInput
    defaultMeasurementUnit: MeasurementUnit
    locale: String
    timezone: String
  }

  """
  Input type for notification preferences
  """
  input NotificationPreferencesInput {
    email: Boolean
    push: Boolean
    inApp: Boolean
  }

  """
  Input type for creating a user (admin only)
  """
  input CreateUserInput {
    email: Email!
    password: String!
    firstName: String!
    lastName: String!
    role: UserRole!
    organizationId: ID!
  }

  """
  Input type for updating user role (admin only)
  """
  input UpdateUserRoleInput {
    role: UserRole!
    permissions: [Permission!]
  }

  extend type Query {
    """
    Get current authenticated user
    """
    me: User @auth(requires: [CASE_READ])

    """
    Get a single user by ID
    """
    user(id: ID!): User @auth(requires: [USER_MANAGE])

    """
    Get all users in the organization
    """
    users(pagination: PaginationInput): UserPage! @auth(requires: [USER_MANAGE])

    """
    Search users by email or name
    """
    searchUsers(query: String!, pagination: PaginationInput): UserPage! @auth(requires: [USER_MANAGE])

    """
    Get user activity log
    """
    userActivity(userId: ID!, pagination: PaginationInput): ActivityPage! @auth(requires: [USER_MANAGE])
  }

  extend type Mutation {
    """
    Register a new user and organization
    """
    register(input: RegisterInput!): AuthPayload!

    """
    Login user
    """
    login(input: LoginInput!): AuthPayload!

    """
    Refresh authentication token
    """
    refreshToken(refreshToken: String!): AuthPayload!

    """
    Logout user
    """
    logout: Boolean! @auth(requires: [CASE_READ])

    """
    Update current user profile
    """
    updateProfile(input: UpdateUserInput!): User! @auth(requires: [CASE_READ])

    """
    Change password
    """
    changePassword(currentPassword: String!, newPassword: String!): Boolean! @auth(requires: [CASE_READ])

    """
    Request password reset
    """
    requestPasswordReset(email: Email!): Boolean!

    """
    Reset password with token
    """
    resetPassword(token: String!, newPassword: String!): Boolean!

    """
    Create a new user (admin only)
    """
    createUser(input: CreateUserInput!): User! @auth(requires: [USER_MANAGE])

    """
    Update user role and permissions (admin only)
    """
    updateUserRole(userId: ID!, input: UpdateUserRoleInput!): User! @auth(requires: [USER_MANAGE])

    """
    Delete user (admin only)
    """
    deleteUser(userId: ID!): Boolean! @auth(requires: [USER_MANAGE])

    """
    Upload user avatar
    """
    uploadAvatar(file: Upload!): User! @auth(requires: [CASE_READ])
  }

  """
  Paginated user results
  """
  type UserPage {
    items: [User!]!
    total: Int!
    page: Int!
    limit: Int!
    totalPages: Int!
    hasNextPage: Boolean!
    hasPreviousPage: Boolean!
  }

  """
  User activity entry
  """
  type Activity {
    id: ID!
    userId: ID!
    user: User
    action: String!
    resource: String!
    resourceId: ID
    metadata: JSONObject
    ipAddress: String
    userAgent: String
    createdAt: DateTime!
  }

  """
  Paginated activity results
  """
  type ActivityPage {
    items: [Activity!]!
    total: Int!
    page: Int!
    limit: Int!
    totalPages: Int!
    hasNextPage: Boolean!
    hasPreviousPage: Boolean!
  }
`;
