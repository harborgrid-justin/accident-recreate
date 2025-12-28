# AccuScene Enterprise API Layer

## Overview

Complete RESTful API implementation for the AccuScene Enterprise Accident Recreation Platform. This API layer provides secure, scalable endpoints for all platform functionality.

## Statistics

- **Total Files**: 33 TypeScript files
- **Total Lines of Code**: ~4,942 lines
- **API Endpoints**: 50+ endpoints across 7 resources
- **Security Features**: JWT authentication, role-based access control, rate limiting
- **Validation**: Zod schema validation for all inputs

## Directory Structure

```
src/api/
├── index.ts                    # Main API module exports
├── server.ts                   # Express server setup and configuration
├── responses.ts                # Standardized response utilities
│
├── controllers/                # Request handlers (8 files)
│   ├── index.ts
│   ├── auth.controller.ts     # Authentication operations
│   ├── users.controller.ts    # User management
│   ├── cases.controller.ts    # Case CRUD and search
│   ├── accidents.controller.ts # Accident diagram management
│   ├── vehicles.controller.ts  # Vehicle operations and physics
│   ├── evidence.controller.ts  # Evidence file management
│   └── reports.controller.ts   # Report generation
│
├── routes/                     # Route definitions (8 files)
│   ├── index.ts
│   ├── auth.routes.ts         # POST /api/auth/login, register, logout
│   ├── users.routes.ts        # /api/users CRUD
│   ├── cases.routes.ts        # /api/cases CRUD, search, assign
│   ├── accidents.routes.ts    # /api/accidents CRUD, diagram export
│   ├── vehicles.routes.ts     # /api/vehicles CRUD, simulate
│   ├── evidence.routes.ts     # /api/evidence upload, download
│   └── reports.routes.ts      # /api/reports generate, download
│
├── middleware/                 # Middleware functions (6 files)
│   ├── index.ts
│   ├── auth.ts                # JWT authentication & authorization
│   ├── errorHandler.ts        # Global error handling
│   ├── logger.ts              # Request/response logging
│   ├── validator.ts           # Input validation and sanitization
│   └── upload.ts              # File upload handling (Multer)
│
└── validators/                 # Validation schemas (8 files)
    ├── index.ts
    ├── auth.schemas.ts        # Login, register, password reset
    ├── user.schemas.ts        # User CRUD validation
    ├── case.schemas.ts        # Case management validation
    ├── accident.schemas.ts    # Accident diagram validation
    ├── vehicle.schemas.ts     # Vehicle and physics validation
    ├── evidence.schemas.ts    # Evidence upload validation
    └── report.schemas.ts      # Report generation validation
```

## API Endpoints

### Authentication (`/api/auth`)
- `POST /login` - User login
- `POST /register` - User registration
- `POST /logout` - User logout
- `POST /refresh` - Refresh access token
- `GET /me` - Get current user
- `POST /change-password` - Change password
- `POST /reset-password-request` - Request password reset
- `POST /reset-password` - Reset password with token
- `POST /verify-email` - Verify email address

### Users (`/api/users`)
- `GET /` - List all users (paginated, filterable)
- `GET /profile` - Get current user profile
- `PUT /profile` - Update current user profile
- `GET /:userId` - Get user by ID
- `POST /` - Create new user (Admin only)
- `PUT /:userId` - Update user (Admin only)
- `DELETE /:userId` - Delete user (Admin only)

### Cases (`/api/cases`)
- `GET /` - List all cases (paginated, filterable)
- `GET /search` - Search cases
- `GET /:caseId` - Get case by ID
- `POST /` - Create new case
- `PUT /:caseId` - Update case
- `DELETE /:caseId` - Delete case (Admin only)
- `PATCH /:caseId/status` - Update case status
- `POST /:caseId/assign` - Assign case to user

### Accidents (`/api/accidents`)
- `GET /` - List all accidents (paginated, filterable)
- `GET /:accidentId` - Get accident by ID
- `POST /` - Create new accident
- `PUT /:accidentId` - Update accident
- `DELETE /:accidentId` - Delete accident
- `PUT /:accidentId/diagram` - Update diagram data
- `POST /:accidentId/export` - Export diagram (PNG, JPG, SVG, PDF)

### Vehicles (`/api/vehicles`)
- `GET /` - List all vehicles (paginated, filterable)
- `GET /:vehicleId` - Get vehicle by ID
- `POST /` - Create new vehicle
- `PUT /:vehicleId` - Update vehicle
- `DELETE /:vehicleId` - Delete vehicle
- `POST /:vehicleId/simulate` - Run physics simulation

### Evidence (`/api/evidence`)
- `GET /` - List all evidence (paginated, filterable)
- `GET /case/:caseId` - Get evidence by case
- `GET /:evidenceId` - Get evidence by ID
- `POST /upload` - Upload evidence files (up to 10 files)
- `PUT /:evidenceId` - Update evidence metadata
- `DELETE /:evidenceId` - Delete evidence (Admin only)
- `GET /:evidenceId/download` - Download evidence file

### Reports (`/api/reports`)
- `GET /` - List all reports (paginated, filterable)
- `GET /case/:caseId` - Get reports by case
- `GET /:reportId` - Get report by ID
- `GET /:reportId/status` - Get report generation status
- `POST /generate` - Generate new report
- `POST /:reportId/regenerate` - Regenerate existing report
- `GET /:reportId/download` - Download report file
- `DELETE /:reportId` - Delete report (Admin only)

## Security Features

### Authentication
- **JWT-based authentication** with access and refresh tokens
- **Access token expiry**: Configurable (default: 1 hour)
- **Refresh token expiry**: Configurable (default: 7 days)
- **Password requirements**: Min 8 chars, uppercase, lowercase, number, special char
- **Password hashing**: bcrypt with configurable rounds

### Authorization
- **Role-based access control (RBAC)** with 4 roles:
  - `ADMIN`: Full system access
  - `INVESTIGATOR`: Create and manage cases, accidents, reports
  - `ADJUSTER`: View cases and reports
  - `VIEWER`: Read-only access
- **Resource ownership**: Users can only access their own resources unless admin
- **Route-level protection**: Middleware enforces role requirements

### Rate Limiting
- **Login**: 5 attempts per 15 minutes
- **Registration**: 3 attempts per hour
- **Password operations**: 5 attempts per hour
- **Configurable per endpoint**

### Input Validation
- **Zod schema validation** for all request bodies, query params, and URL params
- **Input sanitization** to prevent XSS attacks
- **File upload restrictions**: Type, size, and count limits
- **UUID validation** for all IDs

### Security Headers
- **Helmet.js** for comprehensive security headers
- **CORS** with configurable allowed origins
- **Content Security Policy (CSP)**
- **X-Frame-Options, X-XSS-Protection, etc.**

## Middleware

### Authentication Middleware
```typescript
import { authenticate, authorize, requireAdmin } from './middleware/auth';

// Require authentication
router.get('/protected', authenticate, handler);

// Require specific roles
router.post('/create', authorize(UserRole.ADMIN, UserRole.INVESTIGATOR), handler);

// Require admin role
router.delete('/delete/:id', requireAdmin, handler);
```

### Validation Middleware
```typescript
import { validate, validateMultiple } from './middleware/validator';
import { createCaseSchema, caseIdParamSchema } from './validators/case.schemas';

// Validate request body
router.post('/cases', validate(createCaseSchema), handler);

// Validate multiple parts
router.put('/cases/:caseId', validateMultiple({
  params: caseIdParamSchema,
  body: updateCaseSchema
}), handler);
```

### Error Handling
```typescript
import { asyncHandler, AppError, NotFoundError } from './middleware/errorHandler';

// Wrap async handlers
export const handler = asyncHandler(async (req, res) => {
  // Automatic error catching
  const data = await service.getData();
  res.json(success(data));
});

// Throw custom errors
throw new NotFoundError('Case', caseId);
throw new AppError('Custom error', 400, 'CUSTOM_CODE');
```

### File Upload
```typescript
import { uploadMultiple, uploadSingle } from './middleware/upload';

// Single file
router.post('/upload', uploadSingle('file'), handler);

// Multiple files (max 10)
router.post('/upload', uploadMultiple('files', 10), handler);

// Access uploaded files
const files = req.files as Express.Multer.File[];
const fileInfo = getFilesInfo(files);
```

## Response Format

### Success Response
```json
{
  "success": true,
  "data": { ... },
  "message": "Operation successful",
  "meta": {
    "timestamp": "2024-01-15T10:30:00.000Z",
    "requestId": "req_1234567890_abc123"
  }
}
```

### Error Response
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Case with ID 'abc-123' not found",
    "details": { ... }
  },
  "meta": {
    "timestamp": "2024-01-15T10:30:00.000Z",
    "requestId": "req_1234567890_abc123"
  }
}
```

### Paginated Response
```json
{
  "success": true,
  "data": [ ... ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "totalPages": 5,
    "hasNext": true,
    "hasPrev": false
  },
  "meta": {
    "timestamp": "2024-01-15T10:30:00.000Z",
    "requestId": "req_1234567890_abc123"
  }
}
```

## Usage

### Starting the Server

```typescript
import { startServer } from './api';

// Start on default port (3000)
startServer();

// Start on custom port
startServer(8080);
```

### Using the Express App

```typescript
import { createApp } from './api';

const app = createApp();

// Add custom routes
app.get('/custom', (req, res) => {
  res.json({ message: 'Custom route' });
});

// Start server
app.listen(3000);
```

### Making API Calls

```typescript
// Login
const response = await fetch('http://localhost:3000/api/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    email: 'user@example.com',
    password: 'SecurePassword123!'
  })
});

const { data } = await response.json();
const { accessToken } = data.tokens;

// Authenticated request
const casesResponse = await fetch('http://localhost:3000/api/cases', {
  headers: {
    'Authorization': `Bearer ${accessToken}`,
    'Content-Type': 'application/json'
  }
});
```

## Environment Variables

```bash
# Server
PORT=3000
NODE_ENV=development
API_VERSION=1.0.0
API_BASE_URL=http://localhost:3000

# Security
JWT_SECRET=your-secret-key-change-in-production
JWT_REFRESH_SECRET=your-refresh-secret-key
ALLOWED_ORIGINS=http://localhost:3000,http://localhost:5173

# Upload
UPLOAD_DIR=./uploads
MAX_FILE_SIZE=52428800  # 50MB in bytes
```

## Integration with Other Modules

### Database Integration (TODO)
Controllers currently return mock data. Replace with actual database service calls:

```typescript
// Current (mock)
const mockCase = { id: caseId, ... };
res.json(success(mockCase));

// After database integration
const caseService = new CaseService();
const caseData = await caseService.findById(caseId);
if (!caseData) throw new NotFoundError('Case', caseId);
res.json(success(caseData));
```

### Event Bus Integration (TODO)
Add event publishing in controllers:

```typescript
import { eventBus } from '../utils/eventBus';
import { EventType } from '../types/events';

// After creating a case
const newCase = await caseService.create(caseData);
await eventBus.publish(EventType.CASE_CREATED, {
  case: newCase,
  createdBy: userId,
  timestamp: new Date()
});
```

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | User not authenticated |
| `FORBIDDEN` | 403 | User lacks required permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Input validation failed |
| `CONFLICT` | 409 | Resource conflict (e.g., duplicate) |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

## Testing

### Example Test
```typescript
import request from 'supertest';
import { createApp } from './api';

describe('Auth API', () => {
  const app = createApp();

  it('should login successfully', async () => {
    const response = await request(app)
      .post('/api/auth/login')
      .send({
        email: 'test@example.com',
        password: 'Password123!'
      });

    expect(response.status).toBe(200);
    expect(response.body.success).toBe(true);
    expect(response.body.data.tokens.accessToken).toBeDefined();
  });
});
```

## Next Steps

1. **Database Integration**: Replace mock data with actual database calls
2. **Event Bus**: Implement event publishing in controllers
3. **Service Layer**: Create service classes for business logic
4. **Testing**: Add comprehensive unit and integration tests
5. **Documentation**: Generate API documentation (Swagger/OpenAPI)
6. **Performance**: Add caching layer (Redis)
7. **Monitoring**: Add application performance monitoring (APM)

## Notes

- All controllers use `asyncHandler` wrapper for automatic error handling
- All routes require authentication except auth endpoints
- File uploads are organized by type (images, documents, videos)
- Request IDs are generated for all requests for tracing
- Sensitive data is automatically sanitized in logs
- CORS is configured for local development (update for production)

---

**Created by**: Agent 9 - API Layer & Backend Services
**Date**: 2024-12-28
**Status**: ✅ COMPLETE - Ready for database and event bus integration
