//! Storage structures for the DetectionEngine contract

extern crate alloc;

#[allow(unused_imports)]
use alloc::{vec, vec::Vec};

use stylus_sdk::prelude::*;
use stylus_sdk::stylus_proc::sol_storage;

sol_storage! {
    /// Exploit pattern definition
    pub struct ExploitPattern {
        /// Bitmask of pattern types (flash loan=1, price manipulation=2, reentrancy=4, front-running=8)
        uint256 pattern_type;
        /// Severity level (0-100)
        uint256 severity;
        /// Hash of pattern description for gas-efficient storage
        bytes32 description_hash;
        /// Timestamp when pattern was registered
        uint256 created_at;
    }

    /// Transaction analysis result
    pub struct AnalysisResult {
        /// Address that initiated the analyzed transaction
        address analyzed_from;
        /// Target address of the analyzed transaction
        address analyzed_to;
        /// Transaction value in wei
        uint256 tx_value;
        /// Hash of the transaction calldata
        bytes32 calldata_hash;
        /// Gas price of the transaction
        uint256 gas_price;
        /// Computed threat level (0-100)
        uint256 threat_level;
        /// Bitmask of matched exploit patterns
        uint256 matched_patterns;
        /// Timestamp when analysis was performed
        uint256 timestamp;
    }

    /// DetectionEngine contract storage
    /// Uses Solidity-compatible storage layout for proxy upgrade compatibility
    ///
    /// V1 fields are preserved in exact order for upgrade safety.
    /// V2 fields are appended after V1 fields.
    pub struct DetectionEngine {
        // === V1 Fields (preserved for upgrade compatibility) ===

        /// Owner of the contract
        address owner;
        /// Mapping of metric ID to threshold value
        mapping(uint256 => uint256) thresholds;
        /// Mapping of metric ID to current value
        mapping(uint256 => uint256) current_values;
        /// Total number of registered metrics
        uint256 metric_count;

        // === V2 Fields ===

        // --- Role-Based Access Control ---
        /// Mapping of address to role bitmask (ADMIN=0x01, ANALYST=0x02)
        mapping(address => uint256) roles;

        // --- Pattern Registration ---
        /// Mapping of pattern ID to ExploitPattern struct
        mapping(uint256 => ExploitPattern) patterns;
        /// Total number of registered patterns
        uint256 pattern_count;
        /// Mapping of pattern ID to active status
        mapping(uint256 => bool) pattern_active;

        // --- Transaction Analysis ---
        /// Mapping of analysis ID to AnalysisResult struct
        mapping(uint256 => AnalysisResult) analysis_results;
        /// Total number of analyses performed
        uint256 analysis_count;

        // --- Threat Scoring ---
        /// Threshold above which a threat score is considered high (default: 70)
        uint256 high_threat_threshold;
        /// Mapping of address to cumulative threat score
        mapping(address => uint256) address_threat_scores;
        /// Mapping of address to total number of analyses
        mapping(address => uint256) address_analysis_count;

        // --- Whitelist ---
        /// Mapping of address to whitelist status
        mapping(address => bool) whitelisted;

        // --- Pattern Type Configuration ---
        /// Mapping of pattern type to value threshold
        mapping(uint256 => uint256) pattern_type_thresholds;
        /// Mapping of pattern type to rate/gas threshold
        mapping(uint256 => uint256) pattern_type_rate_thresholds;

        // --- Rate-of-Change Tracking ---
        /// Mapping of metric ID to previous value (for rate-of-change detection)
        mapping(uint256 => uint256) previous_values;
        /// Mapping of metric ID to last report timestamp
        mapping(uint256 => uint256) last_report_time;

        // --- ERC-165 Interface Detection ---
        /// Mapping of interface ID to support status
        mapping(bytes4 => bool) supported_interfaces;

        // --- Storage Gap for Future Upgrades ---
        /// Reserved storage slots for future V3 upgrades
        uint256[50] __gap;
    }
}
