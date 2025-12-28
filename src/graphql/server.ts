/**
 * Apollo GraphQL Server Setup
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { ApolloServer } from '@apollo/server';
import { expressMiddleware } from '@apollo/server/express4';
import { ApolloServerPluginDrainHttpServer } from '@apollo/server/plugin/drainHttpServer';
import { ApolloServerPluginLandingPageLocalDefault } from '@apollo/server/plugin/landingPage/default';
import { makeExecutableSchema } from '@graphql-tools/schema';
import { WebSocketServer } from 'ws';
import { useServer } from 'graphql-ws/lib/use/ws';
import { createServer } from 'http';
import express, { Express } from 'express';
import cors from 'cors';
import { json } from 'body-parser';

import { typeDefs } from './schema';
import { resolvers } from './resolvers';
import { createContext, createWebSocketContext } from './context';
import { GraphQLContext } from './types';
import { loggingPlugin } from './middleware/logging.middleware';
import { errorHandlingPlugin, formatError } from './middleware/error.middleware';
import { buildFederatedSchema, getFederationHealth } from './federation';
import { playgroundConfig } from './playground';

/**
 * Create and configure Apollo Server with Federation support
 */
export async function createApolloServer(httpServer: any): Promise<ApolloServer<GraphQLContext>> {
  // Build the federated schema
  const schema = buildFederatedSchema();

  // Create WebSocket server for subscriptions
  const wsServer = new WebSocketServer({
    server: httpServer,
    path: '/graphql',
  });

  // Setup subscription handling
  const serverCleanup = useServer(
    {
      schema,
      context: async (ctx) => {
        return createWebSocketContext(ctx.connectionParams, ctx.extra.socket);
      },
      onConnect: async (ctx) => {
        console.log('WebSocket client connected');
      },
      onDisconnect: async (ctx) => {
        console.log('WebSocket client disconnected');
      },
    },
    wsServer
  );

  // Create Apollo Server
  const server = new ApolloServer<GraphQLContext>({
    schema,

    plugins: [
      // Drain HTTP server on shutdown
      ApolloServerPluginDrainHttpServer({ httpServer }),

      // Cleanup WebSocket server on shutdown
      {
        async serverWillStart() {
          return {
            async drainServer() {
              await serverCleanup.dispose();
            },
          };
        },
      },

      // Logging plugin
      loggingPlugin,

      // Error handling plugin
      errorHandlingPlugin,

      // Landing page (Playground in development)
      process.env.NODE_ENV === 'production'
        ? ApolloServerPluginLandingPageLocalDefault({ footer: false })
        : ApolloServerPluginLandingPageLocalDefault({
            footer: false,
            includeCookies: true,
          }),
    ],

    // Format errors for client
    formatError,

    // Enable introspection in development
    introspection: process.env.NODE_ENV !== 'production',

    // Include stack traces in development
    includeStacktraceInErrorResponses: process.env.NODE_ENV !== 'production',
  });

  await server.start();

  return server;
}

/**
 * Setup Express app with GraphQL middleware
 */
export async function setupGraphQLServer(port: number = 4000): Promise<{
  app: Express;
  server: any;
  apolloServer: ApolloServer<GraphQLContext>;
}> {
  const app = express();
  const httpServer = createServer(app);

  // Create Apollo Server
  const apolloServer = await createApolloServer(httpServer);

  // CORS configuration
  app.use(
    cors({
      origin: process.env.CORS_ORIGIN || 'http://localhost:3000',
      credentials: true,
    })
  );

  // Body parser
  app.use(json({ limit: '10mb' }));

  // Health check endpoint
  app.get('/health', (req, res) => {
    res.json(getFederationHealth());
  });

  // GraphQL endpoint
  app.use(
    '/graphql',
    expressMiddleware(apolloServer, {
      context: createContext,
    })
  );

  // Start HTTP server
  await new Promise<void>((resolve) => {
    httpServer.listen(port, () => {
      console.log(`🚀 GraphQL Server ready at http://localhost:${port}/graphql`);
      console.log(`🔌 WebSocket Server ready at ws://localhost:${port}/graphql`);
      console.log(`❤️  Health check at http://localhost:${port}/health`);
      resolve();
    });
  });

  return {
    app,
    server: httpServer,
    apolloServer,
  };
}

/**
 * Graceful shutdown handler
 */
export async function shutdownServer(
  server: any,
  apolloServer: ApolloServer<GraphQLContext>
): Promise<void> {
  console.log('\n🛑 Shutting down GraphQL server...');

  try {
    // Stop accepting new connections
    await apolloServer.stop();

    // Close HTTP server
    await new Promise<void>((resolve, reject) => {
      server.close((err: Error) => {
        if (err) reject(err);
        else resolve();
      });
    });

    console.log('✅ GraphQL server shut down gracefully');
    process.exit(0);
  } catch (error) {
    console.error('❌ Error during shutdown:', error);
    process.exit(1);
  }
}

/**
 * Start the GraphQL server
 */
export async function startGraphQLServer(): Promise<void> {
  try {
    const port = parseInt(process.env.GRAPHQL_PORT || '4000', 10);
    const { server, apolloServer } = await setupGraphQLServer(port);

    // Graceful shutdown handlers
    process.on('SIGTERM', () => shutdownServer(server, apolloServer));
    process.on('SIGINT', () => shutdownServer(server, apolloServer));

    // Error handlers
    process.on('unhandledRejection', (reason, promise) => {
      console.error('Unhandled Rejection at:', promise, 'reason:', reason);
    });

    process.on('uncaughtException', (error) => {
      console.error('Uncaught Exception:', error);
      shutdownServer(server, apolloServer);
    });
  } catch (error) {
    console.error('Failed to start GraphQL server:', error);
    process.exit(1);
  }
}
