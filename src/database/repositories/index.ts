/**
 * Repository index - Export all custom repositories
 * Provides centralized access to all repository classes
 */

import { DataSource } from 'typeorm';
import { UserRepository } from './UserRepository';
import { CaseRepository } from './CaseRepository';
import { AccidentRepository } from './AccidentRepository';

/**
 * Initialize all repositories with a data source
 */
export const initializeRepositories = (dataSource: DataSource) => {
  return {
    users: new UserRepository(dataSource),
    cases: new CaseRepository(dataSource),
    accidents: new AccidentRepository(dataSource),
  };
};

/**
 * Repository collection type
 */
export type Repositories = ReturnType<typeof initializeRepositories>;

// Export individual repositories
export { UserRepository } from './UserRepository';
export { CaseRepository } from './CaseRepository';
export { AccidentRepository } from './AccidentRepository';
