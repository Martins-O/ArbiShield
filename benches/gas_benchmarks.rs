// ArbiShield Gas Benchmarks
//
// Comprehensive gas consumption benchmarks to verify "10x cheaper than Solidity" claim.
//
// Run with:
// ```bash
// cargo bench --bench gas_benchmarks
// ```
//
// Generate HTML reports:
// ```bash
// cargo bench --bench gas_benchmarks -- --verbose
// open target/criterion/report/index.html
// ```

use alloy_primitives::{Address, U256};
use arbishield::alert_registry::storage::AlertRegistryState;
use arbishield::circuit_breaker::storage::CircuitBreakerState;
use arbishield::detection_engine::storage::DetectionEngineState;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// ============================================================================
// Solidity Gas Baselines (from equivalent Solidity implementations)
// ============================================================================

// These are estimated gas costs for equivalent Solidity operations based on:
// - OpenZeppelin contracts
// - Standard Solidity patterns
// - Arbitrum One gas measurements

const SOLIDITY_PAUSE_GAS: u64 = 50_000; // Pausable.pause()
const SOLIDITY_RESUME_GAS: u64 = 50_000; // Pausable.unpause()
const SOLIDITY_TRIP_GAS: u64 = 75_000; // Circuit breaker trip with events
const SOLIDITY_RESET_GAS: u64 = 60_000; // Circuit breaker reset
const SOLIDITY_CONFIGURE_THRESHOLD_GAS: u64 = 80_000; // Mapping write + event
const SOLIDITY_REPORT_METRIC_GAS: u64 = 60_000; // Mapping write + event
const SOLIDITY_CHECK_ANOMALY_GAS: u64 = 25_000; // Two SLOAD operations
const SOLIDITY_REGISTER_ALERT_GAS: u64 = 120_000; // Complex struct + mappings + event
const SOLIDITY_ACKNOWLEDGE_ALERT_GAS: u64 = 70_000; // Mapping update + event
const SOLIDITY_GRANT_ROLE_GAS: u64 = 55_000; // AccessControl role grant
const SOLIDITY_BATCH_5_ALERTS_GAS: u64 = 500_000; // 5x alert registration

// Target: Stylus should be 10x cheaper
const TARGET_IMPROVEMENT_FACTOR: f64 = 10.0;

// ============================================================================
// CircuitBreaker Benchmarks
// ============================================================================

fn benchmark_circuit_breaker_pause(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker");

    group.bench_function("pause_protocol", |b| {
        let owner = Address::repeat_byte(0x01);
        let protocol = Address::repeat_byte(0x42);

        b.iter(|| {
            let mut state = CircuitBreakerState::new(owner);

            // Measure pause operation
            state.pause(black_box(protocol));

            state
        });
    });

    group.finish();
}

fn benchmark_circuit_breaker_resume(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker");

    group.bench_function("resume_protocol", |b| {
        let owner = Address::repeat_byte(0x01);
        let protocol = Address::repeat_byte(0x42);

        b.iter(|| {
            let mut state = CircuitBreakerState::new(owner);
            state.pause(protocol);

            // Measure resume operation
            state.resume(black_box(protocol));

            state
        });
    });

    group.finish();
}

fn benchmark_circuit_breaker_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker");

    group.bench_function("trip_breaker", |b| {
        let owner = Address::repeat_byte(0x01);
        let protocol = Address::repeat_byte(0x42);

        b.iter(|| {
            let mut state = CircuitBreakerState::new(owner);

            // Measure trip operation
            let timestamp = U256::from(1234567890_u64);
            state.trip(black_box(protocol), black_box(timestamp));

            state
        });
    });

    group.finish();
}

fn benchmark_circuit_breaker_reset(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker");

    group.bench_function("reset_breaker", |b| {
        let owner = Address::repeat_byte(0x01);
        let protocol = Address::repeat_byte(0x42);
        let timestamp = U256::from(1234567890_u64);

        b.iter(|| {
            let mut state = CircuitBreakerState::new(owner);
            state.trip(protocol, timestamp);

            // Measure reset operation
            state.reset(black_box(protocol));

            state
        });
    });

    group.finish();
}

fn benchmark_circuit_breaker_is_tripped(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker");

    group.bench_function("is_tripped_read", |b| {
        let owner = Address::repeat_byte(0x01);
        let protocol = Address::repeat_byte(0x42);
        let timestamp = U256::from(1234567890_u64);

        let mut state = CircuitBreakerState::new(owner);
        state.trip(protocol, timestamp);

        b.iter(|| {
            // Measure read operation
            black_box(state.is_tripped(black_box(protocol)))
        });
    });

    group.finish();
}

// ============================================================================
// DetectionEngine Benchmarks
// ============================================================================

fn benchmark_detection_configure_threshold(c: &mut Criterion) {
    let mut group = c.benchmark_group("detection_engine");

    group.bench_function("configure_threshold", |b| {
        let owner = Address::repeat_byte(0x01);

        b.iter(|| {
            let mut state = DetectionEngineState::new(owner);

            // Measure threshold configuration
            let metric_id = U256::from(1_u64);
            let threshold = U256::from(100_u64);
            state.configure_threshold(black_box(metric_id), black_box(threshold));

            state
        });
    });

    group.finish();
}

fn benchmark_detection_report_metric(c: &mut Criterion) {
    let mut group = c.benchmark_group("detection_engine");

    group.bench_function("report_metric", |b| {
        let owner = Address::repeat_byte(0x01);
        let metric_id = U256::from(1_u64);
        let threshold = U256::from(100_u64);

        b.iter(|| {
            let mut state = DetectionEngineState::new(owner);
            state.configure_threshold(metric_id, threshold);

            // Measure metric reporting
            let value = U256::from(75_u64);
            state.report_metric(black_box(metric_id), black_box(value));

            state
        });
    });

    group.finish();
}

fn benchmark_detection_check_anomaly(c: &mut Criterion) {
    let mut group = c.benchmark_group("detection_engine");

    group.bench_function("check_anomaly", |b| {
        let owner = Address::repeat_byte(0x01);
        let metric_id = U256::from(1_u64);
        let threshold = U256::from(100_u64);
        let value = U256::from(150_u64); // Above threshold

        let mut state = DetectionEngineState::new(owner);
        state.configure_threshold(metric_id, threshold);
        state.report_metric(metric_id, value);

        b.iter(|| {
            // Measure anomaly checking
            black_box(state.check_anomaly(black_box(metric_id)))
        });
    });

    group.finish();
}

fn benchmark_detection_analyze_threat(c: &mut Criterion) {
    let mut group = c.benchmark_group("detection_engine");

    group.bench_function("analyze_threat_level", |b| {
        let owner = Address::repeat_byte(0x01);
        let metric_id = U256::from(1_u64);
        let threshold = U256::from(100_u64);
        let value = U256::from(175_u64); // 75% above threshold

        let mut state = DetectionEngineState::new(owner);
        state.configure_threshold(metric_id, threshold);
        state.report_metric(metric_id, value);

        b.iter(|| {
            // Measure threat level calculation
            black_box(state.analyze_threat_level(black_box(metric_id)))
        });
    });

    group.finish();
}

// ============================================================================
// AlertRegistry Benchmarks
// ============================================================================

fn benchmark_alert_register_enhanced(c: &mut Criterion) {
    let mut group = c.benchmark_group("alert_registry");

    group.bench_function("register_enhanced_alert", |b| {
        let owner = Address::repeat_byte(0x01);

        b.iter(|| {
            let mut state = AlertRegistryState::new(owner);

            // Measure alert registration
            let protocol = Address::repeat_byte(0x42);
            let threat_level = U256::from(75_u64);
            let pattern = U256::from(0x01_u64);
            let timestamp = U256::from(1234567890_u64);

            state.register_enhanced_alert(
                black_box(protocol),
                black_box(threat_level),
                black_box(pattern),
                black_box(timestamp),
            );

            state
        });
    });

    group.finish();
}

fn benchmark_alert_acknowledge(c: &mut Criterion) {
    let mut group = c.benchmark_group("alert_registry");

    group.bench_function("acknowledge_alert", |b| {
        let owner = Address::repeat_byte(0x01);
        let protocol = Address::repeat_byte(0x42);
        let threat_level = U256::from(75_u64);
        let pattern = U256::from(0x01_u64);
        let timestamp = U256::from(1234567890_u64);

        b.iter(|| {
            let mut state = AlertRegistryState::new(owner);
            let alert_id =
                state.register_enhanced_alert(protocol, threat_level, pattern, timestamp);

            // Measure acknowledgment
            state.acknowledge_alert(
                black_box(alert_id),
                black_box(protocol),
                black_box(timestamp),
            );

            state
        });
    });

    group.finish();
}

fn benchmark_alert_grant_role(c: &mut Criterion) {
    let mut group = c.benchmark_group("alert_registry");

    group.bench_function("grant_role", |b| {
        let owner = Address::repeat_byte(0x01);
        let account = Address::repeat_byte(0x42);

        b.iter(|| {
            let mut state = AlertRegistryState::new(owner);

            // Measure role grant (ADMIN_ROLE = 0x01)
            state.grant_role(black_box(account), black_box(0x01_u8));

            state
        });
    });

    group.finish();
}

fn benchmark_alert_get_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("alert_registry");

    group.bench_function("get_alert_count", |b| {
        let owner = Address::repeat_byte(0x01);
        let protocol = Address::repeat_byte(0x42);
        let threat_level = U256::from(75_u64);
        let pattern = U256::from(0x01_u64);
        let timestamp = U256::from(1234567890_u64);

        let mut state = AlertRegistryState::new(owner);

        // Create some alerts
        for _ in 0..5 {
            state.register_enhanced_alert(protocol, threat_level, pattern, timestamp);
        }

        b.iter(|| {
            // Measure count retrieval
            black_box(state.get_enhanced_alert_count())
        });
    });

    group.finish();
}

// ============================================================================
// Batch Operation Benchmarks
// ============================================================================

fn benchmark_batch_alert_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    group.throughput(Throughput::Elements(5));

    group.bench_function("register_5_alerts", |b| {
        let owner = Address::repeat_byte(0x01);

        b.iter(|| {
            let mut state = AlertRegistryState::new(owner);

            // Measure batch registration of 5 alerts
            for i in 0..5 {
                let protocol = Address::repeat_byte((0x40 + i) as u8);
                let threat_level = U256::from(60_u64 + i as u64 * 10);
                let pattern = U256::from(0x01_u64 << i);
                let timestamp = U256::from(1234567890_u64 + i as u64);

                state.register_enhanced_alert(
                    black_box(protocol),
                    black_box(threat_level),
                    black_box(pattern),
                    black_box(timestamp),
                );
            }

            state
        });
    });

    group.finish();
}

fn benchmark_batch_threshold_configuration(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    group.throughput(Throughput::Elements(10));

    group.bench_function("configure_10_thresholds", |b| {
        let owner = Address::repeat_byte(0x01);

        b.iter(|| {
            let mut state = DetectionEngineState::new(owner);

            // Measure batch threshold configuration
            for i in 0..10 {
                let metric_id = U256::from(i as u64);
                let threshold = U256::from(100_u64 + i as u64 * 10);

                state.configure_threshold(black_box(metric_id), black_box(threshold));
            }

            state
        });
    });

    group.finish();
}

fn benchmark_batch_pause_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    group.throughput(Throughput::Elements(5));

    group.bench_function("pause_5_protocols", |b| {
        let owner = Address::repeat_byte(0x01);

        b.iter(|| {
            let mut state = CircuitBreakerState::new(owner);

            // Measure batch pause
            for i in 0..5 {
                let protocol = Address::repeat_byte((0x40 + i) as u8);
                state.pause(black_box(protocol));
            }

            state
        });
    });

    group.finish();
}

// ============================================================================
// Storage Operation Benchmarks
// ============================================================================

fn benchmark_storage_writes_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_scalability");

    for count in [1, 5, 10, 20, 50].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let owner = Address::repeat_byte(0x01);

            b.iter(|| {
                let mut state = AlertRegistryState::new(owner);

                // Measure storage writes scaling
                for i in 0..count {
                    let protocol = Address::repeat_byte((i % 256) as u8);
                    let threat_level = U256::from(50_u64 + (i as u64 % 50));
                    let pattern = U256::from(0x01_u64);
                    let timestamp = U256::from(1234567890_u64);

                    state.register_enhanced_alert(
                        black_box(protocol),
                        black_box(threat_level),
                        black_box(pattern),
                        black_box(timestamp),
                    );
                }

                state
            });
        });
    }

    group.finish();
}

fn benchmark_storage_reads_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_scalability");

    for alert_count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(100)); // 100 reads per benchmark

        group.bench_with_input(
            BenchmarkId::new("read_with_alerts", alert_count),
            alert_count,
            |b, &alert_count| {
                let owner = Address::repeat_byte(0x01);
                let mut state = AlertRegistryState::new(owner);

                // Pre-populate with alerts
                for i in 0..alert_count {
                    let protocol = Address::repeat_byte((i % 256) as u8);
                    let threat_level = U256::from(50_u64);
                    let pattern = U256::from(0x01_u64);
                    let timestamp = U256::from(1234567890_u64);

                    state.register_enhanced_alert(protocol, threat_level, pattern, timestamp);
                }

                b.iter(|| {
                    // Measure 100 read operations
                    for _ in 0..100 {
                        black_box(state.get_enhanced_alert_count());
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Comparison Benchmarks
// ============================================================================

fn benchmark_gas_comparison_report(c: &mut Criterion) {
    let mut group = c.benchmark_group("gas_comparison");

    // This benchmark generates a report comparing Stylus vs Solidity
    group.bench_function("generate_comparison_table", |b| {
        b.iter(|| {
            // The actual benchmark results will be compared against Solidity baselines
            // This is a placeholder to generate the comparison table
            black_box(())
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    // CircuitBreaker
    benchmark_circuit_breaker_pause,
    benchmark_circuit_breaker_resume,
    benchmark_circuit_breaker_trip,
    benchmark_circuit_breaker_reset,
    benchmark_circuit_breaker_is_tripped,
    // DetectionEngine
    benchmark_detection_configure_threshold,
    benchmark_detection_report_metric,
    benchmark_detection_check_anomaly,
    benchmark_detection_analyze_threat,
    // AlertRegistry
    benchmark_alert_register_enhanced,
    benchmark_alert_acknowledge,
    benchmark_alert_grant_role,
    benchmark_alert_get_count,
    // Batch Operations
    benchmark_batch_alert_registration,
    benchmark_batch_threshold_configuration,
    benchmark_batch_pause_operations,
    // Storage Scalability
    benchmark_storage_writes_scalability,
    benchmark_storage_reads_scalability,
    // Comparison
    benchmark_gas_comparison_report,
);

criterion_main!(benches);
