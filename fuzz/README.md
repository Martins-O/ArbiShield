# ArbiShield Fuzzing Tests

This directory contains coverage-guided fuzzing tests using `cargo-fuzz` and `libFuzzer` to discover edge cases and vulnerabilities in the ArbiShield smart contracts.

## Overview

The fuzzing suite tests critical contract logic through 7 specialized fuzzers:

1. **`alert_registry_priority`** - Priority computation from threat levels
2. **`alert_registry_errors`** - Error encoding for all 17 AlertRegistry error types
3. **`circuit_breaker_state`** - State machine transitions (trip/reset cycles)
4. **`detection_engine_anomaly`** - Anomaly detection logic and properties
5. **`cross_contract_errors`** - Error selector consistency across contracts
6. **`rbac_fuzzer`** - Role-based access control bitmask operations
7. **`arithmetic_fuzzer`** - Saturating arithmetic safety properties

## Installation

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Ensure you're using nightly Rust (required for cargo-fuzz)
rustup default nightly
```

## Running Fuzzers

### Run a single fuzzer

```bash
# Run for 60 seconds
cargo fuzz run alert_registry_priority -- -max_total_time=60

# Run with specific number of iterations
cargo fuzz run circuit_breaker_state -- -runs=1000000

# Run indefinitely until failure
cargo fuzz run rbac_fuzzer
```

### Run all fuzzers sequentially

```bash
./run_all_fuzzers.sh
```

### Parallel fuzzing

```bash
# Run with 4 parallel workers
cargo fuzz run arithmetic_fuzzer -- -workers=4 -jobs=4
```

## Fuzzer Details

### 1. Alert Registry Priority Fuzzer
**Target**: `AlertRegistry::compute_priority()`

**Invariants Tested**:
- Priority always in range [0, 3]
- Monotonicity: higher threat → higher/equal priority
- Determinism: same input → same output
- Boundary correctness: 0-39→LOW, 40-69→MEDIUM, 70-89→HIGH, 90+→CRITICAL
- Edge cases: U256::ZERO → LOW, U256::MAX → CRITICAL

**Input**: 32 bytes (U256 threat level)

### 2. Alert Registry Error Encoding Fuzzer
**Target**: All 17 AlertRegistry error types

**Invariants Tested**:
- Minimum 4 bytes (selector)
- Deterministic encoding
- Non-zero selectors
- Correct sizes: Address errors = 36 bytes, two-param errors = 68 bytes
- No uninitialized memory

**Input**: Variable length (error type + parameters)

### 3. Circuit Breaker State Machine Fuzzer
**Target**: CircuitBreaker trip/reset state transitions

**Invariants Tested**:
- Trip count monotonically non-decreasing
- Successful trip: was not tripped, is tripped, count++
- Failed trip (AlreadyTripped): state unchanged
- Successful reset: was tripped, is not tripped, count persists
- Failed reset (NotTripped): state unchanged
- Query operations don't modify state

**Input**: Sequence of operations (trip/reset/query)

### 4. Detection Engine Anomaly Fuzzer
**Target**: Threshold-based anomaly detection logic

**Invariants Tested**:
- Determinism
- Antisymmetry: if a > b, then not (b > a)
- Transitivity: if a > b and b > c, then a > c
- Boundary: a == b → not anomaly, a == b+1 → anomaly
- Zero threshold: any positive → anomaly
- MAX threshold: nothing exceeds it
- Monotonicity: larger value → more likely anomaly
- Commutativity with constant: (a+c) > (b+c) iff a > b

**Input**: 64-96 bytes (value, threshold, optional third value)

### 5. Cross-Contract Error Fuzzer
**Target**: Shared errors across CircuitBreaker, DetectionEngine, AlertRegistry

**Invariants Tested**:
- Same error signature → same selector across contracts
- Identical full encoding for same error with same data
- Consistent sizes (UnauthorizedCaller, InvalidOwner = 36 bytes)
- Address preservation in encoding
- Different errors → different selectors
- Selector collision resistance

**Input**: 20-52 bytes (addresses, IDs)

### 6. RBAC Fuzzer
**Target**: Role bitmask operations (grant/revoke)

**Invariants Tested**:
- Only valid bits set (ADMIN=0x01, MONITOR=0x02)
- Grant increases or maintains roles
- Revoke decreases or maintains roles
- Idempotence: double grant/revoke = single grant/revoke
- Commutativity: order doesn't matter
- Inverse: grant then revoke returns to original
- Consistency between internal state and has_role()

**Input**: Sequence of RBAC operations

### 7. Arithmetic Fuzzer
**Target**: U256 saturating arithmetic operations

**Invariants Tested**:
- **saturating_add**:
  - Result >= both operands
  - Commutativity: a+b = b+a
  - Associativity: (a+b)+c = a+(b+c)
  - Identity: a+0 = a
  - MAX + anything = MAX
- **saturating_sub**:
  - Result <= first operand
  - Identity: a-0 = a
  - Annihilation: a-a = 0
  - 0 - anything = 0
  - b > a → result = 0
- **Interactions**:
  - (a+b)-b = a (no saturation)
  - (a-b)+b = a (no underflow)
  - Monotonicity preservation
  - Increment/decrement roundtrips

**Input**: 64-96 bytes (2-3 U256 values)

## Fuzzing Statistics

Each fuzzer runs with:
- **Iterations**: 10,000+ per run (or until failure/timeout)
- **Coverage**: LibFuzzer tracks code coverage and explores new paths
- **Mutation**: Intelligent input mutation based on coverage feedback
- **Corpus**: Builds a corpus of interesting inputs over time

## Interpreting Results

### Success
```
INFO: Running with entropic power schedule (0xFF, 100).
INFO: Seed: 1234567890
INFO: Loaded 1 modules (1234 inline 8-bit counters): ...
#100000	REDUCE cov: 45 ft: 67 corp: 12/34b exec/s: 50000 rss: 34Mb
```
- `cov`: Code coverage (edges hit)
- `ft`: Features (unique code paths)
- `corp`: Corpus size (interesting inputs found)
- `exec/s`: Executions per second

### Failure
```
==12345==ERROR: libFuzzer: deadly signal
SUMMARY: libFuzzer: deadly signal
artifact_prefix='./'; Test unit written to ./crash-da39a3ee5e6b4b0d
```

Crash artifacts are saved in `fuzz/artifacts/<fuzzer_name>/` for debugging.

## Continuous Integration

Add to `.github/workflows/fuzz.yml`:

```yaml
name: Fuzzing

on:
  schedule:
    - cron: '0 0 * * *'  # Daily
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - alert_registry_priority
          - alert_registry_errors
          - circuit_breaker_state
          - detection_engine_anomaly
          - cross_contract_errors
          - rbac_fuzzer
          - arithmetic_fuzzer
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-fuzz
      - run: cargo fuzz run ${{ matrix.target }} -- -max_total_time=300
```

## Coverage Reports

Generate coverage report:

```bash
# Run fuzzer with coverage tracking
cargo fuzz coverage alert_registry_priority

# Generate HTML report
cargo cov -- show fuzz/target/*/release/alert_registry_priority \
    --format=html \
    --instr-profile=fuzz/coverage/alert_registry_priority/coverage.profdata \
    > coverage.html
```

## Debugging Failures

If a fuzzer finds a crash:

```bash
# Reproduce the crash
cargo fuzz run alert_registry_priority fuzz/artifacts/alert_registry_priority/crash-*

# Run in debugger
rust-lldb -- fuzz/target/*/release/alert_registry_priority fuzz/artifacts/alert_registry_priority/crash-*
```

## Best Practices

1. **Run regularly**: Fuzz daily or on every PR
2. **Long runs**: Let fuzzers run for hours/days to find deep bugs
3. **Save corpus**: Commit interesting corpus inputs for regression testing
4. **Review failures**: Every crash is a potential vulnerability
5. **Combine with proptest**: Fuzzers complement property-based tests

## Resources

- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [cargo-fuzz README](https://github.com/rust-fuzz/cargo-fuzz)
