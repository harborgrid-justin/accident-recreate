/**
 * Vehicle DataLoader
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 *
 * Implements batching and caching for vehicle queries to prevent N+1 problems
 */

import DataLoader from 'dataloader';
import { Vehicle } from '../types';

/**
 * Batch function to load multiple vehicles by ID
 */
async function batchLoadVehicles(ids: readonly string[]): Promise<(Vehicle | Error)[]> {
  try {
    // TODO: Replace with actual database query
    const vehiclesMap = new Map<string, Vehicle>();

    // Simulated database query
    // const results = await database.query(
    //   'SELECT * FROM vehicles WHERE id = ANY($1)',
    //   [ids]
    // );

    return ids.map((id) => {
      const vehicle = vehiclesMap.get(id);
      if (vehicle) {
        return vehicle;
      }
      return new Error(`Vehicle with ID ${id} not found`);
    });
  } catch (error) {
    return ids.map(() => error as Error);
  }
}

/**
 * Create a new Vehicle DataLoader instance
 */
export function createVehicleLoader(): DataLoader<string, Vehicle> {
  return new DataLoader<string, Vehicle>(batchLoadVehicles, {
    cache: true,
    maxBatchSize: 100,
    batchScheduleFn: (callback) => setTimeout(callback, 10),
  });
}
