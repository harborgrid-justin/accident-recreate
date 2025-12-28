/**
 * GraphQL Federation API - Main Export
 * AccuScene Enterprise v0.2.0
 *
 * @module GraphQL
 * @description Comprehensive GraphQL Federation API system with:
 * - Apollo Server with Federation support
 * - Real-time subscriptions via WebSocket
 * - DataLoaders for N+1 query prevention
 * - Custom directives (@auth, @rateLimit, @validate)
 * - Comprehensive error handling and logging
 * - Type-safe resolvers and context
 */

// ============================================================================
// Server & Configuration
// ============================================================================

export {
  createApolloServer,
  setupGraphQLServer,
  startGraphQLServer,
  shutdownServer,
} from './server';

export {
  buildFederatedSchema,
  getFederationHealth,
  federationConfig,
  getMetrics,
  updateMetrics,
  resetMetrics,
} from './federation';

export { playgroundConfig, getExampleQueries } from './playground';

// ============================================================================
// Schema & Resolvers
// ============================================================================

export { typeDefs } from './schema';
export { resolvers } from './resolvers';

export {
  baseTypeDefs,
  caseTypeDefs,
  vehicleTypeDefs,
  sceneTypeDefs,
  simulationTypeDefs,
  userTypeDefs,
  reportTypeDefs,
} from './schema';

// ============================================================================
// Context & Types
// ============================================================================

export { createContext, createWebSocketContext, createJWT, verifyJWT } from './context';

export type {
  GraphQLContext,
  AuthenticatedUser,
  DataLoaders,
  Case,
  Vehicle,
  Scene,
  Simulation,
  User,
  Report,
  CreateCaseInput,
  UpdateCaseInput,
  CreateVehicleInput,
  CreateSimulationInput,
  GenerateReportInput,
  PaginatedResponse,
  CaseFilter,
} from './types';

export {
  UserRole,
  Permission,
  CaseStatus,
  CasePriority,
  VehicleType,
  VehicleRole,
  SimulationStatus,
  ReportType,
  ReportFormat,
  ReportStatus,
} from './types';

// ============================================================================
// Custom Scalars
// ============================================================================

export {
  customScalars,
  DateTimeScalar,
  JSONScalar,
  JSONObjectScalar,
  URLScalar,
  EmailScalar,
  PositiveIntScalar,
  PositiveFloatScalar,
  UploadScalar,
} from './scalars';

// ============================================================================
// Directives
// ============================================================================

export {
  authDirective,
  rateLimitDirective,
  validateDirective,
  registerValidationSchema,
} from './directives';

// ============================================================================
// DataLoaders
// ============================================================================

export { createDataLoaders, clearDataLoaderCaches } from './dataloaders';
export { createCaseLoader } from './dataloaders/case.loader';
export { createVehicleLoader } from './dataloaders/vehicle.loader';

// ============================================================================
// Subscriptions
// ============================================================================

export {
  CASE_TOPICS,
  caseUpdatedSubscription,
  caseCreatedSubscription,
  publishCaseUpdate,
  publishCaseCreated,
  SIMULATION_TOPICS,
  simulationProgressSubscription,
  simulationCompletedSubscription,
  publishSimulationProgress,
  publishSimulationCompleted,
} from './subscriptions';

// ============================================================================
// Middleware
// ============================================================================

export {
  requireAuth,
  requirePermission,
  requireAnyPermission,
  requireAllPermissions,
  requireOrganization,
  requireAdmin,
  requireOwnership,
} from './middleware/auth.middleware';

export {
  loggingPlugin,
  logOperation,
  logError,
  logger,
} from './middleware/logging.middleware';

export {
  errorHandlingPlugin,
  formatError,
  createError,
  createValidationError,
  createNotFoundError,
  createUnauthorizedError,
  createUnauthenticatedError,
  createRateLimitError,
  createConflictError,
  createBadRequestError,
  ErrorCode,
} from './middleware/error.middleware';

// ============================================================================
// Version & Metadata
// ============================================================================

export const GRAPHQL_VERSION = '0.2.0';
export const GRAPHQL_API_NAME = 'AccuScene Enterprise GraphQL API';

/**
 * API Information
 */
export const API_INFO = {
  name: GRAPHQL_API_NAME,
  version: GRAPHQL_VERSION,
  description: 'Federated GraphQL API for AccuScene Enterprise Accident Recreation Platform',
  capabilities: {
    federation: true,
    subscriptions: true,
    fileUploads: true,
    batching: true,
    caching: true,
    customDirectives: true,
    customScalars: true,
    authentication: true,
    authorization: true,
    rateLimit: true,
    validation: true,
    logging: true,
    errorHandling: true,
  },
  features: [
    'Case Management',
    'Vehicle Tracking',
    'Scene Reconstruction',
    'Physics Simulation',
    'Report Generation',
    'User Authentication',
    'Real-time Collaboration',
    'WebSocket Subscriptions',
    'N+1 Query Prevention',
    'Field-level Authorization',
  ],
};

/**
 * Get API information
 */
export function getAPIInfo() {
  return {
    ...API_INFO,
    timestamp: new Date().toISOString(),
    environment: process.env.NODE_ENV || 'development',
    uptime: process.uptime(),
  };
}

// ============================================================================
// Default Export
// ============================================================================

export default {
  startGraphQLServer,
  getAPIInfo,
  API_INFO,
};
