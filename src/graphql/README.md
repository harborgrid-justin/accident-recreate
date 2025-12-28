# AccuScene Enterprise GraphQL Federation API v0.2.0

A comprehensive, production-ready GraphQL Federation API system for the AccuScene Enterprise Accident Recreation Platform.

## Overview

This GraphQL API provides a type-safe, federated interface for managing accident reconstruction cases, vehicles, scenes, simulations, reports, and user authentication.

### Key Features

- **Apollo Server with Federation** - Microservice-ready federated architecture
- **Real-time Subscriptions** - WebSocket support for live updates
- **N+1 Query Prevention** - DataLoader implementation for efficient batching
- **Custom Directives** - @auth, @rateLimit, @validate
- **JWT Authentication** - Secure token-based authentication
- **Field-level Authorization** - Granular permission control
- **Custom Scalars** - DateTime, JSON, Email, URL, and more
- **Comprehensive Error Handling** - Structured error responses
- **Request Logging** - Winston-based logging system
- **Rate Limiting** - Built-in request throttling
- **Type Safety** - 100% TypeScript with strict typing

## Architecture

```
src/graphql/
├── index.ts                    # Main exports
├── server.ts                   # Apollo Server setup
├── context.ts                  # Context factory
├── types.ts                    # TypeScript definitions
├── scalars.ts                  # Custom scalar types
├── federation.ts               # Federation config
├── playground.ts               # GraphQL Playground
│
├── schema/                     # GraphQL schema definitions
│   ├── index.ts
│   ├── case.graphql.ts         # Case types
│   ├── vehicle.graphql.ts      # Vehicle types
│   ├── scene.graphql.ts        # Scene types
│   ├── simulation.graphql.ts   # Simulation types
│   ├── user.graphql.ts         # User/auth types
│   └── report.graphql.ts       # Report types
│
├── resolvers/                  # Query/Mutation resolvers
│   ├── index.ts
│   ├── case.resolver.ts
│   ├── vehicle.resolver.ts
│   ├── scene.resolver.ts
│   ├── simulation.resolver.ts
│   ├── user.resolver.ts
│   └── report.resolver.ts
│
├── directives/                 # Custom directives
│   ├── index.ts
│   ├── auth.directive.ts       # @auth
│   ├── rateLimit.directive.ts  # @rateLimit
│   └── validate.directive.ts   # @validate
│
├── dataloaders/                # N+1 prevention
│   ├── index.ts
│   ├── case.loader.ts
│   └── vehicle.loader.ts
│
├── subscriptions/              # Real-time subscriptions
│   ├── index.ts
│   ├── case.subscription.ts
│   └── simulation.subscription.ts
│
└── middleware/                 # Middleware functions
    ├── auth.middleware.ts      # Authentication
    ├── logging.middleware.ts   # Request logging
    └── error.middleware.ts     # Error handling
```

## Getting Started

### Installation

```bash
npm install @apollo/server @apollo/subgraph graphql graphql-tag
npm install dataloader graphql-subscriptions graphql-ws ws
npm install @graphql-tools/schema @graphql-tools/utils
npm install express cors body-parser
npm install jsonwebtoken bcrypt uuid winston zod
```

### Starting the Server

```typescript
import { startGraphQLServer } from './src/graphql';

// Start the GraphQL server
await startGraphQLServer();

// Server will be available at:
// - GraphQL endpoint: http://localhost:4000/graphql
// - WebSocket endpoint: ws://localhost:4000/graphql
// - Health check: http://localhost:4000/health
```

### Environment Variables

```bash
# Server
GRAPHQL_PORT=4000
NODE_ENV=development

# Authentication
JWT_SECRET=your-secret-key-here

# CORS
CORS_ORIGIN=http://localhost:3000

# Logging
LOG_LEVEL=info
```

## Usage Examples

### Queries

#### Get Cases with Pagination

```graphql
query GetCases($page: Int, $limit: Int) {
  cases(pagination: { page: $page, limit: $limit }) {
    items {
      id
      caseNumber
      title
      status
      priority
      investigator {
        fullName
        email
      }
    }
    total
    hasNextPage
  }
}
```

#### Get Case Details

```graphql
query GetCase($id: ID!) {
  case(id: $id) {
    id
    caseNumber
    title
    description
    status
    priority
    incidentDate
    location {
      address
      city
      state
    }
    vehicles {
      id
      make
      model
      year
      type
    }
    simulations {
      id
      name
      status
      progress
    }
  }
}
```

### Mutations

#### Create a Case

```graphql
mutation CreateCase($input: CreateCaseInput!) {
  createCase(input: $input) {
    id
    caseNumber
    title
    status
  }
}
```

Variables:
```json
{
  "input": {
    "title": "Highway Collision",
    "incidentDate": "2025-12-28T10:30:00Z",
    "priority": "HIGH",
    "location": {
      "address": "I-95 Mile Marker 245",
      "city": "Richmond",
      "state": "VA",
      "zipCode": "23220",
      "country": "USA"
    }
  }
}
```

#### Run Simulation

```graphql
mutation RunSimulation($input: CreateSimulationInput!) {
  createSimulation(input: $input) {
    id
    name
    status
  }
  startSimulation(id: $id) {
    id
    status
    progress
  }
}
```

### Subscriptions

#### Subscribe to Case Updates

```graphql
subscription CaseUpdates($caseId: ID!) {
  caseUpdated(caseId: $caseId) {
    case {
      id
      status
      updatedAt
    }
    mutation
    userId
  }
}
```

#### Subscribe to Simulation Progress

```graphql
subscription SimulationProgress($simulationId: ID!) {
  simulationProgress(simulationId: $simulationId) {
    simulation {
      id
      status
    }
    progress
    currentFrame
    totalFrames
  }
}
```

## Authentication

### Login

```graphql
mutation Login($input: LoginInput!) {
  login(input: $input) {
    token
    refreshToken
    user {
      id
      email
      role
    }
    expiresIn
  }
}
```

### Using Authentication Token

Add the token to your request headers:

```http
Authorization: Bearer YOUR_JWT_TOKEN_HERE
```

## Custom Directives

### @auth - Authorization

Requires specific permissions to access a field:

```graphql
type Query {
  cases: [Case!]! @auth(requires: [CASE_READ])
  deleteCase(id: ID!): Boolean! @auth(requires: [CASE_DELETE])
}
```

### @rateLimit - Rate Limiting

Limits the number of requests:

```graphql
type Query {
  cases: [Case!]! @rateLimit(max: 100, window: 60)
}
```

### @validate - Validation

Validates input against a schema:

```graphql
type Mutation {
  createUser(email: String! @validate(schema: "email")): User!
}
```

## DataLoaders

DataLoaders prevent N+1 query problems by batching requests:

```typescript
// Automatically batches multiple case loads into single query
const case1 = await context.dataloaders.caseLoader.load(id1);
const case2 = await context.dataloaders.caseLoader.load(id2);
const case3 = await context.dataloaders.caseLoader.load(id3);

// Results in single database query: WHERE id IN (id1, id2, id3)
```

## Error Handling

All errors are standardized with error codes:

```json
{
  "errors": [
    {
      "message": "Authentication required",
      "extensions": {
        "code": "UNAUTHENTICATED",
        "http": { "status": 401 }
      }
    }
  ]
}
```

### Error Codes

- `UNAUTHENTICATED` - Authentication required
- `UNAUTHORIZED` - Insufficient permissions
- `BAD_REQUEST` - Invalid request
- `NOT_FOUND` - Resource not found
- `CONFLICT` - Resource conflict
- `INTERNAL_ERROR` - Internal server error
- `VALIDATION_ERROR` - Validation failed
- `RATE_LIMIT_EXCEEDED` - Too many requests

## Permissions

Available permissions:

- `CASE_CREATE` - Create cases
- `CASE_READ` - Read cases
- `CASE_UPDATE` - Update cases
- `CASE_DELETE` - Delete cases
- `SIMULATION_RUN` - Run simulations
- `REPORT_GENERATE` - Generate reports
- `USER_MANAGE` - Manage users

## User Roles

- `ADMIN` - Full access to all features
- `INVESTIGATOR` - Create and manage cases
- `ANALYST` - View and analyze cases
- `VIEWER` - Read-only access

## Health Check

Check API health:

```bash
curl http://localhost:4000/health
```

Response:
```json
{
  "name": "accuscene-graphql-api",
  "version": "0.2.0",
  "status": "healthy",
  "uptime": 12345.67,
  "timestamp": "2025-12-28T...",
  "capabilities": {
    "subscriptions": true,
    "fileUploads": true,
    "customDirectives": true,
    "dataloaders": true
  }
}
```

## Testing

Access GraphQL Playground in development:

```
http://localhost:4000/graphql
```

The playground includes example queries, mutations, and subscriptions.

## Production Deployment

### Recommendations

1. **Use Redis for PubSub** - Replace in-memory PubSub with Redis for distributed subscriptions
2. **Enable HTTPS** - Use SSL/TLS in production
3. **Configure CORS** - Restrict origins to your domains
4. **Set JWT Secret** - Use strong, random JWT secret
5. **Enable Monitoring** - Use APM tools like New Relic or DataDog
6. **Database Connection Pool** - Configure appropriate pool size
7. **Rate Limiting** - Adjust limits based on your needs
8. **Caching** - Implement Redis caching for frequently accessed data

### Security Checklist

- [ ] Strong JWT secret configured
- [ ] HTTPS enabled
- [ ] CORS properly configured
- [ ] Rate limiting enabled
- [ ] Input validation active
- [ ] SQL injection prevention
- [ ] XSS prevention
- [ ] CSRF tokens for mutations
- [ ] Audit logging enabled
- [ ] Error messages sanitized

## Performance

### Optimization Features

- **DataLoader batching** - Automatic query batching
- **Field-level caching** - Cache individual fields
- **Query complexity analysis** - Prevent expensive queries
- **Pagination** - Built-in cursor and offset pagination
- **Lazy loading** - Load data only when requested

## Monitoring

The API includes built-in monitoring:

- Request logging with Winston
- Error tracking and reporting
- Performance metrics
- Federation health checks

## API Documentation

Full API documentation available in GraphQL Playground:

- Schema browser
- Field descriptions
- Type definitions
- Example queries

## Support

For issues or questions:

1. Check the GraphQL Playground examples
2. Review error messages and codes
3. Check server logs in `logs/graphql-*.log`
4. Consult the schema documentation

## Version

Current version: **0.2.0**

## License

PROPRIETARY - AccuScene Enterprise

---

**Built with:**
- Apollo Server
- GraphQL
- TypeScript
- DataLoader
- Winston
- JSON Web Tokens
