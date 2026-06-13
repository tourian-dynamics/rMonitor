#!/bin/bash
# test.sh: Run code checks and unit/integration tests for pulse.
set -e

# Navigate to project root relative to this script
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Running Cargo Check ==="
cargo check

echo "=== Running Cargo Clippy ==="
cargo clippy

echo "=== Running Cargo Test ==="
cargo test

echo "=== All checks and tests passed successfully! ==="
