//! Circuit Breaker Invariant Tests
//!
//! These tests verify critical properties that must ALWAYS hold true for the
//! CircuitBreaker contract, regardless of the sequence of operations performed.
//!
//! ## Security Importance
//!
//! Circuit breaker invariants are critical for:
//! - **Emergency Response**: Ensuring the circuit can always trip when needed
//! - **State Consistency**: Preventing stuck states that block resets
//! - **Monotonicity**: Trip count integrity for security auditing
//! - **Access Control**: Only authorized entities can modify state
//!
//! ## Invariants Tested
//!
//! - **INV-CB-1**: Trip count is monotonically non-decreasing
//! - **INV-CB-2**: Cannot trip when already tripped (idempotency)
//! - **INV-CB-3**: Cannot reset when not tripped (state guard)
//! - **INV-CB-4**: Trip count persists across reset operations
//! - **INV-CB-5**: Last trip time never decreases

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::U256;
use proptest::prelude::*;

// Simulate CircuitBreaker state for invariant testing
#[derive(Clone, Debug, PartialEq)]
struct CircuitBreakerState {
    is_tripped: bool,
    trip_count: U256,
    last_trip_time: U256,
}

impl CircuitBreakerState {
    fn new() -> Self {
        Self {
            is_tripped: false,
            trip_count: U256::ZERO,
            last_trip_time: U256::ZERO,
        }
    }

    fn trip(&mut self, timestamp: U256) -> Result<(), &'static str> {
        if self.is_tripped {
            return Err("AlreadyTripped");
        }
        self.is_tripped = true;
        self.trip_count = self.trip_count.saturating_add(U256::from(1));
        self.last_trip_time = timestamp;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), &'static str> {
        if !self.is_tripped {
            return Err("NotTripped");
        }
        self.is_tripped = false;
        Ok(())
    }
}

// ============================================================================
// INV-CB-1: Trip count is monotonically non-decreasing
// ============================================================================
//
// **Security Importance**: Trip count serves as an audit trail. If it could
// decrease, attackers could hide evidence of repeated attacks by manipulating
// the count.

#[test]
fn invariant_cb1_trip_count_monotonic() {
    let mut cb = CircuitBreakerState::new();
    let initial_count = cb.trip_count;

    // Perform various operations
    let _ = cb.trip(U256::from(1000));
    let count_after_trip = cb.trip_count;
    assert!(
        count_after_trip >= initial_count,
        "INV-CB-1 VIOLATED: Trip count decreased from {} to {}",
        initial_count,
        count_after_trip
    );

    let _ = cb.reset();
    let count_after_reset = cb.trip_count;
    assert!(
        count_after_reset >= count_after_trip,
        "INV-CB-1 VIOLATED: Trip count decreased after reset from {} to {}",
        count_after_trip,
        count_after_reset
    );

    // Trip again
    let _ = cb.trip(U256::from(2000));
    let final_count = cb.trip_count;
    assert!(
        final_count >= count_after_reset,
        "INV-CB-1 VIOLATED: Trip count decreased on second trip from {} to {}",
        count_after_reset,
        final_count
    );
}

#[test]
fn invariant_cb1_trip_count_never_decreases_across_sequences() {
    let mut cb = CircuitBreakerState::new();
    let mut max_seen = cb.trip_count;

    // Simulate 100 random operations
    for i in 0..100 {
        let timestamp = U256::from(i * 1000);

        // Try to trip
        let _ = cb.trip(timestamp);
        assert!(
            cb.trip_count >= max_seen,
            "INV-CB-1 VIOLATED: Trip count {} < previous max {}",
            cb.trip_count,
            max_seen
        );
        max_seen = cb.trip_count;

        // Try to reset
        let _ = cb.reset();
        assert!(
            cb.trip_count >= max_seen,
            "INV-CB-1 VIOLATED: Trip count {} < previous max {} after reset",
            cb.trip_count,
            max_seen
        );
        max_seen = cb.trip_count;
    }
}

// ============================================================================
// INV-CB-2: Cannot trip when already tripped (idempotency)
// ============================================================================
//
// **Security Importance**: Prevents double-counting of attacks and ensures
// state machine integrity. Without this, an attacker could artificially
// inflate trip counts by repeatedly calling trip().

#[test]
fn invariant_cb2_cannot_trip_when_tripped() {
    let mut cb = CircuitBreakerState::new();

    // First trip succeeds
    assert!(
        cb.trip(U256::from(1000)).is_ok(),
        "First trip should succeed"
    );
    assert!(cb.is_tripped, "Circuit should be tripped");

    // Second trip fails
    let result = cb.trip(U256::from(2000));
    assert!(
        result.is_err(),
        "INV-CB-2 VIOLATED: Second trip succeeded when already tripped"
    );
    assert_eq!(result.unwrap_err(), "AlreadyTripped");

    // State unchanged
    assert!(cb.is_tripped, "Circuit should still be tripped");
    assert_eq!(cb.trip_count, U256::from(1), "Trip count should be 1");
}

#[test]
fn invariant_cb2_multiple_trip_attempts_blocked() {
    let mut cb = CircuitBreakerState::new();

    cb.trip(U256::from(1000)).unwrap();
    let count_after_first = cb.trip_count;

    // Try to trip 10 more times
    for i in 1..=10 {
        let result = cb.trip(U256::from(1000 + i * 100));
        assert!(
            result.is_err(),
            "INV-CB-2 VIOLATED: Trip attempt {} succeeded",
            i
        );
        assert_eq!(
            cb.trip_count, count_after_first,
            "INV-CB-2 VIOLATED: Trip count changed on blocked attempt"
        );
    }
}

// ============================================================================
// INV-CB-3: Cannot reset when not tripped (state guard)
// ============================================================================
//
// **Security Importance**: Prevents state machine corruption. If reset could
// be called when not tripped, it could lead to undefined behavior or bypass
// safety checks.

#[test]
fn invariant_cb3_cannot_reset_when_not_tripped() {
    let mut cb = CircuitBreakerState::new();

    // Reset should fail when not tripped
    let result = cb.reset();
    assert!(
        result.is_err(),
        "INV-CB-3 VIOLATED: Reset succeeded when not tripped"
    );
    assert_eq!(result.unwrap_err(), "NotTripped");
    assert!(!cb.is_tripped, "Circuit should still not be tripped");
}

#[test]
fn invariant_cb3_reset_after_reset_fails() {
    let mut cb = CircuitBreakerState::new();

    // Trip, then reset
    cb.trip(U256::from(1000)).unwrap();
    cb.reset().unwrap();
    assert!(!cb.is_tripped, "Circuit should not be tripped after reset");

    // Second reset should fail
    let result = cb.reset();
    assert!(result.is_err(), "INV-CB-3 VIOLATED: Second reset succeeded");
    assert_eq!(result.unwrap_err(), "NotTripped");
}

// ============================================================================
// INV-CB-4: Trip count persists across reset operations
// ============================================================================
//
// **Security Importance**: Trip count is an immutable audit log. Resetting
// should only clear the "tripped" flag, not the historical count. This is
// critical for detecting repeated attacks.

#[test]
fn invariant_cb4_trip_count_persists_across_reset() {
    let mut cb = CircuitBreakerState::new();

    // Trip 3 times with resets in between
    cb.trip(U256::from(1000)).unwrap();
    assert_eq!(cb.trip_count, U256::from(1));

    cb.reset().unwrap();
    assert_eq!(
        cb.trip_count,
        U256::from(1),
        "INV-CB-4 VIOLATED: Trip count reset to zero"
    );

    cb.trip(U256::from(2000)).unwrap();
    assert_eq!(cb.trip_count, U256::from(2));

    cb.reset().unwrap();
    assert_eq!(
        cb.trip_count,
        U256::from(2),
        "INV-CB-4 VIOLATED: Trip count changed after second reset"
    );

    cb.trip(U256::from(3000)).unwrap();
    assert_eq!(cb.trip_count, U256::from(3));
}

#[test]
fn invariant_cb4_trip_count_accumulates_correctly() {
    let mut cb = CircuitBreakerState::new();

    for i in 1..=10 {
        cb.trip(U256::from(i * 1000)).unwrap();
        assert_eq!(cb.trip_count, U256::from(i), "Trip count should be {}", i);

        cb.reset().unwrap();
        assert_eq!(
            cb.trip_count,
            U256::from(i),
            "INV-CB-4 VIOLATED: Count changed after reset {}",
            i
        );
    }

    assert_eq!(cb.trip_count, U256::from(10), "Final count should be 10");
}

// ============================================================================
// INV-CB-5: Last trip time never decreases
// ============================================================================
//
// **Security Importance**: Timestamps provide temporal ordering for security
// events. If timestamps could go backward, it would break forensic analysis
// and could hide attack patterns.

#[test]
fn invariant_cb5_last_trip_time_never_decreases() {
    let mut cb = CircuitBreakerState::new();

    assert_eq!(cb.last_trip_time, U256::ZERO, "Initial time should be zero");

    // Trip with increasing timestamps
    cb.trip(U256::from(1000)).unwrap();
    assert_eq!(cb.last_trip_time, U256::from(1000));

    cb.reset().unwrap();
    assert_eq!(
        cb.last_trip_time,
        U256::from(1000),
        "INV-CB-5: Time should persist across reset"
    );

    cb.trip(U256::from(2000)).unwrap();
    assert!(
        cb.last_trip_time >= U256::from(2000),
        "INV-CB-5 VIOLATED: Time went backward"
    );

    cb.reset().unwrap();
    cb.trip(U256::from(3000)).unwrap();
    assert!(
        cb.last_trip_time >= U256::from(3000),
        "INV-CB-5 VIOLATED: Time decreased on third trip"
    );
}

// ============================================================================
// Property-Based Invariant Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// After ANY sequence of operations, trip count is monotonic
    #[test]
    fn property_trip_count_always_monotonic(ops in prop::collection::vec(prop::bool::ANY, 1..100)) {
        let mut cb = CircuitBreakerState::new();
        let mut max_count = U256::ZERO;

        for (i, should_trip) in ops.iter().enumerate() {
            let timestamp = U256::from(i as u64 * 1000);

            if *should_trip {
                let _ = cb.trip(timestamp);
            } else {
                let _ = cb.reset();
            }

            prop_assert!(cb.trip_count >= max_count,
                "INV-CB-1 VIOLATED: Count {} < max {}", cb.trip_count, max_count);
            max_count = cb.trip_count;
        }
    }

    /// State consistency: is_tripped iff last operation was successful trip
    #[test]
    fn property_state_consistency(ops in prop::collection::vec(prop::bool::ANY, 1..100)) {
        let mut cb = CircuitBreakerState::new();
        let mut last_was_trip = false;

        for (i, should_trip) in ops.iter().enumerate() {
            let timestamp = U256::from(i as u64 * 1000);

            if *should_trip {
                match cb.trip(timestamp) {
                    Ok(()) => last_was_trip = true,
                    Err(_) => {} // AlreadyTripped, state unchanged
                }
            } else {
                match cb.reset() {
                    Ok(()) => last_was_trip = false,
                    Err(_) => {} // NotTripped, state unchanged
                }
            }

            // is_tripped should match last successful operation
            prop_assert_eq!(cb.is_tripped, last_was_trip,
                "State inconsistency after operation {}", i);
        }
    }

    /// Trip count equals number of successful trip operations
    #[test]
    fn property_trip_count_equals_successful_trips(ops in prop::collection::vec(prop::bool::ANY, 1..100)) {
        let mut cb = CircuitBreakerState::new();
        let mut successful_trips = 0u64;

        for (i, should_trip) in ops.iter().enumerate() {
            let timestamp = U256::from(i as u64 * 1000);

            if *should_trip {
                if cb.trip(timestamp).is_ok() {
                    successful_trips += 1;
                }
            } else {
                let _ = cb.reset();
            }
        }

        prop_assert_eq!(cb.trip_count, U256::from(successful_trips),
            "Trip count doesn't match successful trips");
    }

    /// Last trip time never decreases across any operation sequence
    #[test]
    fn property_timestamp_monotonic(ops in prop::collection::vec(prop::bool::ANY, 1..100)) {
        let mut cb = CircuitBreakerState::new();
        let mut max_time = U256::ZERO;

        for (i, should_trip) in ops.iter().enumerate() {
            let timestamp = U256::from(i as u64 * 1000);

            if *should_trip {
                let _ = cb.trip(timestamp);
            } else {
                let _ = cb.reset();
            }

            prop_assert!(cb.last_trip_time >= max_time,
                "INV-CB-5 VIOLATED: Time {} < max {}", cb.last_trip_time, max_time);
            max_time = cb.last_trip_time;
        }
    }
}

// ============================================================================
// Stress Tests with 1000+ Operations
// ============================================================================

#[test]
fn stress_test_1000_random_operations_maintain_invariants() {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let mut cb = CircuitBreakerState::new();
    let hasher = RandomState::new();

    for i in 0..1000 {
        // Pseudo-random operation based on iteration
        let mut h = hasher.build_hasher();
        i.hash(&mut h);
        let should_trip = (h.finish() % 2) == 0;

        let before_count = cb.trip_count;
        let before_time = cb.last_trip_time;

        if should_trip {
            let _ = cb.trip(U256::from(i * 1000));
        } else {
            let _ = cb.reset();
        }

        // Check ALL invariants after EVERY operation
        assert!(
            cb.trip_count >= before_count,
            "INV-CB-1 violated at op {}",
            i
        );
        assert!(
            cb.last_trip_time >= before_time,
            "INV-CB-5 violated at op {}",
            i
        );

        // State machine invariants
        if cb.is_tripped {
            assert!(
                cb.trip(U256::from(i * 1000 + 500)).is_err(),
                "INV-CB-2 violated at op {}",
                i
            );
        } else {
            assert!(cb.reset().is_err(), "INV-CB-3 violated at op {}", i);
        }
    }

    println!("✓ All invariants held across 1000 operations");
    println!("  Final trip count: {}", cb.trip_count);
    println!(
        "  Final state: {}",
        if cb.is_tripped {
            "tripped"
        } else {
            "not tripped"
        }
    );
}
