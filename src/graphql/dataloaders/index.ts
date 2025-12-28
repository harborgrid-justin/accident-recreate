/**
 * DataLoader Factory
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 *
 * Creates and manages all DataLoader instances for efficient data fetching
 */

import DataLoader from 'dataloader';
import { DataLoaders, Case, Vehicle, Scene, User, Report, Simulation } from '../types';
import { createCaseLoader } from './case.loader';
import { createVehicleLoader } from './vehicle.loader';

/**
 * Create all DataLoader instances
 * DataLoaders provide:
 * - Batching: Combines multiple requests into a single database query
 * - Caching: Caches results for the duration of a single request
 */
export function createDataLoaders(): DataLoaders {
  return {
    caseLoader: createCaseLoader(),
    vehicleLoader: createVehicleLoader(),
    sceneLoader: createSceneLoader(),
    userLoader: createUserLoader(),
    reportLoader: createReportLoader(),
    simulationLoader: createSimulationLoader(),
  };
}

/**
 * Scene DataLoader
 */
function createSceneLoader(): DataLoader<string, Scene> {
  return new DataLoader<string, Scene>(async (ids) => {
    try {
      // TODO: Implement database query
      return ids.map((id) => new Error(`Scene with ID ${id} not found`));
    } catch (error) {
      return ids.map(() => error as Error);
    }
  }, {
    cache: true,
    maxBatchSize: 100,
    batchScheduleFn: (callback) => setTimeout(callback, 10),
  });
}

/**
 * User DataLoader
 */
function createUserLoader(): DataLoader<string, User> {
  return new DataLoader<string, User>(async (ids) => {
    try {
      // TODO: Implement database query
      return ids.map((id) => new Error(`User with ID ${id} not found`));
    } catch (error) {
      return ids.map(() => error as Error);
    }
  }, {
    cache: true,
    maxBatchSize: 100,
    batchScheduleFn: (callback) => setTimeout(callback, 10),
  });
}

/**
 * Report DataLoader
 */
function createReportLoader(): DataLoader<string, Report> {
  return new DataLoader<string, Report>(async (ids) => {
    try {
      // TODO: Implement database query
      return ids.map((id) => new Error(`Report with ID ${id} not found`));
    } catch (error) {
      return ids.map(() => error as Error);
    }
  }, {
    cache: true,
    maxBatchSize: 100,
    batchScheduleFn: (callback) => setTimeout(callback, 10),
  });
}

/**
 * Simulation DataLoader
 */
function createSimulationLoader(): DataLoader<string, Simulation> {
  return new DataLoader<string, Simulation>(async (ids) => {
    try {
      // TODO: Implement database query
      return ids.map((id) => new Error(`Simulation with ID ${id} not found`));
    } catch (error) {
      return ids.map(() => error as Error);
    }
  }, {
    cache: true,
    maxBatchSize: 100,
    batchScheduleFn: (callback) => setTimeout(callback, 10),
  });
}

/**
 * Clear all DataLoader caches
 * Useful for testing or when data changes outside the request lifecycle
 */
export function clearDataLoaderCaches(dataloaders: DataLoaders): void {
  dataloaders.caseLoader.clearAll();
  dataloaders.vehicleLoader.clearAll();
  dataloaders.sceneLoader.clearAll();
  dataloaders.userLoader.clearAll();
  dataloaders.reportLoader.clearAll();
  dataloaders.simulationLoader.clearAll();
}
