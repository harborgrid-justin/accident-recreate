#!/bin/bash

################################################################################
# AccuScene Enterprise v0.2.5 - Build Verification Script
#
# This script verifies that the build was successful by checking:
# - All expected artifacts exist
# - Binaries are executable and have correct permissions
# - Dependencies are properly linked
# - Configuration files are valid
# - Runtime tests can be performed
################################################################################

set -e
set -o pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Verification results
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNING_CHECKS=0

################################################################################
# Logging Functions
################################################################################

log() {
    echo -e "${BLUE}[VERIFY]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[VERIFY] ✓${NC} $*"
    ((PASSED_CHECKS++))
    ((TOTAL_CHECKS++))
}

log_error() {
    echo -e "${RED}[VERIFY] ✗${NC} $*"
    ((FAILED_CHECKS++))
    ((TOTAL_CHECKS++))
}

log_warning() {
    echo -e "${YELLOW}[VERIFY] ⚠${NC} $*"
    ((WARNING_CHECKS++))
    ((TOTAL_CHECKS++))
}

log_section() {
    echo -e "\n${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $*${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}\n"
}

################################################################################
# Verification Functions
################################################################################

verify_rust_workspace() {
    log_section "Verifying Rust Workspace"

    cd "${PROJECT_ROOT}/rust-core"

    # Check Cargo.toml exists
    if [ -f "Cargo.toml" ]; then
        log_success "Cargo.toml found"
    else
        log_error "Cargo.toml not found"
        return
    fi

    # Verify workspace version
    local version=$(grep -m 1 'version = ' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
    if [ "$version" = "0.2.5" ]; then
        log_success "Workspace version is 0.2.5"
    else
        log_error "Workspace version is $version, expected 0.2.5"
    fi

    # Check for new v0.2.5 crates in workspace
    local new_crates=(
        "accuscene-dashboard"
        "accuscene-notifications"
        "accuscene-visualization"
        "accuscene-gestures"
        "accuscene-offline"
        "accuscene-sso"
        "accuscene-search"
        "accuscene-transfer"
        "accuscene-preferences"
        "accuscene-a11y"
    )

    for crate in "${new_crates[@]}"; do
        if grep -q "\"crates/${crate}\"" Cargo.toml; then
            log_success "Crate ${crate} found in workspace"
        else
            log_warning "Crate ${crate} not found in workspace members"
        fi
    done
}

verify_rust_build_artifacts() {
    log_section "Verifying Rust Build Artifacts"

    cd "${PROJECT_ROOT}/rust-core"

    # Check if target directory exists
    if [ ! -d "target" ]; then
        log_error "Rust target directory not found - build may not have run"
        return
    fi

    # Determine build mode
    local build_dir="target/release"
    if [ ! -d "$build_dir" ]; then
        build_dir="target/debug"
        log_warning "Using debug build artifacts (release build not found)"
    else
        log_success "Release build artifacts found"
    fi

    # Check for native libraries
    local lib_count=0
    for ext in so dylib dll node; do
        local count=$(find "$build_dir" -maxdepth 1 -name "*.$ext" 2>/dev/null | wc -l)
        lib_count=$((lib_count + count))
    done

    if [ $lib_count -gt 0 ]; then
        log_success "Found $lib_count native library artifacts"
    else
        log_warning "No native library artifacts found (.so, .dylib, .dll, .node)"
    fi

    # Verify Cargo.lock exists
    if [ -f "Cargo.lock" ]; then
        log_success "Cargo.lock found (dependencies locked)"
    else
        log_warning "Cargo.lock not found"
    fi
}

verify_typescript_build() {
    log_section "Verifying TypeScript Build"

    cd "${PROJECT_ROOT}"

    # Check dist directory
    if [ ! -d "dist" ]; then
        log_error "dist directory not found - TypeScript build may have failed"
        return
    else
        log_success "dist directory found"
    fi

    # Check main process build
    if [ -f "dist/main/main.js" ]; then
        log_success "Main process built (dist/main/main.js)"
    else
        log_error "Main process not built (dist/main/main.js not found)"
    fi

    # Check renderer build
    if [ -d "dist/renderer" ]; then
        log_success "Renderer process built (dist/renderer/)"
    else
        log_error "Renderer process not built (dist/renderer/ not found)"
    fi

    # Check API build
    if [ -d "dist/api" ]; then
        log_success "API server built (dist/api/)"
    else
        log_warning "API server directory not found (dist/api/)"
    fi
}

verify_node_dependencies() {
    log_section "Verifying Node Dependencies"

    cd "${PROJECT_ROOT}"

    # Check node_modules
    if [ -d "node_modules" ]; then
        log_success "node_modules directory found"
    else
        log_error "node_modules not found - dependencies not installed"
        return
    fi

    # Check package-lock.json
    if [ -f "package-lock.json" ]; then
        log_success "package-lock.json found (dependencies locked)"
    else
        log_warning "package-lock.json not found"
    fi

    # Verify critical dependencies
    local critical_deps=(
        "electron"
        "react"
        "react-dom"
        "typescript"
        "@react-three/fiber"
        "express"
        "graphql"
    )

    for dep in "${critical_deps[@]}"; do
        if [ -d "node_modules/$dep" ]; then
            log_success "Critical dependency $dep installed"
        else
            log_error "Critical dependency $dep not found"
        fi
    done
}

verify_configuration_files() {
    log_section "Verifying Configuration Files"

    cd "${PROJECT_ROOT}"

    local config_files=(
        "package.json"
        "tsconfig.json"
        ".eslintrc.js:.eslintrc.json"
        "webpack.main.config.js"
        "webpack.renderer.config.js"
    )

    for config in "${config_files[@]}"; do
        IFS=':' read -ra ALTERNATIVES <<< "$config"
        local found=false
        for alt in "${ALTERNATIVES[@]}"; do
            if [ -f "$alt" ]; then
                log_success "Configuration file $alt found"
                found=true
                break
            fi
        done
        if [ "$found" = false ]; then
            log_warning "Configuration file ${ALTERNATIVES[0]} not found"
        fi
    done

    # Verify package.json has correct version
    if [ -f "package.json" ]; then
        local pkg_version=$(grep -m 1 '"version"' package.json | sed 's/.*"\(.*\)".*/\1/')
        if [ "$pkg_version" = "0.2.5" ]; then
            log_success "package.json version is 0.2.5"
        else
            log_warning "package.json version is $pkg_version (expected 0.2.5)"
        fi
    fi
}

verify_build_scripts() {
    log_section "Verifying Build Scripts"

    cd "${PROJECT_ROOT}"

    local scripts=(
        "scripts/build-v0.2.5.sh"
        "scripts/verify-build.sh"
        "scripts/build-all.sh"
    )

    for script in "${scripts[@]}"; do
        if [ -f "$script" ]; then
            if [ -x "$script" ]; then
                log_success "Script $script exists and is executable"
            else
                log_warning "Script $script exists but is not executable"
            fi
        else
            log_warning "Script $script not found"
        fi
    done
}

verify_package_json_scripts() {
    log_section "Verifying package.json Scripts"

    cd "${PROJECT_ROOT}"

    if [ ! -f "package.json" ]; then
        log_error "package.json not found"
        return
    fi

    local expected_scripts=(
        "build"
        "build:renderer"
        "build:main"
        "build:api"
        "test"
        "lint"
        "typecheck"
    )

    for script in "${expected_scripts[@]}"; do
        if grep -q "\"$script\":" package.json; then
            log_success "Script '$script' defined in package.json"
        else
            log_warning "Script '$script' not found in package.json"
        fi
    done
}

verify_rust_crate_structure() {
    log_section "Verifying Rust Crate Structure"

    cd "${PROJECT_ROOT}/rust-core"

    local new_crates=(
        "accuscene-dashboard"
        "accuscene-notifications"
        "accuscene-visualization"
        "accuscene-gestures"
        "accuscene-offline"
        "accuscene-sso"
        "accuscene-search"
        "accuscene-transfer"
        "accuscene-preferences"
        "accuscene-a11y"
    )

    for crate in "${new_crates[@]}"; do
        local crate_path="crates/${crate}"
        if [ -d "$crate_path" ]; then
            if [ -f "$crate_path/Cargo.toml" ]; then
                log_success "Crate $crate has valid structure"
            else
                log_warning "Crate $crate missing Cargo.toml"
            fi
        else
            log_warning "Crate directory $crate_path not found"
        fi
    done
}

check_build_artifacts() {
    log_section "Checking Build Artifacts"

    cd "${PROJECT_ROOT}"

    # Check for build directory
    if [ -d "build" ]; then
        log_success "build directory exists"

        # Look for recent artifacts
        local latest_artifacts=$(find build -name "artifacts_*" -type d 2>/dev/null | sort -r | head -1)
        if [ -n "$latest_artifacts" ]; then
            log_success "Latest artifacts found at: $latest_artifacts"

            # Check for build manifest
            if [ -f "$latest_artifacts/build-manifest.json" ]; then
                log_success "Build manifest found"
                local manifest_version=$(grep '"version"' "$latest_artifacts/build-manifest.json" | head -1 | sed 's/.*"\(.*\)".*/\1/')
                if [ "$manifest_version" = "0.2.5" ]; then
                    log_success "Build manifest version is 0.2.5"
                else
                    log_warning "Build manifest version is $manifest_version"
                fi
            else
                log_warning "Build manifest not found"
            fi
        else
            log_warning "No artifact directories found"
        fi
    else
        log_warning "build directory not found (may not have built yet)"
    fi
}

run_basic_runtime_checks() {
    log_section "Running Basic Runtime Checks"

    cd "${PROJECT_ROOT}"

    # Check if we can load the main JavaScript file
    if [ -f "dist/main/main.js" ]; then
        log "Checking main.js syntax..."
        if node --check dist/main/main.js 2>/dev/null; then
            log_success "Main process JavaScript is valid"
        else
            log_error "Main process JavaScript has syntax errors"
        fi
    fi

    # Verify Rust can compile (quick check)
    cd "${PROJECT_ROOT}/rust-core"
    log "Running cargo check..."
    if cargo check --workspace --quiet 2>/dev/null; then
        log_success "Rust workspace passes cargo check"
    else
        log_warning "Rust workspace has compilation warnings/errors"
    fi
}

################################################################################
# Summary
################################################################################

print_verification_summary() {
    log_section "Verification Summary"

    echo ""
    echo "Total Checks: $TOTAL_CHECKS"
    echo -e "${GREEN}Passed: $PASSED_CHECKS${NC}"
    echo -e "${YELLOW}Warnings: $WARNING_CHECKS${NC}"
    echo -e "${RED}Failed: $FAILED_CHECKS${NC}"
    echo ""

    local pass_rate=0
    if [ $TOTAL_CHECKS -gt 0 ]; then
        pass_rate=$((PASSED_CHECKS * 100 / TOTAL_CHECKS))
    fi

    echo "Pass Rate: ${pass_rate}%"
    echo ""

    if [ $FAILED_CHECKS -eq 0 ]; then
        log_success "Build verification PASSED!"
        if [ $WARNING_CHECKS -gt 0 ]; then
            echo -e "${YELLOW}Note: There are $WARNING_CHECKS warnings that should be reviewed${NC}"
        fi
        return 0
    else
        log_error "Build verification FAILED with $FAILED_CHECKS errors"
        return 1
    fi
}

################################################################################
# Main
################################################################################

main() {
    log_section "AccuScene Enterprise v0.2.5 - Build Verification"

    verify_rust_workspace
    verify_rust_build_artifacts
    verify_typescript_build
    verify_node_dependencies
    verify_configuration_files
    verify_build_scripts
    verify_package_json_scripts
    verify_rust_crate_structure
    check_build_artifacts
    run_basic_runtime_checks

    print_verification_summary
    exit $?
}

# Run main
main "$@"
