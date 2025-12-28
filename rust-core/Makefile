# AccuScene Enterprise Rust - Makefile
# Convenience commands for common development tasks

.PHONY: help build test check fmt lint clean doc install run bench audit deps

# Default target
help:
	@echo "AccuScene Enterprise Rust Development Commands"
	@echo ""
	@echo "Build Commands:"
	@echo "  make build       - Build all workspace crates (debug)"
	@echo "  make release     - Build optimized release version"
	@echo "  make ffi         - Build FFI addon for Node.js"
	@echo ""
	@echo "Test Commands:"
	@echo "  make test        - Run all tests"
	@echo "  make test-unit   - Run unit tests only"
	@echo "  make test-int    - Run integration tests only"
	@echo "  make coverage    - Generate test coverage report"
	@echo ""
	@echo "Quality Commands:"
	@echo "  make check       - Run all checks (fmt, clippy, test)"
	@echo "  make fmt         - Format code with rustfmt"
	@echo "  make fmt-check   - Check code formatting"
	@echo "  make lint        - Run clippy linter"
	@echo "  make audit       - Security audit dependencies"
	@echo ""
	@echo "Documentation:"
	@echo "  make doc         - Generate and open documentation"
	@echo "  make doc-check   - Check documentation builds"
	@echo ""
	@echo "Performance:"
	@echo "  make bench       - Run benchmarks"
	@echo "  make flamegraph  - Generate performance flamegraph"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean       - Remove build artifacts"
	@echo "  make deps        - Check dependency status"
	@echo "  make update      - Update dependencies"
	@echo ""

# Build commands
build:
	cargo build --workspace

release:
	cargo build --workspace --release

ffi:
	cd accuscene-ffi && npm install && npm run build

# Test commands
test:
	cargo test --workspace

test-unit:
	cargo test --workspace --lib

test-int:
	cargo test --workspace --test '*'

coverage:
	cargo tarpaulin --workspace --out Html --output-dir ./coverage

# Quality commands
check: fmt-check lint test
	@echo "✓ All checks passed!"

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

audit:
	cargo audit

# Documentation
doc:
	cargo doc --workspace --no-deps --open

doc-check:
	cargo doc --workspace --no-deps

# Performance
bench:
	cargo bench --workspace

flamegraph:
	cargo flamegraph --bench physics_bench

# Maintenance
clean:
	cargo clean
	rm -rf target/
	rm -rf coverage/
	find . -name "*.node" -delete
	find . -name "node_modules" -type d -exec rm -rf {} +

deps:
	cargo outdated

update:
	cargo update

# Installation
install:
	cargo install cargo-tarpaulin cargo-audit cargo-outdated cargo-flamegraph

# Development server (if applicable)
run:
	cargo run --release

# Watch mode for development
watch:
	cargo watch -x 'build --workspace' -x 'test --workspace'

# Quick validation before commit
pre-commit: fmt lint test
	@echo "✓ Ready to commit!"

# CI simulation
ci: fmt-check lint test doc-check audit
	@echo "✓ CI checks passed!"
