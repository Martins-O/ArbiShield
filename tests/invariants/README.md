# ArbiShield Invariant Tests

Comprehensive invariant testing suite that verifies critical properties which must **ALWAYS** hold true across all ArbiShield security contracts.

## Overview

Invariant tests are formal verification techniques that check whether fundamental correctness properties remain true regardless of the sequence of operations performed on a contract.

### What Makes Invariant Tests Different?

| Test Type | What it Checks | Example |
|-----------|----------------|---------|
| **Unit Test** | Specific function behavior | `trip()` sets `is_tripped = true` |
| **Integration Test** | Multi-contract workflows | Anomaly → Alert → Circuit trip |
| **Property Test** | General properties across random inputs | Priority always in [0, 3] |
| **Invariant Test** | Properties that NEVER change | Trip count NEVER decreases |

## Invariants Tested

### 📊 Summary: 16 Total Invariants

- **CircuitBreaker**: 5 invariants
- **DetectionEngine**: 4 invariants
- **AlertRegistry**: 4 invariants
- **Cross-Contract**: 3 invariants

---

## Circuit Breaker Invariants

### INV-CB-1: Trip Count Monotonicity
```rust
∀ operations: trip_count(t+1) >= trip_count(t)
```

**Security Importance**: Trip count serves as an immutable audit trail. If it could decrease, attackers could hide evidence of repeated attacks by manipulating the count.

**Test Coverage**:
- ✅ Basic monotonicity check
- ✅ Persistence across resets
- ✅ 100 random operation sequence
- ✅ Property-based (1000 iterations)

---

### INV-CB-2: Trip Idempotency
```rust
is_tripped = true ⟹ trip() returns Error
```

**Security Importance**: Prevents double-counting of attacks and ensures state machine integrity. Without this, an attacker could artificially inflate trip counts by repeatedly calling `trip()`.

**Test Coverage**:
- ✅ Single double-trip attempt
- ✅ Multiple rapid trip attempts
- ✅ State unchanged on failed trip

---

### INV-CB-3: Reset State Guard
```rust
is_tripped = false ⟹ reset() returns Error
```

**Security Importance**: Prevents state machine corruption. If reset could be called when not tripped, it could lead to undefined behavior or bypass safety checks.

**Test Coverage**:
- ✅ Reset when never tripped
- ✅ Reset after reset fails
- ✅ State consistency maintained

---

### INV-CB-4: Count Persistence
```rust
reset() ⟹ trip_count unchanged
```

**Security Importance**: Trip count is an immutable audit log. Resetting should only clear the "tripped" flag, not the historical count. Critical for detecting repeated attacks.

**Test Coverage**:
- ✅ 3 trip/reset cycles
- ✅ 10 cycle accumulation test
- ✅ Count strictly increments

---

### INV-CB-5: Timestamp Monotonicity
```rust
∀ operations: last_trip_time(t+1) >= last_trip_time(t)
```

**Security Importance**: Timestamps provide temporal ordering for security events. If timestamps could go backward, it would break forensic analysis and could hide attack patterns.

**Test Coverage**:
- ✅ Increasing timestamp sequence
- ✅ Persistence across reset
- ✅ Three-trip monotonicity

---

## Detection Engine Invariants

### INV-DE-1: Threshold Validity
```rust
∀ thresholds: 0 <= threshold <= U256::MAX
```

**Security Importance**: Thresholds define security boundaries. If they could be set to invalid values, all anomaly detection would fail.

**Test Coverage**:
- ✅ Various threshold values (0, 100, 1M, MAX)
- ✅ Immutability after registration
- ✅ Unchanged by metric reports

---

### INV-DE-2: Deterministic Detection
```rust
check_anomaly(id) at time t = check_anomaly(id) at time t+1
```

**Security Importance**: Non-deterministic detection would allow attackers to bypass security by repeatedly triggering detection until it gives a favorable result.

**Test Coverage**:
- ✅ 100 repeated checks
- ✅ Multiple metrics, same threshold/value
- ✅ Property-based (1000 iterations)

---

### INV-DE-3: Metric Count Monotonicity
```rust
∀ operations: metric_count(t+1) >= metric_count(t)
```

**Security Importance**: Metric count is used for ID generation and audit trails. If it could decrease, metric IDs would collide and tracking would fail.

**Test Coverage**:
- ✅ 100 registrations
- ✅ Sequential increment verification
- ✅ Never decreases across operations

---

### INV-DE-4: Anomaly Detection Correctness
```rust
is_anomaly(value, threshold) ⟺ value > threshold
```

**Security Importance**: This is the core security property. Any violation means either false positives (blocking legitimate traffic) or false negatives (missing real attacks).

**Test Coverage**:
- ✅ Values below threshold (no anomaly)
- ✅ Values above threshold (anomaly)
- ✅ Boundary conditions (equal, just over)
- ✅ Property-based (1000 iterations)

---

## Alert Registry Invariants

### INV-AR-1: Alert ID Monotonicity
```rust
alert_id(i+1) = alert_id(i) + 1 ∧ alert_id(0) = 1
```

**Security Importance**: Alert IDs serve as unique identifiers in audit logs. If IDs could repeat or decrease, alerts would be indistinguishable and security incidents could be hidden.

**Test Coverage**:
- ✅ 100 sequential registrations
- ✅ First ID is 1
- ✅ No gaps in sequence
- ✅ Property-based (1000 iterations)

---

### INV-AR-2: Acknowledgment Immutability
```rust
acknowledged(id) = true ⟹ ∀ future operations: acknowledged(id) = true
```

**Security Importance**: Acknowledgment represents a security action (incident response). If it could be reversed, attackers could hide evidence of breaches.

**Test Coverage**:
- ✅ Double acknowledgment fails
- ✅ State remains acknowledged
- ✅ 10 alerts, 100 operations each
- ✅ Property-based (1000 iterations)

---

### INV-AR-3: Alert Count Monotonicity
```rust
∀ operations: alert_count(t+1) >= alert_count(t)
```

**Security Importance**: Alert count tracks total security events. If it could decrease, attack evidence could be erased.

**Test Coverage**:
- ✅ 100 registrations
- ✅ Acknowledgment doesn't affect count
- ✅ Property-based (1000 iterations)

---

### INV-AR-4: Priority Determinism
```rust
compute_priority(threat) = constant for given threat
```

**Security Importance**: Priority drives alert routing and response urgency. Non-deterministic priority could cause critical alerts to be deprioritized.

**Test Coverage**:
- ✅ All threat level ranges
- ✅ 10 repeated computations per level
- ✅ Monotonicity with threat level
- ✅ Property-based (1000 iterations)

---

## Cross-Contract Invariants

### INV-CC-1: Error Selector Consistency
```rust
selector(CB::UnauthorizedCaller) = selector(DE::UnauthorizedCaller) = selector(AR::UnauthorizedCaller)
```

**Security Importance**: Shared error types must have identical selectors across contracts. Different selectors would break tooling, frontends, and security monitoring systems.

**Test Coverage**:
- ✅ UnauthorizedCaller across CB/DE/AR
- ✅ InvalidOwner across CB/DE/AR
- ✅ Multiple address parameters
- ✅ Property-based (1000 iterations)

---

### INV-CC-2: Upgrade Compatibility
```rust
enhanced_alert_count >= v1_alert_count
```

**Security Importance**: When upgrading from V1 to V2, no alerts should be lost. Enhanced alert count must always be >= V1 count to ensure audit trail completeness.

**Test Coverage**:
- ✅ Simulated upgrade from 100 V1 alerts
- ✅ V2 registrations preserve baseline
- ✅ Property-based (1000 iterations)

---

### INV-CC-3: Priority Distribution Completeness
```rust
Σ(priority_counts[i]) = total_alerts
```

**Security Importance**: Every alert must be categorized into exactly one priority level. Missing alerts (sum < total) or double-counting (sum > total) indicates state corruption.

**Test Coverage**:
- ✅ Simulated priority counts
- ✅ 8 diverse threat levels
- ✅ Property-based (1000 iterations)
- ✅ Workflow coherence tests

---

## Running Invariant Tests

### Quick Start

```bash
# Run all invariant tests
cargo test --test invariants

# Run with detailed output
cargo test --test invariants -- --nocapture

# Run specific contract
cargo test --test invariants circuit_breaker
cargo test --test invariants detection_engine
cargo test --test invariants alert_registry
cargo test --test invariants cross_contract
```

### Run Individual Invariants

```bash
# CircuitBreaker invariants
cargo test --test invariants invariant_cb1
cargo test --test invariants invariant_cb2
cargo test --test invariants invariant_cb3
cargo test --test invariants invariant_cb4
cargo test --test invariants invariant_cb5

# DetectionEngine invariants
cargo test --test invariants invariant_de1
cargo test --test invariants invariant_de2
cargo test --test invariants invariant_de3
cargo test --test invariants invariant_de4

# AlertRegistry invariants
cargo test --test invariants invariant_ar1
cargo test --test invariants invariant_ar2
cargo test --test invariants invariant_ar3
cargo test --test invariants invariant_ar4

# Cross-contract invariants
cargo test --test invariants invariant_cc1
cargo test --test invariants invariant_cc2
cargo test --test invariants invariant_cc3
```

### Stress Tests

```bash
# Run 1000+ operation stress tests
cargo test --test invariants stress_test -- --nocapture
```

### Property-Based Tests

```bash
# Run property-based invariant tests (1000 iterations each)
cargo test --test invariants property_ -- --nocapture
```

## Test Statistics

- **Total Test Functions**: 50+
- **Property-Based Iterations**: 1000 per test
- **Stress Test Operations**: 1000 per contract
- **Total Test Executions**: 50,000+ per run

## Understanding Violations

When an invariant is violated, you'll see output like:

```text
thread 'invariant_cb1_trip_count_monotonic' panicked at
tests/invariants/circuit_breaker_invariants.rs:42:9:

INV-CB-1 VIOLATED: Trip count decreased from 5 to 4

SECURITY IMPLICATION: Trip count serves as an audit trail. If it could
decrease, attackers could hide evidence of repeated attacks.
```

### Violation Report Contains:
1. **Invariant ID** (e.g., INV-CB-1)
2. **Violation Description** (what went wrong)
3. **Current vs Expected State**
4. **Security Implication** (why it matters)
5. **File and Line Number**

## Continuous Integration

Invariant tests run automatically on:

- ✅ Every pull request
- ✅ Every commit to `main` and `indev`
- ✅ Nightly builds
- ✅ Pre-release checks

## Best Practices

### Writing New Invariants

1. **Identify the Property**: What must ALWAYS be true?
2. **Document Security Importance**: Why does this matter?
3. **Write Basic Test**: Test the happy path
4. **Write Violation Test**: Test what happens when violated
5. **Add Property Test**: Random inputs (1000+ iterations)
6. **Add Stress Test**: 1000+ operations

### Example Template

```rust
/// INV-XX-N: [One-line description]
///
/// **Security Importance**: [Why this invariant is critical]
#[test]
fn invariant_xx_n_description() {
    // Setup
    let mut contract = setup();

    // Perform operations
    contract.some_operation();

    // Check invariant
    assert!(
        invariant_holds,
        "INV-XX-N VIOLATED: [Description]"
    );
}
```

## Related Testing

This invariant suite complements:

- **Unit Tests** (113 tests): Basic functionality
- **Integration Tests** (38 tests): Multi-contract workflows
- **Property Tests** (54 tests): Random input properties
- **Fuzz Tests** (7 fuzzers): Coverage-guided edge cases

## Further Reading

- [Invariant Testing in Solidity](https://book.getfoundry.sh/forge/invariant-testing)
- [Property-Based Testing](https://hypothesis.works/articles/what-is-property-based-testing/)
- [Formal Verification](https://en.wikipedia.org/wiki/Formal_verification)
- [Smart Contract Security](https://ethereum.org/en/developers/docs/smart-contracts/security/)
