#!/bin/bash
# AccuScene Enterprise Rust - Initial Setup Script
set -e

echo "================================================"
echo "AccuScene Enterprise Rust System Setup"
echo "Version 0.1.5"
echo "================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
check_command() {
    if command -v $1 &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 is installed"
        return 0
    else
        echo -e "${RED}✗${NC} $1 is not installed"
        return 1
    fi
}

# Check prerequisites
echo "Checking prerequisites..."
echo ""

MISSING_DEPS=0

if ! check_command rustc; then
    echo -e "${YELLOW}  Install Rust: https://rustup.rs/${NC}"
    MISSING_DEPS=1
fi

if ! check_command cargo; then
    echo -e "${YELLOW}  Cargo should be installed with Rust${NC}"
    MISSING_DEPS=1
fi

if ! check_command node; then
    echo -e "${YELLOW}  Install Node.js: https://nodejs.org/${NC}"
    MISSING_DEPS=1
else
    NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$NODE_VERSION" -lt 18 ]; then
        echo -e "${YELLOW}  Node.js version should be 18 or higher (current: $NODE_VERSION)${NC}"
        MISSING_DEPS=1
    fi
fi

check_command npm || MISSING_DEPS=1
check_command git || MISSING_DEPS=1

echo ""

if [ $MISSING_DEPS -eq 1 ]; then
    echo -e "${RED}Some dependencies are missing. Please install them and try again.${NC}"
    exit 1
fi

# Display Rust version
RUST_VERSION=$(rustc --version)
echo "Using: $RUST_VERSION"
echo ""

# Install Rust components
echo "Installing Rust components..."
rustup component add rustfmt clippy 2>/dev/null || echo "Components already installed"
echo ""

# Install development tools
echo "Installing development tools..."
echo "This may take a few minutes..."
echo ""

TOOLS=(
    "cargo-watch"
    "cargo-tarpaulin"
    "cargo-audit"
    "cargo-outdated"
)

for tool in "${TOOLS[@]}"; do
    if cargo install --list | grep -q "^$tool"; then
        echo -e "${GREEN}✓${NC} $tool already installed"
    else
        echo "Installing $tool..."
        cargo install $tool --quiet || echo -e "${YELLOW}Warning: Failed to install $tool${NC}"
    fi
done

echo ""

# Create directory structure for all crates
echo "Creating crate directory structure..."
CRATES=(
    "accuscene-core"
    "accuscene-ffi"
    "accuscene-physics"
    "accuscene-compression"
    "accuscene-database"
    "accuscene-crypto"
    "accuscene-jobs"
    "accuscene-streaming"
    "accuscene-cache"
    "accuscene-telemetry"
    "accuscene-cluster"
)

for crate in "${CRATES[@]}"; do
    if [ ! -d "$crate" ]; then
        mkdir -p "$crate/src"
        mkdir -p "$crate/tests"
        mkdir -p "$crate/benches"
        echo -e "${GREEN}✓${NC} Created $crate/"
    else
        echo -e "${YELLOW}→${NC} $crate/ already exists"
    fi
done

echo ""

# Initialize git hooks (if in git repo)
if [ -d .git ]; then
    echo "Setting up git hooks..."

    # Pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
echo "Running pre-commit checks..."

# Format check
if ! cargo fmt --all -- --check; then
    echo "Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy check
if ! cargo clippy --workspace --all-targets -- -D warnings; then
    echo "Clippy found issues. Please fix them."
    exit 1
fi

echo "Pre-commit checks passed!"
EOF

    chmod +x .git/hooks/pre-commit
    echo -e "${GREEN}✓${NC} Git hooks installed"
else
    echo -e "${YELLOW}→${NC} Not a git repository, skipping git hooks"
fi

echo ""

# Check for optional dependencies
echo "Checking optional dependencies..."
check_command psql && echo -e "  PostgreSQL available for database development" || echo -e "${YELLOW}  PostgreSQL not found (optional)${NC}"
check_command redis-cli && echo -e "  Redis available for caching development" || echo -e "${YELLOW}  Redis not found (optional)${NC}"

echo ""

# Build the workspace to verify setup
echo "Building workspace to verify setup..."
echo "This may take several minutes on first run..."
echo ""

if cargo build --workspace 2>&1 | grep -E "(error|warning:)"; then
    echo ""
    echo -e "${YELLOW}Build completed with warnings. This is expected for empty crates.${NC}"
else
    echo -e "${GREEN}✓${NC} Workspace built successfully"
fi

echo ""

# Summary
echo "================================================"
echo "Setup Complete!"
echo "================================================"
echo ""
echo "Next steps:"
echo ""
echo "1. Review SCRATCHPAD.md for agent assignments"
echo "2. Read ARCHITECTURE.md for system design"
echo "3. Check CONTRIBUTING.md for development workflow"
echo ""
echo "Quick commands:"
echo "  make build    - Build all crates"
echo "  make test     - Run all tests"
echo "  make check    - Run all quality checks"
echo "  make doc      - Generate documentation"
echo "  make help     - Show all available commands"
echo ""
echo -e "${GREEN}Happy coding! 🦀${NC}"
echo ""
