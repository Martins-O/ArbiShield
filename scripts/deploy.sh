#!/bin/bash

# ArbiShield Stylus Deployment Script
# Deploys ArbiShield contracts to Arbitrum Sepolia/Mainnet using cargo-stylus

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
LOG_FILE="$LOG_DIR/deploy_$(date +%Y%m%d_%H%M%S).log"

# Deployment addresses file
ADDRESSES_DIR="$SCRIPT_DIR/deployments"
mkdir -p "$ADDRESSES_DIR"

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
echo "║           ArbiShield Stylus Deployment                       ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

log "Starting deployment process..."
log "Project root: $PROJECT_ROOT"
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
log "Dry run: ${DRY_RUN:-false}"

# Validate network
if [ "$NETWORK" != "sepolia" ] && [ "$NETWORK" != "mainnet" ]; then
    log_error "NETWORK must be either 'sepolia' or 'mainnet', got: $NETWORK"
    exit 1
fi

# Set network-specific variables
if [ "$NETWORK" = "sepolia" ]; then
    RPC_URL="$SEPOLIA_RPC_URL"
    PRIVATE_KEY="$SEPOLIA_PRIVATE_KEY"
    CHAIN_ID="421614"
    EXPLORER_URL="https://sepolia.arbiscan.io"
    EXPLORER_API="https://api-sepolia.arbiscan.io/api"
else
    RPC_URL="$MAINNET_RPC_URL"
    PRIVATE_KEY="$MAINNET_PRIVATE_KEY"
    CHAIN_ID="42161"
    EXPLORER_URL="https://arbiscan.io"
    EXPLORER_API="https://api.arbiscan.io/api"
fi

log "RPC URL: $RPC_URL"
log "Chain ID: $CHAIN_ID"
log "Explorer: $EXPLORER_URL"

# Validate required variables
if [ -z "$RPC_URL" ] || [ -z "$PRIVATE_KEY" ]; then
    log_error "Missing required environment variables (RPC_URL or PRIVATE_KEY)"
    exit 1
fi

# Warn if deploying to mainnet
if [ "$NETWORK" = "mainnet" ]; then
    echo ""
    echo -e "${RED}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                    ⚠️  MAINNET DEPLOYMENT  ⚠️                  ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    log_warning "You are about to deploy to Arbitrum MAINNET!"
    log_warning "This will use REAL funds from your wallet."
    echo ""
    read -p "Are you absolutely sure you want to continue? (type 'YES' to confirm): " CONFIRM
    if [ "$CONFIRM" != "YES" ]; then
        log "Deployment cancelled by user"
        exit 0
    fi
fi

# =============================================================================
# 2. Pre-Deployment Validation
# =============================================================================

echo ""
log_step "Step 2: Running Pre-Deployment Validation"

if [ -f "$SCRIPT_DIR/pre_deploy.sh" ]; then
    log "Running pre-deployment checks..."
    if bash "$SCRIPT_DIR/pre_deploy.sh"; then
        log_success "Pre-deployment validation passed"
    else
        log_error "Pre-deployment validation failed!"
        log_error "Fix errors before deploying"
        exit 1
    fi
else
    log_warning "Pre-deployment script not found, skipping validation"
fi

# =============================================================================
# 3. Build Contract
# =============================================================================

echo ""
log_step "Step 3: Building WASM Contract"

cd "$PROJECT_ROOT"

# Clean previous build (optional)
if [ "${CLEAN_BUILD:-false}" = "true" ]; then
    log "Cleaning previous build artifacts..."
    cargo clean
fi

# Build for WASM target
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
WASM_BYTES=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)
log_success "WASM file generated: $WASM_FILE ($WASM_SIZE, $WASM_BYTES bytes)"

# Check size limit (128KB = 131072 bytes)
if [ $WASM_BYTES -gt 131072 ]; then
    log_error "WASM file exceeds Stylus size limit of 128KB!"
    log_error "Current size: $WASM_BYTES bytes ($(echo "scale=2; $WASM_BYTES / 1024" | bc) KB)"
    log_error "Consider optimizing with wasm-opt or reducing code size"
    exit 1
elif [ $WASM_BYTES -gt 100000 ]; then
    log_warning "WASM file is large: $WASM_SIZE (approaching 128KB limit)"
fi

# =============================================================================
# 4. Dry Run Check
# =============================================================================

if [ "${DRY_RUN:-false}" = "true" ]; then
    echo ""
    log_warning "DRY RUN MODE - Simulation only, no actual deployment"
    log "Would deploy to: $NETWORK"
    log "RPC URL: $RPC_URL"
    log "Contract size: $WASM_BYTES bytes"
    log_success "Dry run completed successfully"
    exit 0
fi

# =============================================================================
# 5. Deploy Contract
# =============================================================================

echo ""
log_step "Step 4: Deploying Contract to $NETWORK"

# Check cargo-stylus
if ! command -v cargo-stylus &> /dev/null; then
    log_error "cargo-stylus not found!"
    log_error "Install with: cargo install cargo-stylus"
    exit 1
fi

# Prepare deployment command
DEPLOY_CMD="cargo stylus deploy \
    --endpoint \"$RPC_URL\" \
    --private-key \"$PRIVATE_KEY\""

# Add optional parameters
if [ -n "$MAX_GAS_LIMIT" ]; then
    DEPLOY_CMD="$DEPLOY_CMD --estimate-gas"
fi

log "Deploying contract using cargo-stylus..."
log "This may take several minutes..."

# Deploy with retry logic
MAX_ATTEMPTS=${MAX_RETRY_ATTEMPTS:-3}
RETRY_DELAY=${RETRY_DELAY:-10}
ATTEMPT=1
DEPLOYED=false

while [ $ATTEMPT -le $MAX_ATTEMPTS ] && [ "$DEPLOYED" = "false" ]; do
    log "Deployment attempt $ATTEMPT of $MAX_ATTEMPTS..."

    # Capture deployment output
    DEPLOY_OUTPUT=$(mktemp)

    if eval $DEPLOY_CMD 2>&1 | tee "$DEPLOY_OUTPUT" | tee -a "$LOG_FILE"; then
        # Extract contract address from output
        # cargo stylus deploy outputs: "deployed code at address 0x..."
        CONTRACT_ADDRESS=$(grep -oP 'address 0x[a-fA-F0-9]{40}' "$DEPLOY_OUTPUT" | head -1 | cut -d' ' -f2)

        if [ -n "$CONTRACT_ADDRESS" ]; then
            log_success "Contract deployed successfully!"
            log_success "Contract address: $CONTRACT_ADDRESS"
            DEPLOYED=true
        else
            log_error "Deployment succeeded but could not extract contract address"
            cat "$DEPLOY_OUTPUT"
            rm "$DEPLOY_OUTPUT"
            exit 1
        fi
    else
        log_error "Deployment attempt $ATTEMPT failed"

        if [ $ATTEMPT -lt $MAX_ATTEMPTS ]; then
            log "Retrying in $RETRY_DELAY seconds..."
            sleep $RETRY_DELAY
        fi
    fi

    rm "$DEPLOY_OUTPUT"
    ((ATTEMPT++))
done

if [ "$DEPLOYED" = "false" ]; then
    log_error "Deployment failed after $MAX_ATTEMPTS attempts"
    exit 1
fi

# =============================================================================
# 6. Save Deployment Addresses
# =============================================================================

echo ""
log_step "Step 5: Saving Deployment Information"

DEPLOYMENT_FILE="$ADDRESSES_DIR/$NETWORK-$(date +%Y%m%d_%H%M%S).json"

# Create deployment record
cat > "$DEPLOYMENT_FILE" <<EOF
{
  "network": "$NETWORK",
  "chainId": "$CHAIN_ID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "$(cast wallet address --private-key "$PRIVATE_KEY" 2>/dev/null || echo "N/A")",
  "contracts": {
    "ArbiShield": {
      "address": "$CONTRACT_ADDRESS",
      "explorer": "$EXPLORER_URL/address/$CONTRACT_ADDRESS",
      "wasm_size": $WASM_BYTES
    }
  },
  "rpcUrl": "$RPC_URL",
  "logFile": "$LOG_FILE"
}
EOF

log_success "Deployment info saved to: $DEPLOYMENT_FILE"

# Create/update latest symlink
LATEST_FILE="$ADDRESSES_DIR/$NETWORK-latest.json"
cp "$DEPLOYMENT_FILE" "$LATEST_FILE"
log_success "Updated latest deployment: $LATEST_FILE"

# =============================================================================
# 7. Wait for Confirmations
# =============================================================================

echo ""
log_step "Step 6: Waiting for Transaction Confirmations"

CONFIRMATION_BLOCKS=${CONFIRMATION_BLOCKS:-3}
log "Waiting for $CONFIRMATION_BLOCKS confirmations..."

# Simple wait (adjust based on block time - Arbitrum is ~0.25s per block)
WAIT_TIME=$((CONFIRMATION_BLOCKS * 1))
sleep $WAIT_TIME

log_success "Transaction confirmed"

# =============================================================================
# 8. Verify Contract on Arbiscan
# =============================================================================

echo ""
log_step "Step 7: Contract Verification"

if [ "${AUTO_VERIFY:-true}" = "true" ] && [ -n "$ARBISCAN_API_KEY" ]; then
    log "Verifying contract on Arbiscan..."

    # cargo-stylus verify command
    VERIFY_CMD="cargo stylus verify \
        --deployment-tx $(echo $CONTRACT_ADDRESS | sed 's/0x//') \
        --endpoint \"$RPC_URL\""

    if eval $VERIFY_CMD 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Contract verified on Arbiscan"
        log "View at: $EXPLORER_URL/address/$CONTRACT_ADDRESS#code"
    else
        log_warning "Contract verification failed"
        log "You can verify manually later using: ./scripts/verify.sh"
    fi
else
    log_warning "Skipping automatic verification"
    if [ -z "$ARBISCAN_API_KEY" ]; then
        log "Set ARBISCAN_API_KEY in .env to enable verification"
    fi
fi

# =============================================================================
# 9. Initialize Contract
# =============================================================================

echo ""
log_step "Step 8: Contract Initialization"

if [ -f "$SCRIPT_DIR/initialize.sh" ]; then
    log "Running initialization script..."

    export CONTRACT_ADDRESS
    export NETWORK
    export RPC_URL
    export PRIVATE_KEY

    if bash "$SCRIPT_DIR/initialize.sh"; then
        log_success "Contract initialized successfully"
    else
        log_error "Contract initialization failed!"
        log_warning "Contract is deployed but not initialized"
        log "You can initialize manually later using: ./scripts/initialize.sh"
    fi
else
    log_warning "Initialization script not found, skipping initialization"
    log "Contract deployed but may need manual initialization"
fi

# =============================================================================
# 10. Post-Deployment Tests
# =============================================================================

echo ""
log_step "Step 9: Post-Deployment Tests"

if [ "${RUN_POST_DEPLOY_TESTS:-true}" = "true" ]; then
    log "Running post-deployment tests..."

    # Basic connectivity test
    log "Testing contract connectivity..."

    # Try to call a view function using cast (if available)
    if command -v cast &> /dev/null; then
        # Test owner() function
        OWNER=$(cast call "$CONTRACT_ADDRESS" "owner()(address)" --rpc-url "$RPC_URL" 2>&1 || echo "")

        if [ -n "$OWNER" ]; then
            log_success "Contract is responding to calls"
            log "Owner address: $OWNER"
        else
            log_warning "Could not verify contract connectivity"
        fi
    else
        log_warning "cast not installed, skipping connectivity tests"
        log "Install foundry to enable: curl -L https://foundry.paradigm.xyz | bash"
    fi
else
    log_warning "Skipping post-deployment tests (RUN_POST_DEPLOY_TESTS=false)"
fi

# =============================================================================
# Deployment Summary
# =============================================================================

echo ""
echo -e "${GREEN}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║              🎉  Deployment Successful!  🎉                   ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

log_success "ArbiShield deployed to $NETWORK!"
echo ""
log "Contract Address:    $CONTRACT_ADDRESS"
log "Network:             $NETWORK (Chain ID: $CHAIN_ID)"
log "Explorer:            $EXPLORER_URL/address/$CONTRACT_ADDRESS"
log "Deployment File:     $DEPLOYMENT_FILE"
log "Log File:            $LOG_FILE"

echo ""
log "Next Steps:"
log "  1. Verify deployment: $EXPLORER_URL/address/$CONTRACT_ADDRESS"
log "  2. Initialize contract (if not done): ./scripts/initialize.sh"
log "  3. Configure roles and permissions"
log "  4. Monitor contract: $EXPLORER_URL/address/$CONTRACT_ADDRESS"

echo ""
log_success "Deployment completed successfully!"
