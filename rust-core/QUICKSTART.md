# AccuScene Enterprise Rust - Quick Start Guide

**Get up and running in 5 minutes!**

## Prerequisites

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs)
- **Node.js 18+**: Install from [nodejs.org](https://nodejs.org)
- **Git**: For version control

## Installation

### 1. Automated Setup (Recommended)

```bash
cd /home/user/accident-recreate/rust-core
./setup.sh
```

This script will:
- ✅ Check all prerequisites
- ✅ Install Rust components (rustfmt, clippy)
- ✅ Install development tools
- ✅ Create directory structure
- ✅ Set up git hooks
- ✅ Build the workspace

### 2. Manual Setup

```bash
# Install Rust components
rustup component add rustfmt clippy

# Install development tools
cargo install cargo-watch cargo-tarpaulin cargo-audit cargo-outdated

# Build workspace
cargo build --workspace

# Run tests
cargo test --workspace
```

## Project Structure

```
rust-core/
├── Cargo.toml                 # Workspace configuration
├── SCRATCHPAD.md             # Agent coordination (IMPORTANT!)
├── README.md                 # Project overview
├── ARCHITECTURE.md           # System design
├── CONTRIBUTING.md           # Development guidelines
├── Makefile                  # Convenient commands
├── .rustfmt.toml            # Code formatting rules
├── clippy.toml              # Linting configuration
├── .gitignore               # Git ignore rules
└── accuscene-*/             # Individual crates (to be created)
```

## Agent Assignments

Each agent is responsible for specific crates:

| Agent | Crate | Responsibility |
|-------|-------|----------------|
| 1 | `accuscene-core` | Common types, errors, utilities |
| 2 | `accuscene-ffi` | Node.js FFI bridge (napi-rs) |
| 3 | `accuscene-physics` | Physics simulation engine |
| 4 | `accuscene-compression` | Data compression (LZ4, ZSTD) |
| 5 | `accuscene-database` | Database layer (PostgreSQL, SQLite) |
| 6 | `accuscene-crypto` | Cryptographic security |
| 7 | `accuscene-jobs` | Job processing & scheduling |
| 8 | `accuscene-streaming` | Event streaming |
| 9 | `accuscene-cache` | Caching system (Redis) |
| 10 | `accuscene-telemetry` | Metrics & logging |
| 11 | `accuscene-cluster` | Distributed clustering (Raft) |
| 12 | Testing | Quality assurance |
| 13 | Documentation | API docs, guides |
| 14 | Build/CI | CI/CD pipeline |

## Essential Commands

### Building

```bash
# Build all crates (debug)
make build
# or
cargo build --workspace

# Build release version
make release
# or
cargo build --workspace --release
```

### Testing

```bash
# Run all tests
make test
# or
cargo test --workspace

# Run specific crate tests
cargo test -p accuscene-core

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
make fmt
# or
cargo fmt --all

# Run linter
make lint
# or
cargo clippy --workspace --all-targets -- -D warnings

# Run all checks
make check
```

### Documentation

```bash
# Generate and open docs
make doc
# or
cargo doc --workspace --no-deps --open
```

## Development Workflow

### For Each Agent

1. **Check SCRATCHPAD.md** for your assignment and dependencies
2. **Create feature branch**: `agent-{N}/{feature-name}`
3. **Implement your crate** following the conventions
4. **Write tests** (aim for >80% coverage)
5. **Update SCRATCHPAD.md** with your progress
6. **Run quality checks**: `make check`
7. **Commit with conventional commits**: `feat(core): add entity types`
8. **Create pull request**: `[Agent N] Brief description`

### Example: Agent 1 (Core Library)

```bash
# Create branch
git checkout -b agent-1/core-types

# Create crate structure
cd accuscene-core
cargo init --lib

# Edit Cargo.toml
# Add dependencies, implement types, write tests

# Update SCRATCHPAD.md
# Mark "Common types and traits" as completed

# Run checks
cd ..
make check

# Commit and push
git add .
git commit -m "feat(core): implement common types and error handling"
git push origin agent-1/core-types

# Create PR
gh pr create --title "[Agent 1] Implement core types and error handling"
```

## Common Patterns

### Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Something went wrong: {0}")]
    Generic(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, MyError>;
```

### Logging

```rust
use tracing::{info, debug, instrument};

#[instrument]
pub async fn process_data(id: Uuid) -> Result<()> {
    info!(?id, "Processing started");
    debug!("Loading data...");
    // ... implementation
    Ok(())
}
```

### Async Functions

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Repository: Send + Sync {
    async fn find(&self, id: Uuid) -> Result<Option<Entity>>;
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_function() {
        let result = my_function();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = my_async_function().await;
        assert!(result.is_ok());
    }
}
```

## Dependency Graph

**Build Order (Important!):**

1. **Tier 0:** `accuscene-core` ← Start here!
2. **Tier 1:** `accuscene-physics`, `accuscene-compression`, `accuscene-crypto`, `accuscene-telemetry`
3. **Tier 2:** `accuscene-database`, `accuscene-cache`, `accuscene-streaming`, `accuscene-cluster`
4. **Tier 3:** `accuscene-jobs`
5. **Tier 4:** `accuscene-ffi` ← Integration layer (last!)

**Note:** Higher tiers depend on lower tiers. Complete lower tiers first!

## SCRATCHPAD.md Usage

**CRITICAL:** Always update `SCRATCHPAD.md` before and after work!

### Before Starting Work

```markdown
### Agent 3: Physics Simulation
- **Status:** 🚧 In Progress
- **Current Task:** Implementing collision detection
- **Blockers:** None
```

### After Completing Work

```markdown
### Agent 3: Physics Simulation
- **Status:** ✅ Completed
- **Deliverables:**
  - [x] Collision detection ← Mark completed
  - [ ] Collision response ← Next task
```

### If Blocked

```markdown
### Agent 7: Job Processing
- **Status:** ❌ Blocked
- **Blocker:** Waiting for Agent 5 (database layer)
- **Notes:** Need `Repository` trait to be implemented
```

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build --workspace

# Update dependencies
cargo update
```

### Test Failures

```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run single-threaded
cargo test -- --test-threads=1
```

### Format/Lint Issues

```bash
# Auto-fix formatting
cargo fmt --all

# See clippy suggestions
cargo clippy --workspace --all-targets
```

## Next Steps

1. ✅ **Read SCRATCHPAD.md** - Understand your assignment
2. ✅ **Read ARCHITECTURE.md** - Understand system design
3. ✅ **Read CONTRIBUTING.md** - Understand workflow
4. ✅ **Start coding!** - Follow your agent assignment

## Getting Help

- **SCRATCHPAD.md** - Current status and assignments
- **ARCHITECTURE.md** - System architecture and design decisions
- **CONTRIBUTING.md** - Detailed development guidelines
- **README.md** - Project overview and documentation
- **Rust Book** - https://doc.rust-lang.org/book/
- **GitHub Issues** - Report bugs and ask questions

## Useful Links

- **Workspace:** `/home/user/accident-recreate/rust-core/`
- **Main App:** `/home/user/accident-recreate/` (Electron/React/TypeScript)
- **Rust Docs:** https://docs.rs/
- **Tokio Docs:** https://docs.rs/tokio/
- **napi-rs Docs:** https://napi.rs/

---

**You're all set! Start building! 🚀🦀**
