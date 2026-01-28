//! Storage structures for the DetectionEngine contract

extern crate alloc;

#[allow(unused_imports)]
use alloc::{vec, vec::Vec};

use stylus_sdk::prelude::*;
use stylus_sdk::stylus_proc::sol_storage;

sol_storage! {
    /// DetectionEngine contract storage
    /// Uses Solidity-compatible storage layout for proxy upgrade compatibility
    pub struct DetectionEngine {
        /// Owner of the contract
        address owner;
        /// Mapping of metric ID to threshold value
        mapping(uint256 => uint256) thresholds;
        /// Mapping of metric ID to current value
        mapping(uint256 => uint256) current_values;
        /// Total number of registered metrics
        uint256 metric_count;
    }
}
