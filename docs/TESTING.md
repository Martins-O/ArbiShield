# Testing Guide

Comprehensive testing guide for ArbiShield contracts.

## Testing Philosophy

ArbiShield follows a multi-layered testing approach:

1. **Unit Tests**: Test individual functions and logic
2. **Integration Tests**: Test contract interactions
3. **Property-Based Tests**: Test invariants with random inputs
4. **Manual Testing**: Interactive testing on testnets

## Test Structure

```
arbishield/
├── src/
│   ├── detection_engine/
│   │   └── mod.rs          # Unit tests in #[cfg(test)]
│   ├── circuit_breaker/
│   │   └── mod.rs          # Unit tests
│   └── alert_registry/
│       └── mod.rs          # Unit tests
└── tests/
    └── integration.rs      # Integration tests
```

## Running Tests

### Quick Test

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_threshold_logic
```

### Comprehensive Test Suite

```bash
# Run full validation
./scripts/check-all.sh
```

This includes:
- Unit tests
- Integration tests
- Format checks
- Linter checks
- Build validation

### Test with Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage
```

## Unit Tests

### Writing Unit Tests

Each contract module includes unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_comparison() {
        let threshold = U256::from(100);
        let value = U256::from(150);
        assert!(value > threshold);
    }

    #[test]
    fn test_zero_address() {
        let addr = Address::ZERO;
        assert_eq!(addr, Address::from([0u8; 20]));
    }
}
```

### Test Coverage Goals

- **Functions**: 100% of public functions
- **Branches**: >90% of conditional branches
- **Error paths**: All error cases tested

### Example: DetectionEngine Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_registration() {
        // Test metric registration logic
        let id = U256::from(1);
        let threshold = U256::from(1000);
        assert_eq!(threshold, U256::from(1000));
    }

    #[test]
    fn test_anomaly_detection() {
        // Test anomaly detection logic
        let threshold = U256::from(100);
        let value_normal = U256::from(50);
        let value_anomaly = U256::from(150);

        assert!(value_normal <= threshold);
        assert!(value_anomaly > threshold);
    }

    #[test]
    fn test_ownership_validation() {
        // Test zero address check
        let zero = Address::ZERO;
        let valid = Address::from([1u8; 20]);

        assert_eq!(zero, Address::from([0u8; 20]));
        assert_ne!(valid, Address::ZERO);
    }
}
```

## Integration Tests

### Writing Integration Tests

Integration tests live in `tests/` directory:

```rust
// tests/integration.rs
use alloy_primitives::{Address, U256};

#[test]
fn test_coordinated_response() {
    // Test interaction between contracts
    let threshold = U256::from(1000);
    let anomaly_value = U256::from(1500);

    // Simulate anomaly detection
    assert!(anomaly_value > threshold);

    // Simulate circuit breaker trip
    let mut is_tripped = false;
    is_tripped = true;
    assert!(is_tripped);
}
```

### Integration Test Scenarios

Test these workflows:

1. **Detection → Circuit Breaker**
   - Anomaly triggers breaker
   - Breaker prevents operations

2. **Detection → Alert Registry**
   - Anomaly logs alert
   - Alert data is accurate

3. **Full Workflow**
   - Detection → Breaker → Alert
   - All events emitted correctly

## Property-Based Testing

### Using proptest

Add property-based tests for invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn threshold_invariant(
        threshold in 0u64..1000000,
        value in 0u64..1000000
    ) {
        let t = U256::from(threshold);
        let v = U256::from(value);

        let is_anomaly = v > t;
        let expected = value > threshold;

        assert_eq!(is_anomaly, expected);
    }
}
```

### Invariants to Test

For DetectionEngine:
- `current_value > threshold` ⟺ anomaly
- `metric_count` ≥ number of registered metrics
- Owner is never zero address after initialization

For CircuitBreaker:
- `trip_count` always increases
- `is_tripped` ⟺ last operation was trip (not reset)
- `last_trip_time` ≤ current block timestamp

For AlertRegistry:
- `alert_count` = length of alerts array
- All alert timestamps ≤ current block timestamp
- No alert has zero source address

## Manual Testing

### Local Testnet Setup

Use Arbitrum local testnode:

```bash
# Clone nitro-testnode
git clone https://github.com/OffchainLabs/nitro-testnode.git
cd nitro-testnode

# Start local node
./test-node.bash --init

# In another terminal, deploy contracts
cd ../arbishield
export RPC_URL=http://localhost:8547
./scripts/deploy.sh
```

### Testnet Deployment

Deploy to Arbitrum Sepolia:

```bash
# Configure for Sepolia
export RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
export PRIVATE_KEY=your_key_here

# Deploy
./scripts/deploy.sh
```

### Manual Test Scenarios

#### Scenario 1: Register and Check Metrics

```javascript
const engine = new ethers.Contract(address, abi, signer);

// Register metric
await engine.register_metric(1, 1000);

// Report normal value
await engine.report_metric(1, 500);
let isAnomaly = await engine.check_anomaly(1);
console.log("Anomaly (should be false):", isAnomaly);

// Report anomaly
await engine.report_metric(1, 1500);
isAnomaly = await engine.check_anomaly(1);
console.log("Anomaly (should be true):", isAnomaly);
```

#### Scenario 2: Trip and Reset Circuit Breaker

```javascript
const breaker = new ethers.Contract(address, abi, signer);

// Check initial state
let isActive = await breaker.is_active();
console.log("Initially active:", isActive); // false

// Trip
await breaker.trip();
isActive = await breaker.is_active();
console.log("After trip:", isActive); // true

// Try to trip again (should fail)
try {
  await breaker.trip();
  console.error("Should have failed!");
} catch (e) {
  console.log("Correctly rejected double trip");
}

// Reset
await breaker.reset();
isActive = await breaker.is_active();
console.log("After reset:", isActive); // false
```

#### Scenario 3: Log Alerts

```javascript
const registry = new ethers.Contract(address, abi, signer);

// Register alerts
await registry.register_alert(
  10,  // high severity
  ethers.constants.AddressZero, // source (example)
  ethers.utils.keccak256(ethers.utils.toUtf8Bytes("Test alert"))
);

// Query alert
const count = await registry.get_alert_count();
console.log("Total alerts:", count.toString());

const alert = await registry.get_alert(0);
console.log("Alert 0:", {
  timestamp: alert[0].toString(),
  severity: alert[1],
  source: alert[2],
  hash: alert[3]
});
```

## Performance Testing

### Gas Profiling

Profile gas usage:

```bash
# Build with gas reporting
RUSTFLAGS="-C instrument-coverage" cargo test

# Analyze gas costs
cargo stylus check --verbose
```

### Benchmark Tests

Create benchmarks:

```rust
#[bench]
fn bench_metric_registration(b: &mut Bencher) {
    b.iter(|| {
        // Benchmark metric registration
    });
}
```

### Load Testing

Test with high volume:

```javascript
// Register 1000 metrics
for (let i = 0; i < 1000; i++) {
  await engine.register_metric(i, 1000);
}

// Measure gas costs
const receipt = await engine.report_metric(500, 1500);
console.log("Gas used:", receipt.gasUsed.toString());
```

## Continuous Integration

### GitHub Actions

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.81
          components: rustfmt, clippy
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt --check
```

## Test Checklist

Before deployment:

### Unit Tests
- [ ] All public functions tested
- [ ] Error cases covered
- [ ] Edge cases handled
- [ ] Invariants verified

### Integration Tests
- [ ] Contract interactions work
- [ ] Events emitted correctly
- [ ] Cross-contract calls succeed
- [ ] Error propagation correct

### Manual Tests
- [ ] Deployed to testnet
- [ ] All functions callable
- [ ] Events visible on explorer
- [ ] Gas costs acceptable

### Security Tests
- [ ] Ownership checks enforced
- [ ] Zero address validation works
- [ ] Unauthorized calls rejected
- [ ] State transitions valid

## Debugging

### Common Issues

#### Test Fails: "No such file or directory"

Ensure you're in the project root:
```bash
cd /path/to/arbishield
cargo test
```

#### Test Fails: "Cannot find package"

Update dependencies:
```bash
cargo update
cargo build
```

#### Integration Test Fails

Check contract addresses and ABI:
```bash
cargo clean
cargo build --release
```

### Debug Output

Add debug output to tests:

```rust
#[test]
fn test_with_debug() {
    let value = U256::from(100);
    println!("Value: {:?}", value);
    assert_eq!(value, U256::from(100));
}
```

Run with output:
```bash
cargo test -- --nocapture
```

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [proptest Guide](https://altsysrq.github.io/proptest-book/intro.html)
- [Stylus Testing Guide](https://docs.arbitrum.io/stylus/testing)

---

For more information, see [ARCHITECTURE.md](./ARCHITECTURE.md) and [DEPLOYMENT.md](./DEPLOYMENT.md).
