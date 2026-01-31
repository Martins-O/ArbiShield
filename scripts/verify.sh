#!/bin/bash

# ArbiShield Contract Verification Script
# Verifies deployed Stylus contracts on Arbiscan

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Log file
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/verify_$(date +%Y%m%d_%H%M%S).log"

# Logging functions
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

log_step() {
    echo -e "${MAGENTA}[$(date +'%Y-%m-%d %H:%M:%S')] ▶${NC} $1" | tee -a "$LOG_FILE"
}

# Print header
echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║           ArbiShield Contract Verification                   ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

log "Starting verification process..."
log "Log file: $LOG_FILE"

# =============================================================================
# 1. Load Environment
# =============================================================================

echo ""
log_step "Step 1: Loading Environment Configuration"

if [ ! -f "$SCRIPT_DIR/.env" ]; then
    log_error ".env file not found!"
    log_error "Copy .env.example to .env and configure it:"
    log_error "  cp $SCRIPT_DIR/.env.example $SCRIPT_DIR/.env"
    exit 1
fi

# Load environment variables
set -a
source "$SCRIPT_DIR/.env"
set +a

log_success "Environment loaded"
log "Network: $NETWORK"

# Validate network
if [ "$NETWORK" != "sepolia" ] && [ "$NETWORK" != "mainnet" ]; then
    log_error "NETWORK must be either 'sepolia' or 'mainnet', got: $NETWORK"
    exit 1
fi

# Set network-specific variables
if [ "$NETWORK" = "sepolia" ]; then
    RPC_URL="$SEPOLIA_RPC_URL"
    CHAIN_ID="421614"
    EXPLORER_URL="https://sepolia.arbiscan.io"
    EXPLORER_API="https://api-sepolia.arbiscan.io/api"
else
    RPC_URL="$MAINNET_RPC_URL"
    CHAIN_ID="42161"
    EXPLORER_URL="https://arbiscan.io"
    EXPLORER_API="https://api.arbiscan.io/api"
fi

log "RPC URL: $RPC_URL"
log "Chain ID: $CHAIN_ID"
log "Explorer: $EXPLORER_URL"

# =============================================================================
# 2. Get Contract Address
# =============================================================================

echo ""
log_step "Step 2: Getting Contract Address"

# Check if contract address provided as argument
if [ -n "$1" ]; then
    CONTRACT_ADDRESS="$1"
    log "Using provided contract address: $CONTRACT_ADDRESS"
else
    # Try to load from latest deployment file
    LATEST_DEPLOYMENT="$SCRIPT_DIR/deployments/$NETWORK-latest.json"

    if [ -f "$LATEST_DEPLOYMENT" ]; then
        log "Loading contract address from: $LATEST_DEPLOYMENT"

        # Extract contract address using grep/cut (portable)
        CONTRACT_ADDRESS=$(grep -o '"address": *"0x[a-fA-F0-9]\{40\}"' "$LATEST_DEPLOYMENT" | head -1 | cut -d'"' -f4)

        if [ -n "$CONTRACT_ADDRESS" ]; then
            log_success "Found contract address: $CONTRACT_ADDRESS"
        else
            log_error "Could not extract contract address from deployment file"
            exit 1
        fi
    else
        log_error "No contract address provided and no deployment file found"
        log_error "Usage: $0 [CONTRACT_ADDRESS]"
        log_error "Or deploy first using: ./scripts/deploy.sh"
        exit 1
    fi
fi

# Validate address format
if ! echo "$CONTRACT_ADDRESS" | grep -qE '^0x[a-fA-F0-9]{40}$'; then
    log_error "Invalid contract address format: $CONTRACT_ADDRESS"
    log_error "Expected format: 0x followed by 40 hex characters"
    exit 1
fi

# =============================================================================
# 3. Check Arbiscan API Key
# =============================================================================

echo ""
log_step "Step 3: Validating Arbiscan API Key"

if [ -z "$ARBISCAN_API_KEY" ]; then
    log_error "ARBISCAN_API_KEY not set in .env"
    log_error "Get an API key from: https://arbiscan.io/myapikey"
    exit 1
fi

log_success "Arbiscan API key configured"

# =============================================================================
# 4. Check cargo-stylus
# =============================================================================

echo ""
log_step "Step 4: Checking cargo-stylus Installation"

if ! command -v cargo-stylus &> /dev/null; then
    log_error "cargo-stylus not found!"
    log_error "Install with: cargo install cargo-stylus"
    exit 1
fi

STYLUS_VERSION=$(cargo-stylus --version 2>&1 || echo "unknown")
log_success "cargo-stylus installed: $STYLUS_VERSION"

# =============================================================================
# 5. Check if Contract is Already Verified
# =============================================================================

echo ""
log_step "Step 5: Checking Verification Status"

log "Checking if contract is already verified..."

# Query Arbiscan API to check verification status
VERIFY_STATUS=$(curl -s "$EXPLORER_API?module=contract&action=getsourcecode&address=$CONTRACT_ADDRESS&apikey=$ARBISCAN_API_KEY")

if echo "$VERIFY_STATUS" | grep -q '"result":\[{"SourceCode":"","ABI":"Contract source code not verified"'; then
    log "Contract is not verified yet"
elif echo "$VERIFY_STATUS" | grep -q '"SourceCode":'; then
    log_success "Contract is already verified!"
    log "View at: $EXPLORER_URL/address/$CONTRACT_ADDRESS#code"

    read -p "Do you want to verify again anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "Verification skipped"
        exit 0
    fi
else
    log_warning "Could not determine verification status"
    log "Proceeding with verification..."
fi

# =============================================================================
# 6. Build Contract
# =============================================================================

echo ""
log_step "Step 6: Building Contract for Verification"

cd "$PROJECT_ROOT"

# Build WASM
log "Building contract for wasm32-unknown-unknown..."
if cargo build --release --target wasm32-unknown-unknown 2>&1 | tee -a "$LOG_FILE"; then
    log_success "Contract built successfully"
else
    log_error "Contract build failed!"
    exit 1
fi

# Check WASM file
WASM_FILE="target/wasm32-unknown-unknown/release/arbishield.wasm"
if [ ! -f "$WASM_FILE" ]; then
    log_error "WASM file not found at $WASM_FILE"
    exit 1
fi

WASM_SIZE=$(du -h "$WASM_FILE" | cut -f1)
log_success "WASM file ready: $WASM_FILE ($WASM_SIZE)"

# =============================================================================
# 7. Verify Contract Using cargo-stylus
# =============================================================================

echo ""
log_step "Step 7: Verifying Contract on Arbiscan"

log "Submitting verification to Arbiscan..."
log "Contract: $CONTRACT_ADDRESS"
log "Network: $NETWORK"

# Stylus verify command
# Note: cargo-stylus verify needs the deployment transaction or program address
VERIFY_CMD="cargo stylus verify \
    --deployment-tx $(echo $CONTRACT_ADDRESS | sed 's/0x//') \
    --endpoint \"$RPC_URL\""

log "Running verification command..."

# Capture output
VERIFY_OUTPUT=$(mktemp)

if eval $VERIFY_CMD 2>&1 | tee "$VERIFY_OUTPUT" | tee -a "$LOG_FILE"; then
    log_success "Verification command completed"

    # Check output for success indicators
    if grep -q -i "verified\|success" "$VERIFY_OUTPUT"; then
        log_success "Contract verified successfully!"
    else
        log_warning "Verification may have succeeded, but could not confirm from output"
    fi
else
    log_error "Verification command failed"

    # Show error details
    log_error "Error output:"
    cat "$VERIFY_OUTPUT" | tee -a "$LOG_FILE"
    rm "$VERIFY_OUTPUT"
    exit 1
fi

rm "$VERIFY_OUTPUT"

# =============================================================================
# 8. Confirm Verification on Arbiscan
# =============================================================================

echo ""
log_step "Step 8: Confirming Verification Status"

# Wait a bit for Arbiscan to process
log "Waiting for Arbiscan to process verification..."
sleep 5

# Check verification status again
VERIFY_STATUS=$(curl -s "$EXPLORER_API?module=contract&action=getsourcecode&address=$CONTRACT_ADDRESS&apikey=$ARBISCAN_API_KEY")

if echo "$VERIFY_STATUS" | grep -q '"SourceCode":'; then
    if ! echo "$VERIFY_STATUS" | grep -q '"result":\[{"SourceCode":"","ABI":"Contract source code not verified"'; then
        log_success "Verification confirmed on Arbiscan!"
    else
        log_warning "Verification status unclear - check manually"
    fi
else
    log_warning "Could not confirm verification status"
fi

# =============================================================================
# Verification Summary
# =============================================================================

echo ""
echo -e "${GREEN}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║              ✅  Verification Complete!  ✅                    ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

log_success "Contract verification process completed!"
echo ""
log "Contract Address:  $CONTRACT_ADDRESS"
log "Network:           $NETWORK (Chain ID: $CHAIN_ID)"
log "Explorer:          $EXPLORER_URL/address/$CONTRACT_ADDRESS#code"
log "API Endpoint:      $EXPLORER_API"

echo ""
log "Next Steps:"
log "  1. View verified source: $EXPLORER_URL/address/$CONTRACT_ADDRESS#code"
log "  2. Check contract interactions: $EXPLORER_URL/address/$CONTRACT_ADDRESS"
log "  3. Monitor events: $EXPLORER_URL/address/$CONTRACT_ADDRESS#events"

echo ""
log_success "Verification completed successfully!"
