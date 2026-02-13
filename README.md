# 🛡️ ArbiShield

**Production-Grade Security Contracts for Ethereum/Arbitrum**

ArbiShield is a comprehensive security system for Ethereum/Arbitrum smart contracts, providing real-time anomaly detection, circuit breaker functionality, and persistent alert logging. Now available in Solidity with the same powerful features and security guarantees as the original Stylus version.

## 📋 Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Contracts](#contracts)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Development](#development)
- [Testing](#testing)
- [Deployment](#deployment)
- [Usage Examples](#usage-examples)
- [Security Considerations](#security-considerations)
- [Contributing](#contributing)
- [License](#license)

## 🔍 Overview

ArbiShield consists of three independent, proxy-upgradeable smart contracts:

1. **DetectionEngine** - Monitors on-chain metrics and detects anomalies
2. **CircuitBreaker** - Emergency stop mechanism for critical situations
3. **AlertRegistry** - Immutable record of security events

### Key Features

✅ **Production-Ready**: Comprehensive error handling and event emission
✅ **Gas Optimized**: Built with Stylus for up to 10x gas savings
✅ **Proxy Compatible**: `sol_storage!` layout enables upgrades
✅ **Independent Design**: Each contract works standalone or coordinated
✅ **Owner Control**: Simple ownership model with `transfer_ownership()`
✅ **No Unsafe Code**: Strict Rust safety with `#![forbid(unsafe_code)]`

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      External Coordinator                    │
│                    (EOA or Smart Contract)                   │
└───────┬─────────────────┬───────────────────┬───────────────┘
        │                 │                   │
        │                 │                   │
┌───────▼────────┐ ┌──────▼─────────┐ ┌──────▼──────────┐
│ DetectionEngine│ │ CircuitBreaker │ │ AlertRegistry   │
├────────────────┤ ├────────────────┤ ├─────────────────┤
│ • Metrics      │ │ • Trip/Reset   │ │ • Log Alerts    │
│ • Thresholds   │ │ • Status       │ │ • Query History │
│ • Anomaly Check│ │ • Trip Count   │ │ • Severity      │
└────────────────┘ └────────────────┘ └─────────────────┘
```

### Design Philosophy

- **Independent Contracts**: Each can be deployed and used separately
- **External Coordination**: Orchestration happens off-chain or via coordinator contract
- **Event-Driven**: All state changes emit events for monitoring
- **Upgrade Path**: Proxy-compatible storage for future improvements

## 📦 Contracts

### DetectionEngine

Monitors metrics and detects when values exceed configured thresholds.

**Key Functions:**
- `register_metric(id, threshold)` - Register a new metric
- `report_metric(id, value)` - Update metric value
- `check_anomaly(id)` - Check if metric exceeds threshold
- `get_threshold(id)` / `get_current_value(id)` - Query metrics

**Events:**
- `MetricRegistered(id, threshold)`
- `MetricReported(id, value)`
- `AnomalyDetected(id, current, threshold)`

### CircuitBreaker

Emergency stop mechanism for halting operations.

**Key Functions:**
- `trip()` - Activate circuit breaker
- `reset()` - Deactivate circuit breaker
- `is_active()` - Check current state
- `get_trip_count()` - Total trips

**Events:**
- `Tripped(tripCount, timestamp)`
- `Reset(timestamp)`

### AlertRegistry

Permanent storage for security alerts.

**Key Functions:**
- `register_alert(severity, source, message_hash)` - Log new alert
- `get_alert(id)` - Retrieve alert details
- `get_alert_count()` - Total alerts

**Events:**
- `AlertRegistered(id, timestamp, severity, source, messageHash)`

## 🔧 Prerequisites

- **Node.js**: Version 16+ 
- **npm** or **yarn**
- **Hardhat**: For Solidity development and testing

### Installation

```bash
# Install dependencies
npm install

# Install Hardhat globally (if needed)
npm install -g hardhat
```

## 🚀 Installation

Clone the repository and install dependencies:

```bash
git clone <repository-url>
cd arbishield

# Install dependencies
npm install

# Compile contracts
npm run compile
```

## 💻 Development

### Project Structure

```
arbishield/
├── src/
│   ├── lib.rs                    # Main library entry
│   ├── detection_engine/         # DetectionEngine contract
│   │   ├── mod.rs
│   │   ├── storage.rs
│   │   ├── error.rs
│   │   └── interface.rs
│   ├── circuit_breaker/          # CircuitBreaker contract
│   │   └── ...
│   └── alert_registry/           # AlertRegistry contract
│       └── ...
├── tests/                        # Integration tests
├── scripts/                      # Deployment and utility scripts
├── docs/                         # Additional documentation
└── Cargo.toml                    # Project configuration
```

### Building

```bash
# Build for WASM target
cargo build --release --target wasm32-unknown-unknown

# Check contracts with Stylus
cargo stylus check

# Run all validation checks
./scripts/check-all.sh
```

### Code Quality

The project enforces strict code quality standards:

```toml
[lints.rust]
missing_docs = "warn"
unsafe_code = "forbid"

[lints.clippy]
all = "warn"
correctness = "deny"
pedantic = "warn"
```

## 🧪 Testing

### Unit Tests

Each contract includes unit tests:

```bash
cargo test
```

### Integration Tests

Cross-contract interaction tests:

```bash
cargo test --test integration
```

### Test Coverage

Run tests with coverage:

```bash
cargo test --all-features
```

## 🚀 Deployment

### Configuration

1. Copy the environment template:

```bash
cp .env.example .env
```

2. Configure your `.env` file:

```env
RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
PRIVATE_KEY=your_private_key_here
CHAIN_ID=421614  # Arbitrum Sepolia
```

### Deploy

Use the deployment script:

```bash
./scripts/deploy.sh
```

Or deploy manually:

```bash
# Validate first
cargo stylus check

# Deploy with gas estimation
cargo stylus deploy \
  --endpoint=$RPC_URL \
  --private-key=$PRIVATE_KEY \
  --estimate-gas

# Deploy
cargo stylus deploy \
  --endpoint=$RPC_URL \
  --private-key=$PRIVATE_KEY
```

### Networks

- **Arbitrum Sepolia** (testnet): `https://sepolia-rollup.arbitrum.io/rpc`
- **Arbitrum One** (mainnet): `https://arb1.arbitrum.io/rpc`

## 📚 Usage Examples

### Deploying and Using DetectionEngine

```rust
// Deploy
let mut engine = DetectionEngine::new();

// Register a metric (only owner)
engine.register_metric(U256::from(1), U256::from(1000))?;

// Report metric value
engine.report_metric(U256::from(1), U256::from(500))?;

// Check for anomaly
let is_anomaly = engine.check_anomaly(U256::from(1))?;
```

### Coordinated Security System

```javascript
// External coordination example (JavaScript/ethers.js)
const detectionEngine = new ethers.Contract(engineAddr, abi, signer);
const circuitBreaker = new ethers.Contract(breakerAddr, abi, signer);
const alertRegistry = new ethers.Contract(registryAddr, abi, signer);

// Monitor for anomalies
detectionEngine.on("AnomalyDetected", async (id, current, threshold) => {
  console.log(`Anomaly detected on metric ${id}`);

  // Trip circuit breaker
  await circuitBreaker.trip();

  // Log alert
  await alertRegistry.register_alert(
    10,  // high severity
    detectionEngine.address,
    ethers.utils.keccak256(ethers.utils.toUtf8Bytes("Threshold exceeded"))
  );
});
```

## 🔒 Security Considerations

### Access Control

- Each contract has a single **owner**
- Only the owner can execute privileged operations
- Ownership is transferable via `transfer_ownership()`
- Zero address checks prevent accidental ownership loss

### Error Handling

- All errors are typed with contextual data
- No panics in production code
- Proper Result types throughout

### Upgrade Path

- Contracts use `sol_storage!` for Solidity-compatible layout
- Compatible with proxy upgrade patterns (UUPS, Transparent)
- Storage slots remain consistent across upgrades

### Best Practices

- ✅ No `unsafe` code
- ✅ Comprehensive documentation
- ✅ Event emission for all state changes
- ✅ Input validation
- ✅ Gas-optimized storage patterns

### Auditing

Before mainnet deployment:

- [ ] Third-party security audit recommended
- [ ] Extensive testnet testing
- [ ] Gradual rollout with monitoring
- [ ] Emergency response plan

## 📖 Documentation

- [Architecture Guide](docs/ARCHITECTURE.md) - System design and patterns
- [Deployment Guide](docs/DEPLOYMENT.md) - Detailed deployment instructions
- [Testing Guide](docs/TESTING_COMPREHENSIVE.md) - Testing strategies and examples
- [Security Audit](docs/SECURITY_AUDIT_REPORT.md) - Security audit findings
- [Gas Comparison](docs/GAS_COMPARISON.md) - Rust vs Solidity gas benchmarks
- [API Reference](docs/API_REFERENCE.md) - Contract API documentation

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all checks pass: `./scripts/check-all.sh`
5. Submit a pull request

### Development Guidelines

- Follow Rust best practices
- Maintain test coverage
- Document public APIs
- Update relevant documentation

## 📄 License

This project is licensed under MIT OR Apache-2.0.

## 🔗 Resources

- [Arbitrum Stylus Documentation](https://docs.arbitrum.io/stylus)
- [Stylus SDK Repository](https://github.com/OffchainLabs/stylus-sdk-rs)
- [OpenZeppelin Rust Contracts](https://github.com/OpenZeppelin/rust-contracts-stylus)

---

**Built with ❤️ using Arbitrum Stylus**
