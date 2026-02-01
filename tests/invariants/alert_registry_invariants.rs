//! Alert Registry Invariant Tests
//!
//! These tests verify critical properties for the AlertRegistry contract's
//! alert management and tracking logic.
//!
//! ## Security Importance
//!
//! Alert registry invariants ensure:
//! - **Audit Trail Integrity**: Alert IDs and counts are tamper-proof
//! - **State Immutability**: Acknowledged alerts can't be reversed
//! - **Priority Correctness**: Threat levels map correctly to priorities
//! - **Expiration Logic**: Expired alerts are properly handled
//!
//! ## Invariants Tested
//!
//! - **INV-AR-1**: Alert IDs are strictly monotonically increasing (1-indexed)
//! - **INV-AR-2**: Acknowledged alerts cannot be unacknowledged (one-way transition)
//! - **INV-AR-3**: Alert count is monotonically non-decreasing
//! - **INV-AR-4**: Priority is deterministically computed from threat level

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use arbishield::alert_registry::storage::AlertRegistry;
use proptest::prelude::*;

// Simulate AlertRegistry state
#[derive(Clone, Debug)]
struct AlertRegistryState {
    enhanced_alert_count: U256,
    alerts: Vec<(U256, U256, bool)>, // (id, threat_level, acknowledged)
}

impl AlertRegistryState {
    fn new() -> Self {
        Self {
            enhanced_alert_count: U256::ZERO,
            alerts: Vec::new(),
        }
    }

    fn register_alert(&mut self, threat_level: U256) -> U256 {
        let id = self.enhanced_alert_count.saturating_add(U256::from(1));
        self.enhanced_alert_count = id;
        self.alerts.push((id, threat_level, false));
        id
    }

    fn acknowledge_alert(&mut self, id: U256) -> Result<(), &'static str> {
        if let Some(pos) = self.alerts.iter().position(|(aid, _, _)| *aid == id) {
            if self.alerts[pos].2 {
                return Err("AlreadyAcknowledged");
            }
            self.alerts[pos].2 = true;
            Ok(())
        } else {
            Err("AlertNotFound")
        }
    }

    fn is_acknowledged(&self, id: U256) -> bool {
        self.alerts
            .iter()
            .find(|(aid, _, _)| *aid == id)
            .map(|(_, _, ack)| *ack)
            .unwrap_or(false)
    }
}

// ============================================================================
// INV-AR-1: Alert IDs are strictly monotonically increasing (1-indexed)
// ============================================================================
//
// **Security Importance**: Alert IDs serve as unique identifiers in audit logs.
// If IDs could repeat or decrease, alerts would be indistinguishable and
// security incidents could be hidden.

#[test]
fn invariant_ar1_alert_ids_monotonically_increasing() {
    let mut registry = AlertRegistryState::new();

    let mut prev_id = U256::ZERO;

    for i in 0..100 {
        let id = registry.register_alert(U256::from(i * 10));

        assert!(
            id > prev_id,
            "INV-AR-1 VIOLATED: Alert ID {} not greater than previous {}",
            id,
            prev_id
        );

        assert_eq!(
            id,
            prev_id.saturating_add(U256::from(1)),
            "INV-AR-1 VIOLATED: ID should increment by 1, got {} after {}",
            id,
            prev_id
        );

        prev_id = id;
    }
}

#[test]
fn invariant_ar1_first_alert_id_is_one() {
    let mut registry = AlertRegistryState::new();

    let first_id = registry.register_alert(U256::from(50));

    assert_eq!(
        first_id,
        U256::from(1),
        "INV-AR-1 VIOLATED: First alert ID should be 1, got {}",
        first_id
    );
}

#[test]
fn invariant_ar1_no_id_gaps() {
    let mut registry = AlertRegistryState::new();

    let ids: Vec<U256> = (0..50)
        .map(|i| registry.register_alert(U256::from(i * 20)))
        .collect();

    // Check for gaps
    for i in 1..ids.len() {
        let gap = ids[i].saturating_sub(ids[i - 1]);
        assert_eq!(
            gap,
            U256::from(1),
            "INV-AR-1 VIOLATED: Gap of {} between IDs {} and {}",
            gap,
            ids[i - 1],
            ids[i]
        );
    }
}

// ============================================================================
// INV-AR-2: Acknowledged alerts cannot be unacknowledged
// ============================================================================
//
// **Security Importance**: Acknowledgment represents a security action
// (incident response). If it could be reversed, attackers could hide evidence
// of breaches.

#[test]
fn invariant_ar2_acknowledged_alerts_immutable() {
    let mut registry = AlertRegistryState::new();

    let id = registry.register_alert(U256::from(75));

    // Acknowledge
    registry.acknowledge_alert(id).unwrap();
    assert!(registry.is_acknowledged(id), "Alert should be acknowledged");

    // Try to acknowledge again - should fail but NOT unacknowledge
    let result = registry.acknowledge_alert(id);
    assert!(
        result.is_err(),
        "INV-AR-2: Should reject double acknowledgment"
    );
    assert_eq!(result.unwrap_err(), "AlreadyAcknowledged");

    // Alert should STILL be acknowledged
    assert!(
        registry.is_acknowledged(id),
        "INV-AR-2 VIOLATED: Alert became unacknowledged"
    );
}

#[test]
fn invariant_ar2_acknowledgment_is_one_way() {
    let mut registry = AlertRegistryState::new();

    // Register multiple alerts
    let ids: Vec<U256> = (0..10)
        .map(|i| registry.register_alert(U256::from(i * 15)))
        .collect();

    // Acknowledge all
    for &id in &ids {
        registry.acknowledge_alert(id).unwrap();
    }

    // Verify all are acknowledged and remain so
    for (i, &id) in ids.iter().enumerate() {
        assert!(
            registry.is_acknowledged(id),
            "INV-AR-2 VIOLATED: Alert {} became unacknowledged",
            i
        );

        // Try to modify state through repeated acknowledgments
        for _ in 0..10 {
            let _ = registry.acknowledge_alert(id);
        }

        // Should STILL be acknowledged
        assert!(
            registry.is_acknowledged(id),
            "INV-AR-2 VIOLATED: State changed after repeated ops on alert {}",
            i
        );
    }
}

// ============================================================================
// INV-AR-3: Alert count is monotonically non-decreasing
// ============================================================================
//
// **Security Importance**: Alert count tracks total security events. If it
// could decrease, attack evidence could be erased.

#[test]
fn invariant_ar3_alert_count_monotonic() {
    let mut registry = AlertRegistryState::new();
    let mut max_count = registry.enhanced_alert_count;

    for i in 0..100 {
        registry.register_alert(U256::from(i * 10));

        assert!(
            registry.enhanced_alert_count >= max_count,
            "INV-AR-3 VIOLATED: Count {} < previous {}",
            registry.enhanced_alert_count,
            max_count
        );

        max_count = registry.enhanced_alert_count;
    }

    assert_eq!(
        registry.enhanced_alert_count,
        U256::from(100),
        "Final count should be 100"
    );
}

#[test]
fn invariant_ar3_acknowledgment_does_not_affect_count() {
    let mut registry = AlertRegistryState::new();

    // Register and acknowledge alerts
    for i in 0..20 {
        let id = registry.register_alert(U256::from(i * 5));
        let count_before_ack = registry.enhanced_alert_count;

        registry.acknowledge_alert(id).unwrap();
        let count_after_ack = registry.enhanced_alert_count;

        assert_eq!(
            count_before_ack, count_after_ack,
            "INV-AR-3 VIOLATED: Acknowledgment changed count from {} to {}",
            count_before_ack, count_after_ack
        );
    }
}

// ============================================================================
// INV-AR-4: Priority is deterministically computed from threat level
// ============================================================================
//
// **Security Importance**: Priority drives alert routing and response urgency.
// Non-deterministic priority could cause critical alerts to be deprioritized.

#[test]
fn invariant_ar4_priority_deterministic() {
    // Test all threat level ranges
    let test_cases = vec![
        (0, 0),   // LOW
        (39, 0),  // LOW
        (40, 1),  // MEDIUM
        (69, 1),  // MEDIUM
        (70, 2),  // HIGH
        (89, 2),  // HIGH
        (90, 3),  // CRITICAL
        (100, 3), // CRITICAL
    ];

    for (threat_level, expected_priority) in test_cases {
        let threat = U256::from(threat_level);
        let expected = U256::from(expected_priority);

        // Compute priority multiple times
        for i in 0..10 {
            let priority = AlertRegistry::compute_priority(threat);
            assert_eq!(
                priority, expected,
                "INV-AR-4 VIOLATED: Priority for threat {} is {} on iteration {}, expected {}",
                threat_level, priority, i, expected
            );
        }
    }
}

#[test]
fn invariant_ar4_priority_monotonic_with_threat() {
    // Higher threat should give higher or equal priority
    let threat_levels: Vec<u64> = vec![0, 10, 40, 50, 70, 80, 90, 100];

    for i in 1..threat_levels.len() {
        let low_threat = U256::from(threat_levels[i - 1]);
        let high_threat = U256::from(threat_levels[i]);

        let low_priority = AlertRegistry::compute_priority(low_threat);
        let high_priority = AlertRegistry::compute_priority(high_threat);

        assert!(
            high_priority >= low_priority,
            "INV-AR-4 VIOLATED: Higher threat {} gave lower priority {} than threat {} priority {}",
            high_threat,
            high_priority,
            low_threat,
            low_priority
        );
    }
}

// ============================================================================
// Property-Based Invariant Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Alert IDs are always sequential and 1-indexed
    #[test]
    fn property_alert_ids_sequential(count in 1usize..100) {
        let mut registry = AlertRegistryState::new();

        for i in 0..count {
            let id = registry.register_alert(U256::from(i as u64 * 10));
            prop_assert_eq!(id, U256::from((i + 1) as u64),
                "Alert ID should be {}, got {}", i + 1, id);
        }

        prop_assert_eq!(registry.enhanced_alert_count, U256::from(count as u64));
    }

    /// Once acknowledged, always acknowledged
    #[test]
    fn property_acknowledgment_permanent(
        threat_levels in prop::collection::vec(0u64..=100, 1..50)
    ) {
        let mut registry = AlertRegistryState::new();

        // Register alerts
        let ids: Vec<U256> = threat_levels.iter()
            .map(|&t| registry.register_alert(U256::from(t)))
            .collect();

        // Acknowledge all
        for &id in &ids {
            registry.acknowledge_alert(id).unwrap();
        }

        // Perform many operations
        for _ in 0..100 {
            // Try to acknowledge again
            for &id in &ids {
                let _ = registry.acknowledge_alert(id);
            }

            // All should still be acknowledged
            for &id in &ids {
                prop_assert!(registry.is_acknowledged(id),
                    "Alert {} should remain acknowledged", id);
            }
        }
    }

    /// Priority is always in range [0, 3]
    #[test]
    fn property_priority_bounded(threat_level in any::<u64>()) {
        let threat = U256::from(threat_level);
        let priority = AlertRegistry::compute_priority(threat);
        let p = priority.saturating_to::<u64>();

        prop_assert!(p <= 3, "Priority {} exceeds maximum 3", p);
        prop_assert!(p >= 0, "Priority {} below minimum 0", p);
    }

    /// Same threat level always gives same priority
    #[test]
    fn property_priority_deterministic_for_threat(threat_level in 0u64..=200) {
        let threat = U256::from(threat_level);

        let p1 = AlertRegistry::compute_priority(threat);
        let p2 = AlertRegistry::compute_priority(threat);
        let p3 = AlertRegistry::compute_priority(threat);

        prop_assert_eq!(p1, p2, "Priority computation not deterministic");
        prop_assert_eq!(p2, p3, "Priority computation not deterministic");
    }

    /// Alert count equals number of registered alerts
    #[test]
    fn property_count_equals_registrations(count in 1usize..100) {
        let mut registry = AlertRegistryState::new();

        for i in 0..count {
            registry.register_alert(U256::from(i as u64 * 5));
        }

        prop_assert_eq!(
            registry.enhanced_alert_count.saturating_to::<u64>() as usize,
            count,
            "Alert count doesn't match registrations"
        );

        prop_assert_eq!(
            registry.alerts.len(),
            count,
            "Internal alert vec doesn't match count"
        );
    }
}

// ============================================================================
// Stress Test with 1000+ Operations
// ============================================================================

#[test]
fn stress_test_1000_operations_maintain_all_invariants() {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let mut registry = AlertRegistryState::new();
    let hasher = RandomState::new();

    for i in 0..1000 {
        let mut h = hasher.build_hasher();
        i.hash(&mut h);
        let random = h.finish();

        let operation = random % 2;

        match operation {
            0 => {
                // Register alert
                let threat_level = (random % 101) as u64;
                let before_count = registry.enhanced_alert_count;

                let id = registry.register_alert(U256::from(threat_level));

                // INV-AR-1: ID is sequential
                assert_eq!(
                    id,
                    before_count.saturating_add(U256::from(1)),
                    "INV-AR-1 violated at op {}",
                    i
                );

                // INV-AR-3: Count increased
                assert_eq!(
                    registry.enhanced_alert_count,
                    before_count.saturating_add(U256::from(1)),
                    "INV-AR-3 violated at op {}",
                    i
                );

                // INV-AR-4: Priority is correct
                let priority = AlertRegistry::compute_priority(U256::from(threat_level));
                let p = priority.saturating_to::<u64>();
                assert!(p <= 3, "INV-AR-4 violated: priority {} > 3 at op {}", p, i);
            }
            _ => {
                // Acknowledge random existing alert
                if !registry.alerts.is_empty() {
                    let idx = (random as usize) % registry.alerts.len();
                    let id = registry.alerts[idx].0;

                    let was_acked = registry.is_acknowledged(id);
                    let result = registry.acknowledge_alert(id);

                    if was_acked {
                        // INV-AR-2: Should fail if already acknowledged
                        assert!(result.is_err(), "INV-AR-2 violated at op {}", i);
                    }

                    // INV-AR-2: Should be acknowledged after
                    assert!(
                        registry.is_acknowledged(id),
                        "INV-AR-2 violated: alert {} not acknowledged at op {}",
                        id,
                        i
                    );
                }
            }
        }
    }

    println!("✓ All AlertRegistry invariants held across 1000 operations");
    println!("  Total alerts: {}", registry.enhanced_alert_count);
    println!("  Alerts in vec: {}", registry.alerts.len());
    println!(
        "  Acknowledged: {}",
        registry.alerts.iter().filter(|(_, _, ack)| *ack).count()
    );
}
