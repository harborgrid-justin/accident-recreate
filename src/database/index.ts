import { DataSource } from 'typeorm';
import { databaseConfig, getDbConfig, connectionOptions } from './config';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Database Connection Manager for AccuScene Enterprise Platform
 * Handles database initialization, connection lifecycle, and graceful shutdown
 */

export class DatabaseManager {
  private static instance: DatabaseManager;
  private dataSource: DataSource | null = null;
  private isInitialized = false;

  private constructor() {}

  /**
   * Get singleton instance of DatabaseManager
   */
  public static getInstance(): DatabaseManager {
    if (!DatabaseManager.instance) {
      DatabaseManager.instance = new DatabaseManager();
    }
    return DatabaseManager.instance;
  }

  /**
   * Initialize database connection
   * Creates data directory if it doesn't exist
   */
  public async initialize(): Promise<DataSource> {
    if (this.isInitialized && this.dataSource?.isInitialized) {
      return this.dataSource;
    }

    try {
      // Ensure data directory exists (skip for in-memory databases)
      const config = getDbConfig();
      if (config.database !== ':memory:') {
        const dbPath = config.database as string;
        const dbDir = path.dirname(dbPath);

        if (!fs.existsSync(dbDir)) {
          fs.mkdirSync(dbDir, { recursive: true });
          console.log(`Created database directory: ${dbDir}`);
        }
      }

      // Create and initialize data source
      this.dataSource = new DataSource(config);
      await this.dataSource.initialize();

      this.isInitialized = true;
      console.log('Database connection established successfully');
      console.log(`Database type: ${config.type}`);
      console.log(`Database location: ${config.database}`);

      // Run migrations if needed
      if (config.migrationsRun) {
        const pendingMigrations = await this.dataSource.showMigrations();
        if (pendingMigrations) {
          console.log('Running pending migrations...');
          await this.dataSource.runMigrations();
          console.log('Migrations completed successfully');
        }
      }

      return this.dataSource;
    } catch (error) {
      console.error('Failed to initialize database connection:', error);
      throw new Error(`Database initialization failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Get active data source
   * Throws error if not initialized
   */
  public getDataSource(): DataSource {
    if (!this.dataSource || !this.isInitialized) {
      throw new Error('Database not initialized. Call initialize() first.');
    }
    return this.dataSource;
  }

  /**
   * Check if database is connected and initialized
   */
  public isConnected(): boolean {
    return this.isInitialized && this.dataSource?.isInitialized === true;
  }

  /**
   * Close database connection gracefully
   */
  public async close(): Promise<void> {
    if (this.dataSource?.isInitialized) {
      await this.dataSource.destroy();
      this.isInitialized = false;
      console.log('Database connection closed');
    }
  }

  /**
   * Reset database (for testing purposes only)
   * Drops all data and re-runs migrations
   */
  public async reset(): Promise<void> {
    if (!this.dataSource?.isInitialized) {
      throw new Error('Database not initialized');
    }

    if (process.env.NODE_ENV === 'production') {
      throw new Error('Database reset is not allowed in production');
    }

    try {
      await this.dataSource.synchronize(true); // Drop and recreate schema
      console.log('Database reset successfully');
    } catch (error) {
      console.error('Failed to reset database:', error);
      throw error;
    }
  }

  /**
   * Execute health check on database connection
   */
  public async healthCheck(): Promise<{
    connected: boolean;
    responseTime: number;
    error?: string;
  }> {
    const startTime = Date.now();

    try {
      if (!this.isConnected()) {
        return {
          connected: false,
          responseTime: Date.now() - startTime,
          error: 'Database not initialized',
        };
      }

      // Simple query to test connection
      await this.dataSource!.query('SELECT 1');

      return {
        connected: true,
        responseTime: Date.now() - startTime,
      };
    } catch (error) {
      return {
        connected: false,
        responseTime: Date.now() - startTime,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }
}

// Export singleton instance methods
export const dbManager = DatabaseManager.getInstance();

/**
 * Initialize database connection
 * Convenience function for common use case
 */
export const initializeDatabase = async (): Promise<DataSource> => {
  return dbManager.initialize();
};

/**
 * Get active database connection
 * Convenience function for common use case
 */
export const getConnection = (): DataSource => {
  return dbManager.getDataSource();
};

/**
 * Close database connection
 * Convenience function for common use case
 */
export const closeDatabase = async (): Promise<void> => {
  return dbManager.close();
};

// Graceful shutdown handlers
process.on('SIGINT', async () => {
  console.log('SIGINT received: closing database connection...');
  await closeDatabase();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  console.log('SIGTERM received: closing database connection...');
  await closeDatabase();
  process.exit(0);
});

// Export all entities for convenience
export * from './entities/User';
export * from './entities/Case';
export * from './entities/Accident';
export * from './entities/Vehicle';
export * from './entities/Witness';
export * from './entities/Evidence';
export * from './entities/InsuranceClaim';
