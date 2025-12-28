/**
 * AccuScene Enterprise v0.2.0
 * Main Entry Point
 *
 * @version 0.2.0
 * @module accuscene-enterprise
 */

// Export all enterprise features
export * from './enterprise';

// Export all type definitions
export * from './types';

// Core application version
export const VERSION = '0.2.0';
export const APPLICATION_NAME = 'AccuScene Enterprise';
export const BUILD_DATE = new Date().toISOString();

// Application metadata
export const APP_METADATA = {
  name: APPLICATION_NAME,
  version: VERSION,
  buildDate: BUILD_DATE,
  features: [
    'Accident Recreation',
    'Physics Simulation',
    'GraphQL Federation API',
    'Real-time Collaboration (CRDT)',
    'Advanced UI Components (3D, AR)',
    'Plugin Architecture',
    'Performance Monitoring',
  ],
  enterprise: true,
};
