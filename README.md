# AccuScene Enterprise - Accident Recreation Platform

Professional accident recreation and analysis platform built with Electron, React, and TypeScript.

## Overview

AccuScene Enterprise is a comprehensive desktop application for accident reconstruction professionals, providing advanced tools for:

- Interactive accident scene diagram editing
- Physics-based collision simulation
- Vehicle dynamics modeling
- Professional PDF report generation
- Case management and data persistence
- Multi-user authentication and authorization

## Technology Stack

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type-safe development
- **Konva/Fabric.js** - Canvas-based diagram editor
- **Electron** - Desktop application framework

### Backend
- **Express** - REST API server
- **TypeORM** - Database ORM
- **SQLite3/better-sqlite3** - Embedded database
- **JWT** - Authentication

### Build & Tooling
- **Webpack** - Module bundler
- **Jest** - Testing framework
- **ESLint & Prettier** - Code quality

## Project Structure

```
accident-recreate/
├── src/
│   ├── main/          # Electron main process
│   ├── renderer/      # React frontend
│   ├── api/           # Express backend
│   ├── database/      # TypeORM entities
│   ├── physics/       # Simulation engine
│   ├── editor/        # Diagram editor
│   ├── reports/       # PDF generation
│   ├── types/         # Shared TypeScript types
│   └── utils/         # Helper utilities
├── dist/              # Build output
├── assets/            # Application assets
└── data/              # SQLite database files
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

## License

PROPRIETARY - © 2024 AccuScene Enterprise. All rights reserved.

## Development Notes

This project uses:
- Strict TypeScript configuration
- Path aliases for clean imports (@/, @main/, @renderer/, etc.)
- Secure Electron context isolation
- ESLint with TypeScript rules
- Prettier for consistent formatting