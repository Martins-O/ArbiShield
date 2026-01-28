//! Storage structures for the CircuitBreaker contract

extern crate alloc;

#[allow(unused_imports)]
use alloc::{vec, vec::Vec};

use stylus_sdk::prelude::*;
use stylus_sdk::stylus_proc::sol_storage;

sol_storage! {
    /// CircuitBreaker contract storage
    /// Uses Solidity-compatible storage layout for proxy upgrade compatibility
    pub struct CircuitBreaker {
        /// Owner of the contract
        address owner;
        /// Whether the circuit is currently tripped
        bool is_tripped;
        /// Total number of times the circuit has been tripped
        uint256 trip_count;
        /// Timestamp of the last trip
        uint256 last_trip_time;
    }
}
