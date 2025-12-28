# AccuScene Enterprise Coding Standards v0.2.5

**Last Updated:** 2025-12-28

## Table of Contents

- [Overview](#overview)
- [General Principles](#general-principles)
- [Rust Coding Standards](#rust-coding-standards)
- [TypeScript Coding Standards](#typescript-coding-standards)
- [Git Workflow](#git-workflow)
- [Code Review Guidelines](#code-review-guidelines)
- [Testing Standards](#testing-standards)
- [Documentation Standards](#documentation-standards)
- [Security Standards](#security-standards)

---

## Overview

This document defines the coding standards for AccuScene Enterprise. All contributors must follow these standards to ensure code quality, maintainability, and consistency across the codebase.

### Goals

- **Consistency**: Code should look like it was written by one person
- **Readability**: Code should be self-documenting and easy to understand
- **Maintainability**: Code should be easy to modify and extend
- **Reliability**: Code should be robust and handle edge cases
- **Performance**: Code should be efficient and scalable

---

## General Principles

### 1. Code Clarity

- **Write self-documenting code**: Variable and function names should clearly express intent
- **Avoid clever tricks**: Prefer readable code over "clever" one-liners
- **Use meaningful names**: Names should describe what, not how
- **Keep functions small**: Each function should do one thing well
- **Limit nesting**: Maximum 3-4 levels of nesting

### 2. DRY (Don't Repeat Yourself)

- Extract common logic into reusable functions
- Create utilities for repeated patterns
- Use configuration over duplication

### 3. SOLID Principles

- **Single Responsibility**: Each module/class should have one reason to change
- **Open/Closed**: Open for extension, closed for modification
- **Liskov Substitution**: Subtypes should be substitutable for base types
- **Interface Segregation**: Many specific interfaces over one general interface
- **Dependency Inversion**: Depend on abstractions, not concretions

### 4. Error Handling

- **Never ignore errors**: All errors must be handled
- **Use Result types**: Prefer `Result<T, E>` over exceptions (Rust)
- **Provide context**: Error messages should be actionable
- **Fail fast**: Validate inputs early
- **Log appropriately**: Log errors with sufficient context

### 5. Performance

- **Profile before optimizing**: Don't guess, measure
- **Optimize the critical path**: Focus on hot code paths
- **Consider space-time tradeoffs**: Balance memory vs CPU
- **Use appropriate data structures**: Choose based on access patterns
- **Lazy evaluation**: Compute only what's needed

---

## Rust Coding Standards

### Naming Conventions

```rust
// Modules: snake_case
mod accident_reconstruction;

// Types: PascalCase
struct SceneData { }
enum VehicleType { }
trait Simulatable { }

// Functions: snake_case
fn calculate_impact_force() -> f64 { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_VELOCITY: f64 = 200.0;
const DEFAULT_TIMESTEP: f64 = 0.01;

// Variables: snake_case
let vehicle_position = Point3D::new(0.0, 0.0, 0.0);

// Lifetimes: 'short, 'a, 'b
fn process_scene<'a>(scene: &'a Scene) -> &'a str { }

// Generic parameters: single uppercase letter or PascalCase
fn calculate<T: Numeric>(value: T) -> T { }
fn process<TInput, TOutput>(input: TInput) -> TOutput { }
```

### Code Organization

```rust
// File structure
// 1. Module documentation
//! Module for scene simulation
//!
//! This module provides core simulation functionality...

// 2. Imports (group by std, external crates, internal)
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::physics::PhysicsEngine;
use crate::types::{Scene, Vehicle};

// 3. Type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub timestep: f64,
    pub iterations: usize,
}

// 4. Constants
const DEFAULT_GRAVITY: f64 = 9.81;

// 5. Implementation
impl SimulationConfig {
    pub fn new(timestep: f64, iterations: usize) -> Self {
        Self { timestep, iterations }
    }
}

// 6. Functions
pub fn run_simulation(config: SimulationConfig) -> Result<SimulationResult, SimulationError> {
    // Implementation
}

// 7. Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_config() {
        // Test implementation
    }
}
```

### Error Handling

```rust
// Use thiserror for error definitions
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SceneError {
    #[error("Scene not found: {0}")]
    NotFound(String),

    #[error("Invalid scene data: {0}")]
    InvalidData(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

// Always return Result for fallible operations
pub fn load_scene(path: &str) -> Result<Scene, SceneError> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| SceneError::Io(e))?;

    serde_json::from_str(&data)
        .map_err(|_| SceneError::InvalidData(path.to_string()))
}

// Use ? operator for error propagation
pub async fn process_scene(scene_id: &str) -> Result<ProcessedScene, SceneError> {
    let scene = load_scene(scene_id)?;
    let validated = validate_scene(&scene)?;
    let processed = transform_scene(validated).await?;
    Ok(processed)
}
```

### Async/Await

```rust
// Always use async for I/O operations
pub async fn fetch_scene(id: &str) -> Result<Scene, SceneError> {
    let response = reqwest::get(format!("/api/scenes/{}", id))
        .await?
        .json::<Scene>()
        .await?;
    Ok(response)
}

// Use tokio::spawn for concurrent tasks
pub async fn process_multiple_scenes(ids: Vec<String>) -> Result<Vec<Scene>, SceneError> {
    let tasks: Vec<_> = ids
        .into_iter()
        .map(|id| tokio::spawn(fetch_scene(id)))
        .collect();

    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await??);
    }
    Ok(results)
}

// Use channels for message passing
use tokio::sync::mpsc;

pub async fn simulation_worker(mut rx: mpsc::Receiver<SimulationJob>) {
    while let Some(job) = rx.recv().await {
        process_simulation(job).await;
    }
}
```

### Type Safety

```rust
// Use newtype pattern for type safety
#[derive(Debug, Clone, Copy)]
pub struct SceneId(uuid::Uuid);

#[derive(Debug, Clone, Copy)]
pub struct VehicleId(uuid::Uuid);

// This prevents mixing scene IDs with vehicle IDs
fn get_scene(id: SceneId) -> Option<Scene> { }
fn get_vehicle(id: VehicleId) -> Option<Vehicle> { }

// Use builder pattern for complex construction
pub struct SimulationBuilder {
    timestep: Option<f64>,
    iterations: Option<usize>,
    gravity: Option<f64>,
}

impl SimulationBuilder {
    pub fn new() -> Self {
        Self {
            timestep: None,
            iterations: None,
            gravity: None,
        }
    }

    pub fn timestep(mut self, timestep: f64) -> Self {
        self.timestep = Some(timestep);
        self
    }

    pub fn iterations(mut self, iterations: usize) -> Self {
        self.iterations = Some(iterations);
        self
    }

    pub fn build(self) -> Result<Simulation, BuildError> {
        Ok(Simulation {
            timestep: self.timestep.ok_or(BuildError::MissingTimestep)?,
            iterations: self.iterations.ok_or(BuildError::MissingIterations)?,
            gravity: self.gravity.unwrap_or(9.81),
        })
    }
}
```

### Performance Guidelines

```rust
// Use references to avoid unnecessary cloning
pub fn calculate_distance(a: &Point3D, b: &Point3D) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
}

// Use Cow for flexible ownership
use std::borrow::Cow;

pub fn process_data(data: Cow<str>) -> String {
    if data.contains("special") {
        data.to_uppercase() // Only allocates if needed
    } else {
        data.into_owned()
    }
}

// Use Arc/Rc for shared ownership
use std::sync::Arc;

pub struct SharedScene {
    data: Arc<SceneData>,
}

// Use iterators instead of collecting unnecessarily
pub fn sum_velocities(vehicles: &[Vehicle]) -> f64 {
    vehicles.iter()
        .map(|v| v.velocity.magnitude())
        .sum()
}

// Pre-allocate collections when size is known
pub fn create_vehicles(count: usize) -> Vec<Vehicle> {
    let mut vehicles = Vec::with_capacity(count);
    for i in 0..count {
        vehicles.push(Vehicle::default());
    }
    vehicles
}
```

---

## TypeScript Coding Standards

### Naming Conventions

```typescript
// Interfaces and Types: PascalCase
interface SceneData { }
type VehicleType = 'sedan' | 'suv' | 'truck';

// Classes: PascalCase
class SceneManager { }

// Functions: camelCase
function calculateImpactForce(): number { }

// Variables: camelCase
const vehiclePosition = { x: 0, y: 0, z: 0 };

// Constants: SCREAMING_SNAKE_CASE
const MAX_VELOCITY = 200;
const DEFAULT_TIMESTEP = 0.01;

// Private members: prefix with _
class Scene {
  private _data: SceneData;

  get data(): SceneData {
    return this._data;
  }
}

// Enums: PascalCase for enum, SCREAMING_SNAKE_CASE for values
enum VehicleStatus {
  STOPPED = 'STOPPED',
  MOVING = 'MOVING',
  CRASHED = 'CRASHED',
}
```

### Type Annotations

```typescript
// Always provide explicit return types for functions
function calculateDistance(a: Point3D, b: Point3D): number {
  return Math.sqrt(
    Math.pow(a.x - b.x, 2) +
    Math.pow(a.y - b.y, 2) +
    Math.pow(a.z - b.z, 2)
  );
}

// Use type inference for simple variables
const count = 10; // Type inferred as number
const name = "Scene 1"; // Type inferred as string

// Explicitly type complex objects
const config: SimulationConfig = {
  timestep: 0.01,
  iterations: 1000,
  gravity: 9.81,
};

// Use generics for reusable code
function findById<T extends { id: string }>(items: T[], id: string): T | undefined {
  return items.find(item => item.id === id);
}

// Avoid 'any' - use 'unknown' if type is truly unknown
function processData(data: unknown): void {
  if (typeof data === 'string') {
    console.log(data.toUpperCase());
  }
}
```

### Error Handling

```typescript
// Use custom error classes
class SceneError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: unknown
  ) {
    super(message);
    this.name = 'SceneError';
  }
}

// Always handle promise rejections
async function loadScene(id: string): Promise<Scene> {
  try {
    const response = await fetch(`/api/scenes/${id}`);
    if (!response.ok) {
      throw new SceneError(
        `Failed to load scene: ${response.statusText}`,
        'LOAD_FAILED',
        { statusCode: response.status }
      );
    }
    return await response.json();
  } catch (error) {
    if (error instanceof SceneError) {
      throw error;
    }
    throw new SceneError(
      'Unexpected error loading scene',
      'UNEXPECTED_ERROR',
      error
    );
  }
}

// Use Result type for explicit error handling
type Result<T, E = Error> =
  | { success: true; value: T }
  | { success: false; error: E };

function parseScene(data: string): Result<Scene> {
  try {
    const scene = JSON.parse(data);
    return { success: true, value: scene };
  } catch (error) {
    return {
      success: false,
      error: new SceneError('Parse failed', 'PARSE_ERROR', error)
    };
  }
}
```

### Async/Await

```typescript
// Always use async/await over .then()
async function processScene(id: string): Promise<ProcessedScene> {
  const scene = await loadScene(id);
  const validated = await validateScene(scene);
  const processed = await transformScene(validated);
  return processed;
}

// Use Promise.all for parallel operations
async function loadMultipleScenes(ids: string[]): Promise<Scene[]> {
  return Promise.all(ids.map(id => loadScene(id)));
}

// Use Promise.allSettled for error resilience
async function loadScenesWithErrors(ids: string[]): Promise<Scene[]> {
  const results = await Promise.allSettled(
    ids.map(id => loadScene(id))
  );

  return results
    .filter((r): r is PromiseFulfilledResult<Scene> => r.status === 'fulfilled')
    .map(r => r.value);
}
```

### React Best Practices

```typescript
// Use functional components with TypeScript
interface SceneViewerProps {
  sceneId: string;
  onSceneLoad?: (scene: Scene) => void;
}

export const SceneViewer: React.FC<SceneViewerProps> = ({
  sceneId,
  onSceneLoad,
}) => {
  const [scene, setScene] = useState<Scene | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let cancelled = false;

    const loadData = async () => {
      setLoading(true);
      try {
        const data = await loadScene(sceneId);
        if (!cancelled) {
          setScene(data);
          onSceneLoad?.(data);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err as Error);
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };

    loadData();

    return () => {
      cancelled = true;
    };
  }, [sceneId, onSceneLoad]);

  if (loading) {
    return <LoadingSpinner />;
  }

  if (error) {
    return <ErrorDisplay error={error} />;
  }

  if (!scene) {
    return null;
  }

  return <SceneCanvas scene={scene} />;
};

// Use custom hooks for reusable logic
function useScene(sceneId: string) {
  const [scene, setScene] = useState<Scene | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    // ... implementation
  }, [sceneId]);

  return { scene, loading, error };
}
```

---

## Git Workflow

### Branch Naming

- `main` - Production-ready code
- `develop` - Integration branch
- `feature/description` - New features
- `fix/description` - Bug fixes
- `hotfix/description` - Urgent production fixes
- `refactor/description` - Code refactoring
- `docs/description` - Documentation updates

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(simulation): add collision detection algorithm

fix(scene): correct vehicle position calculation

docs(api): update endpoint documentation

refactor(physics): simplify force calculation
```

### Pull Request Process

1. Create feature branch from `develop`
2. Make changes with atomic commits
3. Write/update tests
4. Update documentation
5. Run linters and tests
6. Create PR with description
7. Address review comments
8. Squash commits if needed
9. Merge to `develop`

---

## Code Review Guidelines

### For Authors

- Keep PRs small and focused
- Provide clear description
- Link related issues
- Add screenshots for UI changes
- Respond to feedback promptly
- Don't take feedback personally

### For Reviewers

- Be constructive and respectful
- Focus on code, not the person
- Explain the "why" behind suggestions
- Approve if minor issues remain
- Use PR templates
- Review within 24 hours

### Review Checklist

- [ ] Code follows style guidelines
- [ ] Tests are included and passing
- [ ] Documentation is updated
- [ ] No security vulnerabilities
- [ ] Performance impact considered
- [ ] Error handling is appropriate
- [ ] Backward compatibility maintained

---

## Testing Standards

### Unit Tests

- Test one thing per test
- Use descriptive test names
- Follow AAA pattern (Arrange, Act, Assert)
- Mock external dependencies
- Aim for >80% code coverage

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_distance_returns_correct_value() {
        // Arrange
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(3.0, 4.0, 0.0);

        // Act
        let distance = calculate_distance(&a, &b);

        // Assert
        assert_eq!(distance, 5.0);
    }
}
```

### Integration Tests

- Test component interactions
- Use real dependencies when possible
- Test happy and error paths
- Clean up test data

### End-to-End Tests

- Test critical user journeys
- Use production-like environment
- Keep tests maintainable
- Run in CI/CD pipeline

---

## Documentation Standards

### Code Comments

```rust
// Good: Explain WHY, not WHAT
// Use RK4 integration for better stability at large timesteps
let next_state = rk4_integrate(current_state, timestep);

// Bad: Repeat what code does
// Call the rk4_integrate function
let next_state = rk4_integrate(current_state, timestep);
```

### Function Documentation

```rust
/// Calculates the impact force between two vehicles
///
/// # Arguments
///
/// * `vehicle1` - The first vehicle involved in collision
/// * `vehicle2` - The second vehicle involved in collision
/// * `contact_point` - The point of contact in world coordinates
///
/// # Returns
///
/// The magnitude of impact force in Newtons
///
/// # Examples
///
/// ```
/// let force = calculate_impact_force(&car1, &car2, contact_point);
/// assert!(force > 0.0);
/// ```
pub fn calculate_impact_force(
    vehicle1: &Vehicle,
    vehicle2: &Vehicle,
    contact_point: Point3D,
) -> f64 {
    // Implementation
}
```

---

## Security Standards

### Input Validation

- Validate all user inputs
- Sanitize data before storage
- Use parameterized queries
- Implement rate limiting
- Check file uploads

### Authentication & Authorization

- Use JWT for API authentication
- Implement role-based access control
- Encrypt sensitive data
- Use HTTPS everywhere
- Implement session timeout

### Secret Management

- Never commit secrets to git
- Use environment variables
- Rotate credentials regularly
- Use secret management tools
- Audit access logs

---

## Enforcement

These standards are enforced through:

1. **Automated Linting**: Clippy (Rust), ESLint (TypeScript)
2. **Pre-commit Hooks**: Format and lint checks
3. **CI/CD Pipeline**: Automated tests and checks
4. **Code Review**: Manual review process
5. **Documentation**: This document and inline comments

---

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [Clean Code by Robert C. Martin](https://www.amazon.com/Clean-Code-Handbook-Software-Craftsmanship/dp/0132350882)
- [The Pragmatic Programmer](https://pragprog.com/titles/tpp20/the-pragmatic-programmer-20th-anniversary-edition/)
