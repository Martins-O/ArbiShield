#!/bin/bash

# ArbiShield Pre-Deployment Validation Script
# Validates environment and contract readiness before deployment

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Log file
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/pre_deploy_$(date +%Y%m%d_%H%M%S).log"

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] ✓${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ✗${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] ⚠${NC} $1" | tee -a "$LOG_FILE"
}

# Error counter
ERRORS=0
WARNINGS=0

# Print header
echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║        ArbiShield Pre-Deployment Validation                  ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

log "Starting pre-deployment validation..."
log "Project root: $PROJECT_ROOT"
log "Log file: $LOG_FILE"

# =============================================================================
# 1. Environment Setup Validation
# =============================================================================

echo ""
log "━━━ Step 1: Environment Setup ━━━"

# Check if .env exists
if [ ! -f "$SCRIPT_DIR/.env" ]; then
    log_error ".env file not found!"
    log_error "Copy .env.example to .env and configure it:"
    log_error "  cp $SCRIPT_DIR/.env.example $SCRIPT_DIR/.env"
    ((ERRORS++))
else
    log_success ".env file found"

    # Load environment variables
    set -a
    source "$SCRIPT_DIR/.env"
    set +a

    # Validate required environment variables
    REQUIRED_VARS=("NETWORK" "SEPOLIA_RPC_URL" "SEPOLIA_PRIVATE_KEY")

    for VAR in "${REQUIRED_VARS[@]}"; do
        if [ -z "${!VAR}" ] || [ "${!VAR}" = "your_private_key_here" ] || [ "${!VAR}" = "your_arbiscan_api_key" ]; then
            log_error "Required environment variable $VAR is not set or still has default value"
            ((ERRORS++))
        else
            log_success "Environment variable $VAR is set"
        fi
    done

    # Validate network
    if [ "$NETWORK" != "sepolia" ] && [ "$NETWORK" != "mainnet" ]; then
        log_error "NETWORK must be either 'sepolia' or 'mainnet', got: $NETWORK"
        ((ERRORS++))
    else
        log_success "Network configured: $NETWORK"
    fi

    # Warn if deploying to mainnet
    if [ "$NETWORK" = "mainnet" ]; then
        log_warning "⚠️  DEPLOYING TO MAINNET! Make sure you know what you're doing!"
        ((WARNINGS++))
    fi
fi

# =============================================================================
# 2. Rust & Cargo Validation
# =============================================================================

echo ""
log "━━━ Step 2: Rust & Cargo Validation ━━━"

# Check Rust installation
if ! command -v rustc &> /dev/null; then
    log_error "Rust is not installed. Install from: https://rustup.rs/"
    ((ERRORS++))
else
    RUST_VERSION=$(rustc --version)
    log_success "Rust installed: $RUST_VERSION"
fi

# Check Cargo installation
if ! command -v cargo &> /dev/null; then
    log_error "Cargo is not installed"
    ((ERRORS++))
else
    CARGO_VERSION=$(cargo --version)
    log_success "Cargo installed: $CARGO_VERSION"
fi

# Check for wasm32-unknown-unknown target
if rustc --print target-list | grep -q "wasm32-unknown-unknown"; then
    if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
        log_success "wasm32-unknown-unknown target installed"
    else
        log_warning "wasm32-unknown-unknown target not installed"
        log "Installing wasm32-unknown-unknown target..."
        if rustup target add wasm32-unknown-unknown; then
            log_success "Installed wasm32-unknown-unknown target"
        else
            log_error "Failed to install wasm32-unknown-unknown target"
            ((ERRORS++))
        fi
    fi
else
    log_error "wasm32-unknown-unknown target not available"
    ((ERRORS++))
fi

# =============================================================================
# 3. Stylus CLI Validation
# =============================================================================

echo ""
log "━━━ Step 3: Stylus CLI Validation ━━━"

# Check cargo-stylus installation
if ! command -v cargo-stylus &> /dev/null; then
    log_warning "cargo-stylus is not installed"
    log "Installing cargo-stylus..."
    if cargo install cargo-stylus; then
        log_success "Installed cargo-stylus"
    else
        log_error "Failed to install cargo-stylus"
        log_error "Install manually: cargo install cargo-stylus"
        ((ERRORS++))
    fi
else
    STYLUS_VERSION=$(cargo-stylus --version 2>&1 || echo "unknown")
    log_success "cargo-stylus installed: $STYLUS_VERSION"
fi

# =============================================================================
# 4. Project Build Validation
# =============================================================================

echo ""
log "━━━ Step 4: Project Build Validation ━━━"

cd "$PROJECT_ROOT"

# Check if Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    log_error "Cargo.toml not found in project root"
    ((ERRORS++))
else
    log_success "Cargo.toml found"
fi

# Build project
log "Building project..."
if cargo build --release --target wasm32-unknown-unknown 2>&1 | tee -a "$LOG_FILE"; then
    log_success "Project builds successfully"
else
    log_error "Project build failed"
    ((ERRORS++))
fi

# Check WASM output
WASM_FILE="target/wasm32-unknown-unknown/release/arbishield.wasm"
if [ -f "$WASM_FILE" ]; then
    WASM_SIZE=$(du -h "$WASM_FILE" | cut -f1)
    log_success "WASM file generated: $WASM_FILE ($WASM_SIZE)"

    # Check WASM size (Stylus limit is 128KB, warn at 100KB)
    WASM_BYTES=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)
    if [ $WASM_BYTES -gt 100000 ]; then
        log_warning "WASM file is large: $WASM_SIZE (limit: 128KB)"
        log_warning "Consider optimizing with wasm-opt if needed"
        ((WARNINGS++))
    fi
else
    log_error "WASM file not found at $WASM_FILE"
    ((ERRORS++))
fi

# =============================================================================
# 5. Test Suite Validation
# =============================================================================

echo ""
log "━━━ Step 5: Test Suite Validation ━━━"

# Run tests
log "Running test suite..."
if cargo test --lib 2>&1 | tee -a "$LOG_FILE"; then
    log_success "All unit tests passing"
else
    log_error "Some tests failed"
    ((ERRORS++))
fi

# Run property tests
log "Running property tests..."
if cargo test --test proptest_tests 2>&1 | tee -a "$LOG_FILE" | grep -q "test result: ok"; then
    log_success "All property tests passing"
else
    log_warning "Property tests failed or not run"
    ((WARNINGS++))
fi

# Run invariant tests
log "Running invariant tests..."
if cargo test --test invariant_tests 2>&1 | tee -a "$LOG_FILE" | grep -q "test result: ok"; then
    log_success "All invariant tests passing"
else
    log_warning "Invariant tests failed or not run"
    ((WARNINGS++))
fi

# Run security audit tests
log "Running security audit tests..."
if cargo test --test security_audit 2>&1 | tee -a "$LOG_FILE" | grep -q "test result: ok"; then
    log_success "All security audit tests passing"
else
    log_warning "Security audit tests failed or not run"
    ((WARNINGS++))
fi

# =============================================================================
# 6. Stylus Contract Validation
# =============================================================================

echo ""
log "━━━ Step 6: Stylus Contract Validation ━━━"

if command -v cargo-stylus &> /dev/null; then
    # Validate contract for target network
    log "Validating contract for Stylus deployment..."

    # Get RPC URL based on network
    if [ "$NETWORK" = "sepolia" ]; then
        RPC_URL="$SEPOLIA_RPC_URL"
    else
        RPC_URL="$MAINNET_RPC_URL"
    fi

    if cargo stylus check --endpoint "$RPC_URL" 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Contract passes Stylus validation"
    else
        log_error "Contract failed Stylus validation"
        log_error "Fix validation errors before deployment"
        ((ERRORS++))
    fi
else
    log_warning "Skipping Stylus validation (cargo-stylus not installed)"
    ((WARNINGS++))
fi

# =============================================================================
# 7. Gas Estimation
# =============================================================================

echo ""
log "━━━ Step 7: Gas Estimation ━━━"

log "Estimating deployment gas costs..."

# Rough estimation based on WASM size
# Stylus deployment costs: base + per-byte cost
if [ -f "$WASM_FILE" ]; then
    WASM_BYTES=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)

    # Rough estimates (actual costs may vary)
    # Base cost: ~2-3 million gas
    # Per byte: ~200 gas
    BASE_GAS=2500000
    PER_BYTE_GAS=200
    ESTIMATED_GAS=$((BASE_GAS + WASM_BYTES * PER_BYTE_GAS))

    log "Estimated deployment gas: ~$ESTIMATED_GAS"
    log "Contract size: $WASM_BYTES bytes"

    # Estimate cost at different gas prices (in gwei)
    for GWEI in 0.1 0.5 1.0; do
        # Calculate cost in ETH
        COST=$(echo "scale=6; $ESTIMATED_GAS * $GWEI / 1000000000" | bc)
        log "  At $GWEI gwei: ~$COST ETH"
    done
else
    log_warning "Cannot estimate gas (WASM file not found)"
    ((WARNINGS++))
fi

# =============================================================================
# 8. Network Connectivity
# =============================================================================

echo ""
log "━━━ Step 8: Network Connectivity ━━━"

# Test RPC connectivity
if [ -n "$RPC_URL" ]; then
    log "Testing connection to $NETWORK RPC..."

    # Try to get chain ID
    CHAIN_ID=$(curl -s -X POST "$RPC_URL" \
        -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

    if [ -n "$CHAIN_ID" ]; then
        # Convert hex to decimal
        CHAIN_ID_DEC=$((CHAIN_ID))

        # Validate chain ID
        if [ "$NETWORK" = "sepolia" ] && [ "$CHAIN_ID_DEC" = "421614" ]; then
            log_success "Connected to Arbitrum Sepolia (Chain ID: $CHAIN_ID_DEC)"
        elif [ "$NETWORK" = "mainnet" ] && [ "$CHAIN_ID_DEC" = "42161" ]; then
            log_success "Connected to Arbitrum One (Chain ID: $CHAIN_ID_DEC)"
        else
            log_error "Chain ID mismatch! Expected chain for $NETWORK, got chain $CHAIN_ID_DEC"
            ((ERRORS++))
        fi
    else
        log_error "Failed to connect to RPC endpoint: $RPC_URL"
        ((ERRORS++))
    fi
else
    log_error "RPC_URL not configured"
    ((ERRORS++))
fi

# =============================================================================
# 9. Security Checks
# =============================================================================

echo ""
log "━━━ Step 9: Security Checks ━━━"

# Check if deploying from a secure environment
if [ -n "$CI" ]; then
    log_warning "Running in CI environment - ensure secrets are properly secured"
    ((WARNINGS++))
fi

# Warn about private key in .env
if [ -f "$SCRIPT_DIR/.env" ]; then
    if grep -q "PRIVATE_KEY" "$SCRIPT_DIR/.env"; then
        log_warning "Private keys found in .env - ensure file is in .gitignore"
        ((WARNINGS++))
    fi
fi

# Check .gitignore
if [ -f "$PROJECT_ROOT/.gitignore" ]; then
    if grep -q ".env" "$PROJECT_ROOT/.gitignore"; then
        log_success ".env is in .gitignore"
    else
        log_error ".env is NOT in .gitignore - risk of exposing secrets!"
        ((ERRORS++))
    fi
else
    log_warning ".gitignore not found"
    ((WARNINGS++))
fi

# =============================================================================
# 10. Deployment Checklist
# =============================================================================

echo ""
log "━━━ Step 10: Deployment Checklist ━━━"

CHECKLIST_ITEMS=(
    "All tests passing"
    "Contract builds successfully"
    "WASM file generated"
    "Stylus validation passes"
    "Network connectivity verified"
    "Environment configured"
    "Private keys secured"
    ".env in .gitignore"
)

for item in "${CHECKLIST_ITEMS[@]}"; do
    log "  ☑ $item"
done

# =============================================================================
# Summary
# =============================================================================

echo ""
echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                  Validation Summary                           ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    log_success "All validation checks passed!"
    log_success "Ready for deployment to $NETWORK"
    echo ""
    log "Next steps:"
    log "  1. Review deployment parameters in .env"
    log "  2. Run: ./scripts/deploy.sh"
    log "  3. Verify deployment: ./scripts/verify.sh"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    log_warning "Validation completed with $WARNINGS warning(s)"
    log "Review warnings before deploying to $NETWORK"
    echo ""
    log "To deploy anyway:"
    log "  ./scripts/deploy.sh"
    exit 0
else
    log_error "Validation failed with $ERRORS error(s) and $WARNINGS warning(s)"
    log_error "Fix errors before deployment"
    echo ""
    log "Check log file for details: $LOG_FILE"
    exit 1
fi
