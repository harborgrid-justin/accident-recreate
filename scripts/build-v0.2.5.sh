#!/bin/bash

################################################################################
# AccuScene Enterprise v0.2.5 - Comprehensive Build Script
#
# This script builds all components of the AccuScene Enterprise platform:
# - Rust core libraries and services
# - TypeScript/JavaScript frontend and API
# - Runs linting and tests
# - Creates build artifacts
################################################################################

set -e  # Exit on error
set -o pipefail  # Catch errors in pipes

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Build configuration
BUILD_MODE="${BUILD_MODE:-release}"
SKIP_TESTS="${SKIP_TESTS:-false}"
SKIP_LINT="${SKIP_LINT:-false}"
PARALLEL_JOBS="${PARALLEL_JOBS:-4}"
BUILD_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BUILD_DIR="${PROJECT_ROOT}/build"
ARTIFACTS_DIR="${BUILD_DIR}/artifacts_${BUILD_TIMESTAMP}"

# Logging
LOG_FILE="${BUILD_DIR}/build_${BUILD_TIMESTAMP}.log"

################################################################################
# Logging Functions
################################################################################

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $*" | tee -a "${LOG_FILE}"
}

log_success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] ✓${NC} $*" | tee -a "${LOG_FILE}"
}

log_error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ✗${NC} $*" | tee -a "${LOG_FILE}"
}

log_warning() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] ⚠${NC} $*" | tee -a "${LOG_FILE}"
}

log_section() {
    echo -e "\n${BLUE}═══════════════════════════════════════════════════════════${NC}" | tee -a "${LOG_FILE}"
    echo -e "${BLUE}  $*${NC}" | tee -a "${LOG_FILE}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}\n" | tee -a "${LOG_FILE}"
}

################################################################################
# Error Handling
################################################################################

error_exit() {
    log_error "$1"
    log_error "Build failed! Check log at: ${LOG_FILE}"
    exit 1
}

trap 'error_exit "Build interrupted"' INT TERM

################################################################################
# Environment Check
################################################################################

check_environment() {
    log_section "Environment Check"

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        error_exit "cargo not found. Please install Rust."
    fi
    log "Rust version: $(rustc --version)"
    log "Cargo version: $(cargo --version)"

    # Check Node.js
    if ! command -v node &> /dev/null; then
        error_exit "node not found. Please install Node.js."
    fi
    log "Node.js version: $(node --version)"
    log "npm version: $(npm --version)"

    # Check TypeScript
    if ! command -v tsc &> /dev/null; then
        log_warning "tsc not found globally, will use local version"
    else
        log "TypeScript version: $(tsc --version)"
    fi

    log_success "Environment check passed"
}

################################################################################
# Setup
################################################################################

setup_build() {
    log_section "Build Setup"

    # Create build directories
    mkdir -p "${BUILD_DIR}"
    mkdir -p "${ARTIFACTS_DIR}"

    log "Build mode: ${BUILD_MODE}"
    log "Build directory: ${BUILD_DIR}"
    log "Artifacts directory: ${ARTIFACTS_DIR}"
    log "Parallel jobs: ${PARALLEL_JOBS}"

    # Initialize log file
    echo "AccuScene Enterprise v0.2.5 Build Log" > "${LOG_FILE}"
    echo "Build started at: $(date)" >> "${LOG_FILE}"
    echo "Build mode: ${BUILD_MODE}" >> "${LOG_FILE}"
    echo "----------------------------------------" >> "${LOG_FILE}"

    log_success "Build setup complete"
}

################################################################################
# Clean Previous Builds
################################################################################

clean_previous_builds() {
    log_section "Cleaning Previous Builds"

    cd "${PROJECT_ROOT}"

    # Clean Rust builds
    if [ -d "rust-core/target" ]; then
        log "Cleaning Rust target directory..."
        cd rust-core
        cargo clean 2>&1 | tee -a "${LOG_FILE}" || log_warning "Cargo clean had warnings"
        cd ..
    fi

    # Clean Node builds
    if [ -d "dist" ]; then
        log "Cleaning dist directory..."
        rm -rf dist
    fi

    if [ -d "build/electron" ]; then
        log "Cleaning electron build directory..."
        rm -rf build/electron
    fi

    log_success "Clean complete"
}

################################################################################
# Install Dependencies
################################################################################

install_dependencies() {
    log_section "Installing Dependencies"

    cd "${PROJECT_ROOT}"

    # Install Node dependencies
    log "Installing npm dependencies..."
    npm ci 2>&1 | tee -a "${LOG_FILE}" || error_exit "npm ci failed"

    log_success "Dependencies installed"
}

################################################################################
# Build Rust Core
################################################################################

build_rust_core() {
    log_section "Building Rust Core (v0.2.5)"

    cd "${PROJECT_ROOT}/rust-core"

    local cargo_flags=""
    if [ "${BUILD_MODE}" = "release" ]; then
        cargo_flags="--release"
    fi

    log "Building all workspace crates..."
    log "Crates include:"
    log "  - accuscene-core, accuscene-ffi, accuscene-physics"
    log "  - accuscene-compression, accuscene-database, accuscene-crypto"
    log "  - accuscene-jobs, accuscene-streaming, accuscene-cache"
    log "  - accuscene-telemetry, accuscene-cluster, accuscene-eventsourcing"
    log "  - accuscene-analytics, accuscene-ml, accuscene-security"
    log "  - NEW: accuscene-dashboard, accuscene-notifications, accuscene-visualization"
    log "  - NEW: accuscene-gestures, accuscene-offline, accuscene-sso"
    log "  - NEW: accuscene-search, accuscene-transfer, accuscene-preferences, accuscene-a11y"

    # Build all workspace members
    cargo build --workspace ${cargo_flags} -j ${PARALLEL_JOBS} 2>&1 | tee -a "${LOG_FILE}" || error_exit "Rust build failed"

    # Copy build artifacts
    log "Copying Rust build artifacts..."
    if [ "${BUILD_MODE}" = "release" ]; then
        cp -r target/release/*.so "${ARTIFACTS_DIR}/" 2>/dev/null || true
        cp -r target/release/*.dylib "${ARTIFACTS_DIR}/" 2>/dev/null || true
        cp -r target/release/*.dll "${ARTIFACTS_DIR}/" 2>/dev/null || true
        cp -r target/release/*.node "${ARTIFACTS_DIR}/" 2>/dev/null || true
    else
        cp -r target/debug/*.so "${ARTIFACTS_DIR}/" 2>/dev/null || true
        cp -r target/debug/*.dylib "${ARTIFACTS_DIR}/" 2>/dev/null || true
        cp -r target/debug/*.dll "${ARTIFACTS_DIR}/" 2>/dev/null || true
        cp -r target/debug/*.node "${ARTIFACTS_DIR}/" 2>/dev/null || true
    fi

    log_success "Rust core build complete"
}

################################################################################
# Run Rust Tests
################################################################################

run_rust_tests() {
    if [ "${SKIP_TESTS}" = "true" ]; then
        log_warning "Skipping Rust tests (SKIP_TESTS=true)"
        return
    fi

    log_section "Running Rust Tests"

    cd "${PROJECT_ROOT}/rust-core"

    log "Running cargo test..."
    cargo test --workspace --all-features 2>&1 | tee -a "${LOG_FILE}" || error_exit "Rust tests failed"

    log_success "Rust tests passed"
}

################################################################################
# TypeScript Compilation
################################################################################

build_typescript() {
    log_section "Building TypeScript"

    cd "${PROJECT_ROOT}"

    # Run TypeScript compiler
    log "Running TypeScript compilation..."
    npx tsc --noEmit 2>&1 | tee -a "${LOG_FILE}" || error_exit "TypeScript compilation failed"

    # Build renderer
    log "Building renderer..."
    npm run build:renderer 2>&1 | tee -a "${LOG_FILE}" || error_exit "Renderer build failed"

    # Build main
    log "Building main process..."
    npm run build:main 2>&1 | tee -a "${LOG_FILE}" || error_exit "Main process build failed"

    # Build API
    log "Building API..."
    npm run build:api 2>&1 | tee -a "${LOG_FILE}" || error_exit "API build failed"

    # Copy TypeScript artifacts
    log "Copying TypeScript build artifacts..."
    cp -r dist/* "${ARTIFACTS_DIR}/" 2>/dev/null || true

    log_success "TypeScript build complete"
}

################################################################################
# Run ESLint
################################################################################

run_eslint() {
    if [ "${SKIP_LINT}" = "true" ]; then
        log_warning "Skipping ESLint (SKIP_LINT=true)"
        return
    fi

    log_section "Running ESLint"

    cd "${PROJECT_ROOT}"

    log "Running ESLint..."
    npm run lint 2>&1 | tee -a "${LOG_FILE}" || error_exit "ESLint failed"

    log_success "ESLint passed"
}

################################################################################
# Run TypeScript/JavaScript Tests
################################################################################

run_js_tests() {
    if [ "${SKIP_TESTS}" = "true" ]; then
        log_warning "Skipping JavaScript tests (SKIP_TESTS=true)"
        return
    fi

    log_section "Running JavaScript/TypeScript Tests"

    cd "${PROJECT_ROOT}"

    log "Running Jest tests..."
    npm run test 2>&1 | tee -a "${LOG_FILE}" || error_exit "Jest tests failed"

    log_success "JavaScript tests passed"
}

################################################################################
# Create Build Manifest
################################################################################

create_build_manifest() {
    log_section "Creating Build Manifest"

    local manifest_file="${ARTIFACTS_DIR}/build-manifest.json"

    cat > "${manifest_file}" <<EOF
{
  "version": "0.2.5",
  "build_timestamp": "${BUILD_TIMESTAMP}",
  "build_date": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "build_mode": "${BUILD_MODE}",
  "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')",
  "rust_version": "$(rustc --version)",
  "node_version": "$(node --version)",
  "components": {
    "rust_crates": 25,
    "new_crates_v0.2.5": [
      "accuscene-dashboard",
      "accuscene-notifications",
      "accuscene-visualization",
      "accuscene-gestures",
      "accuscene-offline",
      "accuscene-sso",
      "accuscene-search",
      "accuscene-transfer",
      "accuscene-preferences",
      "accuscene-a11y"
    ],
    "typescript": true,
    "tests_passed": $([ "${SKIP_TESTS}" = "true" ] && echo "false" || echo "true"),
    "lint_passed": $([ "${SKIP_LINT}" = "true" ] && echo "false" || echo "true")
  },
  "build_artifacts": {
    "rust_binaries": "target/${BUILD_MODE}/",
    "typescript_output": "dist/",
    "artifacts_dir": "${ARTIFACTS_DIR}"
  }
}
EOF

    log "Build manifest created at: ${manifest_file}"
    log_success "Build manifest complete"
}

################################################################################
# Build Summary
################################################################################

print_build_summary() {
    log_section "Build Summary"

    log_success "AccuScene Enterprise v0.2.5 Build Complete!"
    log ""
    log "Build Details:"
    log "  Build Mode: ${BUILD_MODE}"
    log "  Build Time: ${BUILD_TIMESTAMP}"
    log "  Log File: ${LOG_FILE}"
    log "  Artifacts: ${ARTIFACTS_DIR}"
    log ""
    log "Components Built:"
    log "  ✓ Rust Core (25 crates, 10 new in v0.2.5)"
    log "  ✓ TypeScript/JavaScript"
    log "  ✓ Electron Main Process"
    log "  ✓ Renderer Process"
    log "  ✓ API Server"
    log ""
    if [ "${SKIP_TESTS}" = "false" ]; then
        log "  ✓ All Tests Passed"
    fi
    if [ "${SKIP_LINT}" = "false" ]; then
        log "  ✓ Linting Passed"
    fi
    log ""
    log "Next Steps:"
    log "  1. Run verification: ./scripts/verify-build.sh"
    log "  2. Package application: npm run package"
    log "  3. Deploy artifacts from: ${ARTIFACTS_DIR}"
    log ""
}

################################################################################
# Main Build Process
################################################################################

main() {
    log_section "AccuScene Enterprise v0.2.5 - Build Process Starting"

    check_environment
    setup_build

    # Clean if requested
    if [ "${CLEAN_BUILD:-false}" = "true" ]; then
        clean_previous_builds
    fi

    install_dependencies
    build_rust_core
    run_rust_tests
    build_typescript
    run_eslint
    run_js_tests
    create_build_manifest
    print_build_summary

    log_success "Build process completed successfully!"
    exit 0
}

# Run main function
main "$@"
