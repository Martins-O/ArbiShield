# ArbiShield Deployment Guide

Complete guide for deploying ArbiShield Stylus contracts to Arbitrum Sepolia testnet and mainnet.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Environment Setup](#environment-setup)
- [Deployment Process](#deployment-process)
- [Post-Deployment](#post-deployment)
- [Verification](#verification)
- [Initialization](#initialization)
- [Troubleshooting](#troubleshooting)
- [Network Information](#network-information)
- [Gas Estimation](#gas-estimation)
- [Security Best Practices](#security-best-practices)

## Prerequisites

### Required Tools

1. **Rust Toolchain** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **WASM Target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Cargo Stylus CLI**
   ```bash
   cargo install cargo-stylus
   ```

4. **Foundry** (optional, for advanced features)
   ```bash
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   ```

### System Requirements

- Linux, macOS, or WSL2 on Windows
- Bash shell
- curl, grep, jq (for scripts)
- At least 4GB RAM
- 1GB free disk space

### Verify Installation

```bash
# Check Rust
rustc --version
cargo --version

# Check WASM target
rustup target list | grep wasm32-unknown-unknown

# Check Cargo Stylus
cargo-stylus --version

# Check Foundry (optional)
cast --version
```

## Environment Setup

### 1. Clone and Build

```bash
cd /path/to/arbishield
cargo build --release --target wasm32-unknown-unknown
cargo test
```

### 2. Configure Environment

```bash
# Copy environment template
cd scripts
cp .env.example .env

# Edit configuration
nano .env  # or vim, code, etc.
```

### 3. Required Environment Variables

Edit [scripts/.env](scripts/.env) and configure:

#### Network Configuration
```bash
# Network: sepolia or mainnet
NETWORK=sepolia
```

#### RPC Endpoints
```bash
# Arbitrum Sepolia
SEPOLIA_RPC_URL=https://sepolia-rollup.arbitrum.io/rpc

# Arbitrum Mainnet (for production)
MAINNET_RPC_URL=https://arb1.arbitrum.io/rpc
```

#### Private Keys
```bash
# Testnet private key (without 0x prefix)
SEPOLIA_PRIVATE_KEY=your_testnet_private_key_here

# Mainnet private key (KEEP SECURE!)
MAINNET_PRIVATE_KEY=your_mainnet_private_key_here
```

**⚠️ SECURITY WARNING:**
- Never commit `.env` to version control
- Use different private keys for testnet and mainnet
- Never share your private keys
- Consider using hardware wallets for mainnet

#### API Keys
```bash
# Get from https://arbiscan.io/myapikey
ARBISCAN_API_KEY=your_arbiscan_api_key
```

#### Deployment Parameters (Optional)
```bash
# Initial admin (defaults to deployer)
ADMIN_ADDRESS=0x...

# Initial monitor
MONITOR_ADDRESS=0x...

# DetectionEngine address (can be set after deployment)
DETECTION_ENGINE_ADDRESS=0x...

# Alert expiration (default: 30 days)
DEFAULT_ALERT_EXPIRATION=2592000

# Gas settings
GAS_PRICE_MULTIPLIER=1.2
MAX_GAS_LIMIT=10000000
CONFIRMATION_BLOCKS=3
MAX_RETRY_ATTEMPTS=3
RETRY_DELAY=10
```

### 4. Fund Your Wallet

#### For Sepolia Testnet
- Get free ETH from [Arbitrum Sepolia Faucet](https://faucet.quicknode.com/arbitrum/sepolia)
- Or bridge from [Ethereum Sepolia Faucet](https://sepoliafaucet.com/)

Check balance:
```bash
cast balance YOUR_ADDRESS --rpc-url https://sepolia-rollup.arbitrum.io/rpc
```

You'll need approximately 0.01-0.05 ETH for deployment.

#### For Mainnet
- Bridge ETH to Arbitrum One using [Arbitrum Bridge](https://bridge.arbitrum.io/)
- Ensure sufficient ETH for deployment (~0.01-0.1 ETH depending on gas prices)

## Deployment Process

### Overview

The deployment process consists of:
1. **Pre-deployment validation** - Checks environment, builds, runs tests
2. **Contract deployment** - Deploys WASM contract to Arbitrum
3. **Verification** - Verifies contract source on Arbiscan
4. **Initialization** - Configures roles, permissions, settings

### Step 1: Pre-Deployment Validation

Run validation checks before deploying:

```bash
cd scripts
./pre_deploy.sh
```

This script validates:
- ✅ Environment configuration
- ✅ Rust & Cargo installation
- ✅ Stylus CLI availability
- ✅ Project builds successfully
- ✅ All tests passing (unit, property, invariant, security)
- ✅ Stylus contract validation
- ✅ Gas estimation
- ✅ Network connectivity
- ✅ Security checks (.gitignore, private keys)

**Expected Output:**
```
╔═══════════════════════════════════════════════════════════════╗
║        ArbiShield Pre-Deployment Validation                  ║
╚═══════════════════════════════════════════════════════════════╝

[2024-01-15 10:00:00] ━━━ Step 1: Environment Setup ━━━
[2024-01-15 10:00:01] ✓ .env file found
[2024-01-15 10:00:01] ✓ Environment variable NETWORK is set
...
[2024-01-15 10:05:00] ✓ All validation checks passed!
[2024-01-15 10:05:00] ✓ Ready for deployment to sepolia
```

### Step 2: Deploy Contract

Deploy to Arbitrum:

```bash
./deploy.sh
```

**For mainnet deployment, you'll see a confirmation prompt:**
```
╔═══════════════════════════════════════════════════════════════╗
║                    ⚠️  MAINNET DEPLOYMENT  ⚠️                  ║
╚═══════════════════════════════════════════════════════════════╝

Are you absolutely sure you want to continue? (type 'YES' to confirm):
```

The deployment script will:
1. Load environment configuration
2. Run pre-deployment validation
3. Build WASM contract
4. Check contract size (must be < 128KB)
5. Deploy to Arbitrum using cargo-stylus
6. Save deployment addresses to `deployments/`
7. Wait for confirmations
8. Verify contract on Arbiscan (if `AUTO_VERIFY=true`)
9. Initialize contract (if `initialize.sh` exists)
10. Run post-deployment tests

**Expected Output:**
```
╔═══════════════════════════════════════════════════════════════╗
║           ArbiShield Stylus Deployment                       ║
╚═══════════════════════════════════════════════════════════════╝

[2024-01-15 10:10:00] Starting deployment process...
...
[2024-01-15 10:15:00] ✓ Contract deployed successfully!
[2024-01-15 10:15:00] ✓ Contract address: 0x1234567890123456789012345678901234567890

╔═══════════════════════════════════════════════════════════════╗
║              🎉  Deployment Successful!  🎉                   ║
╚═══════════════════════════════════════════════════════════════╝
```

### Step 3: Verify Contract Source

Verify the contract on Arbiscan (automatic or manual):

```bash
# If not done automatically during deployment
./verify.sh [CONTRACT_ADDRESS]
```

The verification script:
- Checks if contract is already verified
- Builds WASM contract
- Submits verification to Arbiscan
- Confirms verification status

**Expected Output:**
```
╔═══════════════════════════════════════════════════════════════╗
║           ArbiShield Contract Verification                   ║
╚═══════════════════════════════════════════════════════════════╝

[2024-01-15 10:20:00] ✓ Contract verified successfully!
View at: https://sepolia.arbiscan.io/address/0x1234...#code
```

### Step 4: Initialize Contract

Configure roles and initial settings:

```bash
./initialize.sh [CONTRACT_ADDRESS]
```

The initialization script:
- Verifies contract ownership
- Grants ADMIN_ROLE to configured admin address
- Grants MONITOR_ROLE to configured monitor address
- Sets alert expiration duration
- Links DetectionEngine (if configured)
- Verifies configuration

**Expected Output:**
```
╔═══════════════════════════════════════════════════════════════╗
║          ArbiShield Contract Initialization                  ║
╚═══════════════════════════════════════════════════════════════╝

[2024-01-15 10:25:00] ✓ ADMIN_ROLE granted - TX: 0xabc...
[2024-01-15 10:25:05] ✓ MONITOR_ROLE granted - TX: 0xdef...
[2024-01-15 10:25:10] ✓ Alert expiration configured - TX: 0x123...
...
╔═══════════════════════════════════════════════════════════════╗
║           ✅  Initialization Complete!  ✅                    ║
╚═══════════════════════════════════════════════════════════════╝
```

## Post-Deployment

### Deployment Artifacts

Deployment creates the following files:

```
scripts/
├── deployments/
│   ├── sepolia-20240115_101500.json    # Timestamped deployment
│   └── sepolia-latest.json              # Symlink to latest
└── logs/
    ├── pre_deploy_20240115_100000.log
    ├── deploy_20240115_101000.log
    ├── verify_20240115_102000.log
    └── initialize_20240115_102500.log
```

### Deployment JSON Format

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
  "logFile": "./logs/deploy_20240115_101000.log"
}
```

### Verify Deployment

1. **Check on Arbiscan:**
   ```
   https://sepolia.arbiscan.io/address/[CONTRACT_ADDRESS]
   ```

2. **Test contract calls:**
   ```bash
   # Get owner
   cast call [CONTRACT_ADDRESS] "owner()(address)" --rpc-url $RPC_URL

   # Get alert count
   cast call [CONTRACT_ADDRESS] "get_enhanced_alert_count()(uint256)" --rpc-url $RPC_URL

   # Check role
   cast call [CONTRACT_ADDRESS] "has_role(address,uint8)(bool)" [ADDRESS] 1 --rpc-url $RPC_URL
   ```

3. **Monitor events:**
   ```bash
   # Watch for new alerts
   cast logs --address [CONTRACT_ADDRESS] --rpc-url $RPC_URL
   ```

## Verification

### Manual Verification

If automatic verification fails:

```bash
# Run verification script
./verify.sh 0xYourContractAddress
```

### Troubleshooting Verification

**Issue: "Contract already verified"**
- Contract source is already on Arbiscan
- No action needed

**Issue: "API key not set"**
- Add `ARBISCAN_API_KEY` to `.env`
- Get key from https://arbiscan.io/myapikey

**Issue: "Verification failed"**
- Ensure contract is deployed
- Wait a few minutes after deployment
- Check Arbiscan API status
- Verify WASM binary matches deployed code

## Initialization

### Manual Initialization

Initialize contract with custom configuration:

```bash
# Export required variables
export CONTRACT_ADDRESS=0x...
export NETWORK=sepolia
export RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
export PRIVATE_KEY=your_private_key

# Run initialization
./initialize.sh
```

### Configuration Options

Configure in `.env`:

```bash
# Roles
ADMIN_ADDRESS=0x...          # Admin for contract management
MONITOR_ADDRESS=0x...        # Monitor for alert registration

# Settings
DEFAULT_ALERT_EXPIRATION=2592000  # 30 days in seconds
DETECTION_ENGINE_ADDRESS=0x...    # DetectionEngine contract
```

### Role Permissions

| Role | Permission | Value |
|------|-----------|-------|
| ADMIN_ROLE | Full contract management, grant/revoke roles | 0x01 |
| MONITOR_ROLE | Register alerts, report metrics | 0x02 |

## Troubleshooting

### Common Issues

#### 1. "Private key not set"

**Error:**
```
Missing required environment variables (RPC_URL or PRIVATE_KEY)
```

**Solution:**
- Edit `scripts/.env`
- Set `SEPOLIA_PRIVATE_KEY` or `MAINNET_PRIVATE_KEY`
- Remove `0x` prefix from private key

#### 2. "WASM file exceeds size limit"

**Error:**
```
WASM file exceeds Stylus size limit of 128KB!
Current size: 150000 bytes
```

**Solution:**
- Optimize code to reduce size
- Use `wasm-opt` for additional optimization:
  ```bash
  wasm-opt -Oz -o optimized.wasm target/wasm32-unknown-unknown/release/arbishield.wasm
  ```
- Remove unused features/dependencies

#### 3. "cargo-stylus not found"

**Error:**
```
cargo-stylus not found!
```

**Solution:**
```bash
cargo install cargo-stylus
```

#### 4. "Deployment failed"

**Possible causes:**
- Insufficient gas/ETH
- Network connectivity issues
- Invalid private key
- RPC endpoint down

**Solutions:**
- Check wallet balance
- Verify RPC URL is correct
- Test network connectivity: `curl $RPC_URL`
- Increase `MAX_RETRY_ATTEMPTS` in `.env`
- Check Arbitrum network status

#### 5. "Tests failing"

**Error:**
```
Some tests failed
```

**Solution:**
```bash
# Run specific test suite
cargo test --lib                      # Unit tests
cargo test --test proptest_tests      # Property tests
cargo test --test invariant_tests     # Invariant tests
cargo test --test security_audit      # Security tests

# Check specific failure
cargo test --lib -- --nocapture
```

#### 6. "Network chain ID mismatch"

**Error:**
```
Chain ID mismatch! Expected chain for sepolia, got chain 421613
```

**Solution:**
- Verify RPC URL matches network
- Sepolia: `https://sepolia-rollup.arbitrum.io/rpc` (Chain ID: 421614)
- Mainnet: `https://arb1.arbitrum.io/rpc` (Chain ID: 42161)

### Debug Mode

Enable verbose logging:

```bash
# Set in .env
ENABLE_LOGGING=true
LOG_FILE=./scripts/logs/debug.log

# Check logs
tail -f scripts/logs/deploy_*.log
```

## Network Information

### Arbitrum Sepolia (Testnet)

- **Chain ID:** 421614
- **RPC URL:** https://sepolia-rollup.arbitrum.io/rpc
- **Explorer:** https://sepolia.arbiscan.io
- **Faucet:** https://faucet.quicknode.com/arbitrum/sepolia
- **Bridge:** https://bridge.arbitrum.io/?destinationChain=arbitrum-sepolia

### Arbitrum One (Mainnet)

- **Chain ID:** 42161
- **RPC URL:** https://arb1.arbitrum.io/rpc
- **Explorer:** https://arbiscan.io
- **Bridge:** https://bridge.arbitrum.io/

### Additional RPC Endpoints

**Public RPCs (rate-limited):**
```
# Sepolia
https://sepolia-rollup.arbitrum.io/rpc
https://arbitrum-sepolia.blockpi.network/v1/rpc/public

# Mainnet
https://arb1.arbitrum.io/rpc
https://arbitrum.blockpi.network/v1/rpc/public
```

**Private RPC providers (recommended for production):**
- [Alchemy](https://www.alchemy.com/)
- [Infura](https://infura.io/)
- [QuickNode](https://www.quicknode.com/)
- [Chainstack](https://chainstack.com/)

## Gas Estimation

### Deployment Costs

Typical costs for ArbiShield deployment:

| Network | Gas Used | @ 0.1 gwei | @ 0.5 gwei | @ 1.0 gwei |
|---------|----------|------------|------------|------------|
| Sepolia | ~3-4M gas | ~0.0003 ETH | ~0.0015 ETH | ~0.003 ETH |
| Mainnet | ~3-4M gas | ~0.0003 ETH | ~0.0015 ETH | ~0.003 ETH |

**Factors affecting cost:**
- Contract size (larger = more expensive)
- Gas price at deployment time
- Network congestion

### Operation Costs

| Operation | Estimated Gas | Cost @ 0.1 gwei |
|-----------|---------------|-----------------|
| register_enhanced_alert | ~100,000 | ~0.00001 ETH |
| acknowledge_alert | ~50,000 | ~0.000005 ETH |
| grant_role | ~50,000 | ~0.000005 ETH |
| set_alert_expiration | ~45,000 | ~0.0000045 ETH |

### Gas Optimization Tips

1. **Batch operations** when possible
2. **Use lower gas price** during off-peak hours
3. **Minimize storage writes** in contract design
4. **Optimize WASM size** to reduce deployment costs

## Security Best Practices

### Private Key Management

1. **Never commit `.env` to git**
   ```bash
   # Verify .gitignore includes .env
   grep ".env" .gitignore
   ```

2. **Use different keys for testnet/mainnet**
   - Never use mainnet keys on testnet
   - Consider hardware wallets for mainnet

3. **Secure key storage**
   - Use encrypted storage
   - Set file permissions: `chmod 600 .env`
   - Consider using environment variables or secret managers

### Deployment Safety

1. **Always test on Sepolia first**
   ```bash
   # Test full deployment flow
   NETWORK=sepolia ./deploy.sh
   ```

2. **Verify contract source**
   - Ensures transparency
   - Allows users to audit code
   - Enables interaction through Arbiscan

3. **Use dry run mode**
   ```bash
   # Set in .env
   DRY_RUN=true
   ./deploy.sh
   ```

4. **Review deployment parameters**
   - Double-check addresses
   - Verify network configuration
   - Confirm gas settings

### Post-Deployment Security

1. **Transfer ownership** if needed
   ```bash
   cast send [CONTRACT] "transferOwnership(address)" [NEW_OWNER] \
     --private-key $PRIVATE_KEY \
     --rpc-url $RPC_URL
   ```

2. **Revoke unnecessary permissions**
   ```bash
   cast send [CONTRACT] "revoke_role(address,uint8)" [ADDRESS] [ROLE] \
     --private-key $PRIVATE_KEY \
     --rpc-url $RPC_URL
   ```

3. **Monitor contract activity**
   - Watch for unexpected transactions
   - Set up alerts for role changes
   - Monitor alert registrations

4. **Regular security audits**
   - Review access control
   - Check for anomalies
   - Verify role assignments

### Emergency Procedures

If you suspect a security issue:

1. **Pause operations** (if implemented)
2. **Revoke suspicious addresses**
3. **Investigate logs and events**
4. **Notify users if necessary**
5. **Deploy fixed version if needed**

## Additional Resources

- [Arbitrum Documentation](https://docs.arbitrum.io/)
- [Stylus Documentation](https://docs.arbitrum.io/stylus/stylus-overview)
- [cargo-stylus CLI](https://github.com/OffchainLabs/cargo-stylus)
- [Foundry Book](https://book.getfoundry.sh/)
- [Arbiscan](https://arbiscan.io/)

## Support

For issues or questions:

1. Check this documentation
2. Review logs in `scripts/logs/`
3. Check [Arbitrum Discord](https://discord.gg/arbitrum)
4. Review [Stylus GitHub Issues](https://github.com/OffchainLabs/cargo-stylus/issues)

---

**Last Updated:** 2024-01-15
**Version:** 1.0.0
