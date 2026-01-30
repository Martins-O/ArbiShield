#![no_main]

use libfuzzer_sys::fuzz_target;
use alloy_primitives::U256;

extern crate alloc;

/// Simulate CircuitBreaker state transitions
#[derive(Clone, Debug)]
struct CircuitBreakerSim {
    is_tripped: bool,
    trip_count: U256,
    last_trip_time: U256,
}

impl CircuitBreakerSim {
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
        // Note: trip_count and last_trip_time persist
        Ok(())
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let mut cb = CircuitBreakerSim::new();
    let initial_count = cb.trip_count;

    // Simulate random operations
    for (i, &byte) in data.iter().enumerate() {
        let operation = byte % 3; // 0 = trip, 1 = reset, 2 = query
        let timestamp = U256::from((i as u64) * 1000);

        match operation {
            0 => {
                // Trip
                let prev_count = cb.trip_count;
                let prev_tripped = cb.is_tripped;

                match cb.trip(timestamp) {
                    Ok(()) => {
                        // === Invariant Checks for successful trip ===
                        assert!(cb.is_tripped, "Circuit should be tripped after trip()");
                        assert_eq!(cb.trip_count, prev_count.saturating_add(U256::from(1)),
                                   "Trip count should increment");
                        assert_eq!(cb.last_trip_time, timestamp,
                                   "Last trip time should be updated");
                        assert!(!prev_tripped, "Should not have been tripped before");
                    }
                    Err("AlreadyTripped") => {
                        // === Invariant Checks for failed trip ===
                        assert!(prev_tripped, "Should have been tripped to fail");
                        assert_eq!(cb.trip_count, prev_count,
                                   "Trip count should not change on failed trip");
                        assert!(cb.is_tripped, "Should remain tripped");
                    }
                    Err(e) => panic!("Unexpected error: {}", e),
                }
            }
            1 => {
                // Reset
                let prev_count = cb.trip_count;
                let prev_time = cb.last_trip_time;
                let prev_tripped = cb.is_tripped;

                match cb.reset() {
                    Ok(()) => {
                        // === Invariant Checks for successful reset ===
                        assert!(!cb.is_tripped, "Circuit should not be tripped after reset()");
                        assert_eq!(cb.trip_count, prev_count,
                                   "Trip count should persist across reset");
                        assert_eq!(cb.last_trip_time, prev_time,
                                   "Last trip time should persist across reset");
                        assert!(prev_tripped, "Should have been tripped before reset");
                    }
                    Err("NotTripped") => {
                        // === Invariant Checks for failed reset ===
                        assert!(!prev_tripped, "Should not have been tripped to fail");
                        assert_eq!(cb.trip_count, prev_count,
                                   "Trip count should not change on failed reset");
                        assert!(!cb.is_tripped, "Should remain not tripped");
                    }
                    Err(e) => panic!("Unexpected error: {}", e),
                }
            }
            2 => {
                // Query state (read-only, no state changes)
                let state_before = cb.clone();
                let _is_tripped = cb.is_tripped;
                let _count = cb.trip_count;
                let _time = cb.last_trip_time;

                // === Invariant: Query operations don't modify state ===
                assert_eq!(cb.is_tripped, state_before.is_tripped, "Query modified is_tripped");
                assert_eq!(cb.trip_count, state_before.trip_count, "Query modified trip_count");
                assert_eq!(cb.last_trip_time, state_before.last_trip_time, "Query modified last_trip_time");
            }
            _ => unreachable!(),
        }

        // === Global Invariants (checked after every operation) ===

        // 1. Trip count is monotonically non-decreasing
        assert!(cb.trip_count >= initial_count,
                "Trip count {} decreased below initial {}",
                cb.trip_count, initial_count);

        // 2. is_tripped is always boolean (enforced by type)
        // (No check needed, Rust type system ensures this)

        // 3. Trip count never overflows (saturating arithmetic)
        let max_possible_trips = U256::from(data.len() as u64);
        assert!(cb.trip_count <= max_possible_trips.saturating_add(initial_count),
                "Trip count {} exceeds maximum possible {}",
                cb.trip_count, max_possible_trips);

        // 4. last_trip_time should be <= current timestamp if ever tripped
        if !cb.last_trip_time.is_zero() {
            assert!(cb.last_trip_time <= timestamp,
                    "Last trip time {} is in the future (now: {})",
                    cb.last_trip_time, timestamp);
        }

        // 5. If never tripped, last_trip_time should be zero
        if cb.trip_count.is_zero() {
            assert_eq!(cb.last_trip_time, U256::ZERO,
                       "Last trip time should be zero when never tripped");
        }
    }

    // === Final State Invariants ===

    // Trip count should match number of successful trip operations
    let trip_operations = data.iter().filter(|&&b| b % 3 == 0).count();
    // Can't exceed this (but may be less due to AlreadyTripped errors)
    assert!(cb.trip_count.saturating_to::<u64>() as usize <= trip_operations + initial_count.saturating_to::<u64>() as usize,
            "Trip count exceeds number of trip operations");
});
