//! Storage structures for the AlertRegistry contract

extern crate alloc;

#[allow(unused_imports)]
use alloc::{vec, vec::Vec};

use stylus_sdk::prelude::*;
use stylus_sdk::stylus_proc::sol_storage;

sol_storage! {
    /// AlertRegistry contract storage
    /// Uses Solidity-compatible storage layout for proxy upgrade compatibility
    pub struct AlertRegistry {
        /// Owner of the contract
        address owner;
        /// Array of all alerts
        Alert[] alerts;
    }

    /// Individual alert structure
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
}
