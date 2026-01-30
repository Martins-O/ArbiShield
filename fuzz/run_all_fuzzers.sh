#!/bin/bash

# ArbiShield Fuzzing Test Runner
# Runs all fuzzers sequentially with configurable time limits

set -e

# Configuration
FUZZ_TIME=${FUZZ_TIME:-60}  # Default: 60 seconds per fuzzer
FUZZ_RUNS=${FUZZ_RUNS:-100000}  # Default: 100k iterations per fuzzer

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Fuzzers to run
FUZZERS=(
    "alert_registry_priority"
    "alert_registry_errors"
    "circuit_breaker_state"
    "detection_engine_anomaly"
    "cross_contract_errors"
    "rbac_fuzzer"
    "arithmetic_fuzzer"
)

# Check if cargo-fuzz is installed
if ! command -v cargo-fuzz &> /dev/null; then
    echo -e "${RED}Error: cargo-fuzz not found${NC}"
    echo "Install with: cargo install cargo-fuzz"
    echo "Note: cargo-fuzz requires nightly Rust"
    exit 1
fi

# Check if using nightly
if ! rustc --version | grep -q "nightly"; then
    echo -e "${YELLOW}Warning: Not using nightly Rust${NC}"
    echo "cargo-fuzz requires nightly. Switch with: rustup default nightly"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}ArbiShield Fuzzing Test Suite${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Configuration:"
echo "  Time per fuzzer: ${FUZZ_TIME}s"
echo "  Runs per fuzzer: ${FUZZ_RUNS}"
echo "  Total fuzzers: ${#FUZZERS[@]}"
echo ""

# Track results
PASSED=0
FAILED=0
FAILED_FUZZERS=()

# Run each fuzzer
for fuzzer in "${FUZZERS[@]}"; do
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Running: ${fuzzer}${NC}"
    echo -e "${BLUE}========================================${NC}"

    START_TIME=$(date +%s)

    if cargo fuzz run "${fuzzer}" -- -max_total_time="${FUZZ_TIME}" -runs="${FUZZ_RUNS}" 2>&1 | tee "/tmp/fuzz_${fuzzer}.log"; then
        END_TIME=$(date +%s)
        DURATION=$((END_TIME - START_TIME))

        echo -e "${GREEN}✓ ${fuzzer} PASSED (${DURATION}s)${NC}"
        PASSED=$((PASSED + 1))
    else
        END_TIME=$(date +%s)
        DURATION=$((END_TIME - START_TIME))

        echo -e "${RED}✗ ${fuzzer} FAILED (${DURATION}s)${NC}"
        FAILED=$((FAILED + 1))
        FAILED_FUZZERS+=("${fuzzer}")

        # Check for crash artifacts
        if [ -d "artifacts/${fuzzer}" ]; then
            echo -e "${YELLOW}Crash artifacts found in artifacts/${fuzzer}/${NC}"
            ls -lh "artifacts/${fuzzer}/" | tail -5
        fi
    fi

    echo ""
done

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Fuzzing Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Total fuzzers: ${#FUZZERS[@]}"
echo -e "${GREEN}Passed: ${PASSED}${NC}"
if [ ${FAILED} -gt 0 ]; then
    echo -e "${RED}Failed: ${FAILED}${NC}"
    echo ""
    echo "Failed fuzzers:"
    for fuzzer in "${FAILED_FUZZERS[@]}"; do
        echo -e "  ${RED}✗${NC} ${fuzzer}"
    done
fi

echo ""

# Exit with appropriate code
if [ ${FAILED} -gt 0 ]; then
    echo -e "${RED}Fuzzing FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}All fuzzers PASSED${NC}"
    exit 0
fi
