#!/bin/bash

# ArbiShield Contract Initialization Script
# Initializes deployed contracts with roles, permissions, and initial configuration

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
LOG_FILE="$LOG_DIR/initialize_$(date +%Y%m%d_%H%M%S).log"

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
echo "║          ArbiShield Contract Initialization                  ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

log "Starting initialization process..."
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
    PRIVATE_KEY="$SEPOLIA_PRIVATE_KEY"
    CHAIN_ID="421614"
    EXPLORER_URL="https://sepolia.arbiscan.io"
else
    RPC_URL="$MAINNET_RPC_URL"
    PRIVATE_KEY="$MAINNET_PRIVATE_KEY"
    CHAIN_ID="42161"
    EXPLORER_URL="https://arbiscan.io"
fi

log "RPC URL: $RPC_URL"
log "Chain ID: $CHAIN_ID"

# =============================================================================
# 2. Get Contract Address
# =============================================================================

echo ""
log_step "Step 2: Getting Contract Address"

# Check if contract address provided as argument or environment variable
if [ -n "$CONTRACT_ADDRESS" ]; then
    log "Using contract address from environment: $CONTRACT_ADDRESS"
elif [ -n "$1" ]; then
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
        log_error "Or set CONTRACT_ADDRESS environment variable"
        exit 1
    fi
fi

# Validate address format
if ! echo "$CONTRACT_ADDRESS" | grep -qE '^0x[a-fA-F0-9]{40}$'; then
    log_error "Invalid contract address format: $CONTRACT_ADDRESS"
    exit 1
fi

# =============================================================================
# 3. Check Dependencies
# =============================================================================

echo ""
log_step "Step 3: Checking Dependencies"

# Check if cast is available (from Foundry)
if ! command -v cast &> /dev/null; then
    log_warning "cast (Foundry) not found - some features will be limited"
    log "Install Foundry: curl -L https://foundry.paradigm.xyz | bash"
    USE_CAST=false
else
    CAST_VERSION=$(cast --version 2>&1 | head -1 || echo "unknown")
    log_success "cast installed: $CAST_VERSION"
    USE_CAST=true
fi

# =============================================================================
# 4. Verify Contract Ownership
# =============================================================================

echo ""
log_step "Step 4: Verifying Contract Ownership"

if [ "$USE_CAST" = "true" ]; then
    log "Checking contract owner..."

    # Get deployer address from private key
    DEPLOYER=$(cast wallet address --private-key "$PRIVATE_KEY")
    log "Deployer address: $DEPLOYER"

    # Get contract owner
    OWNER=$(cast call "$CONTRACT_ADDRESS" "owner()(address)" --rpc-url "$RPC_URL" 2>&1 || echo "")

    if [ -n "$OWNER" ]; then
        # Convert to lowercase for comparison
        OWNER_LOWER=$(echo "$OWNER" | tr '[:upper:]' '[:lower:]')
        DEPLOYER_LOWER=$(echo "$DEPLOYER" | tr '[:upper:]' '[:lower:]')

        if [ "$OWNER_LOWER" = "$DEPLOYER_LOWER" ]; then
            log_success "Deployer is the contract owner"
        else
            log_error "Deployer ($DEPLOYER) is not the contract owner ($OWNER)"
            log_error "Initialization requires owner privileges"
            exit 1
        fi
    else
        log_warning "Could not verify contract ownership - proceeding anyway"
    fi
else
    log_warning "Skipping ownership verification (cast not available)"
fi

# =============================================================================
# 5. Configure AlertRegistry Roles
# =============================================================================

echo ""
log_step "Step 5: Configuring AlertRegistry Roles"

# Role constants (from AlertRegistry contract)
ADMIN_ROLE=1    # 0x01
MONITOR_ROLE=2  # 0x02

# Grant ADMIN_ROLE to specified admin address
if [ -n "$ADMIN_ADDRESS" ] && [ "$ADMIN_ADDRESS" != "" ]; then
    log "Granting ADMIN_ROLE to: $ADMIN_ADDRESS"

    if [ "$USE_CAST" = "true" ]; then
        # Call grant_role(address, uint8)
        TX_HASH=$(cast send "$CONTRACT_ADDRESS" \
            "grant_role(address,uint8)" \
            "$ADMIN_ADDRESS" \
            "$ADMIN_ROLE" \
            --private-key "$PRIVATE_KEY" \
            --rpc-url "$RPC_URL" \
            --json 2>&1 | grep -o '"transactionHash":"0x[^"]*"' | cut -d'"' -f4 || echo "")

        if [ -n "$TX_HASH" ]; then
            log_success "ADMIN_ROLE granted - TX: $TX_HASH"
            log "View: $EXPLORER_URL/tx/$TX_HASH"
        else
            log_warning "Could not grant ADMIN_ROLE"
        fi
    else
        log_warning "Skipping role grant (cast not available)"
    fi
else
    log_warning "ADMIN_ADDRESS not configured - skipping admin role grant"
fi

# Grant MONITOR_ROLE to specified monitor address
if [ -n "$MONITOR_ADDRESS" ] && [ "$MONITOR_ADDRESS" != "" ]; then
    log "Granting MONITOR_ROLE to: $MONITOR_ADDRESS"

    if [ "$USE_CAST" = "true" ]; then
        # Call grant_role(address, uint8)
        TX_HASH=$(cast send "$CONTRACT_ADDRESS" \
            "grant_role(address,uint8)" \
            "$MONITOR_ADDRESS" \
            "$MONITOR_ROLE" \
            --private-key "$PRIVATE_KEY" \
            --rpc-url "$RPC_URL" \
            --json 2>&1 | grep -o '"transactionHash":"0x[^"]*"' | cut -d'"' -f4 || echo "")

        if [ -n "$TX_HASH" ]; then
            log_success "MONITOR_ROLE granted - TX: $TX_HASH"
            log "View: $EXPLORER_URL/tx/$TX_HASH"
        else
            log_warning "Could not grant MONITOR_ROLE"
        fi
    else
        log_warning "Skipping role grant (cast not available)"
    fi
else
    log_warning "MONITOR_ADDRESS not configured - skipping monitor role grant"
fi

# =============================================================================
# 6. Configure Alert Expiration
# =============================================================================

echo ""
log_step "Step 6: Configuring Alert Expiration"

if [ -n "$DEFAULT_ALERT_EXPIRATION" ] && [ "$DEFAULT_ALERT_EXPIRATION" != "" ]; then
    log "Setting alert expiration duration to: $DEFAULT_ALERT_EXPIRATION seconds"

    if [ "$USE_CAST" = "true" ]; then
        # Call set_alert_expiration_duration(uint256)
        TX_HASH=$(cast send "$CONTRACT_ADDRESS" \
            "set_alert_expiration_duration(uint256)" \
            "$DEFAULT_ALERT_EXPIRATION" \
            --private-key "$PRIVATE_KEY" \
            --rpc-url "$RPC_URL" \
            --json 2>&1 | grep -o '"transactionHash":"0x[^"]*"' | cut -d'"' -f4 || echo "")

        if [ -n "$TX_HASH" ]; then
            log_success "Alert expiration configured - TX: $TX_HASH"
            log "View: $EXPLORER_URL/tx/$TX_HASH"
        else
            log_warning "Could not set alert expiration"
        fi
    else
        log_warning "Skipping expiration config (cast not available)"
    fi
else
    log_warning "DEFAULT_ALERT_EXPIRATION not configured - using contract default"
fi

# =============================================================================
# 7. Link DetectionEngine
# =============================================================================

echo ""
log_step "Step 7: Linking DetectionEngine"

if [ -n "$DETECTION_ENGINE_ADDRESS" ] && [ "$DETECTION_ENGINE_ADDRESS" != "" ]; then
    log "Setting DetectionEngine address to: $DETECTION_ENGINE_ADDRESS"

    if [ "$USE_CAST" = "true" ]; then
        # Call set_detection_engine(address)
        TX_HASH=$(cast send "$CONTRACT_ADDRESS" \
            "set_detection_engine(address)" \
            "$DETECTION_ENGINE_ADDRESS" \
            --private-key "$PRIVATE_KEY" \
            --rpc-url "$RPC_URL" \
            --json 2>&1 | grep -o '"transactionHash":"0x[^"]*"' | cut -d'"' -f4 || echo "")

        if [ -n "$TX_HASH" ]; then
            log_success "DetectionEngine linked - TX: $TX_HASH"
            log "View: $EXPLORER_URL/tx/$TX_HASH"
        else
            log_warning "Could not set DetectionEngine address"
        fi
    else
        log_warning "Skipping DetectionEngine link (cast not available)"
    fi
else
    log_warning "DETECTION_ENGINE_ADDRESS not configured - skipping"
    log "You can set this later using set_detection_engine()"
fi

# =============================================================================
# 8. Configure DetectionEngine Thresholds
# =============================================================================

echo ""
log_step "Step 8: Configuring DetectionEngine Thresholds"

if [ -n "$DEFAULT_ANOMALY_THRESHOLD" ] && [ "$DEFAULT_ANOMALY_THRESHOLD" != "" ]; then
    log "Setting default anomaly threshold to: $DEFAULT_ANOMALY_THRESHOLD"

    if [ "$USE_CAST" = "true" ] && [ -n "$DETECTION_ENGINE_ADDRESS" ]; then
        # Note: This assumes DetectionEngine has a set_default_threshold function
        # Adjust based on actual DetectionEngine interface
        log_warning "DetectionEngine threshold configuration requires custom implementation"
        log "Configure thresholds manually using DetectionEngine contract"
    else
        log_warning "Skipping threshold config"
    fi
else
    log_warning "DEFAULT_ANOMALY_THRESHOLD not configured"
fi

# =============================================================================
# 9. Verify Initialization
# =============================================================================

echo ""
log_step "Step 9: Verifying Initialization"

if [ "$USE_CAST" = "true" ]; then
    log "Verifying contract configuration..."

    # Check owner
    OWNER=$(cast call "$CONTRACT_ADDRESS" "owner()(address)" --rpc-url "$RPC_URL" 2>&1 || echo "")
    if [ -n "$OWNER" ]; then
        log_success "Owner: $OWNER"
    fi

    # Check alert expiration duration
    EXPIRATION=$(cast call "$CONTRACT_ADDRESS" "get_alert_expiration_duration()(uint256)" --rpc-url "$RPC_URL" 2>&1 || echo "")
    if [ -n "$EXPIRATION" ]; then
        log_success "Alert expiration: $EXPIRATION seconds"
    fi

    # Check detection engine
    ENGINE=$(cast call "$CONTRACT_ADDRESS" "get_detection_engine()(address)" --rpc-url "$RPC_URL" 2>&1 || echo "")
    if [ -n "$ENGINE" ]; then
        log_success "DetectionEngine: $ENGINE"
    fi

    # Check roles (if addresses configured)
    if [ -n "$ADMIN_ADDRESS" ]; then
        HAS_ADMIN=$(cast call "$CONTRACT_ADDRESS" "has_role(address,uint8)(bool)" "$ADMIN_ADDRESS" "$ADMIN_ROLE" --rpc-url "$RPC_URL" 2>&1 || echo "")
        if [ "$HAS_ADMIN" = "true" ]; then
            log_success "ADMIN_ROLE verified for: $ADMIN_ADDRESS"
        else
            log_warning "ADMIN_ROLE not found for: $ADMIN_ADDRESS"
        fi
    fi

    if [ -n "$MONITOR_ADDRESS" ]; then
        HAS_MONITOR=$(cast call "$CONTRACT_ADDRESS" "has_role(address,uint8)(bool)" "$MONITOR_ADDRESS" "$MONITOR_ROLE" --rpc-url "$RPC_URL" 2>&1 || echo "")
        if [ "$HAS_MONITOR" = "true" ]; then
            log_success "MONITOR_ROLE verified for: $MONITOR_ADDRESS"
        else
            log_warning "MONITOR_ROLE not found for: $MONITOR_ADDRESS"
        fi
    fi
else
    log_warning "Skipping verification (cast not available)"
fi

# =============================================================================
# Initialization Summary
# =============================================================================

echo ""
echo -e "${GREEN}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║           ✅  Initialization Complete!  ✅                    ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

log_success "Contract initialization completed!"
echo ""
log "Contract Address:    $CONTRACT_ADDRESS"
log "Network:             $NETWORK (Chain ID: $CHAIN_ID)"
log "Explorer:            $EXPLORER_URL/address/$CONTRACT_ADDRESS"

echo ""
log "Configuration Summary:"
log "  Owner:             ${OWNER:-N/A}"
log "  Admin:             ${ADMIN_ADDRESS:-Not configured}"
log "  Monitor:           ${MONITOR_ADDRESS:-Not configured}"
log "  Alert Expiration:  ${DEFAULT_ALERT_EXPIRATION:-Contract default} seconds"
log "  DetectionEngine:   ${DETECTION_ENGINE_ADDRESS:-Not configured}"

echo ""
log "Next Steps:"
log "  1. Verify roles: $EXPLORER_URL/address/$CONTRACT_ADDRESS#readContract"
log "  2. Test alert registration with MONITOR_ROLE"
log "  3. Configure additional subscribers if needed"
log "  4. Set up monitoring and alerting"

echo ""
log_success "Initialization completed successfully!"
