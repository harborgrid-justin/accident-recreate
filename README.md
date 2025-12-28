# AccuScene Enterprise - Accident Recreation Platform

**Version 0.2.0** - Professional accident recreation and analysis platform built with Electron, React, TypeScript, and Rust.

## Overview

AccuScene Enterprise is a comprehensive desktop application for accident reconstruction professionals, providing advanced tools for:

### Core Features
- Interactive accident scene diagram editing
- Physics-based collision simulation
- Vehicle dynamics modeling
- Professional PDF report generation
- Case management and data persistence
- Multi-user authentication and authorization

### Enterprise Features (v0.2.0)
- **GraphQL Federation API** - Unified API gateway with real-time subscriptions
- **Real-time Collaboration** - Multi-user editing with CRDT-based conflict resolution
- **Advanced 3D/AR UI** - React Three Fiber components with AR preview
- **Plugin Architecture** - Extensible system with marketplace integration
- **Performance Monitoring** - APM dashboard with Prometheus, Datadog, New Relic support

## Technology Stack

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type-safe development
- **Konva/Fabric.js** - Canvas-based diagram editor
- **Electron** - Desktop application framework
- **React Three Fiber** - 3D rendering with Three.js *(v0.2.0)*
- **@react-three/drei** - 3D component helpers *(v0.2.0)*
- **Yjs** - CRDT framework for collaboration *(v0.2.0)*

### Backend
- **Express** - REST API server
- **Apollo Server** - GraphQL Federation server *(v0.2.0)*
- **Socket.io** - Real-time WebSocket communication *(v0.2.0)*
- **TypeORM** - Database ORM
- **SQLite3/better-sqlite3** - Embedded database
- **JWT** - Authentication
- **DataLoader** - Batching and caching *(v0.2.0)*

### Rust Core (v0.2.0)
- **15 Rust crates** providing high-performance backend:
  - Event sourcing & CQRS
  - Advanced analytics engine
  - Real-time streaming pipeline (Apache Arrow)
  - Machine learning integration
  - Security & audit system
  - Physics simulation
  - Compression & crypto
  - Database layer
  - Job processing
  - Distributed clustering

### Build & Tooling
- **Webpack** - Module bundler
- **Jest** - Testing framework
- **ESLint & Prettier** - Code quality
- **Cargo** - Rust package manager

## Project Structure

```
accident-recreate/
├── src/
│   ├── main/                    # Electron main process
│   ├── renderer/                # React frontend
│   │   └── components/
│   │       └── advanced/        # 3D/AR components (v0.2.0)
│   ├── api/                     # Express backend
│   ├── graphql/                 # GraphQL Federation API (v0.2.0)
│   ├── collaboration/           # Real-time collaboration (v0.2.0)
│   ├── plugins/                 # Plugin architecture (v0.2.0)
│   ├── monitoring/              # Performance monitoring (v0.2.0)
│   ├── enterprise/              # Enterprise module exports (v0.2.0)
│   ├── database/                # TypeORM entities
│   ├── physics/                 # Simulation engine
│   ├── editor/                  # Diagram editor
│   ├── reports/                 # PDF generation
│   ├── types/                   # Shared TypeScript types
│   └── utils/                   # Helper utilities
├── rust-core/                   # Rust backend (v0.2.0)
│   ├── crates/
│   │   ├── accuscene-core/      # Core library
│   │   ├── accuscene-eventsourcing/  # Event sourcing & CQRS
│   │   ├── accuscene-analytics/      # Analytics engine
│   │   ├── accuscene-streaming/      # Streaming pipeline
│   │   ├── accuscene-ml/             # ML integration
│   │   ├── accuscene-security/       # Security & audit
│   │   └── ... (10 more crates)
│   └── Cargo.toml               # Workspace configuration
├── dist/                        # Build output
├── assets/                      # Application assets
├── data/                        # SQLite database files
└── docs/                        # Documentation (v0.2.0)
```

## Getting Started

### Prerequisites

- Node.js 18+ and npm
- Git

### Installation

```bash
# Install dependencies
npm install

# Copy environment configuration
cp .env.example .env
```

### Development

```bash
# Run all services in development mode
npm run dev

# Or run individually:
npm run dev:renderer   # React dev server (port 8080)
npm run dev:api       # Express API server (port 3001)
npm run dev:electron  # Electron main process
```

### Building

```bash
# Build all components
npm run build

# Package for distribution
npm run package

# Platform-specific builds
npm run package:win
npm run package:mac
npm run package:linux
```

### Testing

```bash
# Run tests
npm test

# Run tests in watch mode
npm test:watch

# Generate coverage report
npm test:coverage
```

### Code Quality

```bash
# Lint code
npm run lint

# Fix linting issues
npm run lint:fix

# Format code
npm run format

# Check formatting
npm run format:check

# Type checking
npm run typecheck
```

## Configuration

See `.env.example` for available environment variables.

## Enterprise Features (v0.2.0)

### GraphQL Federation API

Unified API gateway with:
- Apollo Server with Federation support
- Real-time subscriptions via WebSocket
- DataLoaders for N+1 query prevention
- Custom directives (@auth, @rateLimit, @validate)
- Type-safe resolvers with comprehensive error handling
- GraphQL Playground with documentation

**Location**: `src/graphql/`
**Files**: 33 TypeScript files (~4,500 lines)
**Documentation**: `src/graphql/README.md`

### Real-time Collaboration

Multi-user editing system with:
- CRDT-based conflict-free replication (6 CRDT types)
- Operational Transformation for concurrent edits
- Vector clock causality tracking
- Real-time presence and cursor tracking
- WebSocket server with room management
- Offline-first with automatic sync
- Permission system (READ, WRITE, ADMIN, OWNER)

**Location**: `src/collaboration/`
**Files**: 36 TypeScript files (~5,500 lines)
**Documentation**: `src/collaboration/README.md`

### Advanced 3D/AR UI Components

React Three Fiber components including:
- 3D Scene viewer with camera controls
- 3D Vehicle models with damage visualization
- 3D Simulation playback with timeline
- AR Preview with marker detection
- Advanced charts (Force, Velocity, Energy, Impact)
- Heatmaps for damage and force distribution
- Timeline editor with events and keyframes
- Custom hooks (useScene3D, useAnimation, useAR)

**Location**: `src/renderer/components/advanced/`
**Files**: 58 files (~5,000 lines)
**Technologies**: React Three Fiber, Three.js, @react-three/drei

### Plugin Architecture

Extensible plugin system with:
- Hot-reloading for development
- Permission-based security (12 permissions)
- Capability detection (7 capabilities)
- Dependency resolution with version constraints
- Marketplace integration
- Manifest validation (JSON schema + semantic)
- UI extension points (toolbars, panels, menus, context menus)
- Import/export format extensions
- Built-in plugins (measurements, annotations, exports)

**Location**: `src/plugins/`
**Files**: 46 TypeScript files (~6,000 lines)
**Documentation**: `src/plugins/README.md`

### Performance Monitoring

APM dashboard with:
- Real-time metrics collection
- Distributed tracing
- CPU and memory profiling
- Health check system
- Alert management
- Multiple integrations (Prometheus, Datadog, New Relic)
- Interactive monitoring dashboard

**Location**: `src/monitoring/`
**Files**: 42 TypeScript files
**Integrations**: Prometheus, Datadog, New Relic

### Rust Backend

High-performance backend with 15 crates:
- **Event Sourcing & CQRS** - Full event-driven architecture
- **Analytics Engine** - Real-time metrics and insights
- **Streaming Pipeline** - Apache Arrow-based data flow
- **ML Integration** - Accident prediction models
- **Security & Audit** - SOC2 compliance, audit trails
- Plus 10 more crates for core functionality

**Location**: `rust-core/`
**Version**: 0.2.0
**Language**: Rust 2021 Edition

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.

## License

PROPRIETARY - © 2024 AccuScene Enterprise. All rights reserved.

## Development Notes

This project uses:
- Strict TypeScript configuration
- Path aliases for clean imports (@/, @main/, @renderer/, etc.)
- Secure Electron context isolation
- ESLint with TypeScript rules
- Prettier for consistent formatting