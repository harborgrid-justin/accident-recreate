# Contributing to AccuScene Enterprise Rust System

Thank you for your interest in contributing to AccuScene Enterprise! This document provides guidelines for the multi-agent development process.

## Agent Assignment System

This project uses a **multi-agent development model** where each agent is responsible for specific crates:

- **Agent 1:** `accuscene-core` (foundation)
- **Agent 2:** `accuscene-ffi` (FFI bridge)
- **Agent 3:** `accuscene-physics` (physics simulation)
- **Agent 4:** `accuscene-compression` (data compression)
- **Agent 5:** `accuscene-database` (database layer)
- **Agent 6:** `accuscene-crypto` (cryptographic security)
- **Agent 7:** `accuscene-jobs` (job processing)
- **Agent 8:** `accuscene-streaming` (event streaming)
- **Agent 9:** `accuscene-cache` (caching system)
- **Agent 10:** `accuscene-telemetry` (metrics/logging)
- **Agent 11:** `accuscene-cluster` (distributed clustering)
- **Agent 12:** Testing & Quality Assurance
- **Agent 13:** Documentation
- **Agent 14:** Build & CI/CD

## Getting Started

### Prerequisites

1. **Rust:** Install via [rustup](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default stable
   rustup component add rustfmt clippy
   ```

2. **Development Tools:**
   ```bash
   cargo install cargo-watch cargo-tarpaulin cargo-audit cargo-outdated
   ```

3. **Node.js:** Version 18+ for FFI development
   ```bash
   # Using nvm (recommended)
   nvm install 18
   nvm use 18
   ```

4. **Database (Optional):**
   ```bash
   # PostgreSQL
   sudo apt-get install postgresql-14

   # Redis
   sudo apt-get install redis-server
   ```

### First Time Setup

```bash
# Clone repository
git clone https://github.com/accuscene/enterprise.git
cd accident-recreate/rust-core

# Build workspace
make build

# Run tests
make test

# Verify everything works
make check
```

## Development Workflow

### 1. Update SCRATCHPAD.md

**CRITICAL:** Before starting any work:

1. Open `/home/user/accident-recreate/rust-core/SCRATCHPAD.md`
2. Update your agent's status section
3. Mark tasks as in-progress (🚧)
4. Note any blockers (❌)

Example:
```markdown
### Agent 3: Physics Simulation (`accuscene-physics`)
- **Status:** 🚧 In Progress
- **Current Task:** Implementing collision detection
- **Blockers:** None
- **Notes:** ETA 2 hours
```

### 2. Create Feature Branch

```bash
# Branch naming: agent-{number}/{feature-name}
git checkout -b agent-3/collision-detection
```

### 3. Implement Feature

Follow these guidelines:

#### Code Style

- **Format:** Run `cargo fmt` before committing
- **Lints:** Fix all `cargo clippy` warnings
- **Comments:** Document public APIs with `///` doc comments
- **Tests:** Write tests for all new functionality

#### Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Descriptive error message: {details}")]
    MyErrorType { details: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

#### Logging

```rust
use tracing::{info, debug, warn, error, instrument};

#[instrument(skip(self))]
pub async fn my_function(&self, id: Uuid) -> Result<()> {
    info!(?id, "Starting operation");
    debug!("Detailed debug info");
    Ok(())
}
```

#### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = my_function(input);

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_functionality() {
        let result = async_function().await;
        assert_eq!(result, expected);
    }
}
```

### 4. Run Quality Checks

Before committing:

```bash
# Format code
make fmt

# Run linter
make lint

# Run tests
make test

# Or run all checks
make check
```

### 5. Commit Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Format: <type>(<scope>): <description>

git add .
git commit -m "feat(physics): add collision detection algorithm"

# Types:
# - feat: New feature
# - fix: Bug fix
# - docs: Documentation only
# - test: Adding tests
# - refactor: Code refactoring
# - perf: Performance improvement
# - chore: Maintenance tasks
```

### 6. Update SCRATCHPAD.md Again

Mark completed tasks:

```markdown
### Agent 3: Physics Simulation (`accuscene-physics`)
- **Status:** ✅ Completed
- **Deliverables:**
  - [x] Collision detection algorithm
  - [ ] Collision response (next)
```

### 7. Create Pull Request

```bash
# Push to remote
git push origin agent-3/collision-detection

# Create PR using GitHub CLI
gh pr create \
  --title "[Agent 3] Add collision detection algorithm" \
  --body "$(cat <<EOF
## Summary
- Implemented broad-phase collision detection using spatial hashing
- Added narrow-phase detection with GJK algorithm
- Performance: <1ms for 100 objects

## Testing
- Added unit tests for edge cases
- Benchmarks show 10x improvement over naive approach

## Checklist
- [x] Tests pass
- [x] Clippy clean
- [x] Formatted
- [x] Documentation updated
- [x] SCRATCHPAD.md updated
EOF
)"
```

## Code Review Guidelines

### As a Reviewer

- **Functionality:** Does it work as intended?
- **Tests:** Are there adequate tests?
- **Performance:** Any performance concerns?
- **Security:** Any security implications?
- **Documentation:** Is it well documented?
- **Style:** Follows project conventions?

### Approval Criteria

Pull requests must meet ALL criteria:

- ✅ All tests pass
- ✅ Clippy produces no warnings
- ✅ Code is formatted
- ✅ Documentation is complete
- ✅ SCRATCHPAD.md is updated
- ✅ No security vulnerabilities
- ✅ At least one approval

## Testing Standards

### Unit Tests

- **Coverage:** Aim for >80% code coverage
- **Scope:** Test individual functions/methods
- **Speed:** Should run in milliseconds

```rust
#[test]
fn test_vehicle_mass_validation() {
    assert!(Vehicle::new("Car", -100.0).is_err());
    assert!(Vehicle::new("Car", 1500.0).is_ok());
}
```

### Integration Tests

- **Scope:** Test module interactions
- **Location:** `tests/` directory
- **Setup:** May use test fixtures

```rust
// tests/physics_integration.rs
#[tokio::test]
async fn test_full_simulation() {
    let simulator = Simulator::new(config).await?;
    let scene = load_test_scene();
    let result = simulator.simulate(scene).await?;
    assert!(result.is_valid());
}
```

### Property-Based Tests

Use `proptest` for property testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_collision_symmetry(mass_a in 100.0..5000.0, mass_b in 100.0..5000.0) {
        let result_ab = collide(mass_a, mass_b);
        let result_ba = collide(mass_b, mass_a);
        prop_assert_eq!(result_ab.magnitude(), result_ba.magnitude());
    }
}
```

### Benchmarks

Use `criterion` for benchmarks:

```rust
// benches/physics_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn collision_benchmark(c: &mut Criterion) {
    c.bench_function("collision_detection", |b| {
        b.iter(|| {
            detect_collision(black_box(&vehicle_a), black_box(&vehicle_b))
        })
    });
}

criterion_group!(benches, collision_benchmark);
criterion_main!(benches);
```

## Performance Guidelines

### Critical Paths

These paths must meet strict performance requirements:

1. **Physics Simulation:** <16ms per frame
2. **FFI Calls:** <1ms overhead
3. **Database Queries:** <10ms p99
4. **Compression:** >100 MB/s

### Optimization Checklist

- [ ] Profile before optimizing (`cargo flamegraph`)
- [ ] Use `#[inline]` for hot small functions
- [ ] Preallocate collections when size is known
- [ ] Use `&str` instead of `String` when possible
- [ ] Avoid clones in hot paths
- [ ] Use `Cow<'_, str>` for conditional ownership
- [ ] Consider SIMD for data-parallel operations

### Example: Optimized Hot Path

```rust
// ❌ Slow: Multiple allocations
pub fn format_ids(ids: &[Uuid]) -> Vec<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

// ✅ Fast: Preallocated, inline
#[inline]
pub fn format_ids(ids: &[Uuid]) -> Vec<String> {
    let mut result = Vec::with_capacity(ids.len());
    for id in ids {
        result.push(id.to_string());
    }
    result
}
```

## Security Guidelines

### Input Validation

Always validate external inputs:

```rust
pub fn create_vehicle(mass: f64) -> Result<Vehicle> {
    if mass <= 0.0 {
        return Err(Error::InvalidMass { mass });
    }
    if mass > 100_000.0 {
        return Err(Error::MassTooLarge { mass });
    }
    Ok(Vehicle { mass, ..Default::default() })
}
```

### Unsafe Code

Minimize `unsafe` usage. When necessary:

```rust
/// # Safety
///
/// This function is safe if:
/// - `ptr` points to valid, initialized memory
/// - `ptr` is properly aligned
/// - No other references to this memory exist
///
/// # Example
///
/// ```
/// let data = vec![1, 2, 3];
/// let ptr = data.as_ptr();
/// unsafe {
///     let value = read_unchecked(ptr);
/// }
/// ```
#[inline]
pub unsafe fn read_unchecked<T>(ptr: *const T) -> T {
    debug_assert!(!ptr.is_null(), "Pointer must not be null");
    std::ptr::read(ptr)
}
```

### Dependency Auditing

Run security audits weekly:

```bash
# Check for known vulnerabilities
cargo audit

# Check dependency licenses
cargo deny check licenses

# Check for outdated dependencies
cargo outdated
```

## Documentation Standards

### Module Documentation

```rust
//! # Physics Simulation Module
//!
//! This module provides high-performance physics simulation capabilities
//! for accident reconstruction.
//!
//! ## Features
//!
//! - Vehicle dynamics simulation
//! - Collision detection and response
//! - Real-time performance (60 FPS)
//!
//! ## Example
//!
//! ```rust
//! use accuscene_physics::{Simulator, Vehicle};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let simulator = Simulator::new();
//! let vehicle = Vehicle::new("Car", 1500.0)?;
//! let result = simulator.simulate(vehicle).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance
//!
//! Typical simulation performance on modern hardware:
//! - Single vehicle: <1ms
//! - 10 vehicles: <5ms
//! - 100 vehicles: <16ms (60 FPS)
```

### Function Documentation

```rust
/// Simulates a collision between two vehicles.
///
/// This function uses the impulse-based collision response algorithm
/// to calculate post-collision velocities.
///
/// # Arguments
///
/// * `vehicle_a` - The first vehicle in the collision
/// * `vehicle_b` - The second vehicle in the collision
/// * `contact_point` - The point of contact in world space
///
/// # Returns
///
/// Returns `Ok(CollisionResult)` with the collision outcome, or
/// `Err(PhysicsError)` if the simulation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Either vehicle has invalid mass (≤0 or >100,000 kg)
/// - The contact point is outside the collision bounds
/// - The simulation diverges (rare)
///
/// # Example
///
/// ```rust
/// use accuscene_physics::{Vehicle, Point3, simulate_collision};
///
/// let car1 = Vehicle::new("Car 1", 1500.0)?;
/// let car2 = Vehicle::new("Car 2", 1600.0)?;
/// let contact = Point3::new(0.0, 0.0, 0.0);
///
/// let result = simulate_collision(&car1, &car2, contact)?;
/// println!("Impact force: {} N", result.impact_force);
/// ```
///
/// # Performance
///
/// This function typically completes in <1ms for simple collisions.
///
/// # See Also
///
/// - [`detect_collision`] for collision detection
/// - [`Vehicle`] for vehicle configuration
pub fn simulate_collision(
    vehicle_a: &Vehicle,
    vehicle_b: &Vehicle,
    contact_point: Point3,
) -> Result<CollisionResult> {
    // Implementation...
}
```

## Inter-Agent Communication

### Dependency Management

If your crate depends on another agent's work:

1. **Check SCRATCHPAD.md** for their status
2. **Use trait-based interfaces** for loose coupling
3. **Document requirements** clearly
4. **Communicate blockers** immediately

### Example: Trait-Based Interface

```rust
// Agent 5 defines trait (accuscene-database)
#[async_trait]
pub trait SceneRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Scene>>;
}

// Agent 3 uses trait (accuscene-physics)
pub async fn load_scene<R: SceneRepository>(
    repo: &R,
    id: Uuid,
) -> Result<Scene> {
    repo.find_by_id(id)
        .await?
        .ok_or(Error::SceneNotFound { id })
}
```

### Blocking Issues

If blocked by another agent:

1. Update SCRATCHPAD.md with blocker details
2. Create GitHub issue tagging responsible agent
3. Work on non-blocked tasks
4. Consider creating mock implementation

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **Major (1.0.0):** Breaking changes
- **Minor (0.1.0):** New features, backward compatible
- **Patch (0.0.1):** Bug fixes

### Release Checklist

- [ ] All tests pass
- [ ] Benchmarks meet targets
- [ ] Documentation is complete
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Security audit clean
- [ ] No clippy warnings
- [ ] FFI addon builds on all platforms

### Creating a Release

```bash
# Update version
cargo release version minor

# Build release
make release

# Tag release
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# Publish to crates.io (when public)
cargo publish --workspace
```

## Getting Help

### Resources

- **SCRATCHPAD.md:** Current project status
- **ARCHITECTURE.md:** System architecture
- **README.md:** Project overview
- **Rust Book:** https://doc.rust-lang.org/book/
- **Cargo Book:** https://doc.rust-lang.org/cargo/

### Communication

- **GitHub Issues:** Bug reports, feature requests
- **GitHub Discussions:** Architecture discussions
- **Slack:** #accuscene-rust-dev (for team members)
- **Email:** support@accuscene.com

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

---

**Thank you for contributing to AccuScene Enterprise!** 🦀
