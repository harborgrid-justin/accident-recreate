# AccuScene Enterprise - Warning Prevention Guide v0.2.5

**Last Updated:** 2025-12-28

## Overview

This guide explains how to use the warning prevention infrastructure in AccuScene Enterprise to maintain high code quality and prevent common errors.

## Table of Contents

- [Rust Linting](#rust-linting)
- [TypeScript Type Safety](#typescript-type-safety)
- [Using Type Guards](#using-type-guards)
- [Using Assertions](#using-assertions)
- [CI/CD Integration](#cicd-integration)
- [IDE Setup](#ide-setup)
- [Common Workflows](#common-workflows)

---

## Rust Linting

### Running Clippy

```bash
# Standard clippy check
cd rust-core
cargo clippy

# Strict mode (treat warnings as errors)
cargo clippy -- -D warnings

# Check all workspace members
cargo clippy --workspace --all-targets --all-features

# Use the convenient alias
cargo clippy-strict

# Check a specific crate
cargo clippy -p accuscene-core
```

### Running cargo-deny

```bash
# Install cargo-deny (one time)
cargo install cargo-deny

# Check everything
cargo deny check

# Check specific categories
cargo deny check advisories  # Security vulnerabilities
cargo deny check licenses    # License compliance
cargo deny check bans        # Banned/duplicate dependencies
cargo deny check sources     # Source registry validation

# Update advisory database
cargo deny fetch
```

### Configuration Files

- **clippy.toml**: Clippy-specific settings
  - Complexity thresholds
  - Naming conventions
  - Disallowed patterns

- **.cargo/config.toml**: Compiler flags and aliases
  - Warning levels
  - Platform-specific settings
  - Convenient cargo commands

- **deny.toml**: Dependency auditing
  - Security vulnerability scanning
  - License compliance
  - Duplicate detection

---

## TypeScript Type Safety

### Type Declaration Files

Three main type declaration files provide complete type coverage:

1. **global.d.ts** - Global types and utilities
2. **modules.d.ts** - Module and asset declarations
3. **enterprise.d.ts** - Business domain types

### Using Global Types

```typescript
import {
  UUID,
  Point3D,
  Vector3D,
  Result,
  Option,
  DeepPartial,
  DeepReadonly,
} from '@types/global';

// Branded types prevent mixing IDs
const sceneId: UUID = createSceneId();
const userId: UUID = createUserId();

// Result type for explicit error handling
function loadScene(id: UUID): Result<Scene, Error> {
  try {
    const scene = fetchScene(id);
    return { success: true, value: scene };
  } catch (error) {
    return { success: false, error: error as Error };
  }
}

// Use Result with pattern matching
const result = loadScene(sceneId);
if (result.success) {
  console.log('Loaded:', result.value);
} else {
  console.error('Error:', result.error);
}

// Option type for nullable values
function findVehicle(id: UUID): Option<Vehicle> {
  return vehicles.find(v => v.id === id);
}

// Utility types for transformations
type PartialScene = DeepPartial<Scene>;
type ReadonlyConfig = DeepReadonly<Configuration>;
```

### Using Enterprise Types

```typescript
import {
  Scene,
  Vehicle,
  VehicleType,
  User,
  UserRole,
  Permission,
  Project,
  ProjectStatus,
} from '@types/enterprise';

// Type-safe enums
const vehicleType: VehicleType = VehicleType.SEDAN;
const userRole: UserRole = UserRole.INVESTIGATOR;

// Discriminated unions for state
type LoadingState =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: Scene }
  | { status: 'error'; error: Error };

function handleState(state: LoadingState) {
  switch (state.status) {
    case 'idle':
      return 'Ready';
    case 'loading':
      return 'Loading...';
    case 'success':
      return `Loaded: ${state.data.name}`;
    case 'error':
      return `Error: ${state.error.message}`;
  }
}
```

---

## Using Type Guards

Type guards provide runtime type checking with TypeScript type narrowing.

### Basic Type Guards

```typescript
import {
  isString,
  isNumber,
  isBoolean,
  isObject,
  isArray,
  isDefined,
  isNullish,
} from '@utils/typeGuards';

function processValue(value: unknown) {
  if (isString(value)) {
    // TypeScript knows value is string
    console.log(value.toUpperCase());
  } else if (isNumber(value)) {
    // TypeScript knows value is number
    console.log(value.toFixed(2));
  } else if (isArray(value)) {
    // TypeScript knows value is array
    console.log(value.length);
  }
}

// Check if value is defined (not null/undefined)
function greet(name: string | null | undefined) {
  if (isDefined(name)) {
    // TypeScript knows name is string
    console.log(`Hello, ${name}!`);
  }
}
```

### Format Validation

```typescript
import {
  isUUID,
  isEmail,
  isURL,
  isISODateString,
  isHexColor,
} from '@utils/typeGuards';

function validateInput(input: unknown) {
  if (isUUID(input)) {
    // Use as UUID
    fetchScene(input);
  }

  if (isEmail(input)) {
    // Use as email
    sendNotification(input);
  }

  if (isURL(input)) {
    // Use as URL
    fetch(input);
  }
}
```

### Geometric Validation

```typescript
import {
  isPoint3D,
  isVector3D,
  isDimensions3D,
} from '@utils/typeGuards';

function processPosition(pos: unknown) {
  if (isPoint3D(pos)) {
    // TypeScript knows pos has x, y, z properties
    const distance = Math.sqrt(
      pos.x ** 2 + pos.y ** 2 + pos.z ** 2
    );
    console.log(`Distance: ${distance}`);
  }
}
```

### Enterprise Type Validation

```typescript
import {
  isVehicle,
  isScene,
  isUser,
  isProject,
} from '@utils/typeGuards';

function handleApiResponse(data: unknown) {
  if (isScene(data)) {
    // TypeScript knows data is Scene
    console.log(`Scene: ${data.name}`);
    data.vehicles.forEach(v => processVehicle(v));
  }
}

// Validate array elements
import { isArrayOf } from '@utils/typeGuards';

function processVehicles(data: unknown) {
  if (isArrayOf(data, isVehicle)) {
    // TypeScript knows data is Vehicle[]
    data.forEach(vehicle => {
      console.log(`${vehicle.type}: ${vehicle.name}`);
    });
  }
}
```

### Composing Type Guards

```typescript
import {
  andGuards,
  orGuards,
  notGuard,
  createTypeGuard,
} from '@utils/typeGuards';

// Combine guards with AND
const isPositiveNumber = andGuards(isNumber, isPositive);

// Combine guards with OR
const isStringOrNumber = orGuards(isString, isNumber);

// Negate a guard
const isNotNull = notGuard(isNull);

// Create custom type guard
const isAdultAge = createTypeGuard<number>(
  (value): value is number => isNumber(value) && value >= 18
);
```

---

## Using Assertions

Assertions throw errors when conditions aren't met, providing fail-fast behavior.

### Basic Assertions

```typescript
import {
  assert,
  assertDefined,
  assertNotNull,
  assertNever,
  AssertionError,
} from '@utils/assertions';

function processScene(scene: Scene | null) {
  // Assert scene is not null
  assertNotNull(scene, 'Scene must not be null');
  // TypeScript now knows scene is Scene

  // Assert condition
  assert(scene.vehicles.length > 0, 'Scene must have vehicles');

  // Use scene safely
  const firstVehicle = scene.vehicles[0];
}

// Exhaustive switch with assertNever
type Status = 'idle' | 'loading' | 'success' | 'error';

function handleStatus(status: Status) {
  switch (status) {
    case 'idle':
      return 'Ready';
    case 'loading':
      return 'Loading...';
    case 'success':
      return 'Complete';
    case 'error':
      return 'Failed';
    default:
      // Ensures all cases are handled
      assertNever(status, 'Unhandled status');
  }
}
```

### Type Assertions

```typescript
import {
  assertString,
  assertNumber,
  assertBoolean,
  assertObject,
  assertArray,
} from '@utils/assertions';

function processApiResponse(data: unknown) {
  assertObject(data, 'Response must be an object');
  // data is now Record<string, unknown>

  assertString(data.name, 'Name must be a string');
  assertNumber(data.id, 'ID must be a number');
  assertArray(data.items, 'Items must be an array');
}
```

### Range Assertions

```typescript
import {
  assertInRange,
  assertPositive,
  assertNonNegative,
  assertGreaterThan,
  assertLessThan,
} from '@utils/assertions';

function setSpeed(speed: number) {
  assertNonNegative(speed, 'Speed cannot be negative');
  assertLessThan(speed, 300, 'Speed exceeds maximum');

  // Use speed safely
  vehicle.velocity = speed;
}

function setOpacity(opacity: number) {
  assertInRange(opacity, 0, 1, 'Opacity must be between 0 and 1');
  material.opacity = opacity;
}
```

### Array Assertions

```typescript
import {
  assertNonEmptyArray,
  assertMinArrayLength,
  assertArrayElements,
} from '@utils/assertions';

function calculateAverage(numbers: number[]) {
  assertNonEmptyArray(numbers, 'Cannot calculate average of empty array');
  // TypeScript knows array has at least one element

  const sum = numbers.reduce((a, b) => a + b);
  return sum / numbers.length;
}

function processVehicles(vehicles: unknown[]) {
  assertMinArrayLength(vehicles, 1, 'At least one vehicle required');
  assertArrayElements(vehicles, isVehicle, 'Invalid vehicle data');
  // vehicles is now Vehicle[]

  vehicles.forEach(v => console.log(v.name));
}
```

### Object Property Assertions

```typescript
import {
  assertHasProperty,
  assertHasProperties,
  assertPropertyType,
} from '@utils/assertions';

function processConfig(config: unknown) {
  assertObject(config);
  assertHasProperty(config, 'apiUrl', 'Config must have apiUrl');
  assertPropertyType(
    config,
    'apiUrl',
    isString,
    'apiUrl must be a string'
  );

  // config.apiUrl is now string
  fetch(config.apiUrl);
}
```

### Geometric Assertions

```typescript
import {
  assertPoint3D,
  assertVector3D,
  assertCoordinatesInBounds,
} from '@utils/assertions';

function setVehiclePosition(vehicle: Vehicle, pos: unknown) {
  assertPoint3D(pos, 'Invalid position');

  // Check bounds
  const bounds = {
    minX: -1000, maxX: 1000,
    minY: -1000, maxY: 1000,
    minZ: 0, maxZ: 100,
  };

  assertCoordinatesInBounds(
    pos.x, pos.y, pos.z,
    bounds,
    'Position out of scene bounds'
  );

  vehicle.position = pos;
}
```

### Error Handling

```typescript
import { AssertionError } from '@utils/assertions';

try {
  processScene(scene);
} catch (error) {
  if (error instanceof AssertionError) {
    console.error('Assertion failed:', error.message);
    console.error('Context:', error.context);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Lint and Type Check

on: [push, pull_request]

jobs:
  rust-lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy
      - name: Run Clippy
        run: |
          cd rust-core
          cargo clippy --workspace --all-targets -- -D warnings
      - name: Run cargo-deny
        run: |
          cargo install cargo-deny
          cd rust-core
          cargo deny check

  typescript-lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '20'
      - name: Install dependencies
        run: npm ci
      - name: Run ESLint
        run: npm run lint
      - name: Run TypeScript
        run: npm run typecheck
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running pre-commit checks..."

# Rust checks
if [ -d "rust-core" ]; then
  echo "Checking Rust code..."
  cd rust-core
  cargo clippy --workspace --quiet -- -D warnings
  if [ $? -ne 0 ]; then
    echo "Clippy failed. Please fix the warnings."
    exit 1
  fi
  cd ..
fi

# TypeScript checks
echo "Checking TypeScript code..."
npm run lint --silent
if [ $? -ne 0 ]; then
  echo "ESLint failed. Please fix the errors."
  exit 1
fi

npm run typecheck --silent
if [ $? -ne 0 ]; then
  echo "TypeScript check failed. Please fix the type errors."
  exit 1
fi

echo "All checks passed!"
exit 0
```

---

## IDE Setup

### VS Code

**.vscode/settings.json**:

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": ["--all-targets"],
  "rust-analyzer.diagnostics.disabled": [],
  "rust-analyzer.diagnostics.enable": true,

  "typescript.tsdk": "node_modules/typescript/lib",
  "typescript.enablePromptUseWorkspaceTsdk": true,

  "eslint.enable": true,
  "eslint.validate": [
    "javascript",
    "javascriptreact",
    "typescript",
    "typescriptreact"
  ],

  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true,
    "source.organizeImports": true
  },

  "editor.formatOnSave": true,
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### IntelliJ / WebStorm

1. Enable TypeScript language service
2. Enable ESLint inspection
3. Configure Rust plugin for clippy
4. Enable "Run clippy" on save

---

## Common Workflows

### Adding a New Feature

1. **Create feature branch**:
   ```bash
   git checkout -b feature/new-simulation
   ```

2. **Write code with types**:
   ```typescript
   // Use enterprise types
   import { Scene, Vehicle } from '@types/enterprise';

   // Use type guards
   import { isScene, isVehicle } from '@utils/typeGuards';

   // Use assertions
   import { assertDefined, assertPositive } from '@utils/assertions';
   ```

3. **Run linters**:
   ```bash
   # Rust
   cd rust-core && cargo clippy-strict

   # TypeScript
   npm run lint
   npm run typecheck
   ```

4. **Fix warnings before committing**

### Debugging Type Issues

```typescript
// Use type assertions to narrow types
import { assertObject, assertHasProperty } from '@utils/assertions';

function debugApiResponse(data: unknown) {
  console.log('Type:', typeof data);
  console.log('Value:', data);

  try {
    assertObject(data);
    console.log('✓ Is object');

    assertHasProperty(data, 'id');
    console.log('✓ Has id property');

    assertHasProperty(data, 'name');
    console.log('✓ Has name property');
  } catch (error) {
    if (error instanceof AssertionError) {
      console.error('Validation failed:', error.message);
      console.error('Context:', error.context);
    }
  }
}
```

### Refactoring with Safety

```typescript
// Before: unsafe
function calculateDistance(a: any, b: any): number {
  return Math.sqrt(
    (a.x - b.x) ** 2 + (a.y - b.y) ** 2 + (a.z - b.z) ** 2
  );
}

// After: type-safe
import { Point3D } from '@types/global';
import { assertPoint3D } from '@utils/assertions';

function calculateDistance(a: Point3D, b: Point3D): number {
  assertPoint3D(a, 'First point is invalid');
  assertPoint3D(b, 'Second point is invalid');

  return Math.sqrt(
    (a.x - b.x) ** 2 + (a.y - b.y) ** 2 + (a.z - b.z) ** 2
  );
}
```

---

## Best Practices

### Type Guards vs Assertions

**Use Type Guards when**:
- Validating external input
- Conditional logic based on type
- Non-critical validation

**Use Assertions when**:
- Internal invariants
- Pre/post conditions
- Critical validation that should fail fast

```typescript
// Type guard for conditional logic
if (isVehicle(data)) {
  processVehicle(data);
} else {
  console.warn('Invalid vehicle data');
}

// Assertion for invariants
function processVehicle(vehicle: Vehicle) {
  assertDefined(vehicle.id, 'Vehicle must have ID');
  assertPositive(vehicle.mass, 'Vehicle mass must be positive');
  // Process vehicle...
}
```

### Progressive Type Safety

Start with basic types and gradually add more specificity:

```typescript
// Level 1: Basic types
function loadScene(id: string): Scene {
  // ...
}

// Level 2: Branded types
function loadScene(id: UUID): Scene {
  // ...
}

// Level 3: Validated input
function loadScene(id: UUID): Result<Scene, Error> {
  assertUUID(id, 'Invalid scene ID format');
  // ...
}

// Level 4: Full validation
function loadScene(id: UUID): Result<Scene, SceneError> {
  assertUUID(id, 'Invalid scene ID format');

  const result = fetchScene(id);
  if (result.success) {
    assertScene(result.data);
    return result;
  }
  return result;
}
```

---

## Resources

- [CODING_STANDARDS.md](./CODING_STANDARDS.md) - Full coding standards
- [TYPE_CONVENTIONS.md](./TYPE_CONVENTIONS.md) - Type system guide
- [Rust Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [cargo-deny Documentation](https://embarkstudios.github.io/cargo-deny/)

---

**For questions or issues, contact the AccuScene Enterprise development team.**
