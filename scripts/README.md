# ArbiShield Deployment Scripts

Production-ready deployment scripts for ArbiShield Stylus contracts on Arbitrum.

## Quick Start

```bash
# 1. Configure environment
cp .env.example .env
nano .env  # Edit configuration

# 2. Validate setup
./pre_deploy.sh

# 3. Deploy contract
./deploy.sh

# 4. Verify on Arbiscan (if not done automatically)
./verify.sh

# 5. Initialize contract (if not done automatically)
./initialize.sh
```

## Scripts

### pre_deploy.sh
**Pre-deployment validation script**

Validates environment, dependencies, builds, tests, and network connectivity before deployment.

```bash
./pre_deploy.sh
```

**Checks:**
- ✅ Environment configuration (.env)
- ✅ Rust & Cargo installation
- ✅ Stylus CLI (cargo-stylus)
- ✅ Project builds successfully
- ✅ All tests passing (113 unit + 54 property + 56 invariant + 43 security)
- ✅ Stylus contract validation
- ✅ Gas estimation
- ✅ Network connectivity
- ✅ Security (.gitignore, private keys)

**Output:**
- Logs to `logs/pre_deploy_YYYYMMDD_HHMMSS.log`
- Exit code 0 on success, 1 on failure

---

### deploy.sh
**Main deployment script**

Deploys ArbiShield contract to Arbitrum Sepolia or Mainnet using cargo-stylus.

```bash
./deploy.sh
```

**Process:**
1. Load environment configuration
2. Run pre-deployment validation
3. Build WASM contract
4. Deploy to Arbitrum
5. Save deployment addresses
6. Wait for confirmations
7. Verify on Arbiscan (optional)
8. Initialize contract (optional)
9. Run post-deployment tests

**Output:**
- Deployment JSON: `deployments/<network>-YYYYMMDD_HHMMSS.json`
- Latest symlink: `deployments/<network>-latest.json`
- Logs to `logs/deploy_YYYYMMDD_HHMMSS.log`

**Configuration (.env):**
```bash
NETWORK=sepolia                      # sepolia or mainnet
SEPOLIA_RPC_URL=https://...          # RPC endpoint
SEPOLIA_PRIVATE_KEY=...              # Private key (no 0x)
ARBISCAN_API_KEY=...                 # For verification
AUTO_VERIFY=true                     # Auto-verify on Arbiscan
DRY_RUN=false                        # Simulate without deploying
```

---

### verify.sh
**Contract verification script**

Verifies deployed Stylus contracts on Arbiscan.

```bash
./verify.sh [CONTRACT_ADDRESS]
```

**Arguments:**
- `CONTRACT_ADDRESS` (optional): Contract address to verify
  - If not provided, loads from `deployments/<network>-latest.json`

**Process:**
1. Load contract address
2. Check if already verified
3. Build WASM contract
4. Submit verification to Arbiscan
5. Confirm verification status

**Output:**
- Logs to `logs/verify_YYYYMMDD_HHMMSS.log`
- Contract source visible on Arbiscan

---

### initialize.sh
**Contract initialization script**

Configures deployed contract with roles, permissions, and initial settings.

```bash
./initialize.sh [CONTRACT_ADDRESS]
```

**Arguments:**
- `CONTRACT_ADDRESS` (optional): Contract address to initialize
  - If not provided, loads from deployment JSON or `$CONTRACT_ADDRESS` env var

**Process:**
1. Verify contract ownership
2. Grant ADMIN_ROLE to configured admin
3. Grant MONITOR_ROLE to configured monitor
4. Set alert expiration duration
5. Link DetectionEngine
6. Verify configuration

**Configuration (.env):**
```bash
ADMIN_ADDRESS=0x...                  # Admin for contract management
MONITOR_ADDRESS=0x...                # Monitor for alert registration
DEFAULT_ALERT_EXPIRATION=2592000     # 30 days in seconds
DETECTION_ENGINE_ADDRESS=0x...       # DetectionEngine contract
```

**Output:**
- Transaction hashes for each configuration step
- Logs to `logs/initialize_YYYYMMDD_HHMMSS.log`

---

## Configuration

### Environment Variables (.env)

Create `.env` from `.env.example` and configure:

#### Required
```bash
NETWORK=sepolia                      # sepolia or mainnet
SEPOLIA_RPC_URL=https://...          # Arbitrum Sepolia RPC
SEPOLIA_PRIVATE_KEY=...              # Private key (no 0x prefix)
```

#### Optional
```bash
# API Keys
ARBISCAN_API_KEY=...                 # For contract verification

# Deployment Parameters
ADMIN_ADDRESS=0x...                  # Initial admin (defaults to deployer)
MONITOR_ADDRESS=0x...                # Initial monitor
DETECTION_ENGINE_ADDRESS=0x...       # DetectionEngine contract

# Gas Settings
GAS_PRICE_MULTIPLIER=1.2             # Gas price multiplier
MAX_GAS_LIMIT=10000000               # Max gas limit
CONFIRMATION_BLOCKS=3                # Confirmations to wait

# Deployment Options
AUTO_VERIFY=true                     # Auto-verify on Arbiscan
SAVE_ADDRESSES=true                  # Save deployment addresses
RUN_POST_DEPLOY_TESTS=true           # Run post-deployment tests
DRY_RUN=false                        # Simulate without deploying

# Advanced
MAX_RETRY_ATTEMPTS=3                 # Max retry attempts
RETRY_DELAY=10                       # Delay between retries (seconds)
```

### Network Information

#### Arbitrum Sepolia (Testnet)
- **Chain ID:** 421614
- **RPC:** https://sepolia-rollup.arbitrum.io/rpc
- **Explorer:** https://sepolia.arbiscan.io
- **Faucet:** https://faucet.quicknode.com/arbitrum/sepolia

#### Arbitrum One (Mainnet)
- **Chain ID:** 42161
- **RPC:** https://arb1.arbitrum.io/rpc
- **Explorer:** https://arbiscan.io
- **Bridge:** https://bridge.arbitrum.io/

---

## Directory Structure

```
scripts/
├── README.md                        # This file
├── DEPLOYMENT.md                    # Comprehensive deployment guide
├── .env.example                     # Environment template
├── .env                             # Your configuration (git-ignored)
│
├── pre_deploy.sh                    # Pre-deployment validation
├── deploy.sh                        # Main deployment script
├── verify.sh                        # Contract verification
├── initialize.sh                    # Contract initialization
│
├── deployments/                     # Deployment artifacts
│   ├── sepolia-YYYYMMDD_HHMMSS.json
│   ├── sepolia-latest.json
│   ├── mainnet-YYYYMMDD_HHMMSS.json
│   └── mainnet-latest.json
│
└── logs/                            # Execution logs
    ├── pre_deploy_YYYYMMDD_HHMMSS.log
    ├── deploy_YYYYMMDD_HHMMSS.log
    ├── verify_YYYYMMDD_HHMMSS.log
    └── initialize_YYYYMMDD_HHMMSS.log
```

---

## Deployment JSON Format

Example: `deployments/sepolia-20240115_101500.json`

```json
{
  "network": "sepolia",
  "chainId": "421614",
  "timestamp": "2024-01-15T10:15:00Z",
  "deployer": "0xYourAddress...",
  "contracts": {
    "ArbiShield": {
      "address": "0x1234567890123456789012345678901234567890",
      "explorer": "https://sepolia.arbiscan.io/address/0x1234...",
      "wasm_size": 45678
    }
  },
  "rpcUrl": "https://sepolia-rollup.arbitrum.io/rpc",
  "logFile": "./logs/deploy_20240115_101500.log"
}
```

---

## Examples

### Full Deployment Flow (Testnet)

```bash
# 1. Setup
cd scripts
cp .env.example .env
nano .env  # Set NETWORK=sepolia, SEPOLIA_RPC_URL, SEPOLIA_PRIVATE_KEY

# 2. Validate
./pre_deploy.sh
# ✓ All checks pass

# 3. Deploy
./deploy.sh
# ✓ Contract deployed: 0x1234567890123456789012345678901234567890

# 4. Verify deployment
cat deployments/sepolia-latest.json
# View on Arbiscan: https://sepolia.arbiscan.io/address/0x1234...

# 5. Initialize (if needed)
./initialize.sh
# ✓ Roles configured, settings applied
```

### Manual Verification

```bash
# Verify specific contract
./verify.sh 0x1234567890123456789012345678901234567890

# Check verification status
curl "https://api-sepolia.arbiscan.io/api?module=contract&action=getsourcecode&address=0x1234...&apikey=$ARBISCAN_API_KEY"
```

### Using Foundry (cast)

```bash
# Load deployment
CONTRACT=$(jq -r '.contracts.ArbiShield.address' deployments/sepolia-latest.json)
RPC_URL=$(jq -r '.rpcUrl' deployments/sepolia-latest.json)

# Get owner
cast call $CONTRACT "owner()(address)" --rpc-url $RPC_URL

# Get alert count
cast call $CONTRACT "get_enhanced_alert_count()(uint256)" --rpc-url $RPC_URL

# Check role
cast call $CONTRACT "has_role(address,uint8)(bool)" $ADMIN_ADDRESS 1 --rpc-url $RPC_URL

# Register alert (requires MONITOR_ROLE)
cast send $CONTRACT \
  "register_enhanced_alert(address,uint256,uint256,bytes32,address)" \
  $PROTOCOL_ADDRESS \
  75 \
  0x01 \
  0x0000000000000000000000000000000000000000000000000000000000000001 \
  $SOURCE_ADDRESS \
  --private-key $PRIVATE_KEY \
  --rpc-url $RPC_URL
```

---

## Troubleshooting

### "Private key not set"
- Edit `.env` and set `SEPOLIA_PRIVATE_KEY` or `MAINNET_PRIVATE_KEY`
- Remove `0x` prefix from private key

### "cargo-stylus not found"
```bash
cargo install cargo-stylus
```

### "WASM file exceeds size limit"
- Contract must be < 128KB
- Optimize code or use `wasm-opt`

### "Tests failing"
```bash
# Run specific test suite
cargo test --lib
cargo test --test proptest_tests
cargo test --test invariant_tests
cargo test --test security_audit
```

### "Deployment failed"
- Check wallet balance (need ~0.01-0.05 ETH)
- Verify RPC URL is correct
- Check network connectivity
- Review logs in `logs/deploy_*.log`

### "Verification failed"
- Wait a few minutes after deployment
- Check `ARBISCAN_API_KEY` is set correctly
- Ensure WASM binary matches deployed code
- Review logs in `logs/verify_*.log`

---

## Security Best Practices

### 1. Private Key Management
- ✅ Never commit `.env` to git (check `.gitignore`)
- ✅ Use different keys for testnet/mainnet
- ✅ Set file permissions: `chmod 600 .env`
- ✅ Consider hardware wallets for mainnet

### 2. Deployment Safety
- ✅ Always test on Sepolia first
- ✅ Use dry run mode: `DRY_RUN=true`
- ✅ Review deployment parameters
- ✅ Verify contract source on Arbiscan

### 3. Post-Deployment
- ✅ Transfer ownership if needed
- ✅ Revoke unnecessary permissions
- ✅ Monitor contract activity
- ✅ Regular security audits

---

## Advanced Usage

### Custom RPC Endpoints

Use private RPC providers for better reliability:

```bash
# Alchemy
SEPOLIA_RPC_URL=https://arb-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# Infura
SEPOLIA_RPC_URL=https://arbitrum-sepolia.infura.io/v3/YOUR_API_KEY

# QuickNode
SEPOLIA_RPC_URL=https://YOUR_NODE.arbitrum-sepolia.quiknode.pro/YOUR_API_KEY/
```

### Batch Deployment

Deploy multiple contracts:

```bash
# Deploy to Sepolia
NETWORK=sepolia ./deploy.sh

# Deploy to Mainnet (with confirmation)
NETWORK=mainnet ./deploy.sh
```

### Scripted Initialization

```bash
# Load deployment
CONTRACT=$(jq -r '.contracts.ArbiShield.address' deployments/sepolia-latest.json)

# Set multiple configurations
export CONTRACT_ADDRESS=$CONTRACT
export ADMIN_ADDRESS=0x...
export MONITOR_ADDRESS=0x...

./initialize.sh
```

---

## Resources

- **[Comprehensive Deployment Guide](DEPLOYMENT.md)** - Full deployment documentation
- [Arbitrum Documentation](https://docs.arbitrum.io/)
- [Stylus Documentation](https://docs.arbitrum.io/stylus/stylus-overview)
- [cargo-stylus CLI](https://github.com/OffchainLabs/cargo-stylus)
- [Foundry Book](https://book.getfoundry.sh/)

---

## Support

For issues:
1. Check [DEPLOYMENT.md](DEPLOYMENT.md)
2. Review logs in `logs/`
3. [Arbitrum Discord](https://discord.gg/arbitrum)
4. [Stylus GitHub](https://github.com/OffchainLabs/cargo-stylus/issues)

---

**Version:** 1.0.0
**Last Updated:** 2024-01-15
