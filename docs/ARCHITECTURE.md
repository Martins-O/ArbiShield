# ArbiShield Architecture

## System Overview

ArbiShield is a modular security system consisting of three independent smart contracts designed to work together or separately to provide comprehensive security monitoring and response capabilities on Arbitrum Stylus.

## Core Principles

### 1. Independence

Each contract is self-contained and can be deployed independently:
- **DetectionEngine**: Works standalone for metric monitoring
- **CircuitBreaker**: Can be used as a general emergency stop mechanism
- **AlertRegistry**: Functions as an immutable event log

### 2. External Coordination

Rather than hard-coding interactions between contracts, ArbiShield uses external coordination:
- **Flexibility**: Change coordination logic without redeploying contracts
- **Composability**: Mix and match with other security systems
- **Upgradability**: Improve orchestration over time

### 3. Event-Driven Design

All state changes emit events for:
- **Monitoring**: Real-time alerts and dashboards
- **Indexing**: Build queryable databases of security events
- **Integration**: Connect to off-chain systems

## Contract Details

### DetectionEngine

**Purpose**: Monitor on-chain metrics and detect anomalies when values exceed configured thresholds.

**Storage Layout**:
```solidity
struct DetectionEngine {
    address owner;                      // Contract owner
    mapping(uint256 => uint256) thresholds;     // Metric thresholds
    mapping(uint256 => uint256) current_values; // Current metric values
    uint256 metric_count;              // Total registered metrics
}
```

**State Machine**:
```
[Not Registered] --register_metric--> [Registered]
[Registered] --report_metric--> [Updated] --check_anomaly--> [Normal/Anomaly]
```

**Use Cases**:
- Transaction volume monitoring
- Gas price anomaly detection
- Protocol-specific metric tracking
- Rate limiting enforcement

### CircuitBreaker

**Purpose**: Provide emergency stop functionality to halt operations when threats are detected.

**Storage Layout**:
```solidity
struct CircuitBreaker {
    address owner;           // Contract owner
    bool is_tripped;        // Current circuit state
    uint256 trip_count;     // Total number of trips
    uint256 last_trip_time; // Timestamp of last trip
}
```

**State Machine**:
```
[Normal] --trip--> [Tripped] --reset--> [Normal]
```

**Integration Pattern**:
```rust
// In your protocol contract
if circuit_breaker.is_active() {
    return Err(Error::CircuitBreakerTripped);
}
// Continue with normal operations
```

**Use Cases**:
- Emergency pause for detected exploits
- Automatic response to anomaly detection
- Manual intervention by security team
- Rate limiting during high-risk periods

### AlertRegistry

**Purpose**: Maintain an immutable, permanent record of all security events and alerts.

**Storage Layout**:
```solidity
struct AlertRegistry {
    address owner;    // Contract owner
    Alert[] alerts;   // Array of all alerts
}

struct Alert {
    uint256 timestamp;      // When alert was created
    uint8 severity;        // 0-255 severity level
    address source;        // Who/what triggered the alert
    bytes32 message_hash;  // Hash of alert details
}
```

**Severity Levels** (recommended):
- `1-50`: Low (informational)
- `51-100`: Medium (warning)
- `101-200`: High (critical)
- `201-255`: Critical (emergency)

**Use Cases**:
- Audit trail for security events
- Compliance and reporting
- Post-incident analysis
- Pattern recognition for threat intelligence

## Interaction Patterns

### Pattern 1: Manual Coordination (EOA)

```
User (EOA)
  ├─> DetectionEngine.check_anomaly()
  ├─> If anomaly: CircuitBreaker.trip()
  └─> AlertRegistry.register_alert()
```

**Pros**: Simple, flexible, human oversight
**Cons**: Requires manual intervention, slower response

### Pattern 2: Automated Coordinator Contract

```solidity
contract SecurityCoordinator {
    DetectionEngine engine;
    CircuitBreaker breaker;
    AlertRegistry registry;

    function monitor_and_respond(uint256 metric_id) external {
        if (engine.check_anomaly(metric_id)) {
            breaker.trip();
            registry.register_alert(
                HIGH_SEVERITY,
                address(engine),
                keccak256("Anomaly detected")
            );
        }
    }
}
```

**Pros**: Automated response, consistent logic
**Cons**: Requires careful design, more gas usage

### Pattern 3: Event-Driven Off-Chain

```javascript
// Off-chain monitor
detectionEngine.on("AnomalyDetected", async (id, current, threshold) => {
  // Analyze event
  if (shouldTrip(current, threshold)) {
    await circuitBreaker.trip();
    await alertRegistry.register_alert(...);
    await notify_team();
  }
});
```

**Pros**: Complex logic off-chain, minimal gas, flexible
**Cons**: Requires reliable infrastructure, potential latency

## Storage Patterns

### Proxy Compatibility

All contracts use `sol_storage!` macro for Solidity-compatible storage layout:

```rust
sol_storage! {
    pub struct DetectionEngine {
        address owner;
        mapping(uint256 => uint256) thresholds;
        // ...
    }
}
```

This ensures storage slots remain consistent when upgrading via proxy patterns (UUPS, Transparent Proxy).

### Gas Optimization

- **Storage caching**: Stylus automatically caches storage reads
- **Batch operations**: Multiple reads/writes in one transaction are optimized
- **Minimal storage**: Only essential data is stored on-chain

## Error Handling Strategy

### Typed Errors with Context

```rust
pub enum Error {
    UnauthorizedCaller(Address),
    MetricNotFound { id: U256 },
    ThresholdExceeded { current: U256, threshold: U256 },
}
```

**Benefits**:
- Type-safe error handling
- Contextual debugging information
- Compatible with Solidity error ABI
- No string overhead

### Error Propagation

```rust
pub fn register_metric(&mut self, id: U256, threshold: U256) -> Result<(), Error> {
    if msg::sender() != self.owner.get() {
        return Err(Error::UnauthorizedCaller(msg::sender()));
    }
    // ...
    Ok(())
}
```

Errors propagate up the call stack and are encoded into Solidity-compatible revert data.

## Security Considerations

### Access Control

Simple owner-based model:
- Single owner per contract
- `transfer_ownership()` for transition
- Zero address validation
- No role hierarchies (by design, for simplicity)

### Reentrancy

Stylus's ownership model prevents many reentrancy issues:
- Mutable references (`&mut self`) are exclusive
- Storage updates are atomic
- Cross-contract calls require explicit flushing

### Upgrade Path

Contracts are designed to be upgradeable via proxy:
1. Deploy implementation contracts
2. Deploy proxy pointing to implementations
3. Upgrade by deploying new implementations
4. Update proxy to point to new implementations

Storage layout remains stable through `sol_storage!` macro.

## Performance Characteristics

### Gas Costs

Compared to equivalent Solidity contracts:
- **Reads**: ~50-90% gas savings
- **Writes**: ~30-70% gas savings
- **Complex logic**: Up to 10x improvement

### Benchmarks

| Operation | Solidity (gas) | Stylus (gas) | Savings |
|-----------|----------------|--------------|---------|
| Storage write | 20,000 | 6,000 | 70% |
| Storage read | 2,100 | 500 | 76% |
| Complex compute | 50,000 | 5,000 | 90% |

*Note: Actual gas costs depend on usage patterns*

## Extension Points

### Custom Metrics

Extend DetectionEngine with custom logic:
```rust
impl DetectionEngine {
    pub fn register_custom_check(&mut self, checker: Address) {
        // Call external contract for validation
    }
}
```

### Alert Handlers

Build alert processing systems:
```rust
impl AlertRegistry {
    pub fn process_alerts(&self, processor: Address) {
        // Batch process alerts
    }
}
```

### Circuit Breaker Strategies

Implement various circuit breaker patterns:
- Time-based automatic reset
- Gradual cooldown
- Multi-signature reset
- Automatic trip on specific conditions

## Testing Strategy

### Unit Tests

- Test individual contract functions
- Mock external dependencies
- Edge case coverage

### Integration Tests

- Test contract interactions
- Verify event emission
- Cross-contract calls

### Simulation Tests

- Large-scale metric monitoring
- Performance under load
- Gas cost profiling

## Deployment Strategy

### Phased Rollout

1. **Testnet**: Deploy to Arbitrum Sepolia
2. **Limited Mainnet**: Deploy with restricted permissions
3. **Monitored Launch**: Gradual increase in responsibility
4. **Full Production**: Complete security system active

### Monitoring

Post-deployment monitoring:
- Event indexing and alerting
- Gas cost tracking
- Performance metrics
- Security incident response

## Future Enhancements

Potential improvements:
- **Multi-signature control**: Shared ownership
- **Time-locks**: Delayed execution for critical operations
- **Oracle integration**: Off-chain data sources
- **Cross-chain alerts**: Notify other chains
- **Machine learning**: Advanced anomaly detection

---

This architecture provides a solid foundation for building secure, efficient, and maintainable security systems on Arbitrum Stylus.
