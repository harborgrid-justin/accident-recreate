import { DataSourceOptions } from 'typeorm';
import { User } from './entities/User';
import { Case } from './entities/Case';
import { Accident } from './entities/Accident';
import { Vehicle } from './entities/Vehicle';
import { Witness } from './entities/Witness';
import { Evidence } from './entities/Evidence';
import { InsuranceClaim } from './entities/InsuranceClaim';
import * as path from 'path';

/**
 * TypeORM Database Configuration for AccuScene Enterprise Platform
 * SQLite database for portability and zero external dependencies
 */

const isDevelopment = process.env.NODE_ENV !== 'production';
const isTest = process.env.NODE_ENV === 'test';

// Determine database path based on environment
const getDatabasePath = (): string => {
  if (isTest) {
    return ':memory:'; // In-memory database for testing
  }

  const dbDir = process.env.DB_PATH || path.join(process.cwd(), 'data');
  const dbFile = process.env.DB_NAME || 'accuscene.db';

  return path.join(dbDir, dbFile);
};

export const databaseConfig: DataSourceOptions = {
  type: 'sqlite',
  database: getDatabasePath(),

  // Entity registration
  entities: [
    User,
    Case,
    Accident,
    Vehicle,
    Witness,
    Evidence,
    InsuranceClaim,
  ],

  // Migration configuration
  migrations: [path.join(__dirname, 'migrations', '*.{ts,js}')],
  migrationsTableName: 'migrations_history',
  migrationsRun: true, // Auto-run migrations on startup

  // Synchronize schema in development (WARNING: Don't use in production)
  synchronize: isDevelopment && !isTest,

  // Logging configuration
  logging: isDevelopment ? ['query', 'error', 'warn'] : ['error'],
  logger: 'advanced-console',

  // Connection pool settings (SQLite has limited concurrency)
  maxQueryExecutionTime: 5000, // Log slow queries > 5s

  // Additional SQLite-specific settings
  enableWAL: true, // Write-Ahead Logging for better concurrency
  cache: {
    type: 'database',
    duration: 30000, // Cache results for 30 seconds
  },
};

/**
 * Database configuration for testing environment
 * Uses in-memory SQLite database for fast, isolated tests
 */
export const testDatabaseConfig: DataSourceOptions = {
  ...databaseConfig,
  database: ':memory:',
  synchronize: true,
  logging: false,
  dropSchema: true, // Clean database before each test run
  migrationsRun: false,
};

/**
 * Get appropriate database configuration based on environment
 */
export const getDbConfig = (): DataSourceOptions => {
  return isTest ? testDatabaseConfig : databaseConfig;
};

/**
 * Database connection options
 */
export const connectionOptions = {
  retryAttempts: 10,
  retryDelay: 3000,
  autoLoadEntities: true,
};
