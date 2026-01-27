# Deployment Guide

This guide covers deploying ArbiShield contracts to Arbitrum networks.

## Prerequisites

Before deploying, ensure you have:

1. **Rust toolchain** (1.81+)
2. **cargo-stylus** CLI tool
3. **WASM target** installed
4. **Funded wallet** with ETH on target network
5. **RPC endpoint** for the target network

## Environment Setup

### 1. Install Dependencies

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-stylus
cargo install --force cargo-stylus

# Add WASM target
rustup target add wasm32-unknown-unknown

# Verify installation
cargo stylus --version
```

### 2. Configure Environment

Create a `.env` file from the template:

```bash
cp .env.example .env
```

Edit `.env`:

```env
# Network RPC endpoint
RPC_URL=https://sepolia-rollup.arbitrum.io/rpc

# Your deployment wallet private key
PRIVATE_KEY=0x...your_private_key_here

# Chain ID
CHAIN_ID=421614  # Arbitrum Sepolia

# Optional: Custom gas settings
GAS_LIMIT=10000000
GAS_PRICE=
```

**⚠️ Security Warning**: Never commit your `.env` file! It contains your private key.

## Network Configuration

### Arbitrum Sepolia (Testnet)

```env
RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
CHAIN_ID=421614
```

**Faucet**: Get testnet ETH from [Arbitrum Sepolia Faucet](https://faucet.quicknode.com/arbitrum/sepolia)

### Arbitrum One (Mainnet)

```env
RPC_URL=https://arb1.arbitrum.io/rpc
CHAIN_ID=42161
```

**⚠️ Mainnet Checklist**:
- [ ] Contracts audited by third party
- [ ] Extensive testnet testing completed
- [ ] Emergency response plan in place
- [ ] Monitoring and alerting configured
- [ ] Sufficient ETH for deployment

## Pre-Deployment Validation

### 1. Code Quality Checks

Run all validation checks:

```bash
./scripts/check-all.sh
```

This runs:
- Format check (`cargo fmt --check`)
- Linter (`cargo clippy`)
- Build (`cargo build`)
- Stylus validation (`cargo stylus check`)
- Tests (`cargo test`)

### 2. Manual Verification

```bash
# Check formatting
cargo fmt --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Build for WASM
cargo build --release --target wasm32-unknown-unknown

# Validate contracts
cargo stylus check
```

### 3. Test Suite

```bash
# Run all tests
cargo test --all-features

# Run integration tests
cargo test --test integration
```

## Deployment Methods

### Method 1: Interactive Script (Recommended)

Use the provided deployment script:

```bash
./scripts/deploy.sh
```

The script will:
1. Check prerequisites
2. Validate contracts
3. Estimate gas for each deployment
4. Prompt for confirmation before each deployment
5. Deploy each contract sequentially

### Method 2: Manual Deployment

Deploy each contract individually:

```bash
# Load environment variables
source .env

# Deploy DetectionEngine
echo "Deploying DetectionEngine..."
cargo stylus deploy \
  --endpoint="$RPC_URL" \
  --private-key="$PRIVATE_KEY" \
  --estimate-gas

# If gas estimate looks good, deploy:
cargo stylus deploy \
  --endpoint="$RPC_URL" \
  --private-key="$PRIVATE_KEY"

# Note the deployed address!
# Repeat for CircuitBreaker and AlertRegistry
```

### Method 3: CI/CD Pipeline

For automated deployments, use a CI/CD pipeline:

```yaml
# .github/workflows/deploy.yml
name: Deploy Contracts

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.81
          target: wasm32-unknown-unknown
      - name: Install cargo-stylus
        run: cargo install --force cargo-stylus
      - name: Deploy
        env:
          RPC_URL: ${{ secrets.RPC_URL }}
          PRIVATE_KEY: ${{ secrets.PRIVATE_KEY }}
        run: ./scripts/deploy.sh
```

## Gas Estimation

### Understanding Gas Costs

Stylus contracts have two gas components:

1. **Deployment gas**: One-time cost to deploy the contract
2. **Execution gas**: Per-transaction cost (much lower than Solidity)

### Estimate Deployment Gas

```bash
cargo stylus deploy \
  --endpoint="$RPC_URL" \
  --private-key="$PRIVATE_KEY" \
  --estimate-gas
```

Example output:
```
Deployment gas estimate: 850,000 gas
At 0.1 gwei: ~0.000085 ETH
At 1 gwei: ~0.00085 ETH
```

### Optimize Gas Usage

To minimize deployment gas:

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Enable link-time optimization
codegen-units = 1    # Single codegen unit
strip = true         # Strip symbols
```

## Post-Deployment

### 1. Record Contract Addresses

Update your `.env` file with deployed addresses:

```env
DETECTION_ENGINE_ADDRESS=0x...
CIRCUIT_BREAKER_ADDRESS=0x...
ALERT_REGISTRY_ADDRESS=0x...
```

### 2. Verify Deployment

Check contract on block explorer:
- **Arbitrum Sepolia**: https://sepolia.arbiscan.io/
- **Arbitrum One**: https://arbiscan.io/

### 3. Initial Configuration

Set up initial state:

```rust
// Register initial metrics
await detectionEngine.register_metric(
  1,     // metric ID
  1000   // threshold
);

// Verify ownership
const owner = await detectionEngine.owner();
console.log("Owner:", owner);
```

### 4. Transfer Ownership (Optional)

If using a multi-sig or different admin:

```rust
await detectionEngine.transfer_ownership(newOwner);
await circuitBreaker.transfer_ownership(newOwner);
await alertRegistry.transfer_ownership(newOwner);
```

## Monitoring Setup

### Event Monitoring

Set up event listeners for all contracts:

```javascript
// Monitor DetectionEngine
detectionEngine.on("AnomalyDetected", (id, current, threshold) => {
  console.log(`⚠️ Anomaly: Metric ${id} at ${current} exceeds ${threshold}`);
  // Trigger alerts, trip circuit breaker, etc.
});

// Monitor CircuitBreaker
circuitBreaker.on("Tripped", (tripCount, timestamp) => {
  console.log(`🚨 Circuit breaker tripped! Count: ${tripCount}`);
  // Send urgent notifications
});

// Monitor AlertRegistry
alertRegistry.on("AlertRegistered", (id, timestamp, severity, source, hash) => {
  console.log(`📝 Alert ${id} registered with severity ${severity}`);
  // Log to database, send notifications
});
```

### Health Checks

Implement periodic health checks:

```javascript
async function healthCheck() {
  const engineHealth = await detectionEngine.get_metric_count();
  const breakerActive = await circuitBreaker.is_active();
  const alertCount = await alertRegistry.get_alert_count();

  console.log({
    metrics: engineHealth.toString(),
    circuitActive: breakerActive,
    alerts: alertCount.toString()
  });
}

setInterval(healthCheck, 60000); // Every minute
```

## Troubleshooting

### Common Issues

#### 1. Deployment Fails: "Insufficient Funds"

**Problem**: Not enough ETH in wallet

**Solution**:
```bash
# Check balance
cast balance $YOUR_ADDRESS --rpc-url $RPC_URL

# Get testnet ETH from faucet
```

#### 2. "WASM target not found"

**Problem**: WebAssembly target not installed

**Solution**:
```bash
rustup target add wasm32-unknown-unknown
```

#### 3. "cargo-stylus: command not found"

**Problem**: CLI tool not installed

**Solution**:
```bash
cargo install --force cargo-stylus
```

#### 4. Deployment Succeeds but Contract Doesn't Work

**Problem**: ABI mismatch or incorrect compilation

**Solution**:
```bash
# Rebuild from scratch
cargo clean
cargo build --release --target wasm32-unknown-unknown
cargo stylus check
```

#### 5. High Gas Costs

**Problem**: Not optimized for size

**Solution**: Ensure release profile is optimized (see Cargo.toml)

## Security Checklist

Before mainnet deployment:

- [ ] All tests passing
- [ ] Code reviewed by team
- [ ] Third-party security audit completed
- [ ] Testnet deployment tested thoroughly
- [ ] Emergency procedures documented
- [ ] Monitoring and alerting configured
- [ ] Ownership transfer plan ready
- [ ] Backup plans for critical failures
- [ ] Team trained on incident response

## Upgrade Strategy

For future upgrades:

### 1. Deploy New Implementation

```bash
# Deploy new version
cargo stylus deploy --endpoint=$RPC_URL --private-key=$PRIVATE_KEY

# Note new address
NEW_IMPL=0x...
```

### 2. Update Proxy

If using proxy pattern:

```javascript
// Point proxy to new implementation
await proxy.upgradeTo(NEW_IMPL);
```

### 3. Verify Upgrade

```javascript
// Check new implementation is active
const impl = await proxy.implementation();
assert(impl === NEW_IMPL);

// Test functionality
await detectionEngine.get_metric_count();
```

## Cost Estimates

### Testnet Deployment

- DetectionEngine: ~800,000 gas (~$0.10 at 0.1 gwei)
- CircuitBreaker: ~500,000 gas (~$0.06 at 0.1 gwei)
- AlertRegistry: ~700,000 gas (~$0.09 at 0.1 gwei)
- **Total**: ~2,000,000 gas (~$0.25 at 0.1 gwei)

### Mainnet Deployment

Costs vary with gas prices. At typical Arbitrum One prices (0.1 gwei):
- Total deployment: ~$0.30-0.50

## Resources

- [Arbitrum Stylus Docs](https://docs.arbitrum.io/stylus)
- [cargo-stylus CLI](https://github.com/OffchainLabs/cargo-stylus)
- [Arbitrum Block Explorer](https://arbiscan.io/)
- [Arbitrum Sepolia Explorer](https://sepolia.arbiscan.io/)

---

For additional help, refer to [ARCHITECTURE.md](./ARCHITECTURE.md) and [TESTING.md](./TESTING.md).
