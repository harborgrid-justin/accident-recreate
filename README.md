# AccuScene Enterprise - Accident Recreation Platform

**Version 0.3.0** - Professional accident recreation and analysis platform built with Electron, React, TypeScript, and Rust.

## Overview

AccuScene Enterprise is a comprehensive desktop application for accident reconstruction professionals, providing advanced tools for:

### Core Features
- Interactive accident scene diagram editing
- Physics-based collision simulation
- Vehicle dynamics modeling
- Professional PDF report generation
- Case management and data persistence
- Multi-user authentication and authorization

### Enterprise Features (v0.2.0 + v0.2.5 + v0.3.0)
- **GraphQL Federation API** - Unified API gateway with real-time subscriptions *(v0.2.0)*
- **Real-time Collaboration** - Multi-user editing with CRDT-based conflict resolution *(v0.2.0)*
- **Advanced 3D/AR UI** - React Three Fiber components with AR preview *(v0.2.0)*
- **Plugin Architecture** - Extensible system with marketplace integration *(v0.2.0)*
- **Performance Monitoring** - APM dashboard with Prometheus, Datadog, New Relic support *(v0.2.0)*
- **Accessibility (a11y)** - Screen reader, high contrast, keyboard navigation support *(v0.2.5)*
- **Interactive Dashboards** - Customizable real-time dashboards with widget system *(v0.2.5)*
- **Gesture Recognition** - Multi-touch gestures for enhanced interaction *(v0.2.5)*
- **Push Notifications** - Real-time notification system with action support *(v0.2.5)*
- **Offline-first** - Full offline capabilities with automatic sync *(v0.2.5)*
- **User Preferences** - Centralized preference management with cloud sync *(v0.2.5)*
- **Full-text Search** - Fast, fuzzy search across all content *(v0.2.5)*
- **Advanced Visualization** - WebGL/WebGPU-powered data visualizations *(v0.2.5)*
- **Single Sign-On (SSO)** - Enterprise SSO integration support *(v0.2.5)*

### v0.3.0 Features - Advanced Physics & Professional CAD/CAM
- **Advanced Physics Engine** - Rigid body dynamics, deformable bodies, tire friction *(v0.3.0)*
- **Professional CAD/CAM GUI** - Multi-viewport, property panels, command palette *(v0.3.0)*
- **Database Algorithms** - LZ4 compression, delta encoding, B-tree indexing *(v0.3.0)*
- **WebGPU Rendering** - Compute shaders, instancing, LOD system, shadows *(v0.3.0)*
- **AI/ML Prediction** - Crash pattern recognition, speed estimation, trajectory prediction *(v0.3.0)*
- **Performance Streaming** - Optimized data streaming and real-time metrics *(v0.3.0)*

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

### Rust Core (v0.3.0)
- **31 Rust crates** providing high-performance backend:
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
  - Accessibility (a11y) support *(v0.2.5)*
  - Dashboard engine *(v0.2.5)*
  - Gesture recognition *(v0.2.5)*
  - Notification system *(v0.2.5)*
  - Offline storage & sync *(v0.2.5)*
  - Preferences management *(v0.2.5)*
  - Full-text search engine *(v0.2.5)*
  - Visualization engine *(v0.2.5)*
  - SSO integration *(v0.2.5)*
  - Data transfer & sync *(v0.2.5)*
  - **Integration layer** - Unified runtime & facade *(v0.2.5)*
  - **Advanced physics v3** - Deformable bodies & tire friction *(v0.3.0)*
  - **Algorithms** - LZ4, delta encoding, B-tree, bloom filters *(v0.3.0)*
  - **ML v3** - Crash patterns & trajectory prediction *(v0.3.0)*
  - **Security v3** - Enhanced enterprise security *(v0.3.0)*
  - **Performance** - Streaming optimization & metrics *(v0.3.0)*

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
│   ├── enterprise/              # Enterprise module exports
│   │   ├── v0.2.5/              # v0.2.5 integration layer
│   │   └── v0.3.0/              # v0.3.0 integration layer
│   ├── database/                # TypeORM entities
│   ├── physics/                 # Simulation engine
│   ├── editor/                  # Diagram editor
│   ├── reports/                 # PDF generation
│   ├── types/                   # Shared TypeScript types
│   └── utils/                   # Helper utilities
├── rust-core/                   # Rust backend (v0.3.0)
│   ├── crates/
│   │   ├── accuscene-core/            # Core library
│   │   ├── accuscene-eventsourcing/   # Event sourcing & CQRS
│   │   ├── accuscene-analytics/       # Analytics engine
│   │   ├── accuscene-streaming/       # Streaming pipeline
│   │   ├── accuscene-ml/              # ML integration
│   │   ├── accuscene-security/        # Security & audit
│   │   ├── accuscene-integration/     # Integration layer (v0.2.5)
│   │   ├── accuscene-physics-v3/      # Advanced physics (v0.3.0)
│   │   ├── accuscene-algorithms/      # Database algorithms (v0.3.0)
│   │   ├── accuscene-ml-v3/           # ML v3 (v0.3.0)
│   │   ├── accuscene-security-v3/     # Security v3 (v0.3.0)
│   │   ├── accuscene-performance/     # Performance (v0.3.0)
│   │   └── ... (19 more crates)
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

High-performance backend with 31 crates:
- **Event Sourcing & CQRS** - Full event-driven architecture
- **Analytics Engine** - Real-time metrics and insights
- **Streaming Pipeline** - Apache Arrow-based data flow
- **ML Integration** - Accident prediction models (v1 & v3)
- **Security & Audit** - SOC2 compliance, audit trails (v1 & v3)
- **Integration Layer** - Unified runtime and facade *(v0.2.5)*
- **Advanced Physics v3** - Deformable bodies, tire friction *(v0.3.0)*
- **Database Algorithms** - LZ4, delta encoding, B-tree *(v0.3.0)*
- **Performance Engine** - Streaming optimization *(v0.3.0)*
- Plus 22 more crates for core and UX functionality

**Location**: `rust-core/`
**Version**: 0.3.0
**Language**: Rust 2021 Edition

## v0.3.0 Advanced Features

### Physics Engineering (v0.3.0)
Professional-grade accident physics simulation:
- **Advanced Rigid Body Dynamics** - Multi-body simulation with constraints
- **Deformable Body Physics** - Plasticity and fracture modeling
- **Tire Friction Models** - Slip angle and lateral/longitudinal stiffness
- **Energy Absorption** - Detailed energy distribution analysis
- **Crush Zone Simulation** - Vehicle deformation with energy absorption
- **Impact Force Distribution** - Contact point analysis with pressure mapping

### Professional CAD/CAM GUI (v0.3.0)
Enterprise-grade user interface:
- **CAD Toolbar System** - Professional drafting tools
- **Property Panels** - Advanced object property editing
- **Command Palette** - Quick access to all functions
- **Measurement Tools** - Distance, angle, area, volume annotations
- **Layer Management** - Multi-layer scene organization
- **Snap-to-Grid** - Magnetic guides and angle snapping
- **Multi-viewport** - Simultaneous view angles
- **Custom Themes** - Fully customizable appearance

### Database & Algorithms (v0.3.0)
High-performance data management:
- **LZ4 Compression** - Real-time scene data compression
- **Delta Encoding** - Efficient scene difference storage
- **B-tree Indexing** - Spatial data indexing
- **Bloom Filters** - Fast object lookup
- **Write-ahead Logging** - Transaction durability
- **MVCC Transactions** - Multi-version concurrency control

### WebGPU Rendering (v0.3.0)
Next-generation graphics:
- **Compute Shaders** - GPU-accelerated physics computations
- **Instanced Rendering** - Efficient multi-object rendering
- **LOD System** - Automatic level-of-detail management
- **Shadow Mapping** - Realistic shadow rendering
- **Post-processing** - Bloom, SSAO, FXAA, tone mapping
- **VR Mode** - Virtual reality support

### AI/ML Capabilities (v0.3.0)
Intelligent accident analysis:
- **Crash Pattern Recognition** - Identify similar accident patterns
- **Speed Estimation** - Estimate speed from damage analysis
- **Trajectory Prediction** - Predict vehicle paths with confidence
- **Anomaly Detection** - Identify unusual accident characteristics

## v0.2.5 Integration Layer

### Unified Runtime & Service Management
The integration layer provides a cohesive foundation for all v0.2.5 features:

**Rust Integration** (`rust-core/crates/accuscene-integration/`):
- **Unified Configuration** - Single config for all 26 crates
- **Runtime Management** - Lifecycle orchestration for all services
- **Facade Pattern** - Simplified API access across all features
- **Event System** - Cross-crate event bus with pub/sub
- **Service Registry** - Dynamic service discovery and management
- **Health Aggregation** - Comprehensive health monitoring

**TypeScript Integration** (`src/enterprise/v0.2.5/`):
- **EnterpriseProvider** - React context for all enterprise features
- **EnterpriseApp** - Main application shell with status monitoring
- **useEnterprise** - Unified React hook for feature access
- **EnterpriseService** - Centralized service management class
- **Type Definitions** - Comprehensive TypeScript types
- **Configuration System** - Environment-based config with validation

### Key Capabilities
- **31 Rust crates** fully integrated under single runtime (v0.3.0)
- **Automatic service initialization** with dependency resolution
- **Real-time health monitoring** across all services
- **Feature flags** for granular control
- **Development tools** with live service inspection
- **Production-ready** deployment validation
- **Event-driven architecture** for loose coupling

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