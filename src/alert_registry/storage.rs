//! Storage structures for the AlertRegistry contract

extern crate alloc;

#[allow(unused_imports)]
use alloc::{vec, vec::Vec};

use stylus_sdk::prelude::*;
use stylus_sdk::stylus_proc::sol_storage;

sol_storage! {
    /// Enhanced alert structure with full metadata
    pub struct EnhancedAlert {
        /// Protocol address the alert pertains to
        address protocol;
        /// Threat level (0-100)
        uint256 threat_level;
        /// Bitmask of matched exploit patterns
        uint256 pattern_matched;
        /// Timestamp when alert was created
        uint256 timestamp;
        /// Priority level (0=LOW, 1=MEDIUM, 2=HIGH, 3=CRITICAL)
        uint256 priority_level;
        /// Whether the alert has been acknowledged
        bool acknowledged;
        /// Address that acknowledged the alert
        address acknowledged_by;
        /// Timestamp when alert was acknowledged
        uint256 acknowledged_at;
        /// Hash of the alert message/description
        bytes32 message_hash;
        /// Source address that triggered the alert (e.g., DetectionEngine)
        address source;
    }

    /// Individual alert structure (V1 - preserved for upgrade compatibility)
    pub struct Alert {
        /// Timestamp when the alert was created
        uint256 timestamp;
        /// Severity level (0-255, stored as uint256 due to SDK compat)
        uint256 severity;
        /// Source address that triggered the alert
        address source;
        /// Hash of the alert message for verification
        bytes32 message_hash;
    }

    /// AlertRegistry contract storage
    /// Uses Solidity-compatible storage layout for proxy upgrade compatibility
    ///
    /// V1 fields are preserved in exact order for upgrade safety.
    /// V2 fields are appended after V1 fields.
    pub struct AlertRegistry {
        // === V1 Fields (preserved for upgrade compatibility) ===

        /// Owner of the contract
        address owner;
        /// Array of all alerts (V1 storage)
        Alert[] alerts;

        // === V2 Fields ===

        // --- Role-Based Access Control ---
        /// Mapping of address to role bitmask (ADMIN=0x01, MONITOR=0x02)
        mapping(address => uint256) roles;

        // --- Enhanced Alert Storage ---
        /// Mapping of alert ID to EnhancedAlert struct
        mapping(uint256 => EnhancedAlert) enhanced_alerts;
        /// Total number of enhanced alerts registered
        uint256 enhanced_alert_count;

        // --- Subscriber Management ---
        /// Mapping of address to subscriber status
        mapping(address => bool) subscribers;
        /// Total number of subscribers
        uint256 subscriber_count;

        // --- Per-Protocol Alert Tracking ---
        /// Mapping of protocol address to alert count
        mapping(address => uint256) protocol_alert_count;

        // --- Priority-Based Counting ---
        /// Mapping of priority level to alert count
        mapping(uint256 => uint256) priority_alert_counts;

        // --- Alert Expiration ---
        /// Duration after which alerts are considered expired (default: 30 days)
        uint256 alert_expiration_duration;

        // --- DetectionEngine Integration ---
        /// Address of the DetectionEngine contract
        address detection_engine;

        // --- ERC-165 Interface Detection ---
        /// Mapping of interface ID to support status
        mapping(bytes4 => bool) supported_interfaces;

        // --- Storage Gap for Future Upgrades ---
        /// Reserved storage slots for future V3 upgrades
        uint256[50] __gap;
    }
}
