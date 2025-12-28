/**
 * Express Server Setup
 * AccuScene Enterprise Accident Recreation Platform
 */

import express, { Application, Request, Response } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import {
  requestLogger,
  securityHeaders,
  requestSizeLimiter,
  corsOptions,
} from './middleware/logger';
import { errorHandler, notFoundHandler } from './middleware/errorHandler';
import { sanitizeInput } from './middleware/validator';

// Import routes
import authRoutes from './routes/auth.routes';
import usersRoutes from './routes/users.routes';
import casesRoutes from './routes/cases.routes';
import accidentsRoutes from './routes/accidents.routes';
import vehiclesRoutes from './routes/vehicles.routes';
import evidenceRoutes from './routes/evidence.routes';
import reportsRoutes from './routes/reports.routes';

/**
 * Create and configure Express application
 */
export function createApp(): Application {
  const app = express();

  // ==========================================
  // Security & Performance Middleware
  // ==========================================

  // Helmet for security headers
  app.use(helmet({
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        styleSrc: ["'self'", "'unsafe-inline'"],
        scriptSrc: ["'self'"],
        imgSrc: ["'self'", 'data:', 'https:'],
      },
    },
    crossOriginEmbedderPolicy: false, // Allow embedding for Electron
  }));

  // Custom security headers
  app.use(securityHeaders);

  // CORS configuration
  app.use(cors(corsOptions()));

  // Compression
  app.use(compression());

  // Request size limiter
  app.use(requestSizeLimiter);

  // ==========================================
  // Body Parsing Middleware
  // ==========================================

  // JSON body parser with limit
  app.use(express.json({ limit: '10mb' }));

  // URL-encoded body parser
  app.use(express.urlencoded({ extended: true, limit: '10mb' }));

  // ==========================================
  // Logging & Sanitization Middleware
  // ==========================================

  // Request logging
  app.use(requestLogger);

  // Input sanitization
  app.use(sanitizeInput);

  // ==========================================
  // Health Check & Info Endpoints
  // ==========================================

  // Health check endpoint
  app.get('/health', (req: Request, res: Response) => {
    res.status(200).json({
      success: true,
      message: 'AccuScene API is running',
      timestamp: new Date().toISOString(),
      version: process.env.API_VERSION || '1.0.0',
      environment: process.env.NODE_ENV || 'development',
    });
  });

  // API info endpoint
  app.get('/api', (req: Request, res: Response) => {
    res.status(200).json({
      success: true,
      message: 'AccuScene Enterprise Accident Recreation API',
      version: process.env.API_VERSION || '1.0.0',
      documentation: '/api/docs',
      endpoints: {
        auth: '/api/auth',
        users: '/api/users',
        cases: '/api/cases',
        accidents: '/api/accidents',
        vehicles: '/api/vehicles',
        evidence: '/api/evidence',
        reports: '/api/reports',
      },
    });
  });

  // ==========================================
  // API Routes
  // ==========================================

  app.use('/api/auth', authRoutes);
  app.use('/api/users', usersRoutes);
  app.use('/api/cases', casesRoutes);
  app.use('/api/accidents', accidentsRoutes);
  app.use('/api/vehicles', vehiclesRoutes);
  app.use('/api/evidence', evidenceRoutes);
  app.use('/api/reports', reportsRoutes);

  // ==========================================
  // Static Files (for uploads and exports)
  // ==========================================

  app.use('/uploads', express.static('uploads'));
  app.use('/reports', express.static('reports'));
  app.use('/exports', express.static('exports'));

  // ==========================================
  // Error Handling
  // ==========================================

  // 404 handler (must be after all routes)
  app.use(notFoundHandler);

  // Global error handler (must be last)
  app.use(errorHandler);

  return app;
}

/**
 * Start the server
 */
export function startServer(port: number = 3000): void {
  const app = createApp();

  const server = app.listen(port, () => {
    console.log('='.repeat(50));
    console.log('AccuScene Enterprise API Server');
    console.log('='.repeat(50));
    console.log(`Environment: ${process.env.NODE_ENV || 'development'}`);
    console.log(`Port: ${port}`);
    console.log(`Health: http://localhost:${port}/health`);
    console.log(`API Info: http://localhost:${port}/api`);
    console.log('='.repeat(50));
    console.log('Available Endpoints:');
    console.log(`  Auth:      http://localhost:${port}/api/auth`);
    console.log(`  Users:     http://localhost:${port}/api/users`);
    console.log(`  Cases:     http://localhost:${port}/api/cases`);
    console.log(`  Accidents: http://localhost:${port}/api/accidents`);
    console.log(`  Vehicles:  http://localhost:${port}/api/vehicles`);
    console.log(`  Evidence:  http://localhost:${port}/api/evidence`);
    console.log(`  Reports:   http://localhost:${port}/api/reports`);
    console.log('='.repeat(50));
  });

  // Graceful shutdown
  process.on('SIGTERM', () => {
    console.log('SIGTERM signal received: closing HTTP server');
    server.close(() => {
      console.log('HTTP server closed');
      process.exit(0);
    });
  });

  process.on('SIGINT', () => {
    console.log('SIGINT signal received: closing HTTP server');
    server.close(() => {
      console.log('HTTP server closed');
      process.exit(0);
    });
  });
}

// Start server if this file is run directly
if (require.main === module) {
  const PORT = parseInt(process.env.PORT || '3000', 10);
  startServer(PORT);
}
