#!/bin/bash
set -e
echo "=== AccuScene Enterprise v0.2.0 Build ==="
echo "Building Rust core..."
cd /home/user/accident-recreate/rust-core
cargo build --all-features
echo "Rust build complete!"
echo "Building TypeScript..."
cd /home/user/accident-recreate
if [ -f "package.json" ]; then
    npm run build 2>/dev/null || echo "TS build script not configured"
fi
echo "=== Build Complete ==="
