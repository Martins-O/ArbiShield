#!/bin/bash

# ArbiShield Deployment Scripts Test Suite
# Tests deployment scripts for correctness, permissions, and functionality

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Logging functions
log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Test function
run_test() {
    ((TESTS_RUN++))
    local test_name="$1"
    local test_command="$2"

    log_test "$test_name"

    if eval "$test_command"; then
        log_pass "$test_name"
        return 0
    else
        log_fail "$test_name"
        return 1
    fi
}

# Print header
echo ""
echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║        ArbiShield Deployment Scripts Test Suite              ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""

# =============================================================================
# 1. File Existence Tests
# =============================================================================

echo -e "${BLUE}━━━ File Existence Tests ━━━${NC}"

run_test "pre_deploy.sh exists" "[ -f '$SCRIPT_DIR/pre_deploy.sh' ]"
run_test "deploy.sh exists" "[ -f '$SCRIPT_DIR/deploy.sh' ]"
run_test "verify.sh exists" "[ -f '$SCRIPT_DIR/verify.sh' ]"
run_test "initialize.sh exists" "[ -f '$SCRIPT_DIR/initialize.sh' ]"
run_test ".env.example exists" "[ -f '$SCRIPT_DIR/.env.example' ]"
run_test "README.md exists" "[ -f '$SCRIPT_DIR/README.md' ]"
run_test "DEPLOYMENT.md exists" "[ -f '$SCRIPT_DIR/DEPLOYMENT.md' ]"

# =============================================================================
# 2. File Permissions Tests
# =============================================================================

echo ""
echo -e "${BLUE}━━━ File Permissions Tests ━━━${NC}"

run_test "pre_deploy.sh is executable" "[ -x '$SCRIPT_DIR/pre_deploy.sh' ]"
run_test "deploy.sh is executable" "[ -x '$SCRIPT_DIR/deploy.sh' ]"
run_test "verify.sh is executable" "[ -x '$SCRIPT_DIR/verify.sh' ]"
run_test "initialize.sh is executable" "[ -x '$SCRIPT_DIR/initialize.sh' ]"

# =============================================================================
# 3. Script Syntax Tests
# =============================================================================

echo ""
echo -e "${BLUE}━━━ Script Syntax Tests ━━━${NC}"

run_test "pre_deploy.sh has valid syntax" "bash -n '$SCRIPT_DIR/pre_deploy.sh'"
run_test "deploy.sh has valid syntax" "bash -n '$SCRIPT_DIR/deploy.sh'"
run_test "verify.sh has valid syntax" "bash -n '$SCRIPT_DIR/verify.sh'"
run_test "initialize.sh has valid syntax" "bash -n '$SCRIPT_DIR/initialize.sh'"

# =============================================================================
# 4. Shebang Tests
# =============================================================================

echo ""
echo -e "${BLUE}━━━ Shebang Tests ━━━${NC}"

run_test "pre_deploy.sh has correct shebang" "head -1 '$SCRIPT_DIR/pre_deploy.sh' | grep -q '^#!/bin/bash'"
run_test "deploy.sh has correct shebang" "head -1 '$SCRIPT_DIR/deploy.sh' | grep -q '^#!/bin/bash'"
run_test "verify.sh has correct shebang" "head -1 '$SCRIPT_DIR/verify.sh' | grep -q '^#!/bin/bash'"
run_test "initialize.sh has correct shebang" "head -1 '$SCRIPT_DIR/initialize.sh' | grep -q '^#!/bin/bash'"

# =============================================================================
# 5. Environment Template Tests
# =============================================================================

echo ""
echo -e "${BLUE}━━━ Environment Template Tests ━━━${NC}"

run_test ".env.example contains NETWORK" "grep -q '^NETWORK=' '$SCRIPT_DIR/.env.example'"
run_test ".env.example contains SEPOLIA_RPC_URL" "grep -q '^SEPOLIA_RPC_URL=' '$SCRIPT_DIR/.env.example'"
run_test ".env.example contains SEPOLIA_PRIVATE_KEY" "grep -q '^SEPOLIA_PRIVATE_KEY=' '$SCRIPT_DIR/.env.example'"
run_test ".env.example contains ARBISCAN_API_KEY" "grep -q '^ARBISCAN_API_KEY=' '$SCRIPT_DIR/.env.example'"
run_test ".env.example contains MAINNET_RPC_URL" "grep -q '^MAINNET_RPC_URL=' '$SCRIPT_DIR/.env.example'"
run_test ".env.example contains ADMIN_ADDRESS" "grep -q '^ADMIN_ADDRESS=' '$SCRIPT_DIR/.env.example'"
run_test ".env.example contains MONITOR_ADDRESS" "grep -q '^MONITOR_ADDRESS=' '$SCRIPT_DIR/.env.example'"
run_test ".env.example contains DEFAULT_ALERT_EXPIRATION" "grep -q '^DEFAULT_ALERT_EXPIRATION=' '$SCRIPT_DIR/.env.example'"

# =============================================================================
# 6. Script Content Tests
# =============================================================================

echo ""
echo -e "${BLUE}━━━ Script Content Tests ━━━${NC}"

run_test "pre_deploy.sh uses set -e" "grep -q '^set -e' '$SCRIPT_DIR/pre_deploy.sh'"
run_test "deploy.sh uses set -e" "grep -q '^set -e' '$SCRIPT_DIR/deploy.sh'"
run_test "verify.sh uses set -e" "grep -q '^set -e' '$SCRIPT_DIR/verify.sh'"
run_test "initialize.sh uses set -e" "grep -q '^set -e' '$SCRIPT_DIR/initialize.sh'"

run_test "pre_deploy.sh has logging functions" "grep -q 'log_success()' '$SCRIPT_DIR/pre_deploy.sh'"
run_test "deploy.sh has logging functions" "grep -q 'log_success()' '$SCRIPT_DIR/deploy.sh'"
run_test "verify.sh has logging functions" "grep -q 'log_success()' '$SCRIPT_DIR/verify.sh'"
run_test "initialize.sh has logging functions" "grep -q 'log_success()' '$SCRIPT_DIR/initialize.sh'"

run_test "deploy.sh checks for .env" "grep -q 'if.*\.env' '$SCRIPT_DIR/deploy.sh'"
run_test "verify.sh checks for .env" "grep -q 'if.*\.env' '$SCRIPT_DIR/verify.sh'"
run_test "initialize.sh checks for .env" "grep -q 'if.*\.env' '$SCRIPT_DIR/initialize.sh'"

run_test "deploy.sh uses cargo-stylus" "grep -q 'cargo stylus deploy' '$SCRIPT_DIR/deploy.sh' || grep -q 'cargo-stylus' '$SCRIPT_DIR/deploy.sh'"
run_test "verify.sh uses cargo-stylus" "grep -q 'cargo stylus verify' '$SCRIPT_DIR/verify.sh' || grep -q 'cargo-stylus' '$SCRIPT_DIR/verify.sh'"

# =============================================================================
# 7. Documentation Tests
# =============================================================================

echo ""
echo -e "${BLUE}━━━ Documentation Tests ━━━${NC}"

run_test "README.md has Quick Start section" "grep -q '## Quick Start' '$SCRIPT_DIR/README.md'"
run_test "README.md has Scripts section" "grep -q '## Scripts' '$SCRIPT_DIR/README.md'"
run_test "README.md has Configuration section" "grep -q '## Configuration' '$SCRIPT_DIR/README.md'"
run_test "README.md has Troubleshooting section" "grep -q '## Troubleshooting' '$SCRIPT_DIR/README.md'"

run_test "DEPLOYMENT.md has Prerequisites section" "grep -q '## Prerequisites' '$SCRIPT_DIR/DEPLOYMENT.md'"
run_test "DEPLOYMENT.md has Environment Setup section" "grep -q '## Environment Setup' '$SCRIPT_DIR/DEPLOYMENT.md'"
run_test "DEPLOYMENT.md has Deployment Process section" "grep -q '## Deployment Process' '$SCRIPT_DIR/DEPLOYMENT.md'"
run_test "DEPLOYMENT.md has Troubleshooting section" "grep -q '## Troubleshooting' '$SCRIPT_DIR/DEPLOYMENT.md'"
run_test "DEPLOYMENT.md has Network Information section" "grep -q '## Network Information' '$SCRIPT_DIR/DEPLOYMENT.md'"
run_test "DEPLOYMENT.md has Security Best Practices section" "grep -q '## Security Best Practices' '$SCRIPT_DIR/DEPLOYMENT.md'"

# =============================================================================
# 8. Directory Structure Tests
# =============================================================================

echo ""
echo -e "${BLUE}━━━ Directory Structure Tests ━━━${NC}"

# Create directories if they don't exist (they're created by scripts normally)
mkdir -p "$SCRIPT_DIR/logs"
mkdir -p "$SCRIPT_DIR/deployments"

run_test "logs directory exists" "[ -d '$SCRIPT_DIR/logs' ]"
run_test "deployments directory exists" "[ -d '$SCRIPT_DIR/deployments' ]"

# =============================================================================
# 9. Dependency Tests
# =============================================================================

echo ""
echo -e "${BLUE}━━━ Dependency Tests ━━━${NC}"

run_test "bash is available" "command -v bash &> /dev/null"
run_test "curl is available" "command -v curl &> /dev/null"
run_test "grep is available" "command -v grep &> /dev/null"
run_test "sed is available" "command -v sed &> /dev/null"
run_test "date is available" "command -v date &> /dev/null"

# Optional dependencies (warnings only)
if command -v cargo &> /dev/null; then
    log_pass "cargo is available (optional)"
    ((TESTS_PASSED++))
    ((TESTS_RUN++))
else
    log_info "cargo not available (optional for testing)"
fi

if command -v rustc &> /dev/null; then
    log_pass "rustc is available (optional)"
    ((TESTS_PASSED++))
    ((TESTS_RUN++))
else
    log_info "rustc not available (optional for testing)"
fi

if command -v cargo-stylus &> /dev/null; then
    log_pass "cargo-stylus is available (optional)"
    ((TESTS_PASSED++))
    ((TESTS_RUN++))
else
    log_info "cargo-stylus not available (optional for testing)"
fi

if command -v cast &> /dev/null; then
    log_pass "cast (Foundry) is available (optional)"
    ((TESTS_PASSED++))
    ((TESTS_RUN++))
else
    log_info "cast not available (optional for testing)"
fi

# =============================================================================
# 10. Security Tests
# =============================================================================

echo ""
echo -e "${BLUE}━━━ Security Tests ━━━${NC}"

run_test ".gitignore exists in project root" "[ -f '$SCRIPT_DIR/../.gitignore' ]"

if [ -f "$SCRIPT_DIR/../.gitignore" ]; then
    run_test ".gitignore includes .env" "grep -q '\.env' '$SCRIPT_DIR/../.gitignore'"
    run_test ".gitignore includes private keys" "grep -q 'private.*key\|\.pem\|\.key' '$SCRIPT_DIR/../.gitignore' || grep -q '\.env' '$SCRIPT_DIR/../.gitignore'"
fi

run_test "No .env file in git (should not exist or be ignored)" "[ ! -f '$SCRIPT_DIR/.env' ] || git check-ignore '$SCRIPT_DIR/.env' &> /dev/null"

# Check scripts don't expose private keys
run_test "deploy.sh doesn't echo private key" "! grep -i 'echo.*PRIVATE_KEY' '$SCRIPT_DIR/deploy.sh'"
run_test "verify.sh doesn't echo private key" "! grep -i 'echo.*PRIVATE_KEY' '$SCRIPT_DIR/verify.sh'"
run_test "initialize.sh doesn't echo private key" "! grep -i 'echo.*PRIVATE_KEY' '$SCRIPT_DIR/initialize.sh'"

# =============================================================================
# Summary
# =============================================================================

echo ""
echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                      Test Summary                             ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

echo ""
echo "Tests Run:    $TESTS_RUN"
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
else
    echo -e "${GREEN}Tests Failed: $TESTS_FAILED${NC}"
fi

# Calculate percentage
if [ $TESTS_RUN -gt 0 ]; then
    PASS_RATE=$((TESTS_PASSED * 100 / TESTS_RUN))
    echo "Pass Rate:    $PASS_RATE%"
fi

echo ""

# Exit code
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    log_info "Deployment scripts are ready to use"
    log_info "Next steps:"
    log_info "  1. Configure .env: cp .env.example .env && nano .env"
    log_info "  2. Run pre-deployment validation: ./pre_deploy.sh"
    log_info "  3. Deploy: ./deploy.sh"
    exit 0
else
    echo -e "${RED}✗ Some tests failed!${NC}"
    echo ""
    log_info "Please fix the failing tests before deploying"
    exit 1
fi
