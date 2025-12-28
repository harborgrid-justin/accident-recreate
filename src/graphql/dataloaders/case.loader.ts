/**
 * Case DataLoader
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 *
 * Implements batching and caching for case queries to prevent N+1 problems
 */

import DataLoader from 'dataloader';
import { Case } from '../types';

/**
 * Batch function to load multiple cases by ID
 * In production, this would query the database with WHERE id IN (...)
 */
async function batchLoadCases(ids: readonly string[]): Promise<(Case | Error)[]> {
  try {
    // TODO: Replace with actual database query
    // Example: const cases = await db.case.findMany({ where: { id: { in: ids } } });

    // Mock implementation - in production, query database
    const casesMap = new Map<string, Case>();

    // Simulated database query
    // const results = await database.query(
    //   'SELECT * FROM cases WHERE id = ANY($1)',
    //   [ids]
    // );

    // For now, return null for each ID (will be replaced with actual data)
    return ids.map((id) => {
      const caseData = casesMap.get(id);
      if (caseData) {
        return caseData;
      }
      // Return null which will be handled as not found
      return new Error(`Case with ID ${id} not found`);
    });
  } catch (error) {
    // Return errors for each ID
    return ids.map(() => error as Error);
  }
}

/**
 * Create a new Case DataLoader instance
 */
export function createCaseLoader(): DataLoader<string, Case> {
  return new DataLoader<string, Case>(batchLoadCases, {
    // Cache configuration
    cache: true,
    // Maximum batch size
    maxBatchSize: 100,
    // Batch scheduling (batches requests within same tick)
    batchScheduleFn: (callback) => setTimeout(callback, 10),
  });
}
