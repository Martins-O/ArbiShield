// ArbiShield Gas Measurement Tests
//
// Comprehensive gas consumption measurements for all major operations.
// These tests measure computational complexity as a proxy for gas usage.
//
// Run with:
// ```bash
// cargo test --test gas_measurements -- --nocapture
// ```

use alloy_primitives::{Address, U256};

// ============================================================================
// Mock State Structures (for gas measurement)
// ============================================================================

#[derive(Debug, Clone)]
struct MockCircuitBreaker {
    owner: Address,
    paused_protocols: Vec<Address>,
    tripped_protocols: Vec<(Address, U256)>, // (protocol, timestamp)
    trip_counts: Vec<(Address, U256)>,
}

impl MockCircuitBreaker {
    fn new(owner: Address) -> Self {
        Self {
            owner,
            paused_protocols: Vec::new(),
            tripped_protocols: Vec::new(),
            trip_counts: Vec::new(),
        }
    }

    fn pause(&mut self, protocol: Address) {
        if !self.paused_protocols.contains(&protocol) {
            self.paused_protocols.push(protocol);
        }
    }

    fn resume(&mut self, protocol: Address) {
        self.paused_protocols.retain(|p| p != &protocol);
    }

    fn trip(&mut self, protocol: Address, timestamp: U256) {
        // Remove if already tripped
        self.tripped_protocols.retain(|(p, _)| p != &protocol);
        self.tripped_protocols.push((protocol, timestamp));

        // Update trip count
        if let Some((_, count)) = self.trip_counts.iter_mut().find(|(p, _)| p == &protocol) {
            *count = count.saturating_add(U256::from(1));
        } else {
            self.trip_counts.push((protocol, U256::from(1)));
        }
    }

    fn reset(&mut self, protocol: Address) {
        self.tripped_protocols.retain(|(p, _)| p != &protocol);
    }

    fn is_tripped(&self, protocol: Address) -> bool {
        self.tripped_protocols.iter().any(|(p, _)| p == &protocol)
    }

    fn is_paused(&self, protocol: Address) -> bool {
        self.paused_protocols.contains(&protocol)
    }
}

#[derive(Debug, Clone)]
struct MockDetectionEngine {
    owner: Address,
    thresholds: Vec<(U256, U256)>,     // (metric_id, threshold)
    current_values: Vec<(U256, U256)>, // (metric_id, value)
    metric_count: U256,
}

impl MockDetectionEngine {
    fn new(owner: Address) -> Self {
        Self {
            owner,
            thresholds: Vec::new(),
            current_values: Vec::new(),
            metric_count: U256::ZERO,
        }
    }

    fn configure_threshold(&mut self, id: U256, threshold: U256) {
        // Update or insert threshold
        if let Some((_, existing_threshold)) =
            self.thresholds.iter_mut().find(|(mid, _)| *mid == id)
        {
            *existing_threshold = threshold;
        } else {
            self.thresholds.push((id, threshold));
            self.metric_count = self.metric_count.saturating_add(U256::from(1));
        }
    }

    fn report_metric(&mut self, id: U256, value: U256) {
        // Update or insert current value
        if let Some((_, existing_value)) =
            self.current_values.iter_mut().find(|(mid, _)| *mid == id)
        {
            *existing_value = value;
        } else {
            self.current_values.push((id, value));
        }
    }

    fn check_anomaly(&self, id: U256) -> bool {
        let threshold = self
            .thresholds
            .iter()
            .find(|(mid, _)| *mid == id)
            .map(|(_, t)| *t)
            .unwrap_or(U256::ZERO);

        let current = self
            .current_values
            .iter()
            .find(|(mid, _)| *mid == id)
            .map(|(_, v)| *v)
            .unwrap_or(U256::ZERO);

        current > threshold
    }

    fn analyze_threat_level(&self, id: U256) -> U256 {
        let threshold = self
            .thresholds
            .iter()
            .find(|(mid, _)| *mid == id)
            .map(|(_, t)| *t)
            .unwrap_or(U256::ZERO);

        let current = self
            .current_values
            .iter()
            .find(|(mid, _)| *mid == id)
            .map(|(_, v)| *v)
            .unwrap_or(U256::ZERO);

        if threshold == U256::ZERO {
            return U256::ZERO;
        }

        if current <= threshold {
            return U256::ZERO;
        }

        // Calculate percentage above threshold (capped at 100)
        let excess = current.saturating_sub(threshold);
        let percentage = excess
            .saturating_mul(U256::from(100))
            / threshold;

        if percentage > U256::from(100) {
            U256::from(100)
        } else {
            percentage
        }
    }
}

#[derive(Debug, Clone)]
struct MockAlertRegistry {
    owner: Address,
    enhanced_alert_count: U256,
    roles: Vec<(Address, u8)>,
    priority_counts: [U256; 4], // LOW, MEDIUM, HIGH, CRITICAL
    acknowledged_alerts: Vec<U256>,
}

impl MockAlertRegistry {
    fn new(owner: Address) -> Self {
        Self {
            owner,
            enhanced_alert_count: U256::ZERO,
            roles: vec![(owner, 0x03)], // Owner has ADMIN | MONITOR
            priority_counts: [U256::ZERO; 4],
            acknowledged_alerts: Vec::new(),
        }
    }

    fn register_enhanced_alert(
        &mut self,
        _protocol: Address,
        threat_level: U256,
        _pattern: U256,
        _timestamp: U256,
    ) -> U256 {
        self.enhanced_alert_count = self.enhanced_alert_count.saturating_add(U256::from(1));
        let alert_id = self.enhanced_alert_count;

        // Compute priority
        let priority = self.compute_priority(threat_level);
        self.priority_counts[priority as usize] =
            self.priority_counts[priority as usize].saturating_add(U256::from(1));

        alert_id
    }

    fn acknowledge_alert(&mut self, alert_id: U256, _protocol: Address, _timestamp: U256) {
        if !self.acknowledged_alerts.contains(&alert_id) {
            self.acknowledged_alerts.push(alert_id);
        }
    }

    fn grant_role(&mut self, account: Address, role: u8) {
        if let Some((_, existing_role)) = self.roles.iter_mut().find(|(addr, _)| *addr == account) {
            *existing_role |= role;
        } else {
            self.roles.push((account, role));
        }
    }

    fn has_role(&self, account: Address, role: u8) -> bool {
        self.roles
            .iter()
            .find(|(addr, _)| *addr == account)
            .map(|(_, r)| (*r & role) != 0)
            .unwrap_or(false)
    }

    fn get_enhanced_alert_count(&self) -> U256 {
        self.enhanced_alert_count
    }

    fn compute_priority(&self, threat_level: U256) -> u8 {
        if threat_level >= U256::from(90) {
            3 // CRITICAL
        } else if threat_level >= U256::from(70) {
            2 // HIGH
        } else if threat_level >= U256::from(40) {
            1 // MEDIUM
        } else {
            0 // LOW
        }
    }
}

// ============================================================================
// Gas Measurement Utilities
// ============================================================================

struct GasMeasurement {
    operation: String,
    iterations: u32,
    total_time_ns: u128,
    avg_time_ns: u128,
}

impl GasMeasurement {
    fn print_report(&self) {
        println!("\n{}", "=".repeat(70));
        println!("Operation: {}", self.operation);
        println!("Iterations: {}", self.iterations);
        println!("Total Time: {} ns", self.total_time_ns);
        println!("Average Time: {} ns", self.avg_time_ns);
        println!("{}", "=".repeat(70));
    }
}

fn measure_operation<F>(operation: &str, iterations: u32, mut f: F) -> GasMeasurement
where
    F: FnMut(),
{
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        f();
    }

    let duration = start.elapsed();
    let total_time_ns = duration.as_nanos();
    let avg_time_ns = total_time_ns / iterations as u128;

    GasMeasurement {
        operation: operation.to_string(),
        iterations,
        total_time_ns,
        avg_time_ns,
    }
}

// ============================================================================
// CircuitBreaker Gas Tests
// ============================================================================

#[test]
fn gas_measurement_circuit_breaker_pause() {
    let owner = Address::repeat_byte(0x01);
    let protocol = Address::repeat_byte(0x42);

    let measurement = measure_operation("CircuitBreaker::pause", 10_000, || {
        let mut cb = MockCircuitBreaker::new(owner);
        cb.pause(protocol);
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_circuit_breaker_resume() {
    let owner = Address::repeat_byte(0x01);
    let protocol = Address::repeat_byte(0x42);

    let measurement = measure_operation("CircuitBreaker::resume", 10_000, || {
        let mut cb = MockCircuitBreaker::new(owner);
        cb.pause(protocol);
        cb.resume(protocol);
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_circuit_breaker_trip() {
    let owner = Address::repeat_byte(0x01);
    let protocol = Address::repeat_byte(0x42);
    let timestamp = U256::from(1234567890_u64);

    let measurement = measure_operation("CircuitBreaker::trip", 10_000, || {
        let mut cb = MockCircuitBreaker::new(owner);
        cb.trip(protocol, timestamp);
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_circuit_breaker_reset() {
    let owner = Address::repeat_byte(0x01);
    let protocol = Address::repeat_byte(0x42);
    let timestamp = U256::from(1234567890_u64);

    let measurement = measure_operation("CircuitBreaker::reset", 10_000, || {
        let mut cb = MockCircuitBreaker::new(owner);
        cb.trip(protocol, timestamp);
        cb.reset(protocol);
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_circuit_breaker_is_tripped() {
    let owner = Address::repeat_byte(0x01);
    let protocol = Address::repeat_byte(0x42);
    let timestamp = U256::from(1234567890_u64);

    let mut cb = MockCircuitBreaker::new(owner);
    cb.trip(protocol, timestamp);

    let measurement = measure_operation("CircuitBreaker::is_tripped", 100_000, || {
        let _ = cb.is_tripped(protocol);
    });

    measurement.print_report();
}

// ============================================================================
// DetectionEngine Gas Tests
// ============================================================================

#[test]
fn gas_measurement_detection_configure_threshold() {
    let owner = Address::repeat_byte(0x01);
    let metric_id = U256::from(1_u64);
    let threshold = U256::from(100_u64);

    let measurement = measure_operation("DetectionEngine::configure_threshold", 10_000, || {
        let mut de = MockDetectionEngine::new(owner);
        de.configure_threshold(metric_id, threshold);
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_detection_report_metric() {
    let owner = Address::repeat_byte(0x01);
    let metric_id = U256::from(1_u64);
    let threshold = U256::from(100_u64);
    let value = U256::from(75_u64);

    let measurement = measure_operation("DetectionEngine::report_metric", 10_000, || {
        let mut de = MockDetectionEngine::new(owner);
        de.configure_threshold(metric_id, threshold);
        de.report_metric(metric_id, value);
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_detection_check_anomaly() {
    let owner = Address::repeat_byte(0x01);
    let metric_id = U256::from(1_u64);
    let threshold = U256::from(100_u64);
    let value = U256::from(150_u64);

    let mut de = MockDetectionEngine::new(owner);
    de.configure_threshold(metric_id, threshold);
    de.report_metric(metric_id, value);

    let measurement = measure_operation("DetectionEngine::check_anomaly", 100_000, || {
        let _ = de.check_anomaly(metric_id);
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_detection_analyze_threat() {
    let owner = Address::repeat_byte(0x01);
    let metric_id = U256::from(1_u64);
    let threshold = U256::from(100_u64);
    let value = U256::from(175_u64);

    let mut de = MockDetectionEngine::new(owner);
    de.configure_threshold(metric_id, threshold);
    de.report_metric(metric_id, value);

    let measurement = measure_operation("DetectionEngine::analyze_threat_level", 100_000, || {
        let _ = de.analyze_threat_level(metric_id);
    });

    measurement.print_report();
}

// ============================================================================
// AlertRegistry Gas Tests
// ============================================================================

#[test]
fn gas_measurement_alert_register_enhanced() {
    let owner = Address::repeat_byte(0x01);
    let protocol = Address::repeat_byte(0x42);
    let threat_level = U256::from(75_u64);
    let pattern = U256::from(0x01_u64);
    let timestamp = U256::from(1234567890_u64);

    let measurement = measure_operation("AlertRegistry::register_enhanced_alert", 10_000, || {
        let mut ar = MockAlertRegistry::new(owner);
        ar.register_enhanced_alert(protocol, threat_level, pattern, timestamp);
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_alert_acknowledge() {
    let owner = Address::repeat_byte(0x01);
    let protocol = Address::repeat_byte(0x42);
    let threat_level = U256::from(75_u64);
    let pattern = U256::from(0x01_u64);
    let timestamp = U256::from(1234567890_u64);

    let measurement = measure_operation("AlertRegistry::acknowledge_alert", 10_000, || {
        let mut ar = MockAlertRegistry::new(owner);
        let alert_id = ar.register_enhanced_alert(protocol, threat_level, pattern, timestamp);
        ar.acknowledge_alert(alert_id, protocol, timestamp);
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_alert_grant_role() {
    let owner = Address::repeat_byte(0x01);
    let account = Address::repeat_byte(0x42);

    let measurement = measure_operation("AlertRegistry::grant_role", 10_000, || {
        let mut ar = MockAlertRegistry::new(owner);
        ar.grant_role(account, 0x01); // ADMIN_ROLE
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_alert_get_count() {
    let owner = Address::repeat_byte(0x01);
    let protocol = Address::repeat_byte(0x42);
    let threat_level = U256::from(75_u64);
    let pattern = U256::from(0x01_u64);
    let timestamp = U256::from(1234567890_u64);

    let mut ar = MockAlertRegistry::new(owner);
    for _ in 0..10 {
        ar.register_enhanced_alert(protocol, threat_level, pattern, timestamp);
    }

    let measurement = measure_operation("AlertRegistry::get_enhanced_alert_count", 100_000, || {
        let _ = ar.get_enhanced_alert_count();
    });

    measurement.print_report();
}

// ============================================================================
// Batch Operation Gas Tests
// ============================================================================

#[test]
fn gas_measurement_batch_alert_registration() {
    let owner = Address::repeat_byte(0x01);

    let measurement = measure_operation("Batch: Register 5 Alerts", 1_000, || {
        let mut ar = MockAlertRegistry::new(owner);

        for i in 0..5 {
            let protocol = Address::repeat_byte((0x40 + i) as u8);
            let threat_level = U256::from(60_u64 + i as u64 * 10);
            let pattern = U256::from(0x01_u64 << i);
            let timestamp = U256::from(1234567890_u64 + i as u64);

            ar.register_enhanced_alert(protocol, threat_level, pattern, timestamp);
        }
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_batch_threshold_configuration() {
    let owner = Address::repeat_byte(0x01);

    let measurement = measure_operation("Batch: Configure 10 Thresholds", 1_000, || {
        let mut de = MockDetectionEngine::new(owner);

        for i in 0..10 {
            let metric_id = U256::from(i as u64);
            let threshold = U256::from(100_u64 + i as u64 * 10);

            de.configure_threshold(metric_id, threshold);
        }
    });

    measurement.print_report();
}

#[test]
fn gas_measurement_batch_pause_operations() {
    let owner = Address::repeat_byte(0x01);

    let measurement = measure_operation("Batch: Pause 5 Protocols", 1_000, || {
        let mut cb = MockCircuitBreaker::new(owner);

        for i in 0..5 {
            let protocol = Address::repeat_byte((0x40 + i) as u8);
            cb.pause(protocol);
        }
    });

    measurement.print_report();
}

// ============================================================================
// Scalability Tests
// ============================================================================

#[test]
fn gas_measurement_storage_scalability() {
    println!("\n{}", "=".repeat(70));
    println!("STORAGE SCALABILITY TEST");
    println!("{}", "=".repeat(70));

    let owner = Address::repeat_byte(0x01);

    for count in [1, 5, 10, 20, 50, 100] {
        let measurement = measure_operation(&format!("Register {} Alerts", count), 100, || {
            let mut ar = MockAlertRegistry::new(owner);

            for i in 0..count {
                let protocol = Address::repeat_byte((i % 256) as u8);
                let threat_level = U256::from(50_u64 + (i as u64 % 50));
                let pattern = U256::from(0x01_u64);
                let timestamp = U256::from(1234567890_u64);

                ar.register_enhanced_alert(protocol, threat_level, pattern, timestamp);
            }
        });

        println!("{} alerts: {} ns (avg)", count, measurement.avg_time_ns);
    }

    println!("{}", "=".repeat(70));
}

// ============================================================================
// Summary Test
// ============================================================================

#[test]
fn gas_measurement_comprehensive_summary() {
    println!("\n{}", "=".repeat(80));
    println!("COMPREHENSIVE GAS MEASUREMENT SUMMARY");
    println!("{}", "=".repeat(80));

    println!(
        "\n{:<40} {:<15} {:<15}",
        "Operation", "Iterations", "Avg Time (ns)"
    );
    println!("{}", "-".repeat(80));

    // Helper macro to measure and print
    macro_rules! measure_and_print {
        ($name:expr, $iterations:expr, $code:block) => {
            let m = measure_operation($name, $iterations, || $code);
            println!("{:<40} {:<15} {:<15}", $name, $iterations, m.avg_time_ns);
        };
    }

    let owner = Address::repeat_byte(0x01);
    let protocol = Address::repeat_byte(0x42);
    let timestamp = U256::from(1234567890_u64);

    // CircuitBreaker
    measure_and_print!("CB: Pause", 10_000, {
        let mut cb = MockCircuitBreaker::new(owner);
        cb.pause(protocol);
    });

    measure_and_print!("CB: Resume", 10_000, {
        let mut cb = MockCircuitBreaker::new(owner);
        cb.pause(protocol);
        cb.resume(protocol);
    });

    measure_and_print!("CB: Trip", 10_000, {
        let mut cb = MockCircuitBreaker::new(owner);
        cb.trip(protocol, timestamp);
    });

    measure_and_print!("CB: Reset", 10_000, {
        let mut cb = MockCircuitBreaker::new(owner);
        cb.trip(protocol, timestamp);
        cb.reset(protocol);
    });

    // DetectionEngine
    measure_and_print!("DE: Configure Threshold", 10_000, {
        let mut de = MockDetectionEngine::new(owner);
        de.configure_threshold(U256::from(1), U256::from(100));
    });

    measure_and_print!("DE: Report Metric", 10_000, {
        let mut de = MockDetectionEngine::new(owner);
        de.configure_threshold(U256::from(1), U256::from(100));
        de.report_metric(U256::from(1), U256::from(75));
    });

    measure_and_print!("DE: Check Anomaly", 10_000, {
        let mut de = MockDetectionEngine::new(owner);
        de.configure_threshold(U256::from(1), U256::from(100));
        de.report_metric(U256::from(1), U256::from(150));
        let _ = de.check_anomaly(U256::from(1));
    });

    // AlertRegistry
    measure_and_print!("AR: Register Alert", 10_000, {
        let mut ar = MockAlertRegistry::new(owner);
        ar.register_enhanced_alert(protocol, U256::from(75), U256::from(1), timestamp);
    });

    measure_and_print!("AR: Acknowledge Alert", 10_000, {
        let mut ar = MockAlertRegistry::new(owner);
        let id = ar.register_enhanced_alert(protocol, U256::from(75), U256::from(1), timestamp);
        ar.acknowledge_alert(id, protocol, timestamp);
    });

    measure_and_print!("AR: Grant Role", 10_000, {
        let mut ar = MockAlertRegistry::new(owner);
        ar.grant_role(protocol, 0x01);
    });

    println!("{}", "=".repeat(80));
    println!(
        "\nNote: These are computational complexity measurements, not actual on-chain gas costs."
    );
    println!("For actual gas costs, deploy to Arbitrum Sepolia and use gas profiling tools.");
    println!("{}", "=".repeat(80));
}
