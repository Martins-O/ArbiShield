//! Interface and event definitions for the AlertRegistry contract

extern crate alloc;

use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::sol;

sol! {
    // === V1 Events (preserved) ===

    /// Emitted when a new alert is registered (V1)
    event AlertRegistered(
        uint256 indexed id,
        uint256 timestamp,
        uint8 severity,
        address indexed source,
        bytes32 messageHash
    );

    /// Emitted when ownership is transferred
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    // === V2: Enhanced Alert Events ===

    /// Emitted when a new enhanced alert is registered
    event EnhancedAlertRegistered(
        uint256 indexed id,
        address indexed protocol,
        uint256 threat_level,
        uint256 priority_level,
        uint256 pattern_matched
    );

    /// Emitted when an alert is acknowledged
    event AlertAcknowledged(
        uint256 indexed id,
        address indexed protocol,
        address indexed acknowledger
    );

    // === V2: Subscriber Events ===

    /// Emitted when a subscriber is added
    event SubscriberAdded(address indexed subscriber, address indexed grantor);

    /// Emitted when a subscriber is removed
    event SubscriberRemoved(address indexed subscriber, address indexed revoker);

    // === V2: Configuration Events ===

    /// Emitted when the alert expiration duration is updated
    event AlertExpirationUpdated(uint256 old_duration, uint256 new_duration);

    /// Emitted when the detection engine address is updated
    event DetectionEngineUpdated(address indexed old_engine, address indexed new_engine);

    // === V2: Role Events ===

    /// Emitted when a role is granted to an account
    event RoleGranted(address indexed account, uint8 indexed role, address indexed grantor);

    /// Emitted when a role is revoked from an account
    event RoleRevoked(address indexed account, uint8 indexed role, address indexed revoker);

    // === V2: Off-Chain Indexing Events ===

    /// Emitted when a high threat alert is created (for off-chain monitoring)
    event HighThreatAlertEmitted(uint256 indexed id, address indexed protocol, uint256 threat_level);

    /// Emitted when a critical alert is created (for off-chain monitoring)
    event CriticalAlertEmitted(uint256 indexed id, address indexed protocol, uint256 threat_level);
}

/// AlertRegistry interface trait
pub trait IAlertRegistry {
    /// Error type for the interface
    type Error;

    // === V1 Methods (preserved) ===

    /// Register a new alert (V1)
    ///
    /// # Arguments
    /// * `severity` - Alert severity level (0-255)
    /// * `source` - Address of the source that triggered the alert
    /// * `message_hash` - Hash of the alert message
    ///
    /// # Returns
    /// The ID of the newly registered alert
    ///
    /// # Errors
    /// Returns `InvalidAlert` if alert data is invalid
    fn register_alert(
        &mut self,
        severity: u8,
        source: Address,
        message_hash: FixedBytes<32>,
    ) -> Result<U256, Self::Error>;

    /// Get alert details by ID (V1)
    ///
    /// # Returns
    /// Tuple of (timestamp, severity, source, message_hash)
    fn get_alert(
        &self,
        id: U256,
    ) -> Result<(U256, u8, Address, FixedBytes<32>), Self::Error>;

    /// Get the total number of registered alerts (V1)
    fn get_alert_count(&self) -> U256;

    /// Get the current owner
    fn owner(&self) -> Address;

    /// Transfer ownership to a new address
    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Self::Error>;

    // === V2: Enhanced Alert Registration ===

    /// Register a new enhanced alert with full metadata
    ///
    /// # Arguments
    /// * `protocol` - Protocol address the alert pertains to
    /// * `threat_level` - Threat level (0-100)
    /// * `pattern_matched` - Bitmask of matched exploit patterns
    /// * `message_hash` - Hash of the alert message/description
    /// * `source` - Address that triggered the alert (e.g., DetectionEngine)
    ///
    /// # Returns
    /// The alert ID
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have MONITOR role and is not DetectionEngine
    /// Returns `InvalidAddress` if protocol is zero address
    fn register_enhanced_alert(
        &mut self,
        protocol: Address,
        threat_level: U256,
        pattern_matched: U256,
        message_hash: FixedBytes<32>,
        source: Address,
    ) -> Result<U256, Self::Error>;

    /// Get enhanced alert details by ID
    ///
    /// # Returns
    /// Tuple of (protocol, threat_level, pattern_matched, timestamp, priority_level, acknowledged, message_hash, source)
    fn get_enhanced_alert(
        &self,
        id: U256,
    ) -> Result<(Address, U256, U256, U256, U256, bool, FixedBytes<32>, Address), Self::Error>;

    /// Get the total number of enhanced alerts
    fn get_enhanced_alert_count(&self) -> U256;

    // === V2: Acknowledgment ===

    /// Acknowledge an alert
    ///
    /// # Errors
    /// Returns `AlertNotFound` if alert does not exist
    /// Returns `AlertAlreadyAcknowledged` if already acknowledged
    /// Returns `NotAlertProtocol` if caller is not the alert's protocol and not ADMIN
    fn acknowledge_alert(&mut self, id: U256) -> Result<(), Self::Error>;

    /// Check if an alert has been acknowledged
    fn is_alert_acknowledged(&self, id: U256) -> bool;

    // === V2: Subscriber Management ===

    /// Add a subscriber
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidSubscriber` if address is zero
    /// Returns `AlreadySubscribed` if address is already subscribed
    fn add_subscriber(&mut self, addr: Address) -> Result<(), Self::Error>;

    /// Remove a subscriber
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `NotSubscribed` if address is not subscribed
    fn remove_subscriber(&mut self, addr: Address) -> Result<(), Self::Error>;

    /// Check if an address is subscribed
    fn is_subscriber(&self, addr: Address) -> bool;

    /// Get the total number of subscribers
    fn get_subscriber_count(&self) -> U256;

    // === V2: Queries ===

    /// Get the number of alerts for a specific protocol
    fn get_protocol_alert_count(&self, protocol: Address) -> U256;

    /// Get the number of alerts at a specific priority level
    fn get_priority_alert_count(&self, priority_level: U256) -> U256;

    /// Check if an alert is expired
    ///
    /// # Errors
    /// Returns `AlertNotFound` if alert does not exist
    fn is_alert_expired(&self, id: U256) -> Result<bool, Self::Error>;

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

    // === V2: Configuration ===

    /// Set the alert expiration duration
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidExpirationDuration` if duration is zero
    fn set_alert_expiration_duration(&mut self, duration: U256) -> Result<(), Self::Error>;

    /// Get the current alert expiration duration
    fn get_alert_expiration_duration(&self) -> U256;

    /// Set the detection engine address
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidAddress` if address is zero
    fn set_detection_engine(&mut self, addr: Address) -> Result<(), Self::Error>;

    /// Get the detection engine address
    fn get_detection_engine(&self) -> Address;

    // === V2: ERC-165 Interface Detection ===

    /// Query if a contract supports an interface (ERC-165)
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool;
}
