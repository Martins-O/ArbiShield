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
        // === V1 Storage (NEVER REORDER - Proxy Compatibility) ===
        /// Owner of the contract
        address owner;
        /// Whether the global circuit is currently tripped
        bool is_tripped;
        /// Total number of times any circuit has been tripped
        uint256 trip_count;
        /// Timestamp of the last global trip
        uint256 last_trip_time;

        // === V2 Storage (NEW - Safe to append) ===

        // Role-Based Access Control
        /// Mapping from address to role bitmask (ADMIN=1, OPERATOR=2, stored as uint256 due to SDK compat)
        mapping(address => uint256) roles;

        // Protocol Registration
        /// Mapping from protocol address to registration status
        mapping(address => bool) registered_protocols;
        /// Array of registered protocol addresses for enumeration
        address[] protocol_list;

        // Protocol-Specific Pause State
        /// Mapping from protocol address to protocol-specific pause state
        mapping(address => bool) protocol_paused;
        /// Mapping from protocol address to protocol-specific pause timestamp
        mapping(address => uint256) protocol_pause_time;
        /// Mapping from protocol address to protocol trip count
        mapping(address => uint256) protocol_trip_count;

        // Time Lock Configuration
        /// Minimum pause duration in seconds (default: 1 hour = 3600)
        uint256 min_pause_duration;
        /// Maximum pause duration in seconds (default: 7 days = 604800)
        uint256 max_pause_duration;
        /// Cooldown period between consecutive trips in seconds (default: 5 minutes = 300)
        uint256 cooldown_period;

        // ERC-165 Interface Detection
        /// Mapping from interface ID to support status
        mapping(bytes4 => bool) supported_interfaces;

        // === Storage Gap for Future Upgrades ===
        /// Reserved storage slots for future versions (50 slots)
        /// This allows adding new storage variables in upgrades without breaking storage layout
        uint256[50] __gap;
    }
}
