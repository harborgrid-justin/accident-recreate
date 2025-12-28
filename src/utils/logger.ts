/**
 * AccuScene Enterprise Accident Recreation Platform
 * Logging Utility
 *
 * This file provides a centralized logging system for the application.
 * All modules should use this logger for consistent log formatting and management.
 */

import { ApplicationError } from './errors';

// ============================================================================
// LOG LEVELS
// ============================================================================

export enum LogLevel {
  DEBUG = 'DEBUG',
  INFO = 'INFO',
  WARN = 'WARN',
  ERROR = 'ERROR',
  FATAL = 'FATAL',
}

// Numeric values for comparison
const LOG_LEVEL_VALUES: Record<LogLevel, number> = {
  [LogLevel.DEBUG]: 0,
  [LogLevel.INFO]: 1,
  [LogLevel.WARN]: 2,
  [LogLevel.ERROR]: 3,
  [LogLevel.FATAL]: 4,
};

// ============================================================================
// LOG ENTRY INTERFACE
// ============================================================================

export interface LogEntry {
  timestamp: Date;
  level: LogLevel;
  module: string;
  message: string;
  context?: Record<string, any>;
  error?: Error | ApplicationError;
  correlationId?: string;
  userId?: string;
  sessionId?: string;
  tags?: string[];
}

// ============================================================================
// LOG TRANSPORT INTERFACE
// ============================================================================

export interface LogTransport {
  name: string;
  minLevel: LogLevel;
  log(entry: LogEntry): void | Promise<void>;
}

// ============================================================================
// CONSOLE TRANSPORT
// ============================================================================

export class ConsoleTransport implements LogTransport {
  public readonly name = 'console';
  public readonly minLevel: LogLevel;
  private readonly colorize: boolean;

  constructor(minLevel: LogLevel = LogLevel.INFO, colorize: boolean = true) {
    this.minLevel = minLevel;
    this.colorize = colorize;
  }

  log(entry: LogEntry): void {
    const formattedMessage = this.format(entry);

    switch (entry.level) {
      case LogLevel.DEBUG:
        console.debug(formattedMessage);
        break;
      case LogLevel.INFO:
        console.info(formattedMessage);
        break;
      case LogLevel.WARN:
        console.warn(formattedMessage);
        break;
      case LogLevel.ERROR:
      case LogLevel.FATAL:
        console.error(formattedMessage);
        if (entry.error?.stack) {
          console.error(entry.error.stack);
        }
        break;
    }
  }

  private format(entry: LogEntry): string {
    const timestamp = entry.timestamp.toISOString();
    const level = this.colorize ? this.colorizeLevel(entry.level) : entry.level;
    const module = `[${entry.module}]`;
    const correlationId = entry.correlationId ? `[${entry.correlationId}]` : '';
    const message = entry.message;
    const context = entry.context ? `\n  Context: ${JSON.stringify(entry.context, null, 2)}` : '';
    const tags = entry.tags && entry.tags.length > 0 ? ` {${entry.tags.join(', ')}}` : '';

    return `${timestamp} ${level} ${module}${correlationId} ${message}${tags}${context}`;
  }

  private colorizeLevel(level: LogLevel): string {
    const colors = {
      [LogLevel.DEBUG]: '\x1b[36m', // Cyan
      [LogLevel.INFO]: '\x1b[32m',  // Green
      [LogLevel.WARN]: '\x1b[33m',  // Yellow
      [LogLevel.ERROR]: '\x1b[31m', // Red
      [LogLevel.FATAL]: '\x1b[35m', // Magenta
    };
    const reset = '\x1b[0m';
    return `${colors[level]}${level}${reset}`;
  }
}

// ============================================================================
// FILE TRANSPORT
// ============================================================================

export class FileTransport implements LogTransport {
  public readonly name = 'file';
  public readonly minLevel: LogLevel;
  private readonly filePath: string;
  private readonly maxFileSize: number; // in bytes
  private readonly maxFiles: number;
  private writeStream: any; // fs.WriteStream - avoiding import for now

  constructor(
    filePath: string,
    minLevel: LogLevel = LogLevel.INFO,
    maxFileSize: number = 10 * 1024 * 1024, // 10MB
    maxFiles: number = 5
  ) {
    this.filePath = filePath;
    this.minLevel = minLevel;
    this.maxFileSize = maxFileSize;
    this.maxFiles = maxFiles;
  }

  async log(entry: LogEntry): Promise<void> {
    const formattedMessage = this.formatJson(entry);

    // File system operations would be implemented here
    // For now, this is a placeholder that would use fs.appendFile or stream
    // In a real implementation, you would:
    // 1. Check file size and rotate if needed
    // 2. Write to file
    // 3. Handle errors
  }

  private formatJson(entry: LogEntry): string {
    const logObject = {
      timestamp: entry.timestamp.toISOString(),
      level: entry.level,
      module: entry.module,
      message: entry.message,
      ...(entry.correlationId && { correlationId: entry.correlationId }),
      ...(entry.userId && { userId: entry.userId }),
      ...(entry.sessionId && { sessionId: entry.sessionId }),
      ...(entry.context && { context: entry.context }),
      ...(entry.tags && entry.tags.length > 0 && { tags: entry.tags }),
      ...(entry.error && {
        error: {
          name: entry.error.name,
          message: entry.error.message,
          stack: entry.error.stack,
          ...(entry.error instanceof ApplicationError && {
            code: entry.error.code,
            statusCode: entry.error.statusCode,
            context: entry.error.context,
          }),
        },
      }),
    };

    return JSON.stringify(logObject);
  }
}

// ============================================================================
// DATABASE TRANSPORT
// ============================================================================

export class DatabaseTransport implements LogTransport {
  public readonly name = 'database';
  public readonly minLevel: LogLevel;
  private readonly tableName: string;
  private readonly batchSize: number;
  private readonly flushInterval: number;
  private batch: LogEntry[] = [];
  private flushTimer?: NodeJS.Timeout;

  constructor(
    tableName: string = 'logs',
    minLevel: LogLevel = LogLevel.WARN,
    batchSize: number = 100,
    flushInterval: number = 5000
  ) {
    this.tableName = tableName;
    this.minLevel = minLevel;
    this.batchSize = batchSize;
    this.flushInterval = flushInterval;
    this.startFlushTimer();
  }

  async log(entry: LogEntry): Promise<void> {
    this.batch.push(entry);

    if (this.batch.length >= this.batchSize) {
      await this.flush();
    }
  }

  private async flush(): Promise<void> {
    if (this.batch.length === 0) return;

    const entriesToFlush = [...this.batch];
    this.batch = [];

    // Database insert would be implemented here
    // In a real implementation, you would:
    // 1. Connect to database
    // 2. Insert batch of log entries
    // 3. Handle errors
  }

  private startFlushTimer(): void {
    this.flushTimer = setInterval(() => {
      this.flush().catch(err => {
        console.error('Failed to flush logs to database:', err);
      });
    }, this.flushInterval);
  }

  async close(): Promise<void> {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
    }
    await this.flush();
  }
}

// ============================================================================
// LOGGER CLASS
// ============================================================================

export class Logger {
  private static instance: Logger;
  private transports: LogTransport[] = [];
  private globalContext: Record<string, any> = {};
  private correlationId?: string;
  private userId?: string;
  private sessionId?: string;
  private defaultTags: string[] = [];

  private constructor() {
    // Private constructor for singleton
  }

  public static getInstance(): Logger {
    if (!Logger.instance) {
      Logger.instance = new Logger();
      // Initialize with console transport by default
      Logger.instance.addTransport(new ConsoleTransport());
    }
    return Logger.instance;
  }

  // ============================================================================
  // TRANSPORT MANAGEMENT
  // ============================================================================

  public addTransport(transport: LogTransport): void {
    this.transports.push(transport);
  }

  public removeTransport(transportName: string): void {
    this.transports = this.transports.filter(t => t.name !== transportName);
  }

  public clearTransports(): void {
    this.transports = [];
  }

  // ============================================================================
  // CONTEXT MANAGEMENT
  // ============================================================================

  public setGlobalContext(context: Record<string, any>): void {
    this.globalContext = { ...this.globalContext, ...context };
  }

  public clearGlobalContext(): void {
    this.globalContext = {};
  }

  public setCorrelationId(correlationId: string): void {
    this.correlationId = correlationId;
  }

  public setUserId(userId: string): void {
    this.userId = userId;
  }

  public setSessionId(sessionId: string): void {
    this.sessionId = sessionId;
  }

  public setDefaultTags(tags: string[]): void {
    this.defaultTags = tags;
  }

  public addDefaultTag(tag: string): void {
    if (!this.defaultTags.includes(tag)) {
      this.defaultTags.push(tag);
    }
  }

  // ============================================================================
  // LOGGING METHODS
  // ============================================================================

  public debug(module: string, message: string, context?: Record<string, any>, tags?: string[]): void {
    this.log(LogLevel.DEBUG, module, message, context, undefined, tags);
  }

  public info(module: string, message: string, context?: Record<string, any>, tags?: string[]): void {
    this.log(LogLevel.INFO, module, message, context, undefined, tags);
  }

  public warn(module: string, message: string, context?: Record<string, any>, tags?: string[]): void {
    this.log(LogLevel.WARN, module, message, context, undefined, tags);
  }

  public error(
    module: string,
    message: string,
    error?: Error | ApplicationError,
    context?: Record<string, any>,
    tags?: string[]
  ): void {
    this.log(LogLevel.ERROR, module, message, context, error, tags);
  }

  public fatal(
    module: string,
    message: string,
    error?: Error | ApplicationError,
    context?: Record<string, any>,
    tags?: string[]
  ): void {
    this.log(LogLevel.FATAL, module, message, context, error, tags);
  }

  // ============================================================================
  // CORE LOGGING METHOD
  // ============================================================================

  private log(
    level: LogLevel,
    module: string,
    message: string,
    context?: Record<string, any>,
    error?: Error | ApplicationError,
    tags?: string[]
  ): void {
    const entry: LogEntry = {
      timestamp: new Date(),
      level,
      module,
      message,
      context: { ...this.globalContext, ...context },
      error,
      correlationId: this.correlationId,
      userId: this.userId,
      sessionId: this.sessionId,
      tags: [...this.defaultTags, ...(tags || [])],
    };

    // Send to all applicable transports
    this.transports.forEach(transport => {
      if (this.shouldLog(level, transport.minLevel)) {
        try {
          const result = transport.log(entry);
          // Handle async transports
          if (result instanceof Promise) {
            result.catch(err => {
              console.error(`Transport ${transport.name} failed:`, err);
            });
          }
        } catch (err) {
          console.error(`Transport ${transport.name} threw error:`, err);
        }
      }
    });
  }

  private shouldLog(messageLevel: LogLevel, transportMinLevel: LogLevel): boolean {
    return LOG_LEVEL_VALUES[messageLevel] >= LOG_LEVEL_VALUES[transportMinLevel];
  }

  // ============================================================================
  // MODULE-SPECIFIC LOGGER
  // ============================================================================

  public createModuleLogger(moduleName: string, tags?: string[]): ModuleLogger {
    return new ModuleLogger(this, moduleName, tags);
  }
}

// ============================================================================
// MODULE LOGGER CLASS
// ============================================================================

export class ModuleLogger {
  constructor(
    private logger: Logger,
    private moduleName: string,
    private moduleTags: string[] = []
  ) {}

  public debug(message: string, context?: Record<string, any>, tags?: string[]): void {
    this.logger.debug(this.moduleName, message, context, this.combineTags(tags));
  }

  public info(message: string, context?: Record<string, any>, tags?: string[]): void {
    this.logger.info(this.moduleName, message, context, this.combineTags(tags));
  }

  public warn(message: string, context?: Record<string, any>, tags?: string[]): void {
    this.logger.warn(this.moduleName, message, context, this.combineTags(tags));
  }

  public error(
    message: string,
    error?: Error | ApplicationError,
    context?: Record<string, any>,
    tags?: string[]
  ): void {
    this.logger.error(this.moduleName, message, error, context, this.combineTags(tags));
  }

  public fatal(
    message: string,
    error?: Error | ApplicationError,
    context?: Record<string, any>,
    tags?: string[]
  ): void {
    this.logger.fatal(this.moduleName, message, error, context, this.combineTags(tags));
  }

  private combineTags(tags?: string[]): string[] {
    return [...this.moduleTags, ...(tags || [])];
  }
}

// ============================================================================
// CONVENIENCE EXPORTS
// ============================================================================

// Export singleton instance
export const logger = Logger.getInstance();

// Export factory function for module loggers
export function createLogger(moduleName: string, tags?: string[]): ModuleLogger {
  return logger.createModuleLogger(moduleName, tags);
}

// ============================================================================
// PERFORMANCE LOGGING UTILITIES
// ============================================================================

export class PerformanceLogger {
  private startTime: number;
  private checkpoints: Map<string, number> = new Map();

  constructor(
    private logger: ModuleLogger,
    private operationName: string
  ) {
    this.startTime = performance.now();
    this.logger.debug(`Starting operation: ${operationName}`);
  }

  public checkpoint(name: string): void {
    const now = performance.now();
    const elapsed = now - this.startTime;
    this.checkpoints.set(name, elapsed);
    this.logger.debug(`Checkpoint '${name}' in ${this.operationName}`, {
      checkpoint: name,
      elapsedMs: elapsed.toFixed(2),
    });
  }

  public end(context?: Record<string, any>): void {
    const endTime = performance.now();
    const totalTime = endTime - this.startTime;

    const checkpointData: Record<string, string> = {};
    this.checkpoints.forEach((time, name) => {
      checkpointData[name] = `${time.toFixed(2)}ms`;
    });

    this.logger.info(`Completed operation: ${this.operationName}`, {
      totalTimeMs: totalTime.toFixed(2),
      checkpoints: checkpointData,
      ...context,
    }, ['performance']);
  }

  public fail(error: Error, context?: Record<string, any>): void {
    const endTime = performance.now();
    const totalTime = endTime - this.startTime;

    this.logger.error(
      `Failed operation: ${this.operationName}`,
      error,
      {
        totalTimeMs: totalTime.toFixed(2),
        ...context,
      },
      ['performance', 'failure']
    );
  }
}

// Helper function to create performance logger
export function createPerformanceLogger(
  moduleName: string,
  operationName: string
): PerformanceLogger {
  const moduleLogger = createLogger(moduleName);
  return new PerformanceLogger(moduleLogger, operationName);
}

// ============================================================================
// AUDIT LOGGING UTILITIES
// ============================================================================

export interface AuditLogEntry {
  userId: string;
  username: string;
  action: string;
  entityType: string;
  entityId: string;
  changes?: Record<string, { old: any; new: any }>;
  ipAddress?: string;
  userAgent?: string;
  success: boolean;
  reason?: string;
}

export function logAudit(entry: AuditLogEntry): void {
  const auditLogger = createLogger('AUDIT');

  const message = `${entry.username} ${entry.action} ${entry.entityType}:${entry.entityId}`;
  const context = {
    userId: entry.userId,
    action: entry.action,
    entityType: entry.entityType,
    entityId: entry.entityId,
    changes: entry.changes,
    ipAddress: entry.ipAddress,
    userAgent: entry.userAgent,
    success: entry.success,
    reason: entry.reason,
  };

  if (entry.success) {
    auditLogger.info(message, context, ['audit', 'success']);
  } else {
    auditLogger.warn(message, context, ['audit', 'failure']);
  }
}

// ============================================================================
// STRUCTURED LOGGING HELPERS
// ============================================================================

export function logDatabaseOperation(
  moduleName: string,
  operation: string,
  tableName: string,
  recordId?: string,
  duration?: number,
  error?: Error
): void {
  const dbLogger = createLogger(moduleName);
  const context = {
    operation,
    tableName,
    recordId,
    durationMs: duration?.toFixed(2),
  };

  if (error) {
    dbLogger.error(`Database ${operation} failed`, error, context, ['database']);
  } else {
    dbLogger.debug(`Database ${operation} completed`, context, ['database']);
  }
}

export function logHttpRequest(
  method: string,
  url: string,
  statusCode: number,
  duration: number,
  userId?: string,
  error?: Error
): void {
  const httpLogger = createLogger('HTTP');
  const context = {
    method,
    url,
    statusCode,
    durationMs: duration.toFixed(2),
    userId,
  };

  const message = `${method} ${url} - ${statusCode}`;

  if (error) {
    httpLogger.error(message, error, context, ['http', 'request']);
  } else if (statusCode >= 400) {
    httpLogger.warn(message, context, ['http', 'request']);
  } else {
    httpLogger.info(message, context, ['http', 'request']);
  }
}
