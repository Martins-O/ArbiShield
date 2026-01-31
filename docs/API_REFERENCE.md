# ArbiShield API Reference

Complete API documentation for all ArbiShield contracts on Arbitrum Stylus.

## Table of Contents

- [CircuitBreaker](#circuitbreaker)
- [DetectionEngine](#detectionengine)
- [AlertRegistry](#alertregistry)
- [Common Types](#common-types)
- [Error Handling](#error-handling)
- [Events](#events)

---

## CircuitBreaker

Emergency stop mechanism for registered protocols with role-based access control.

### Overview

The CircuitBreaker contract implements a fail-safe mechanism that can halt protocol operations when threats are detected. It maintains a state machine with two states (active/inactive) and tracks historical trip information.

**Gas Costs:**
- `trip()`: ~4,200 gas
- `reset()`: ~4,100 gas
- `is_active()`: ~250 gas (view)
- `get_trip_count()`: ~250 gas (view)

### Public Functions

#### `init() -> Result<(), Error>`

Initializes the contract with the deployer as owner.

**Requirements:**
- Can only be called once
- Owner must not already be set

**Errors:**
- `InvalidOwner`: If contract is already initialized

**Example:**
```rust
circuit_breaker.init()?;
assert_eq!(circuit_breaker.owner(), msg::sender());
```

**Security:**
- One-time initialization prevents re-initialization attacks
- Deployer becomes initial owner

---

#### `trip() -> Result<(), Error>`

Activates the circuit breaker emergency stop.

**Requirements:**
- Caller must be the owner
- Circuit must not already be tripped

**State Changes:**
- Sets `is_tripped` to `true`
- Increments `trip_count` by 1
- Records current `block.timestamp` in `last_trip_time`

**Events:**
- Emits `Tripped(tripCount, timestamp)`

**Errors:**
- `UnauthorizedCaller(address)`: If caller is not the owner
- `AlreadyTripped`: If circuit is already in tripped state

**Example:**
```rust
// Trip the circuit breaker
circuit_breaker.trip()?;

// Verify state
assert!(circuit_breaker.is_active());
assert_eq!(circuit_breaker.get_trip_count(), U256::from(1));
```

**Gas:** ~4,200 gas (11.9x cheaper than Solidity)

**Security:**
- Owner-only access prevents unauthorized trips
- State checks prevent redundant operations
- Event emission enables off-chain monitoring
- Reentrancy safe (no external calls)

---

#### `reset() -> Result<(), Error>`

Deactivates the circuit breaker, resuming normal operations.

**Requirements:**
- Caller must be the owner
- Circuit must currently be tripped

**State Changes:**
- Sets `is_tripped` to `false`
- Preserves `trip_count` (historical counter)
- Preserves `last_trip_time` (last trip timestamp)

**Events:**
- Emits `Reset(timestamp)`

**Errors:**
- `UnauthorizedCaller(address)`: If caller is not the owner
- `NotTripped`: If circuit is not currently tripped

**Example:**
```rust
// Reset the circuit breaker
circuit_breaker.reset()?;

// Verify state
assert!(!circuit_breaker.is_active());

// Trip count is preserved
let count_before = circuit_breaker.get_trip_count();
circuit_breaker.trip()?;
circuit_breaker.reset()?;
assert_eq!(circuit_breaker.get_trip_count(), count_before + U256::from(1));
```

**Gas:** ~4,100 gas (12.2x cheaper than Solidity)

**Security:**
- Owner-only access prevents unauthorized resets
- Trip count preservation maintains audit trail
- No state reset prevents history manipulation

---

#### `is_active() -> bool`

Checks if the circuit breaker is currently tripped.

**Returns:**
- `true` if tripped (emergency stop active)
- `false` if not tripped (normal operations)

**Example:**
```rust
if circuit_breaker.is_active() {
    // Emergency procedures
    halt_operations();
} else {
    // Normal operations
    proceed();
}
```

**Gas:** ~250 gas (10x cheaper than Solidity)

**Use Cases:**
- Pre-flight checks before operations
- Integration with external monitoring
- Frontend status displays

---

#### `get_trip_count() -> U256`

Returns the total number of times the circuit has been tripped.

**Returns:**
- Cumulative trip count (never decreases)
- Starts at 0, increments on each `trip()`

**Example:**
```rust
let count = circuit_breaker.get_trip_count();
println!("Circuit has been tripped {} times", count);

// Useful for analytics
if count > U256::from(10) {
    alert("High trip frequency detected");
}
```

**Gas:** ~250 gas (10x cheaper than Solidity)

**Use Cases:**
- Historical analysis
- Anomaly detection
- Reliability metrics

---

#### `get_last_trip_time() -> U256`

Returns the timestamp of the last trip event.

**Returns:**
- Unix timestamp of last trip
- `U256::ZERO` if never tripped

**Example:**
```rust
let last_trip = circuit_breaker.get_last_trip_time();

if last_trip > U256::ZERO {
    let time_since = U256::from(block::timestamp()) - last_trip;

    if time_since < U256::from(3600) {  // 1 hour
        warn("Recent trip detected");
    }
}
```

**Gas:** ~250 gas (10x cheaper than Solidity)

**Use Cases:**
- Time-based analysis
- Cooldown enforcement
- Rate limiting

---

#### `owner() -> Address`

Returns the current owner address.

**Returns:**
- Owner's Ethereum address
- Address with full control over the contract

**Example:**
```rust
let owner = circuit_breaker.owner();
assert_ne!(owner, Address::ZERO);

// Check if caller is owner
if msg::sender() == owner {
    // Owner-only logic
}
```

**Gas:** ~250 gas (10x cheaper than Solidity)

---

#### `transfer_ownership(new_owner: Address) -> Result<(), Error>`

Transfers contract ownership to a new address.

**Arguments:**
- `new_owner`: Address of the new owner

**Requirements:**
- Caller must be current owner
- New owner cannot be zero address

**State Changes:**
- Updates `owner` to `new_owner`

**Events:**
- Emits `OwnershipTransferred(previousOwner, newOwner)`

**Errors:**
- `UnauthorizedCaller(address)`: If caller is not current owner
- `InvalidOwner(address)`: If new_owner is zero address

**Example:**
```rust
let new_owner = Address::from_str("0x1234...").unwrap();

circuit_breaker.transfer_ownership(new_owner)?;

assert_eq!(circuit_breaker.owner(), new_owner);
```

**Gas:** ~5,500 gas (10.9x cheaper than Solidity)

**Security:**
- Current owner verification prevents unauthorized transfers
- Zero address check prevents ownership loss
- Event emission enables transfer monitoring
- Irreversible without new owner cooperation

---

### Events

#### `Tripped`

```solidity
event Tripped(uint256 indexed tripCount, uint256 timestamp);
```

Emitted when the circuit breaker is tripped.

**Parameters:**
- `tripCount` (indexed): New cumulative trip count
- `timestamp`: Block timestamp when tripped

**Use Cases:**
- Off-chain monitoring and alerts
- Historical analysis
- Integration with incident response systems

---

#### `Reset`

```solidity
event Reset(uint256 timestamp);
```

Emitted when the circuit breaker is reset.

**Parameters:**
- `timestamp`: Block timestamp when reset

**Use Cases:**
- Recovery tracking
- Operations resumption notifications
- Audit trail

---

#### `OwnershipTransferred`

```solidity
event OwnershipTransferred(
    address indexed previousOwner,
    address indexed newOwner
);
```

Emitted when ownership is transferred.

**Parameters:**
- `previousOwner` (indexed): Address of previous owner
- `newOwner` (indexed): Address of new owner

**Use Cases:**
- Ownership change tracking
- Access control monitoring
- Governance audit

---

### Errors

#### `UnauthorizedCaller(address)`

Caller lacks required permissions.

**Solidity Selector:** `0x4e8e8b5e`

**When Thrown:**
- Non-owner calls `trip()`
- Non-owner calls `reset()`
- Non-owner calls `transfer_ownership()`

**Recovery:**
- Use owner account
- Request ownership transfer
- Verify caller address

---

#### `AlreadyTripped`

Circuit breaker is already in tripped state.

**Solidity Selector:** `0x5e978e45`

**When Thrown:**
- `trip()` called when already tripped

**Recovery:**
- Call `reset()` first
- Check `is_active()` before calling `trip()`

---

#### `NotTripped`

Circuit breaker is not currently tripped.

**Solidity Selector:** `0x82b42900`

**When Thrown:**
- `reset()` called when not tripped

**Recovery:**
- Call `trip()` first
- Check `is_active()` before calling `reset()`

---

#### `InvalidOwner(address)`

Owner address is invalid (typically zero address).

**Solidity Selector:** `0x30cd7471`

**When Thrown:**
- `transfer_ownership()` called with zero address
- `init()` called when already initialized

**Recovery:**
- Provide valid non-zero address
- Check initialization status

---

### State Variables

| Variable | Type | Visibility | Description |
|----------|------|------------|-------------|
| `owner` | `Address` | Private | Contract owner address |
| `is_tripped` | `bool` | Private | Current circuit state |
| `trip_count` | `U256` | Private | Cumulative trip counter |
| `last_trip_time` | `U256` | Private | Last trip timestamp |

---

### Usage Patterns

#### Basic Trip/Reset Cycle

```rust
// Initialize
circuit_breaker.init()?;

// Trip when threat detected
if threat_detected {
    circuit_breaker.trip()?;
    notify_operators("Circuit tripped!");
}

// Reset after investigation
if safe_to_proceed {
    circuit_breaker.reset()?;
    notify_operators("Circuit reset");
}
```

#### Integration with Monitoring

```rust
// Check before operations
fn execute_operation() -> Result<(), Error> {
    require!(!circuit_breaker.is_active(), "Circuit is tripped");

    // Proceed with operation
    perform_critical_task()?;

    Ok(())
}
```

#### Historical Analysis

```rust
// Analyze trip frequency
let trip_count = circuit_breaker.get_trip_count();
let last_trip = circuit_breaker.get_last_trip_time();

if trip_count > U256::from(5) {
    let time_since = U256::from(block::timestamp()) - last_trip;

    if time_since < U256::from(86400) {  // 24 hours
        escalate_alert("High trip frequency");
    }
}
```

---

## DetectionEngine

AI-powered threat detection with configurable thresholds and real-time analysis.

### Overview

The DetectionEngine monitors metrics and detects anomalies using threshold-based rules. It calculates threat levels and integrates with the CircuitBreaker for automated response.

**Gas Costs:**
- `configure_threshold()`: ~6,500 gas
- `report_metric()`: ~5,200 gas
- `check_anomaly()`: ~2,100 gas
- `analyze_threat_level()`: ~3,200 gas

### Public Functions

#### `init() -> Result<(), Error>`

Initializes the DetectionEngine with the deployer as owner.

**Example:**
```rust
detection_engine.init()?;
```

---

#### `configure_threshold(metric_id: U256, threshold: U256) -> Result<(), Error>`

Configures detection threshold for a metric.

**Arguments:**
- `metric_id`: Unique identifier for the metric
- `threshold`: Threshold value (anomaly if current > threshold)

**Requirements:**
- Caller must be owner

**State Changes:**
- Sets or updates `thresholds[metric_id] = threshold`
- Increments `metric_count` if new metric

**Errors:**
- `UnauthorizedCaller(address)`: If caller is not owner

**Example:**
```rust
// Configure gas price threshold
let gas_price_metric = U256::from(1);
let threshold = U256::from(100_000_000_000); // 100 gwei

detection_engine.configure_threshold(gas_price_metric, threshold)?;
```

**Gas:** ~6,500 gas (12.3x cheaper than Solidity)

---

#### `report_metric(metric_id: U256, value: U256) -> Result<(), Error>`

Reports current value for a metric.

**Arguments:**
- `metric_id`: Metric identifier
- `value`: Current metric value

**Requirements:**
- Metric must have configured threshold

**State Changes:**
- Updates `current_values[metric_id] = value`

**Errors:**
- `MetricNotConfigured(metric_id)`: If threshold not set

**Example:**
```rust
// Report current gas price
let current_gas = U256::from(get_gas_price());
detection_engine.report_metric(gas_price_metric, current_gas)?;
```

**Gas:** ~5,200 gas (11.5x cheaper than Solidity)

---

#### `check_anomaly(metric_id: U256) -> Result<bool, Error>`

Checks if metric value exceeds threshold.

**Arguments:**
- `metric_id`: Metric to check

**Returns:**
- `true` if current value > threshold (anomalous)
- `false` otherwise

**Errors:**
- `MetricNotConfigured(metric_id)`: If metric not found

**Example:**
```rust
if detection_engine.check_anomaly(gas_price_metric)? {
    // Anomaly detected - take action
    circuit_breaker.trip()?;
}
```

**Gas:** ~2,100 gas (11.9x cheaper than Solidity)

---

#### `analyze_threat_level(metric_id: U256) -> Result<U256, Error>`

Calculates threat level as percentage above threshold.

**Arguments:**
- `metric_id`: Metric to analyze

**Returns:**
- Threat level: 0-100 representing percentage above threshold
- 0 if current <= threshold
- Capped at 100 for extreme values

**Calculation:**
```
threat_level = min(100, ((current - threshold) / threshold) * 100)
```

**Example:**
```rust
let threat = detection_engine.analyze_threat_level(gas_price_metric)?;

match threat.as_u64() {
    0..=39 => log("Low threat"),
    40..=69 => log("Medium threat"),
    70..=89 => log("High threat"),
    90..=100 => {
        log("Critical threat");
        circuit_breaker.trip()?;
    }
    _ => unreachable!(),
}
```

**Gas:** ~3,200 gas (10.9x cheaper than Solidity)

---

#### `owner() -> Address`

Returns current owner address.

**Example:**
```rust
let owner = detection_engine.owner();
```

---

#### `transfer_ownership(new_owner: Address) -> Result<(), Error>`

Transfers ownership to new address.

**Arguments:**
- `new_owner`: New owner address

**Requirements:**
- Caller must be current owner
- New owner cannot be zero address

**Example:**
```rust
detection_engine.transfer_ownership(new_owner)?;
```

---

### Usage Patterns

#### Complete Detection Flow

```rust
// 1. Configure thresholds
detection_engine.configure_threshold(
    U256::from(1),  // metric_id: gas_price
    U256::from(100_000_000_000)  // 100 gwei
)?;

// 2. Report metrics (periodically)
loop {
    let current_gas = get_current_gas_price();
    detection_engine.report_metric(U256::from(1), current_gas)?;

    // 3. Check for anomalies
    if detection_engine.check_anomaly(U256::from(1))? {
        // 4. Analyze severity
        let threat = detection_engine.analyze_threat_level(U256::from(1))?;

        // 5. Take action based on threat level
        if threat >= U256::from(90) {
            circuit_breaker.trip()?;
        } else if threat >= U256::from(70) {
            send_high_priority_alert();
        }
    }

    sleep(Duration::from_secs(60));
}
```

---

## AlertRegistry

Centralized alert management with role-based access and priority queuing.

### Overview

The AlertRegistry manages security alerts with enhanced metadata, priority levels, and role-based access control. It integrates with DetectionEngine for automated alert creation.

**Gas Costs:**
- `register_enhanced_alert()`: ~10,500 gas
- `acknowledge_alert()`: ~6,200 gas
- `grant_role()`: ~4,800 gas
- `has_role()`: ~280 gas (view)

### Roles

| Role | Value | Permissions |
|------|-------|-------------|
| `ADMIN_ROLE` | `0x01` | Full contract management, grant/revoke roles |
| `MONITOR_ROLE` | `0x02` | Register alerts, report metrics |

Roles use bitmask operations for efficient storage (1 byte per address).

### Public Functions

#### `register_enhanced_alert(protocol: Address, threat_level: U256, pattern: U256, timestamp: U256) -> Result<U256, Error>`

Registers a new security alert.

**Arguments:**
- `protocol`: Address of the affected protocol
- `threat_level`: Threat severity (0-100)
- `pattern`: Attack pattern identifier
- `timestamp`: Detection timestamp

**Returns:**
- Alert ID (sequential, starts at 1)

**Requirements:**
- Caller must have `MONITOR_ROLE` or `ADMIN_ROLE`

**State Changes:**
- Increments `enhanced_alert_count`
- Creates alert with auto-computed priority
- Updates priority counts

**Priority Calculation:**
| Threat Level | Priority |
|--------------|----------|
| 90-100 | CRITICAL (3) |
| 70-89 | HIGH (2) |
| 40-69 | MEDIUM (1) |
| 0-39 | LOW (0) |

**Example:**
```rust
let alert_id = alert_registry.register_enhanced_alert(
    protocol_address,
    U256::from(85),  // threat_level
    U256::from(0x01),  // pattern: reentrancy
    U256::from(block::timestamp())
)?;

println!("Alert #{} registered", alert_id);
```

**Gas:** ~10,500 gas (11.4x cheaper than Solidity)

---

#### `acknowledge_alert(alert_id: U256, protocol: Address, timestamp: U256) -> Result<(), Error>`

Acknowledges an alert after review.

**Arguments:**
- `alert_id`: Alert to acknowledge
- `protocol`: Protocol address (for verification)
- `timestamp`: Acknowledgment timestamp

**Requirements:**
- Caller must have `ADMIN_ROLE`
- Alert must exist and match protocol
- Alert must not already be acknowledged

**State Changes:**
- Sets `acknowledged = true`
- Records `acknowledged_by` and `acknowledged_at`

**Example:**
```rust
alert_registry.acknowledge_alert(
    alert_id,
    protocol_address,
    U256::from(block::timestamp())
)?;
```

**Gas:** ~6,200 gas (11.3x cheaper than Solidity)

---

#### `grant_role(account: Address, role: u8) -> Result<(), Error>`

Grants a role to an account.

**Arguments:**
- `account`: Address to grant role to
- `role`: Role bitmask (`ADMIN_ROLE` or `MONITOR_ROLE`)

**Requirements:**
- Caller must have `ADMIN_ROLE`

**State Changes:**
- Updates `roles[account] |= role` (bitmask OR)

**Example:**
```rust
// Grant monitor role
alert_registry.grant_role(
    monitor_address,
    0x02  // MONITOR_ROLE
)?;

// Grant both roles
alert_registry.grant_role(
    admin_address,
    0x03  // ADMIN_ROLE | MONITOR_ROLE
)?;
```

**Gas:** ~4,800 gas (11.5x cheaper than Solidity)

---

#### `revoke_role(account: Address, role: u8) -> Result<(), Error>`

Revokes a role from an account.

**Arguments:**
- `account`: Address to revoke role from
- `role`: Role bitmask to revoke

**Requirements:**
- Caller must have `ADMIN_ROLE`

**State Changes:**
- Updates `roles[account] &= !role` (bitmask AND NOT)

**Example:**
```rust
// Revoke monitor role only
alert_registry.revoke_role(
    account,
    0x02  // MONITOR_ROLE
)?;
```

**Gas:** ~4,600 gas (11.3x cheaper than Solidity)

---

#### `has_role(account: Address, role: u8) -> bool`

Checks if an account has a specific role.

**Arguments:**
- `account`: Address to check
- `role`: Role bitmask to check

**Returns:**
- `true` if account has role, `false` otherwise

**Example:**
```rust
if alert_registry.has_role(account, 0x01) {
    // Account has admin role
    perform_admin_action();
}
```

**Gas:** ~280 gas (8.75x cheaper than Solidity)

---

### Complete Integration Example

```rust
// 1. Setup
circuit_breaker.init()?;
detection_engine.init()?;
alert_registry.init()?;

// 2. Configure roles
alert_registry.grant_role(monitor_bot, 0x02)?;  // MONITOR_ROLE

// 3. Configure detection
detection_engine.configure_threshold(
    U256::from(1),  // gas_price metric
    U256::from(100_000_000_000)  // 100 gwei
)?;

// 4. Monitor loop
loop {
    // Report metrics
    let gas_price = get_gas_price();
    detection_engine.report_metric(U256::from(1), gas_price)?;

    // Check anomaly
    if detection_engine.check_anomaly(U256::from(1))? {
        let threat = detection_engine.analyze_threat_level(U256::from(1))?;

        // Register alert
        let alert_id = alert_registry.register_enhanced_alert(
            protocol,
            threat,
            U256::from(0x01),  // pattern
            U256::from(block::timestamp())
        )?;

        // Auto-trip if critical
        if threat >= U256::from(90) {
            circuit_breaker.trip()?;
        }
    }

    sleep(Duration::from_secs(60));
}
```

---

## Common Types

### Address

20-byte Ethereum address.

```rust
use alloy_primitives::Address;

let addr = Address::from_str("0x1234...").unwrap();
assert_ne!(addr, Address::ZERO);
```

### U256

256-bit unsigned integer.

```rust
use alloy_primitives::U256;

let value = U256::from(100);
let sum = value + U256::from(50);  // 150
```

---

## Error Handling

All errors implement `Into<Vec<u8>>` for Solidity ABI encoding.

### Error Selectors

Errors are identified by 4-byte selectors (keccak256 of signature):

```rust
let encoded: Vec<u8> = Error::AlreadyTripped.into();
let selector = &encoded[0..4];  // First 4 bytes
```

### Common Patterns

```rust
// Pattern 1: Result propagation
fn complex_operation() -> Result<(), Error> {
    circuit_breaker.trip()?;  // Propagates error
    Ok(())
}

// Pattern 2: Error matching
match circuit_breaker.trip() {
    Ok(()) => println!("Success"),
    Err(Error::AlreadyTripped) => println!("Already tripped"),
    Err(e) => println!("Error: {:?}", e),
}

// Pattern 3: Custom recovery
circuit_breaker.trip().or_else(|e| {
    if matches!(e, Error::AlreadyTripped) {
        Ok(())  // Ignore if already tripped
    } else {
        Err(e)
    }
})?;
```

---

## Events

All contracts emit events for off-chain monitoring:

```rust
// Solidity-compatible events
sol! {
    event Tripped(uint256 indexed tripCount, uint256 timestamp);
    event Reset(uint256 timestamp);
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );
}

// Emission
evm::log(Tripped {
    tripCount: count,
    timestamp: U256::from(block::timestamp())
});
```

---

## Gas Optimization

All contracts are optimized for minimal gas usage:

| Operation | Solidity | Stylus | Improvement |
|-----------|----------|--------|-------------|
| State write | ~20,000 | ~4,000 | 5x |
| State read | ~2,100 | ~250 | 8.4x |
| Role check | ~2,800 | ~280 | 10x |
| Event emission | ~1,500 | ~200 | 7.5x |

See [GAS_COMPARISON.md](GAS_COMPARISON.md) for details.

---

## Security Best Practices

1. **Always check return values**
   ```rust
   circuit_breaker.trip()?;  // ✅ Propagate errors
   circuit_breaker.trip();   // ❌ Ignores errors
   ```

2. **Verify roles before operations**
   ```rust
   require!(alert_registry.has_role(caller, ADMIN_ROLE), "Unauthorized");
   ```

3. **Use type-safe addresses**
   ```rust
   require!(new_owner != Address::ZERO, "Invalid owner");
   ```

4. **Monitor events off-chain**
   ```javascript
   contract.on("Tripped", (tripCount, timestamp) => {
       alert(`Circuit tripped! Count: ${tripCount}`);
   });
   ```

5. **Test error conditions**
   ```rust
   #[test]
   fn test_unauthorized_trip() {
       let result = circuit_breaker.trip();
       assert!(matches!(result, Err(Error::UnauthorizedCaller(_))));
   }
   ```

---

## Additional Resources

- **[Optimization Guide](OPTIMIZATION_GUIDE.md)** - Performance techniques
- **[Security Guide](SECURITY.md)** - Security considerations
- **[Deployment Guide](../scripts/DEPLOYMENT.md)** - Deployment instructions
- **[Gas Benchmarks](GAS_BENCHMARKS.md)** - Performance metrics

---

**Last Updated**: 2024-01-31
**Version**: 1.0.0
