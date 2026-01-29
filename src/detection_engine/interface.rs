//! Interface and event definitions for the DetectionEngine contract

extern crate alloc;

use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::sol;

sol! {
    // === V1 Events (preserved) ===

    /// Emitted when a new metric is registered
    event MetricRegistered(uint256 indexed id, uint256 threshold);

    /// Emitted when a metric value is reported
    event MetricReported(uint256 indexed id, uint256 value);

    /// Emitted when an anomaly is detected
    event AnomalyDetected(uint256 indexed id, uint256 current, uint256 threshold);

    /// Emitted when ownership is transferred
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    // === V2: Pattern Events ===

    /// Emitted when a new exploit pattern is registered
    event PatternRegistered(
        uint256 indexed id,
        uint256 pattern_type,
        uint256 severity,
        bytes32 description_hash
    );

    /// Emitted when a pattern's severity is updated
    event PatternUpdated(uint256 indexed id, uint256 old_severity, uint256 new_severity);

    /// Emitted when a pattern is deactivated
    event PatternDeactivated(uint256 indexed id);

    /// Emitted when a pattern is activated
    event PatternActivated(uint256 indexed id);

    // === V2: Analysis Events ===

    /// Emitted when a transaction is analyzed
    event TransactionAnalyzed(
        uint256 indexed analysis_id,
        address indexed from_addr,
        address indexed to_addr,
        uint256 threat_level,
        uint256 matched_patterns
    );

    /// Emitted when a high threat is detected
    event HighThreatDetected(
        uint256 indexed analysis_id,
        address indexed from_addr,
        uint256 threat_level,
        uint256 matched_patterns
    );

    // === V2: Whitelist Events ===

    /// Emitted when an address is added to the whitelist
    event AddressWhitelisted(address indexed addr, address indexed grantor);

    /// Emitted when an address is removed from the whitelist
    event AddressRemovedFromWhitelist(address indexed addr, address indexed revoker);

    // === V2: Role Events ===

    /// Emitted when a role is granted to an account
    event RoleGranted(address indexed account, uint8 indexed role, address indexed grantor);

    /// Emitted when a role is revoked from an account
    event RoleRevoked(address indexed account, uint8 indexed role, address indexed revoker);

    // === V2: Configuration Events ===

    /// Emitted when the high threat threshold is updated
    event HighThreatThresholdUpdated(uint256 old_threshold, uint256 new_threshold);

    /// Emitted when a pattern type value threshold is updated
    event PatternTypeThresholdUpdated(uint256 indexed pattern_type, uint256 old_value, uint256 new_value);

    /// Emitted when a pattern type rate threshold is updated
    event PatternTypeRateThresholdUpdated(uint256 indexed pattern_type, uint256 old_value, uint256 new_value);
}

/// DetectionEngine interface trait
pub trait IDetectionEngine {
    /// Error type for the interface
    type Error;

    // === V1 Methods (preserved) ===

    /// Register a new metric with a threshold
    fn register_metric(&mut self, id: U256, threshold: U256) -> Result<(), Self::Error>;

    /// Report a new value for a registered metric
    fn report_metric(&mut self, id: U256, value: U256) -> Result<(), Self::Error>;

    /// Check if a metric value indicates an anomaly
    fn check_anomaly(&self, id: U256) -> Result<bool, Self::Error>;

    /// Get the threshold for a metric
    fn get_threshold(&self, id: U256) -> U256;

    /// Get the current value for a metric
    fn get_current_value(&self, id: U256) -> U256;

    /// Get the total number of registered metrics
    fn get_metric_count(&self) -> U256;

    /// Get the current owner
    fn owner(&self) -> Address;

    /// Transfer ownership to a new address
    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Self::Error>;

    // === V2: Pattern Registration ===

    /// Register a new exploit pattern
    ///
    /// # Arguments
    /// * `pattern_type` - Bitmask of pattern types (flash loan=1, price manipulation=2, reentrancy=4, front-running=8)
    /// * `severity` - Severity level (0-100)
    /// * `description_hash` - Hash of the pattern description
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidPatternType` if pattern type is invalid
    /// Returns `InvalidSeverity` if severity exceeds maximum
    fn register_pattern(
        &mut self,
        pattern_type: U256,
        severity: U256,
        description_hash: FixedBytes<32>,
    ) -> Result<U256, Self::Error>;

    /// Deactivate an exploit pattern
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `PatternNotFound` if pattern does not exist
    /// Returns `PatternInactive` if pattern is already inactive
    fn deactivate_pattern(&mut self, id: U256) -> Result<(), Self::Error>;

    /// Activate an exploit pattern
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `PatternNotFound` if pattern does not exist
    fn activate_pattern(&mut self, id: U256) -> Result<(), Self::Error>;

    /// Get an exploit pattern by ID
    ///
    /// # Returns
    /// Tuple of (pattern_type, severity, description_hash, created_at)
    fn get_pattern(&self, id: U256) -> Result<(U256, U256, FixedBytes<32>, U256), Self::Error>;

    /// Get the total number of registered patterns
    fn get_pattern_count(&self) -> U256;

    /// Check if a pattern is active
    fn is_pattern_active(&self, id: U256) -> bool;

    // === V2: Transaction Analysis ===

    /// Analyze a transaction for potential exploits
    ///
    /// # Arguments
    /// * `from` - Source address
    /// * `to` - Destination address
    /// * `value` - Transaction value
    /// * `calldata_hash` - Hash of the transaction calldata
    /// * `gas_price` - Gas price
    ///
    /// # Returns
    /// The analysis ID
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ANALYST role
    /// Returns `InvalidAnalysisInput` if from or to is zero address
    fn analyze_transaction(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
        calldata_hash: FixedBytes<32>,
        gas_price: U256,
    ) -> Result<U256, Self::Error>;

    /// Get an analysis result by ID
    ///
    /// # Returns
    /// Tuple of (from, to, value, calldata_hash, gas_price, threat_level, matched_patterns, timestamp)
    fn get_analysis_result(
        &self,
        id: U256,
    ) -> Result<(Address, Address, U256, FixedBytes<32>, U256, U256, U256, U256), Self::Error>;

    /// Get the total number of analyses performed
    fn get_analysis_count(&self) -> U256;

    // === V2: Threat Scoring ===

    /// Get the cumulative threat score for an address
    fn get_address_threat_score(&self, addr: Address) -> U256;

    /// Get the total number of analyses for an address
    fn get_address_analysis_count(&self, addr: Address) -> U256;

    /// Set the high threat threshold
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidThresholdValue` if value exceeds maximum threat score
    fn set_high_threat_threshold(&mut self, threshold: U256) -> Result<(), Self::Error>;

    /// Get the current high threat threshold
    fn get_high_threat_threshold(&self) -> U256;

    // === V2: Whitelist Management ===

    /// Add an address to the whitelist
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidAddress` if address is zero
    /// Returns `AddressAlreadyWhitelisted` if address is already whitelisted
    fn add_to_whitelist(&mut self, addr: Address) -> Result<(), Self::Error>;

    /// Remove an address from the whitelist
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `AddressNotWhitelisted` if address is not whitelisted
    fn remove_from_whitelist(&mut self, addr: Address) -> Result<(), Self::Error>;

    /// Check if an address is whitelisted
    fn is_whitelisted(&self, addr: Address) -> bool;

    // === V2: Role-Based Access Control ===

    /// Grant a role to an account
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidRole` if role value is invalid
    /// Returns `InvalidAddress` if account is zero address
    fn grant_role(&mut self, account: Address, role: u8) -> Result<(), Self::Error>;

    /// Revoke a role from an account
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidRole` if role value is invalid
    /// Returns `CannotRevokeOwnRole` if revoking own role
    fn revoke_role(&mut self, account: Address, role: u8) -> Result<(), Self::Error>;

    /// Check if an account has a specific role
    fn has_role(&self, account: Address, role: u8) -> bool;

    /// Get all roles for an account
    fn get_roles(&self, account: Address) -> u8;

    // === V2: Pattern Type Configuration ===

    /// Set the value threshold for a pattern type
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidPatternType` if pattern type is invalid
    fn set_pattern_type_threshold(
        &mut self,
        pattern_type: U256,
        threshold: U256,
    ) -> Result<(), Self::Error>;

    /// Get the value threshold for a pattern type
    fn get_pattern_type_threshold(&self, pattern_type: U256) -> U256;

    /// Set the rate threshold for a pattern type
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidPatternType` if pattern type is invalid
    fn set_pattern_type_rate_threshold(
        &mut self,
        pattern_type: U256,
        threshold: U256,
    ) -> Result<(), Self::Error>;

    /// Get the rate threshold for a pattern type
    fn get_pattern_type_rate_threshold(&self, pattern_type: U256) -> U256;

    // === V2: ERC-165 Interface Detection ===

    /// Query if a contract supports an interface (ERC-165)
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool;
}
