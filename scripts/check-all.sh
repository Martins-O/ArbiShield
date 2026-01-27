#!/bin/bash
# ArbiShield Validation Script
#
# This script runs all checks to ensure code quality and correctness.

set -e

echo "🛡️  ArbiShield Validation Script"
echo "================================"
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
FAILED=0

# Function to run a check and track status
run_check() {
    local name=$1
    local command=$2

    echo "🔍 Running: $name"
    if eval "$command"; then
        echo -e "${GREEN}✅ $name passed${NC}"
    else
        echo -e "${RED}❌ $name failed${NC}"
        FAILED=1
    fi
    echo
}

# Format check
run_check "Format check" "cargo fmt --check"

# Clippy lints
run_check "Clippy lints" "cargo clippy --all-targets --all-features -- -D warnings"

# Build check
run_check "Build check" "cargo build --release --target wasm32-unknown-unknown"

# Stylus validation
run_check "Stylus check" "cargo stylus check"

# Unit tests
run_check "Unit tests" "cargo test"

# Integration tests
run_check "Integration tests" "cargo test --test integration"

# Documentation build
run_check "Documentation" "cargo doc --no-deps"

echo "================================"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All checks passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some checks failed${NC}"
    exit 1
fi
