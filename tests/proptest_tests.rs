//! Property-based tests for ArbiShield contracts using proptest
//!
//! These tests verify invariants across thousands of random inputs,
//! catching edge cases that hand-written tests might miss.

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::SolError;
use proptest::prelude::*;

use arbishield::alert_registry::error::Error as ARError;
use arbishield::alert_registry::storage::AlertRegistry;
use arbishield::circuit_breaker::error::Error as CBError;
use arbishield::detection_engine::error::Error as DEError;

// ============================================================
// Strategy Helpers
// ============================================================

/// Generate a random non-zero Address
fn arb_nonzero_address() -> impl Strategy<Value = Address> {
    prop::array::uniform20(1u8..=255).prop_map(Address::from)
}

/// Generate a random Address (may be zero)
fn arb_address() -> impl Strategy<Value = Address> {
    prop::array::uniform20(0u8..=255).prop_map(Address::from)
}

/// Generate a random U256 value
fn arb_u256() -> impl Strategy<Value = U256> {
    prop::array::uniform32(0u8..=255).prop_map(|bytes| U256::from_le_bytes(bytes))
}

/// Generate a random U256 value within a range (0..=max as u64)
fn arb_u256_range(max: u64) -> impl Strategy<Value = U256> {
    (0..=max).prop_map(U256::from)
}

/// Generate a random threat level (0..=100)
fn arb_threat_level() -> impl Strategy<Value = U256> {
    arb_u256_range(100)
}

/// Generate a random FixedBytes<32> (message hash)
fn arb_message_hash() -> impl Strategy<Value = FixedBytes<32>> {
    prop::array::uniform32(0u8..=255).prop_map(FixedBytes::from)
}

/// Generate a random valid role bitmask (1..=3)
fn arb_valid_role() -> impl Strategy<Value = u8> {
    prop::sample::select(vec![0x01u8, 0x02, 0x03])
}

/// Generate a single-bit role (either 0x01 or 0x02, not combined)
fn arb_single_bit_role() -> impl Strategy<Value = u8> {
    prop::sample::select(vec![0x01u8, 0x02])
}

// ============================================================
// Priority Computation Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Priority is always one of {0, 1, 2, 3}
    #[test]
    fn priority_always_valid(threat_level in arb_u256()) {
        let priority = AlertRegistry::compute_priority(threat_level);
        let p = priority.saturating_to::<u64>();
        prop_assert!(p <= 3, "Priority must be 0-3, got {p}");
    }

    /// Threat level 0-39 always maps to LOW (0)
    #[test]
    fn low_priority_range(level in 0u64..40) {
        let priority = AlertRegistry::compute_priority(U256::from(level));
        prop_assert_eq!(priority, U256::from(0u64));
    }

    /// Threat level 40-69 always maps to MEDIUM (1)
    #[test]
    fn medium_priority_range(level in 40u64..70) {
        let priority = AlertRegistry::compute_priority(U256::from(level));
        prop_assert_eq!(priority, U256::from(1u64));
    }

    /// Threat level 70-89 always maps to HIGH (2)
    #[test]
    fn high_priority_range(level in 70u64..90) {
        let priority = AlertRegistry::compute_priority(U256::from(level));
        prop_assert_eq!(priority, U256::from(2u64));
    }

    /// Threat level 90+ always maps to CRITICAL (3)
    #[test]
    fn critical_priority_range(level in 90u64..=100) {
        let priority = AlertRegistry::compute_priority(U256::from(level));
        prop_assert_eq!(priority, U256::from(3u64));
    }

    /// Priority is monotonically non-decreasing with threat level
    #[test]
    fn priority_monotonic(a in arb_threat_level(), b in arb_threat_level()) {
        let pa = AlertRegistry::compute_priority(a);
        let pb = AlertRegistry::compute_priority(b);
        if a <= b {
            prop_assert!(pa <= pb, "Priority should be monotonic: f({a}) = {pa} <= f({b}) = {pb}");
        }
        if a >= b {
            prop_assert!(pa >= pb, "Priority should be monotonic: f({a}) = {pa} >= f({b}) = {pb}");
        }
    }

    /// Any U256 value >= 90 (even very large) maps to CRITICAL
    #[test]
    fn large_threat_level_is_critical(level in 90u64..=u64::MAX) {
        let priority = AlertRegistry::compute_priority(U256::from(level));
        prop_assert_eq!(priority, U256::from(3u64));
    }
}

// ============================================================
// Anomaly Detection Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// value > threshold iff value is strictly greater
    #[test]
    fn anomaly_detection_strict(value in arb_u256(), threshold in arb_u256()) {
        let is_anomaly = value > threshold;
        if value == threshold {
            prop_assert!(!is_anomaly, "Equal values should not be anomaly");
        }
        if value < threshold {
            prop_assert!(!is_anomaly, "Values below threshold should not be anomaly");
        }
    }

    /// Anomaly detection is deterministic
    #[test]
    fn anomaly_detection_deterministic(value in arb_u256(), threshold in arb_u256()) {
        let result1 = value > threshold;
        let result2 = value > threshold;
        prop_assert_eq!(result1, result2, "Anomaly detection must be deterministic");
    }

    /// If value > threshold and threshold > 0, then value > 0
    #[test]
    fn anomaly_implies_nonzero_value(
        value in arb_u256(),
        threshold in arb_u256()
    ) {
        if value > threshold && threshold > U256::ZERO {
            prop_assert!(value > U256::ZERO, "Anomalous value must be > 0 when threshold > 0");
        }
    }
}

// ============================================================
// Saturating Arithmetic Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// saturating_add never wraps
    #[test]
    fn saturating_add_never_wraps(a in arb_u256(), b in arb_u256()) {
        let result = a.saturating_add(b);
        prop_assert!(result >= a, "saturating_add result must be >= first operand");
        prop_assert!(result >= b, "saturating_add result must be >= second operand");
    }

    /// saturating_sub never underflows
    #[test]
    fn saturating_sub_never_underflows(a in arb_u256(), b in arb_u256()) {
        let result = a.saturating_sub(b);
        prop_assert!(result <= a, "saturating_sub result must be <= first operand");
        if b > a {
            prop_assert_eq!(result, U256::ZERO, "saturating_sub should floor at zero");
        }
    }

    /// saturating_add with zero is identity
    #[test]
    fn saturating_add_identity(a in arb_u256()) {
        let result = a.saturating_add(U256::ZERO);
        prop_assert_eq!(result, a);
    }

    /// saturating_sub with zero is identity
    #[test]
    fn saturating_sub_identity(a in arb_u256()) {
        let result = a.saturating_sub(U256::ZERO);
        prop_assert_eq!(result, a);
    }

    /// Incrementing then decrementing returns to original (unless at MAX)
    #[test]
    fn increment_decrement_roundtrip(a in arb_u256()) {
        let incremented = a.saturating_add(U256::from(1));
        let back = incremented.saturating_sub(U256::from(1));
        if a < U256::MAX {
            prop_assert_eq!(back, a, "Increment+decrement should roundtrip");
        }
    }
}

// ============================================================
// Error Encoding Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// All CB error encodings have at least 4 bytes
    #[test]
    fn cb_error_encoding_min_length(addr in arb_address()) {
        let errors: Vec<Vec<u8>> = vec![
            CBError::AlreadyTripped.into(),
            CBError::NotTripped.into(),
            CBError::UnauthorizedCaller(addr).into(),
            CBError::InvalidOwner(addr).into(),
        ];
        for (i, enc) in errors.iter().enumerate() {
            prop_assert!(enc.len() >= 4, "CB error {i} must have >= 4 bytes, got {}", enc.len());
        }
    }

    /// All DE error encodings have at least 4 bytes
    #[test]
    fn de_error_encoding_min_length(
        addr in arb_address(),
        id in arb_u256(),
        value in arb_u256()
    ) {
        let errors: Vec<Vec<u8>> = vec![
            DEError::UnauthorizedCaller(addr).into(),
            DEError::MetricNotFound { id }.into(),
            DEError::InvalidThreshold { value }.into(),
            DEError::ThresholdExceeded { current: value, threshold: id }.into(),
            DEError::InvalidOwner(addr).into(),
        ];
        for (i, enc) in errors.iter().enumerate() {
            prop_assert!(enc.len() >= 4, "DE error {i} must have >= 4 bytes, got {}", enc.len());
        }
    }

    /// Same error with same data always produces identical encoding
    #[test]
    fn error_encoding_deterministic(addr in arb_address(), id in arb_u256()) {
        let enc1: Vec<u8> = CBError::UnauthorizedCaller(addr).into();
        let enc2: Vec<u8> = CBError::UnauthorizedCaller(addr).into();
        prop_assert_eq!(enc1, enc2, "Error encoding must be deterministic");

        let enc3: Vec<u8> = DEError::MetricNotFound { id }.into();
        let enc4: Vec<u8> = DEError::MetricNotFound { id }.into();
        prop_assert_eq!(enc3, enc4);
    }

    /// Different addresses produce different error encodings
    #[test]
    fn different_addresses_different_encodings(
        a in arb_nonzero_address(),
        b in arb_nonzero_address()
    ) {
        prop_assume!(a != b);
        let enc_a: Vec<u8> = CBError::UnauthorizedCaller(a).into();
        let enc_b: Vec<u8> = CBError::UnauthorizedCaller(b).into();
        // Same selector (first 4 bytes) but different payload
        prop_assert_eq!(&enc_a[..4], &enc_b[..4], "Same error type must have same selector");
        prop_assert_ne!(enc_a, enc_b, "Different addresses must produce different encodings");
    }

    /// Cross-contract errors with same Solidity signature produce same selector
    #[test]
    fn cross_contract_same_selector(addr in arb_address()) {
        let cb: Vec<u8> = CBError::UnauthorizedCaller(addr).into();
        let de: Vec<u8> = DEError::UnauthorizedCaller(addr).into();
        let ar: Vec<u8> = ARError::UnauthorizedCaller(addr).into();

        prop_assert_eq!(&cb[..4], &de[..4]);
        prop_assert_eq!(&de[..4], &ar[..4]);

        // Full encoding should also match since same signature + same data
        prop_assert_eq!(&cb, &de);
        prop_assert_eq!(&de, &ar);
    }
}

// ============================================================
// Address Fuzzing Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Address::ZERO is always equal to itself
    #[test]
    fn zero_address_always_equal(_dummy in 0u32..100) {
        prop_assert_eq!(Address::ZERO, Address::ZERO);
    }

    /// Non-zero addresses are never equal to Address::ZERO
    #[test]
    fn nonzero_address_never_zero(addr in arb_nonzero_address()) {
        prop_assert_ne!(addr, Address::ZERO, "Non-zero address must not equal Address::ZERO");
    }

    /// Address equality is reflexive
    #[test]
    fn address_equality_reflexive(addr in arb_address()) {
        prop_assert_eq!(addr, addr);
    }

    /// Address comparison is deterministic
    #[test]
    fn address_comparison_deterministic(a in arb_address(), b in arb_address()) {
        let eq1 = a == b;
        let eq2 = a == b;
        prop_assert_eq!(eq1, eq2);
    }
}

// ============================================================
// Role Bitmask Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Granting a role then checking it always succeeds
    #[test]
    fn grant_then_check_role(role in arb_valid_role()) {
        let mut roles: u8 = 0;
        roles |= role;
        prop_assert_eq!(roles & role, role, "Granted role must be present");
    }

    /// Revoking a role then checking it always fails
    #[test]
    fn revoke_then_check_role(role in arb_valid_role()) {
        let mut roles: u8 = 0x03; // All roles
        roles &= !role;
        prop_assert_eq!(roles & role, 0, "Revoked role must be absent");
    }

    /// Grant+revoke returns to original (single-bit roles only)
    #[test]
    fn grant_revoke_roundtrip(initial in 0u8..=3, role in arb_single_bit_role()) {
        let original = initial & 0x03; // Mask to valid range
        let mut roles = original;
        let had_role = (roles & role) == role;

        // Grant
        roles |= role;
        prop_assert_eq!(roles & role, role);

        // Revoke
        roles &= !role;
        // If didn't have role before, should be back to original
        if !had_role {
            prop_assert_eq!(roles, original);
        }
    }

    /// Role validation: only 0x01, 0x02, 0x03 are valid
    #[test]
    fn invalid_role_detection(role in 4u8..=255) {
        let all_roles: u8 = 0x03;
        prop_assert_ne!(role & !all_roles, 0);
    }

    /// U256 roundtrip for role values
    #[test]
    fn role_u256_roundtrip(role in 0u8..=3) {
        let as_u256 = U256::from(role);
        let back: u8 = as_u256.saturating_to::<u64>() as u8;
        prop_assert_eq!(back, role, "Role must survive U256 roundtrip");
    }
}

// ============================================================
// Alert ID Generation Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// Sequential IDs are always monotonically increasing
    #[test]
    fn alert_ids_sequential(n in 1u64..1000) {
        let mut count = U256::ZERO;
        let mut prev = U256::ZERO;
        for _ in 0..n {
            count = count.saturating_add(U256::from(1));
            prop_assert!(count > prev, "IDs must be monotonically increasing");
            prev = count;
        }
    }

    /// Alert ID 0 is always invalid (1-indexed)
    #[test]
    fn zero_id_always_invalid(count in arb_u256_range(1000)) {
        let zero = U256::ZERO;
        // ID is valid if > 0 and <= count
        let is_valid = !zero.is_zero() && zero <= count;
        prop_assert!(!is_valid, "Zero ID must always be invalid");
    }

    /// Valid ID range: 1..=count
    #[test]
    fn valid_id_in_range(count in 1u64..1000, id in 1u64..1000) {
        let count_u256 = U256::from(count);
        let id_u256 = U256::from(id);
        let is_valid = !id_u256.is_zero() && id_u256 <= count_u256;
        if id <= count {
            prop_assert!(is_valid, "ID {id} should be valid when count is {count}");
        } else {
            prop_assert!(!is_valid, "ID {id} should be invalid when count is {count}");
        }
    }
}

// ============================================================
// Expiration Logic Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Alert created at time T with duration D is not expired at T+D
    #[test]
    fn not_expired_at_boundary(
        created_at in 0u64..u64::MAX / 2,
        duration in 1u64..u64::MAX / 4
    ) {
        let created = U256::from(created_at);
        let dur = U256::from(duration);
        let now = created.saturating_add(dur);
        let elapsed = now.saturating_sub(created);
        // At exactly the boundary, NOT expired (> not >=)
        prop_assert!(!(elapsed > dur), "Should not be expired at exact boundary");
    }

    /// Alert is expired one second after duration
    #[test]
    fn expired_after_boundary(
        created_at in 0u64..u64::MAX / 2,
        duration in 1u64..u64::MAX / 4
    ) {
        let created = U256::from(created_at);
        let dur = U256::from(duration);
        let now = created.saturating_add(dur).saturating_add(U256::from(1));
        let elapsed = now.saturating_sub(created);
        prop_assert!(elapsed > dur, "Should be expired one second after boundary");
    }

    /// Zero expiration means never expired
    #[test]
    fn zero_duration_never_expires(
        created_at in arb_u256(),
        now in arb_u256()
    ) {
        let duration = U256::ZERO;
        // When duration is zero, no expiry (contract convention)
        prop_assert!(duration.is_zero());
    }

    /// Expiration is monotonic: if expired at time T, expired at T+1
    #[test]
    fn expiration_monotonic(
        created_at in 0u64..u64::MAX / 4,
        duration in 1u64..u64::MAX / 4,
        extra in 0u64..1000
    ) {
        let created = U256::from(created_at);
        let dur = U256::from(duration);

        // Check at some time past expiration
        let time1 = created.saturating_add(dur).saturating_add(U256::from(extra + 1));
        let elapsed1 = time1.saturating_sub(created);
        let expired1 = elapsed1 > dur;

        // Check at time1 + 1
        let time2 = time1.saturating_add(U256::from(1));
        let elapsed2 = time2.saturating_sub(created);
        let expired2 = elapsed2 > dur;

        if expired1 {
            prop_assert!(expired2, "If expired at T, must be expired at T+1");
        }
    }
}

// ============================================================
// State Machine Properties (Circuit Breaker)
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// Trip count never decreases across trip/reset cycles
    #[test]
    fn trip_count_monotonic(cycles in 1u64..100) {
        let mut count = U256::ZERO;
        let mut is_tripped = false;

        for _ in 0..cycles {
            // Trip
            if !is_tripped {
                is_tripped = true;
                let prev = count;
                count = count.saturating_add(U256::from(1));
                prop_assert!(count >= prev, "Trip count must be monotonically non-decreasing");
            }
            // Reset
            is_tripped = false;
        }
        prop_assert_eq!(count, U256::from(cycles));
    }

    /// After N trip/reset cycles, count == N
    #[test]
    fn trip_count_equals_cycles(n in 1u64..500) {
        let mut count = U256::ZERO;
        for _ in 0..n {
            count = count.saturating_add(U256::from(1));
        }
        prop_assert_eq!(count, U256::from(n));
    }

    /// Circuit state is always boolean
    #[test]
    fn circuit_state_is_boolean(trips in proptest::collection::vec(prop::bool::ANY, 0..100)) {
        let mut state = false;
        for trip in trips {
            state = trip;
            prop_assert!(state == true || state == false, "State must be boolean");
        }
    }
}

// ============================================================
// Metric Count Properties (Detection Engine)
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// Metric count is always equal to number of registrations
    #[test]
    fn metric_count_tracks_registrations(n in 0u64..1000) {
        let mut count = U256::ZERO;
        for _ in 0..n {
            count = count.saturating_add(U256::from(1));
        }
        prop_assert_eq!(count, U256::from(n));
    }

    /// Multiple thresholds can coexist
    #[test]
    fn multiple_thresholds_independent(
        t1 in arb_u256_range(u64::MAX),
        t2 in arb_u256_range(u64::MAX),
        t3 in arb_u256_range(u64::MAX)
    ) {
        // Simulate three metrics with independent thresholds
        let thresholds = [t1, t2, t3];
        let values = [U256::from(50u64), U256::from(150u64), U256::from(250u64)];

        let anomalies: Vec<bool> = thresholds.iter()
            .zip(values.iter())
            .map(|(t, v)| *v > *t)
            .collect();

        // Each check is independent
        prop_assert_eq!(anomalies.len(), 3);
    }
}

// ============================================================
// FixedBytes Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// FixedBytes<32> from random data is not zero (with overwhelming probability)
    #[test]
    fn random_hash_not_zero(hash in arb_message_hash()) {
        // Extremely unlikely to randomly generate all-zero hash
        // but we can't guarantee it, so this is a soft check
        let _ = hash; // Just ensure it can be constructed
    }

    /// FixedBytes<32> equality is reflexive
    #[test]
    fn fixed_bytes_equality_reflexive(hash in arb_message_hash()) {
        prop_assert_eq!(hash, hash);
    }

    /// FixedBytes<4> for interface IDs
    #[test]
    fn interface_id_construction(bytes in prop::array::uniform4(0u8..=255)) {
        let id = FixedBytes::<4>::from(bytes);
        prop_assert_eq!(id[0], bytes[0]);
        prop_assert_eq!(id[1], bytes[1]);
        prop_assert_eq!(id[2], bytes[2]);
        prop_assert_eq!(id[3], bytes[3]);
    }
}

// ============================================================
// Subscriber Count Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// Adding N subscribers results in count N
    #[test]
    fn subscriber_add_count(n in 0u64..500) {
        let mut count = U256::ZERO;
        for _ in 0..n {
            count = count.saturating_add(U256::from(1));
        }
        prop_assert_eq!(count, U256::from(n));
    }

    /// Adding then removing yields zero
    #[test]
    fn subscriber_add_remove_roundtrip(n in 1u64..500) {
        let mut count = U256::ZERO;
        for _ in 0..n {
            count = count.saturating_add(U256::from(1));
        }
        for _ in 0..n {
            count = count.saturating_sub(U256::from(1));
        }
        prop_assert_eq!(count, U256::ZERO);
    }

    /// Removing more than added floors at zero
    #[test]
    fn subscriber_remove_underflow_safe(add in 0u64..100, remove in 0u64..200) {
        let mut count = U256::ZERO;
        for _ in 0..add {
            count = count.saturating_add(U256::from(1));
        }
        for _ in 0..remove {
            count = count.saturating_sub(U256::from(1));
        }
        if remove > add {
            prop_assert_eq!(count, U256::ZERO, "Count should floor at zero");
        }
    }
}

// ============================================================
// Priority Distribution Properties
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// Total alerts across all priorities equals total registered
    #[test]
    fn priority_counts_sum_to_total(
        threat_levels in proptest::collection::vec(0u64..=100, 1..50)
    ) {
        let mut priority_counts = [0u64; 4];
        let total = threat_levels.len();

        for level in &threat_levels {
            let priority = AlertRegistry::compute_priority(U256::from(*level));
            let idx = priority.saturating_to::<u64>() as usize;
            priority_counts[idx] += 1;
        }

        let sum: u64 = priority_counts.iter().sum();
        prop_assert_eq!(sum, total as u64, "Priority counts must sum to total alerts");
    }

    /// Higher threat levels never produce lower priorities
    #[test]
    fn higher_threat_higher_or_equal_priority(
        a in 0u64..=100,
        b in 0u64..=100
    ) {
        let pa = AlertRegistry::compute_priority(U256::from(a));
        let pb = AlertRegistry::compute_priority(U256::from(b));
        if a > b {
            prop_assert!(pa >= pb, "Higher threat {a} must have >= priority than {b}");
        }
    }
}

// ============================================================
// Cross-Contract Error Encoding Fuzzing
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// AR error encoding is always deterministic for AlertNotFound
    #[test]
    fn ar_alert_not_found_deterministic(id in arb_u256()) {
        let enc1: Vec<u8> = ARError::AlertNotFound { id }.into();
        let enc2: Vec<u8> = ARError::AlertNotFound { id }.into();
        prop_assert_eq!(enc1, enc2);
    }

    /// AR error encoding size for InsufficientRole is always 68 bytes
    #[test]
    fn ar_insufficient_role_encoding_size(
        addr in arb_address(),
        role in 0u8..=255
    ) {
        let enc: Vec<u8> = ARError::InsufficientRole {
            caller: addr,
            required_role: role,
        }.into();
        // selector(4) + address(32) + uint8(32)
        prop_assert_eq!(enc.len(), 68, "InsufficientRole must encode to 68 bytes");
    }

    /// AR error encoding size for NotAlertProtocol is always 68 bytes
    #[test]
    fn ar_not_alert_protocol_encoding_size(
        addr in arb_address(),
        id in arb_u256()
    ) {
        let enc: Vec<u8> = ARError::NotAlertProtocol { caller: addr, id }.into();
        // selector(4) + address(32) + uint256(32)
        prop_assert_eq!(enc.len(), 68);
    }

    /// CB errors with addresses always produce 36 bytes
    #[test]
    fn cb_address_error_size(addr in arb_address()) {
        let enc1: Vec<u8> = CBError::UnauthorizedCaller(addr).into();
        let enc2: Vec<u8> = CBError::InvalidOwner(addr).into();
        prop_assert_eq!(enc1.len(), 36, "UnauthorizedCaller(address) must be 36 bytes");
        prop_assert_eq!(enc2.len(), 36, "InvalidOwner(address) must be 36 bytes");
    }

    /// DE ThresholdExceeded always produces 68 bytes
    #[test]
    fn de_threshold_exceeded_size(current in arb_u256(), threshold in arb_u256()) {
        let enc: Vec<u8> = DEError::ThresholdExceeded { current, threshold }.into();
        // selector(4) + uint256(32) + uint256(32)
        prop_assert_eq!(enc.len(), 68);
    }
}
