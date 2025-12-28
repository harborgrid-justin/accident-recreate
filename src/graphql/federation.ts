/**
 * GraphQL Federation Configuration
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { buildSubgraphSchema } from '@apollo/subgraph';
import { GraphQLSchema } from 'graphql';
import { typeDefs } from './schema';
import { resolvers } from './resolvers';

/**
 * Build federated GraphQL schema
 * Uses Apollo Federation to enable schema composition across multiple services
 */
export function buildFederatedSchema(): GraphQLSchema {
  return buildSubgraphSchema([
    {
      typeDefs,
      resolvers,
    },
  ]);
}

/**
 * Federation configuration options
 */
export const federationConfig = {
  // Service name for federation
  serviceName: 'accuscene-graphql-api',

  // Service version
  serviceVersion: '0.2.0',

  // Enable federated tracing
  tracing: process.env.NODE_ENV !== 'production',

  // Federation directives
  directives: {
    key: '@key',
    extends: '@extends',
    external: '@external',
    requires: '@requires',
    provides: '@provides',
  },

  // Reference resolvers configuration
  referenceResolvers: {
    // Maximum batch size for reference resolution
    maxBatchSize: 100,

    // Enable caching for reference resolution
    cache: true,
  },

  // Entity resolution configuration
  entityResolution: {
    // Timeout for entity resolution (ms)
    timeout: 5000,

    // Retry configuration
    retry: {
      maxAttempts: 3,
      backoff: 'exponential',
      initialDelay: 100,
    },
  },
};

/**
 * Federation health check
 * Returns service health information for federation gateway
 */
export function getFederationHealth() {
  return {
    name: federationConfig.serviceName,
    version: federationConfig.serviceVersion,
    status: 'healthy',
    uptime: process.uptime(),
    timestamp: new Date().toISOString(),
    capabilities: {
      subscriptions: true,
      fileUploads: true,
      customDirectives: true,
      dataloaders: true,
    },
  };
}

/**
 * Federation metrics
 * Provides metrics for monitoring federation performance
 */
export interface FederationMetrics {
  totalRequests: number;
  totalErrors: number;
  averageResponseTime: number;
  entityResolutions: number;
  cacheHits: number;
  cacheMisses: number;
}

let metrics: FederationMetrics = {
  totalRequests: 0,
  totalErrors: 0,
  averageResponseTime: 0,
  entityResolutions: 0,
  cacheHits: 0,
  cacheMisses: 0,
};

export function updateMetrics(update: Partial<FederationMetrics>): void {
  metrics = { ...metrics, ...update };
}

export function getMetrics(): FederationMetrics {
  return { ...metrics };
}

export function resetMetrics(): void {
  metrics = {
    totalRequests: 0,
    totalErrors: 0,
    averageResponseTime: 0,
    entityResolutions: 0,
    cacheHits: 0,
    cacheMisses: 0,
  };
}
