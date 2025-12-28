# AccuScene Database Layer

Complete TypeORM database implementation for the AccuScene Enterprise Accident Recreation Platform.

## Overview

This database layer provides a complete, production-ready foundation for managing accident investigation data using TypeORM with SQLite.

### Statistics
- **Total Lines of Code**: ~4,649
- **Entities**: 7
- **Custom Repositories**: 3
- **Migrations**: 1 (Initial Schema)

## Architecture

### Core Components

1. **Configuration** (`config.ts`)
   - Environment-aware database configuration
   - SQLite setup with WAL mode for better concurrency
   - Separate test database configuration
   - Query caching and logging

2. **Connection Manager** (`index.ts`)
   - Singleton pattern for database lifecycle
   - Auto-migration on startup
   - Graceful shutdown handling
   - Health check functionality

3. **Entities** (`entities/`)
   - Complete TypeORM entity definitions
   - Validation decorators and hooks
   - Relations and cascade operations
   - Virtual fields for computed properties
   - Business logic methods

4. **Repositories** (`repositories/`)
   - Custom repositories with domain-specific queries
   - Business logic encapsulation
   - Advanced search and filtering
   - Statistics and reporting methods

5. **Migrations** (`migrations/`)
   - Initial schema creation
   - Comprehensive indexes
   - Foreign key constraints
   - Rollback support

## Entity Schema

### User
**Purpose**: System users with authentication and RBAC

**Key Fields**:
- Authentication: email, password (hashed)
- Profile: firstName, lastName, department, phoneNumber
- Security: role, isActive, loginAttempts, lockedUntil
- Recovery: resetPasswordToken, resetPasswordExpires

**Relations**:
- OneToMany: Cases

**Features**:
- Password hashing and validation
- Account locking after failed attempts
- Password reset token generation
- Role-based access control (admin, investigator, analyst, viewer)

### Case
**Purpose**: Central entity for accident investigation cases

**Key Fields**:
- Identification: caseNumber (auto-generated), title
- Status: status, priority, dueDate
- Client: clientName, clientEmail, clientPhone
- Assignment: userId, assignedTo
- Metadata: tags[], notes, description

**Relations**:
- ManyToOne: User
- OneToOne: Accident
- OneToMany: InsuranceClaims

**Features**:
- Auto-generated unique case numbers (ACC-YYYY-XXXXX)
- Overdue case detection
- Tag management
- Status tracking with timestamps

### Accident
**Purpose**: Core accident scene details and conditions

**Key Fields**:
- Location: location, latitude, longitude, intersection
- Conditions: weather, roadConditions, lightConditions
- Environment: temperature, speedLimit, numberOfLanes
- Impact: injuries, fatalities, estimatedDamage
- Diagram: diagram (JSON), environmentalFactors[]

**Relations**:
- OneToOne: Case
- OneToMany: Vehicles, Witnesses, Evidence

**Features**:
- Severity calculation (minor/moderate/severe/catastrophic)
- Damage aggregation from vehicles
- Environmental factor tracking
- Coordinate validation

### Vehicle
**Purpose**: Vehicles involved in accidents

**Key Fields**:
- Identity: make, model, year, color, licensePlate, vin
- Driver: driverName, driverLicense, driverStatement
- Position: initialPosition, finalPosition, impactPoint, speed
- Damage: damageSeverity, damageAreas[], estimatedDamage
- Safety: airbagDeployed, seatbeltUsed
- Impairment: driverImpaired, driverDistracted
- Occupants: occupants, injuredOccupants, occupantDetails[]

**Relations**:
- ManyToOne: Accident

**Features**:
- Position and angle calculations
- Displacement tracking
- Vehicle description formatting
- Damage area management
- Insurance information tracking

### Witness
**Purpose**: Witness testimonies and credibility assessment

**Key Fields**:
- Identity: name, contact, email, address
- Statement: statement, statementDate, observedDetails
- Location: witnessLocation, distanceFromAccident
- Credibility: reliability, reliabilityNotes, hadClearView
- Evidence: audioRecordings[], videoRecordings[], photos[]
- Legal: willingToTestify, contactedByInsurance

**Relations**:
- ManyToOne: Accident

**Features**:
- Reliability rating system
- Multiple recording types support
- Vehicle observation tracking
- Speed estimation

### Evidence
**Purpose**: Physical and digital evidence with chain of custody

**Key Fields**:
- Identification: evidenceNumber (auto-generated), type, source
- File: filePath, fileName, fileSize, fileHash
- Collection: timestamp, collectedBy, collectionLocation
- Custody: custodyStatus, currentCustodian, chainOfCustody[]
- Analysis: findings, analyzedBy, analyzedDate
- Legal: isAdmissible, admissibilityNotes
- Metadata: imageMetadata, videoMetadata

**Relations**:
- ManyToOne: Accident

**Features**:
- Chain of custody tracking
- File integrity verification (SHA-256)
- Evidence transfer management
- Copy tracking (original vs. copies)
- Media metadata extraction
- Priority assignment

### InsuranceClaim
**Purpose**: Insurance claims related to cases

**Key Fields**:
- Identity: claimNumber, type, insurer
- Amounts: amount, approvedAmount, paidAmount, deductible
- Contacts: claimantName, adjusterName, attorneyName
- Timeline: filedDate, submittedDate, decisionDate, settlementDate
- Details: documents[], communications[], payments[]
- Legal: requiresLitigation, subrogate

**Relations**:
- ManyToOne: Case

**Features**:
- Status tracking with automatic timestamps
- Payment recording and tracking
- Communication logging
- Document management
- Recovery percentage calculation

## Custom Repositories

### UserRepository
- `findByEmail()` - Case-insensitive email lookup
- `findByRole()` - Filter users by role
- `createUser()` - Create user with hashed password
- `updatePassword()` - Update and hash new password
- `lockAccount()` / `unlockAccount()` - Account security
- `getUsersWithCaseCounts()` - Users with statistics
- `searchUsers()` - Full-text search

### CaseRepository
- `findByCaseNumber()` - Lookup by case number
- `findByStatus()` - Filter by status (multiple)
- `findActiveCases()` - Exclude closed/archived
- `findOverdueCases()` - Cases past due date
- `searchCases()` - Full-text search across fields
- `findByTag()` - Filter by tags
- `getCaseStatistics()` - Aggregate statistics
- `findOneWithFullDetails()` - Load all relations

### AccidentRepository
- `findByCaseId()` - Get accident for a case
- `findByLocation()` - Search by location
- `findWithInjuries()` / `findWithFatalities()` - Severity filters
- `findBySeverity()` - Filter by calculated severity
- `findMultiVehicle()` - Multi-vehicle accidents
- `findNearLocation()` - Geospatial search (radius)
- `getAccidentStatistics()` - Comprehensive statistics
- `findByPoliceReport()` - Lookup by report number

## Migrations

### Initial Schema (1703000000000)
Creates complete database schema with:
- All 7 entity tables
- Comprehensive indexes for performance
- Foreign key constraints with CASCADE delete
- Default values and nullability
- Support for rollback (`down()` method)

**Indexes Created**:
- Users: email
- Cases: caseNumber, status, userId
- Accidents: caseId, dateTime
- Vehicles: accidentId, licensePlate
- Witnesses: accidentId, contact
- Evidence: accidentId, evidenceNumber, timestamp
- InsuranceClaims: caseId, claimNumber, status, filedDate

## Usage Examples

### Initialize Database

```typescript
import { initializeDatabase, closeDatabase } from './database';

// Initialize connection
const dataSource = await initializeDatabase();

// Use the connection...

// Close when done
await closeDatabase();
```

### Using Repositories

```typescript
import { initializeRepositories } from './database/repositories';
import { getConnection } from './database';

const dataSource = getConnection();
const repos = initializeRepositories(dataSource);

// Create user
const user = await repos.users.createUser({
  email: 'investigator@example.com',
  firstName: 'John',
  lastName: 'Doe',
  role: UserRole.INVESTIGATOR,
}, 'securePassword123');

// Search cases
const activeCases = await repos.cases.findActiveCases();

// Get statistics
const stats = await repos.accidents.getAccidentStatistics();
```

### Direct Entity Usage

```typescript
import { getConnection } from './database';
import { Case, CaseStatus } from './database/entities/Case';

const dataSource = getConnection();
const caseRepo = dataSource.getRepository(Case);

const newCase = caseRepo.create({
  title: 'Highway Collision Investigation',
  status: CaseStatus.ACTIVE,
  userId: user.id,
});

await caseRepo.save(newCase);
```

## Configuration

### Environment Variables

- `NODE_ENV` - Environment mode (development/production/test)
- `DB_PATH` - Database directory path (default: ./data)
- `DB_NAME` - Database filename (default: accuscene.db)

### Database Features

- **WAL Mode**: Write-Ahead Logging enabled for better concurrency
- **Query Cache**: 30-second result caching
- **Auto-Migration**: Migrations run automatically on startup
- **Logging**: Query logging in development, errors only in production

## Data Validation

All entities include comprehensive validation:
- Email format validation
- Coordinate range validation (-90 to 90 lat, -180 to 180 lng)
- Non-negative number validation
- Required field enforcement
- Length constraints
- Date/time validation

## Performance Optimizations

1. **Indexes**: Strategic indexes on frequently queried fields
2. **Eager Loading**: Configurable eager loading for relations
3. **Query Caching**: Built-in query result caching
4. **Lazy Loading**: Relations loaded on-demand by default
5. **Batch Operations**: Support for bulk inserts/updates

## Security Features

1. **Password Hashing**: SHA-256 password hashing
2. **Account Locking**: Automatic lockout after failed attempts
3. **Password Reset**: Secure token-based password recovery
4. **Soft Delete**: Users deactivated rather than deleted
5. **Data Sanitization**: Automatic removal of sensitive fields in JSON responses

## Testing

The database layer supports testing with:
- In-memory SQLite database (`:memory:`)
- Automatic schema synchronization
- Clean state for each test run
- No migration running in test mode

```typescript
import { testDatabaseConfig } from './database/config';

// Test configuration uses in-memory database
```

## Future Enhancements

Potential areas for expansion:
- [ ] Soft delete support for cases and accidents
- [ ] Audit logging for all entity changes
- [ ] Full-text search using FTS5
- [ ] Database backup and restore utilities
- [ ] Data export/import functionality
- [ ] Query performance monitoring
- [ ] Database size monitoring and cleanup
- [ ] Advanced reporting views

## File Structure

```
src/database/
├── config.ts                    # Database configuration
├── index.ts                     # Connection manager
├── README.md                    # This file
├── entities/
│   ├── User.ts                  # User entity (173 lines)
│   ├── Case.ts                  # Case entity (207 lines)
│   ├── Accident.ts              # Accident entity (337 lines)
│   ├── Vehicle.ts               # Vehicle entity (424 lines)
│   ├── Witness.ts               # Witness entity (330 lines)
│   ├── Evidence.ts              # Evidence entity (477 lines)
│   └── InsuranceClaim.ts        # Insurance claim entity (396 lines)
├── repositories/
│   ├── index.ts                 # Repository initialization
│   ├── UserRepository.ts        # User business logic (165 lines)
│   ├── CaseRepository.ts        # Case business logic (212 lines)
│   └── AccidentRepository.ts    # Accident business logic (191 lines)
└── migrations/
    └── 1703000000000-InitialSchema.ts  # Initial migration (1,339 lines)
```

## Dependencies

Required packages (should be in package.json):
- `typeorm` - ORM framework
- `sqlite3` - SQLite driver
- `reflect-metadata` - TypeScript decorators

## License

Part of AccuScene Enterprise Accident Recreation Platform
