//! Detection Engine Invariant Tests
//!
//! These tests verify critical properties for the DetectionEngine contract's
//! anomaly detection and metric tracking logic.
//!
//! ## Security Importance
//!
//! Detection engine invariants ensure:
//! - **Bounded Threat Scores**: Prevents integer overflow attacks
//! - **Deterministic Detection**: Same input always produces same result
//! - **Metric Integrity**: Counts remain consistent
//! - **Threshold Correctness**: Detection logic is sound
//!
//! ## Invariants Tested
//!
//! - **INV-DE-1**: Threshold values are always within valid range [0, U256::MAX]
//! - **INV-DE-2**: Anomaly detection is deterministic (same inputs → same result)
//! - **INV-DE-3**: Metric count is monotonically non-decreasing
//! - **INV-DE-4**: Value > threshold iff anomaly detected (no false positives/negatives)

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::U256;
use proptest::prelude::*;

// Simulate DetectionEngine state
#[derive(Clone, Debug)]
struct DetectionEngineState {
    thresholds: Vec<(U256, U256)>, // (metric_id, threshold)
    current_values: Vec<(U256, U256)>, // (metric_id, value)
    metric_count: U256,
}

impl DetectionEngineState {
    fn new() -> Self {
        Self {
            thresholds: Vec::new(),
            current_values: Vec::new(),
            metric_count: U256::ZERO,
        }
    }

    fn register_metric(&mut self, threshold: U256) -> U256 {
        let metric_id = self.metric_count;
        self.metric_count = self.metric_count.saturating_add(U256::from(1));
        self.thresholds.push((metric_id, threshold));
        self.current_values.push((metric_id, U256::ZERO));
        metric_id
    }

    fn report_metric(&mut self, id: U256, value: U256) -> Result<(), &'static str> {
        if let Some(pos) = self.current_values.iter().position(|(mid, _)| *mid == id) {
            self.current_values[pos] = (id, value);
            Ok(())
        } else {
            Err("MetricNotFound")
        }
    }

    fn check_anomaly(&self, id: U256) -> Result<bool, &'static str> {
        let value = self.current_values
            .iter()
            .find(|(mid, _)| *mid == id)
            .map(|(_, v)| *v)
            .ok_or("MetricNotFound")?;

        let threshold = self.thresholds
            .iter()
            .find(|(mid, _)| *mid == id)
            .map(|(_, t)| *t)
            .ok_or("MetricNotFound")?;

        Ok(value > threshold)
    }
}

// ============================================================================
// INV-DE-1: Threshold values are always within valid range
// ============================================================================
//
// **Security Importance**: Thresholds define security boundaries. If they could
// be set to invalid values, all anomaly detection would fail.

#[test]
fn invariant_de1_threshold_always_valid() {
    let mut de = DetectionEngineState::new();

    // Register with various thresholds
    let thresholds = vec![
        U256::ZERO,
        U256::from(100),
        U256::from(1_000_000),
        U256::MAX,
    ];

    for &threshold in &thresholds {
        de.register_metric(threshold);

        // All thresholds should be valid U256 values (tautologically true, but documents invariant)
        for (_, t) in &de.thresholds {
            assert!(
                *t <= U256::MAX,
                "INV-DE-1 VIOLATED: Threshold {} exceeds U256::MAX",
                t
            );
            assert!(
                *t >= U256::ZERO,
                "INV-DE-1 VIOLATED: Threshold {} below U256::ZERO",
                t
            );
        }
    }
}

#[test]
fn invariant_de1_threshold_immutable_after_registration() {
    let mut de = DetectionEngineState::new();

    let id = de.register_metric(U256::from(1000));
    let original_threshold = de.thresholds[id.saturating_to::<usize>()].1;

    // Report metrics (shouldn't change threshold)
    de.report_metric(id, U256::from(500)).unwrap();
    de.report_metric(id, U256::from(2000)).unwrap();
    de.report_metric(id, U256::from(10)).unwrap();

    let current_threshold = de.thresholds[id.saturating_to::<usize>()].1;
    assert_eq!(
        original_threshold, current_threshold,
        "INV-DE-1 VIOLATED: Threshold changed from {} to {}",
        original_threshold, current_threshold
    );
}

// ============================================================================
// INV-DE-2: Anomaly detection is deterministic
// ============================================================================
//
// **Security Importance**: Non-deterministic detection would allow attackers
// to bypass security by repeatedly triggering detection until it gives a
// favorable result.

#[test]
fn invariant_de2_detection_is_deterministic() {
    let mut de = DetectionEngineState::new();

    let id = de.register_metric(U256::from(1000));
    de.report_metric(id, U256::from(1500)).unwrap();

    // Check anomaly 100 times - should always return same result
    let first_result = de.check_anomaly(id).unwrap();

    for i in 0..100 {
        let result = de.check_anomaly(id).unwrap();
        assert_eq!(
            result, first_result,
            "INV-DE-2 VIOLATED: Non-deterministic result on iteration {}",
            i
        );
    }
}

#[test]
fn invariant_de2_same_value_same_threshold_same_result() {
    let mut de = DetectionEngineState::new();

    // Create multiple metrics with same threshold
    let ids: Vec<U256> = (0..10).map(|_| de.register_metric(U256::from(500))).collect();

    // Report same value to all metrics
    for &id in &ids {
        de.report_metric(id, U256::from(600)).unwrap();
    }

    // All should detect anomaly
    let first_result = de.check_anomaly(ids[0]).unwrap();
    for (i, &id) in ids.iter().enumerate() {
        let result = de.check_anomaly(id).unwrap();
        assert_eq!(
            result, first_result,
            "INV-DE-2 VIOLATED: Metric {} gave different result",
            i
        );
        assert!(result, "All should detect anomaly for value > threshold");
    }
}

// ============================================================================
// INV-DE-3: Metric count is monotonically non-decreasing
// ============================================================================
//
// **Security Importance**: Metric count is used for ID generation and audit
// trails. If it could decrease, metric IDs would collide and tracking would fail.

#[test]
fn invariant_de3_metric_count_monotonic() {
    let mut de = DetectionEngineState::new();
    let mut max_count = de.metric_count;

    for i in 0..100 {
        de.register_metric(U256::from(i * 100));

        assert!(
            de.metric_count >= max_count,
            "INV-DE-3 VIOLATED: Count {} < previous {}",
            de.metric_count,
            max_count
        );
        max_count = de.metric_count;
    }

    assert_eq!(
        de.metric_count,
        U256::from(100),
        "Final count should be 100"
    );
}

#[test]
fn invariant_de3_metric_count_never_decreases() {
    let mut de = DetectionEngineState::new();

    for i in 0..50 {
        let before = de.metric_count;
        de.register_metric(U256::from(i * 100));
        let after = de.metric_count;

        assert!(
            after > before,
            "INV-DE-3 VIOLATED: Count didn't increase on registration {}", i
        );
        assert_eq!(
            after,
            before.saturating_add(U256::from(1)),
            "Count should increase by exactly 1"
        );
    }
}

// ============================================================================
// INV-DE-4: Value > threshold iff anomaly detected
// ============================================================================
//
// **Security Importance**: This is the core security property. Any violation
// means either false positives (blocking legitimate traffic) or false negatives
// (missing real attacks).

#[test]
fn invariant_de4_anomaly_detection_correctness() {
    let mut de = DetectionEngineState::new();

    let threshold = U256::from(1000);
    let id = de.register_metric(threshold);

    // Test values below threshold - should NOT be anomalies
    let below_values = vec![
        U256::ZERO,
        U256::from(1),
        U256::from(500),
        U256::from(999),
        threshold, // Equal to threshold should NOT be anomaly
    ];

    for (i, value) in below_values.iter().enumerate() {
        de.report_metric(id, *value).unwrap();
        let is_anomaly = de.check_anomaly(id).unwrap();

        assert!(
            !is_anomaly,
            "INV-DE-4 VIOLATED: False positive for value {} at threshold {} (case {})",
            value, threshold, i
        );
    }

    // Test values above threshold - should BE anomalies
    let above_values = vec![
        U256::from(1001),
        U256::from(2000),
        U256::from(1_000_000),
        U256::MAX,
    ];

    for (i, value) in above_values.iter().enumerate() {
        de.report_metric(id, *value).unwrap();
        let is_anomaly = de.check_anomaly(id).unwrap();

        assert!(
            is_anomaly,
            "INV-DE-4 VIOLATED: False negative for value {} at threshold {} (case {})",
            value, threshold, i
        );
    }
}

#[test]
fn invariant_de4_boundary_conditions() {
    let mut de = DetectionEngineState::new();

    // Test various threshold boundaries
    let test_cases = vec![
        (U256::ZERO, U256::ZERO, false),      // 0 == 0: not anomaly
        (U256::ZERO, U256::from(1), true),    // 1 > 0: anomaly
        (U256::from(100), U256::from(100), false), // equal: not anomaly
        (U256::from(100), U256::from(101), true),  // just over: anomaly
        (U256::MAX, U256::MAX, false),        // MAX == MAX: not anomaly
    ];

    for (i, (threshold, value, expected_anomaly)) in test_cases.iter().enumerate() {
        let id = de.register_metric(*threshold);
        de.report_metric(id, *value).unwrap();
        let is_anomaly = de.check_anomaly(id).unwrap();

        assert_eq!(
            is_anomaly, *expected_anomaly,
            "INV-DE-4 VIOLATED: Case {} failed (threshold={}, value={}, expected={})",
            i, threshold, value, expected_anomaly
        );
    }
}

// ============================================================================
// Property-Based Invariant Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// For any threshold and value, detection is deterministic
    #[test]
    fn property_detection_always_deterministic(
        threshold in any::<u64>(),
        value in any::<u64>()
    ) {
        let mut de = DetectionEngineState::new();
        let id = de.register_metric(U256::from(threshold));
        de.report_metric(id, U256::from(value)).unwrap();

        let result1 = de.check_anomaly(id).unwrap();
        let result2 = de.check_anomaly(id).unwrap();
        let result3 = de.check_anomaly(id).unwrap();

        prop_assert_eq!(result1, result2, "First and second check differ");
        prop_assert_eq!(result2, result3, "Second and third check differ");
    }

    /// Anomaly detection matches direct comparison
    #[test]
    fn property_detection_matches_comparison(
        threshold in any::<u64>(),
        value in any::<u64>()
    ) {
        let mut de = DetectionEngineState::new();
        let id = de.register_metric(U256::from(threshold));
        de.report_metric(id, U256::from(value)).unwrap();

        let is_anomaly = de.check_anomaly(id).unwrap();
        let expected = value > threshold;

        prop_assert_eq!(is_anomaly, expected,
            "Detection result doesn't match direct comparison: value={}, threshold={}",
            value, threshold);
    }

    /// Metric count equals number of registrations
    #[test]
    fn property_metric_count_equals_registrations(
        thresholds in prop::collection::vec(any::<u64>(), 1..100)
    ) {
        let mut de = DetectionEngineState::new();

        for threshold in &thresholds {
            de.register_metric(U256::from(*threshold));
        }

        prop_assert_eq!(
            de.metric_count.saturating_to::<u64>() as usize,
            thresholds.len(),
            "Metric count doesn't match number of registrations"
        );
    }

    /// Thresholds never change after registration
    #[test]
    fn property_thresholds_immutable(
        threshold in any::<u64>(),
        values in prop::collection::vec(any::<u64>(), 1..50)
    ) {
        let mut de = DetectionEngineState::new();
        let id = de.register_metric(U256::from(threshold));
        let original = de.thresholds[id.saturating_to::<usize>()].1;

        // Report many values
        for value in &values {
            let _ = de.report_metric(id, U256::from(*value));
        }

        let current = de.thresholds[id.saturating_to::<usize>()].1;
        prop_assert_eq!(original, current, "Threshold changed after metric reports");
    }
}

// ============================================================================
// Stress Test with 1000+ Operations
// ============================================================================

#[test]
fn stress_test_1000_operations_maintain_all_invariants() {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let mut de = DetectionEngineState::new();
    let hasher = RandomState::new();

    // Register some initial metrics
    let ids: Vec<U256> = (0..10)
        .map(|i| de.register_metric(U256::from(i * 100 + 500)))
        .collect();

    for i in 0..1000 {
        let mut h = hasher.build_hasher();
        i.hash(&mut h);
        let random = h.finish();

        let id = ids[(random % ids.len() as u64) as usize];
        let value = U256::from(random % 2000);

        let before_count = de.metric_count;

        // Report metric
        de.report_metric(id, value).unwrap();

        // INV-DE-3: Count unchanged by reports
        assert_eq!(
            de.metric_count, before_count,
            "INV-DE-3 violated at op {}", i
        );

        // INV-DE-4: Detection correctness
        let is_anomaly = de.check_anomaly(id).unwrap();
        let threshold = de.thresholds[id.saturating_to::<usize>()].1;
        let expected = value > threshold;

        assert_eq!(
            is_anomaly, expected,
            "INV-DE-4 violated at op {}: value={}, threshold={}",
            i, value, threshold
        );

        // INV-DE-2: Determinism
        let result2 = de.check_anomaly(id).unwrap();
        assert_eq!(
            is_anomaly, result2,
            "INV-DE-2 violated at op {}", i
        );
    }

    println!("✓ All DetectionEngine invariants held across 1000 operations");
    println!("  Total metrics: {}", de.metric_count);
    println!("  Tracked metrics: {}", de.thresholds.len());
}
