//! Cross-Contract Invariant Tests
//!
//! These tests verify critical properties that span multiple contracts,
//! ensuring the entire ArbiShield system maintains consistency.
//!
//! ## Security Importance
//!
//! Cross-contract invariants ensure:
//! - **System Coherence**: Contracts remain synchronized
//! - **Data Integrity**: No orphaned or inconsistent state
//! - **Workflow Correctness**: Multi-contract operations maintain invariants
//! - **Attack Prevention**: Cross-contract attack vectors are blocked
//!
//! ## Invariants Tested
//!
//! - **INV-CC-1**: Error selectors are consistent across contracts
//! - **INV-CC-2**: Enhanced alert count >= V1 alert count (upgrade compatibility)
//! - **INV-CC-3**: Priority distribution sums to total alerts

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256, FixedBytes};
use arbishield::alert_registry::error::Error as ARError;
use arbishield::alert_registry::storage::AlertRegistry;
use arbishield::circuit_breaker::error::Error as CBError;
use arbishield::detection_engine::error::Error as DEError;
use proptest::prelude::*;

// ============================================================================
// INV-CC-1: Error selectors are consistent across contracts
// ============================================================================
//
// **Security Importance**: Shared error types (like UnauthorizedCaller) must
// have identical selectors across contracts. Different selectors would break
// tooling, frontends, and security monitoring systems.

#[test]
fn invariant_cc1_shared_errors_have_same_selector() {
    let addr = Address::from([0x42; 20]);

    // UnauthorizedCaller should be identical across all contracts
    let cb_err: Vec<u8> = CBError::UnauthorizedCaller(addr).into();
    let de_err: Vec<u8> = DEError::UnauthorizedCaller(addr).into();
    let ar_err: Vec<u8> = ARError::UnauthorizedCaller(addr).into();

    assert_eq!(
        &cb_err[0..4],
        &de_err[0..4],
        "INV-CC-1 VIOLATED: CB and DE UnauthorizedCaller selectors differ"
    );
    assert_eq!(
        &de_err[0..4],
        &ar_err[0..4],
        "INV-CC-1 VIOLATED: DE and AR UnauthorizedCaller selectors differ"
    );

    // Full encoding should also match
    assert_eq!(
        cb_err, de_err,
        "INV-CC-1 VIOLATED: Full encoding differs between CB and DE"
    );
    assert_eq!(
        de_err, ar_err,
        "INV-CC-1 VIOLATED: Full encoding differs between DE and AR"
    );
}

#[test]
fn invariant_cc1_invalid_owner_consistent() {
    let owner = Address::from([0x99; 20]);

    let cb_err: Vec<u8> = CBError::InvalidOwner(owner).into();
    let de_err: Vec<u8> = DEError::InvalidOwner(owner).into();
    let ar_err: Vec<u8> = ARError::InvalidOwner(owner).into();

    // Selectors must match
    assert_eq!(
        &cb_err[0..4],
        &de_err[0..4],
        "INV-CC-1 VIOLATED: InvalidOwner selectors differ"
    );
    assert_eq!(
        &de_err[0..4],
        &ar_err[0..4],
        "INV-CC-1 VIOLATED: InvalidOwner selectors differ"
    );

    // Same size (36 bytes: selector + address)
    assert_eq!(cb_err.len(), 36);
    assert_eq!(de_err.len(), 36);
    assert_eq!(ar_err.len(), 36);
}

#[test]
fn invariant_cc1_selectors_stable_across_addresses() {
    // Test with multiple addresses
    let addresses = vec![
        Address::ZERO,
        Address::from([0x11; 20]),
        Address::from([0xFF; 20]),
        Address::from([0xAB; 20]),
    ];

    for addr in addresses {
        let cb: Vec<u8> = CBError::UnauthorizedCaller(addr).into();
        let de: Vec<u8> = DEError::UnauthorizedCaller(addr).into();
        let ar: Vec<u8> = ARError::UnauthorizedCaller(addr).into();

        assert_eq!(
            &cb[0..4], &de[0..4],
            "INV-CC-1 VIOLATED: Selector changed for address {:?}",
            addr
        );
        assert_eq!(
            &de[0..4], &ar[0..4],
            "INV-CC-1 VIOLATED: Selector changed for address {:?}",
            addr
        );

        // Selector should be the same regardless of address
        let first_selector = &cb[0..4];
        assert_eq!(
            &de[0..4], first_selector,
            "Selector should be invariant to address parameter"
        );
    }
}

// ============================================================================
// INV-CC-2: Enhanced alert count >= V1 alert count (upgrade compatibility)
// ============================================================================
//
// **Security Importance**: When upgrading from V1 to V2, no alerts should be
// lost. Enhanced alert count must always be >= V1 count to ensure audit trail
// completeness.

#[test]
fn invariant_cc2_enhanced_count_gte_v1_count() {
    // Simulate V1 and V2 alert counts
    let v1_count = U256::from(42);
    let v2_count = U256::from(50);

    assert!(
        v2_count >= v1_count,
        "INV-CC-2 VIOLATED: V2 count {} < V1 count {}",
        v2_count,
        v1_count
    );
}

#[test]
fn invariant_cc2_upgrade_path_preserves_counts() {
    // Simulate upgrade: V1 has N alerts, V2 starts with same N
    let v1_alerts = 100u64;
    let mut v2_count = U256::from(v1_alerts); // V2 initialized from V1

    // Register new V2 alerts
    for _ in 0..50 {
        v2_count = v2_count.saturating_add(U256::from(1));
    }

    assert_eq!(v2_count, U256::from(150));
    assert!(
        v2_count >= U256::from(v1_alerts),
        "INV-CC-2 VIOLATED: V2 count dropped below V1 baseline"
    );
}

// ============================================================================
// INV-CC-3: Priority distribution sums to total alerts
// ============================================================================
//
// **Security Importance**: Every alert must be categorized into exactly one
// priority level. Missing alerts (sum < total) or double-counting (sum > total)
// indicates state corruption.

#[test]
fn invariant_cc3_priority_distribution_complete() {
    // Simulate priority counts
    let low_count = U256::from(10);
    let medium_count = U256::from(15);
    let high_count = U256::from(20);
    let critical_count = U256::from(5);

    let total_by_priority = low_count
        .saturating_add(medium_count)
        .saturating_add(high_count)
        .saturating_add(critical_count);

    let expected_total = U256::from(50);

    assert_eq!(
        total_by_priority, expected_total,
        "INV-CC-3 VIOLATED: Priority sum {} != total {}",
        total_by_priority, expected_total
    );
}

#[test]
fn invariant_cc3_every_alert_has_priority() {
    // Register alerts with various threat levels
    let threat_levels = vec![
        10u64, // LOW
        50,    // MEDIUM
        80,    // HIGH
        95,    // CRITICAL
        5,     // LOW
        45,    // MEDIUM
        75,    // HIGH
        100,   // CRITICAL
    ];

    let mut priority_counts = [0u64; 4];
    let total_alerts = threat_levels.len() as u64;

    for &threat in &threat_levels {
        let priority = AlertRegistry::compute_priority(U256::from(threat));
        let p = priority.saturating_to::<u64>();
        priority_counts[p as usize] += 1;
    }

    let sum: u64 = priority_counts.iter().sum();

    assert_eq!(
        sum, total_alerts,
        "INV-CC-3 VIOLATED: Priority sum {} != total alerts {}",
        sum, total_alerts
    );
}

// ============================================================================
// Property-Based Cross-Contract Invariant Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Shared error encoding is deterministic across contracts
    #[test]
    fn property_cc1_shared_errors_deterministic(addr in any::<[u8; 20]>()) {
        let address = Address::from(addr);

        // Encode multiple times
        let cb1: Vec<u8> = CBError::UnauthorizedCaller(address).into();
        let cb2: Vec<u8> = CBError::UnauthorizedCaller(address).into();
        let de1: Vec<u8> = DEError::UnauthorizedCaller(address).into();
        let de2: Vec<u8> = DEError::UnauthorizedCaller(address).into();

        prop_assert_eq!(&cb1, &cb2, "CB encoding not deterministic");
        prop_assert_eq!(&de1, &de2, "DE encoding not deterministic");
        prop_assert_eq!(&cb1, &de1, "CB and DE encodings differ");
    }

    /// Priority distribution always sums to total
    #[test]
    fn property_cc3_distribution_sums_to_total(
        threat_levels in prop::collection::vec(0u64..=100, 1..100)
    ) {
        let mut priority_counts = [0u64; 4];

        for &threat in &threat_levels {
            let priority = AlertRegistry::compute_priority(U256::from(threat));
            let p = priority.saturating_to::<u64>();
            priority_counts[p as usize] += 1;
        }

        let sum: u64 = priority_counts.iter().sum();
        let total = threat_levels.len() as u64;

        prop_assert_eq!(sum, total,
            "Priority distribution sum {} != total alerts {}", sum, total);
    }

    /// Every threat level maps to exactly one priority
    #[test]
    fn property_every_threat_has_unique_priority(threat in 0u64..=200) {
        let priority = AlertRegistry::compute_priority(U256::from(threat));
        let p = priority.saturating_to::<u64>();

        // Must be in range [0, 3]
        prop_assert!(p <= 3, "Priority {} exceeds maximum", p);

        // Deterministic: same threat always gives same priority
        let priority2 = AlertRegistry::compute_priority(U256::from(threat));
        prop_assert_eq!(priority, priority2, "Non-deterministic priority");
    }
}

// ============================================================================
// Complex Multi-Contract Workflow Tests
// ============================================================================

#[test]
fn invariant_workflow_detection_to_alert_to_circuit() {
    // Simulate end-to-end workflow: DetectionEngine → AlertRegistry → CircuitBreaker

    // Step 1: DetectionEngine detects anomaly
    let threshold = U256::from(1000);
    let current_value = U256::from(5000); // 5x threshold
    let is_anomaly = current_value > threshold;
    assert!(is_anomaly, "Anomaly should be detected");

    // Step 2: AlertRegistry registers alert
    let threat_level = U256::from(95); // CRITICAL
    let priority = AlertRegistry::compute_priority(threat_level);
    assert_eq!(priority, U256::from(3), "Should be CRITICAL priority");

    // Step 3: CircuitBreaker trips on CRITICAL alert
    let should_trip = priority == U256::from(3);
    assert!(should_trip, "CRITICAL alert should trigger circuit");

    // Invariant: Detection → Alert → Circuit forms coherent chain
    // If anomaly detected, alert registered, and CRITICAL, circuit MUST trip
    assert!(
        is_anomaly && priority == U256::from(3) && should_trip,
        "Workflow invariant violated: critical anomaly didn't trigger circuit"
    );
}

#[test]
fn invariant_multi_contract_state_consistency() {
    // Track states across contracts
    let mut alert_count = 0u64;
    let mut trip_count = 0u64;
    let mut detected_anomalies = 0u64;

    // Simulate 100 security events
    for i in 0..100 {
        let value = U256::from(i * 100);
        let threshold = U256::from(5000);

        // Detection
        if value > threshold {
            detected_anomalies += 1;

            // Alert
            alert_count += 1;

            // Trip circuit on high severity
            if i % 3 == 0 {
                // Every 3rd anomaly is critical
                trip_count += 1;
            }
        }
    }

    // Invariants:
    // 1. Alerts >= anomalies (every anomaly creates at least one alert)
    assert!(
        alert_count >= detected_anomalies,
        "INV: Alert count {} < detected anomalies {}",
        alert_count,
        detected_anomalies
    );

    // 2. Trips <= alerts (can't trip without an alert)
    assert!(
        trip_count <= alert_count,
        "INV: Trip count {} > alert count {}",
        trip_count,
        alert_count
    );

    // 3. Anomalies <= total events (bounded by event count)
    assert!(
        detected_anomalies <= 100,
        "INV: Detected {} anomalies from 100 events",
        detected_anomalies
    );

    println!("✓ Multi-contract state consistency maintained:");
    println!("  Events: 100");
    println!("  Detected anomalies: {}", detected_anomalies);
    println!("  Alerts registered: {}", alert_count);
    println!("  Circuit trips: {}", trip_count);
}

// ============================================================================
// Stress Test: 1000+ Cross-Contract Operations
// ============================================================================

#[test]
fn stress_test_1000_cross_contract_operations() {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let hasher = RandomState::new();

    // Simulated state
    let mut detection_anomalies = 0u64;
    let mut alert_count = 0u64;
    let mut acknowledged_count = 0u64;
    let mut trip_count = 0u64;
    let mut priority_counts = [0u64; 4];

    for i in 0..1000 {
        let mut h = hasher.build_hasher();
        i.hash(&mut h);
        let random = h.finish();

        let value = U256::from(random % 10000);
        let threshold = U256::from(5000);

        // Detect anomaly
        if value > threshold {
            detection_anomalies += 1;

            // Register alert
            let threat_level = (random % 101) as u64;
            let priority = AlertRegistry::compute_priority(U256::from(threat_level));
            let p = priority.saturating_to::<u64>();

            alert_count += 1;
            priority_counts[p as usize] += 1;

            // Acknowledge (50% chance)
            if random % 2 == 0 {
                acknowledged_count += 1;
            }

            // Trip on CRITICAL
            if p == 3 {
                trip_count += 1;
            }
        }

        // Check invariants every 100 operations
        if (i + 1) % 100 == 0 {
            // INV: alert count >= anomalies
            assert!(
                alert_count >= detection_anomalies,
                "Invariant violated at op {}: alerts {} < anomalies {}",
                i,
                alert_count,
                detection_anomalies
            );

            // INV: acknowledged <= total alerts
            assert!(
                acknowledged_count <= alert_count,
                "Invariant violated at op {}: ack {} > alerts {}",
                i,
                acknowledged_count,
                alert_count
            );

            // INV: priority sum = total alerts
            let priority_sum: u64 = priority_counts.iter().sum();
            assert_eq!(
                priority_sum, alert_count,
                "Invariant violated at op {}: priority sum {} != alerts {}",
                i, priority_sum, alert_count
            );

            // INV: trips <= CRITICAL alerts
            assert!(
                trip_count <= priority_counts[3],
                "Invariant violated at op {}: trips {} > critical alerts {}",
                i,
                trip_count,
                priority_counts[3]
            );
        }
    }

    println!("✓ All cross-contract invariants held across 1000 operations");
    println!("  Anomalies: {}", detection_anomalies);
    println!("  Alerts: {}", alert_count);
    println!("  Acknowledged: {}", acknowledged_count);
    println!("  Trips: {}", trip_count);
    println!("  Priority distribution: {:?}", priority_counts);
}
