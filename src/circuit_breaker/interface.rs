//! Interface and event definitions for the CircuitBreaker contract

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256, FixedBytes};
use alloy_sol_types::sol;

sol! {
    // === Global Circuit Events ===
    /// Emitted when the global circuit breaker is tripped
    event Tripped(uint256 indexed tripCount, uint256 timestamp);

    /// Emitted when the global circuit breaker is reset
    event Reset(uint256 timestamp);

    /// Emitted when ownership is transferred
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    // === Protocol Registration Events ===
    /// Emitted when a protocol is registered for circuit breaker protection
    event ProtocolRegistered(address indexed protocol, uint256 timestamp);

    /// Emitted when a protocol is unregistered from circuit breaker protection
    event ProtocolUnregistered(address indexed protocol, uint256 timestamp);

    // === Protocol-Specific Circuit Events ===
    /// Emitted when a specific protocol's circuit is tripped
    event ProtocolTripped(
        address indexed protocol,
        uint256 tripCount,
        uint256 timestamp,
        address indexed operator
    );

    /// Emitted when a specific protocol's circuit is reset
    event ProtocolReset(
        address indexed protocol,
        uint256 pauseDuration,
        uint256 timestamp,
        address indexed operator
    );

    // === Batch Operation Events ===
    /// Emitted when multiple protocols are tripped in a single operation
    event BatchProtocolsTripped(
        uint256 count,
        uint256 timestamp,
        address indexed operator
    );

    /// Emitted when multiple protocols are reset in a single operation
    event BatchProtocolsReset(
        uint256 count,
        uint256 timestamp,
        address indexed operator
    );

    // === Role Management Events ===
    /// Emitted when a role is granted to an account
    event RoleGranted(
        address indexed account,
        uint8 indexed role,
        address indexed grantor
    );

    /// Emitted when a role is revoked from an account
    event RoleRevoked(
        address indexed account,
        uint8 indexed role,
        address indexed revoker
    );

    // === Configuration Events ===
    /// Emitted when the minimum pause duration is updated
    event MinPauseDurationUpdated(uint256 oldDuration, uint256 newDuration);

    /// Emitted when the maximum pause duration is updated
    event MaxPauseDurationUpdated(uint256 oldDuration, uint256 newDuration);

    /// Emitted when the cooldown period is updated
    event CooldownPeriodUpdated(uint256 oldPeriod, uint256 newPeriod);
}

/// CircuitBreaker interface trait
pub trait ICircuitBreaker {
    /// Error type for the interface
    type Error;

    /// Trip the circuit breaker (activate emergency stop)
    ///
    /// # Errors
    /// Returns `UnauthorizedCaller` if caller is not the owner
    /// Returns `AlreadyTripped` if circuit is already tripped
    fn trip(&mut self) -> Result<(), Self::Error>;

    /// Reset the circuit breaker (deactivate emergency stop)
    ///
    /// # Errors
    /// Returns `UnauthorizedCaller` if caller is not the owner
    /// Returns `NotTripped` if circuit is not currently tripped
    fn reset(&mut self) -> Result<(), Self::Error>;

    /// Check if the circuit breaker is currently active (tripped)
    ///
    /// # Returns
    /// `true` if tripped, `false` otherwise
    fn is_active(&self) -> bool;

    /// Get the total number of times the circuit has been tripped
    ///
    /// # Returns
    /// The trip count
    fn get_trip_count(&self) -> U256;

    /// Get the timestamp of the last trip
    ///
    /// # Returns
    /// The last trip timestamp (0 if never tripped)
    fn get_last_trip_time(&self) -> U256;

    /// Get the current owner
    ///
    /// # Returns
    /// The owner's address
    fn owner(&self) -> Address;

    /// Transfer ownership to a new address
    ///
    /// # Arguments
    /// * `new_owner` - Address of the new owner
    ///
    /// # Errors
    /// Returns `UnauthorizedCaller` if caller is not the owner
    /// Returns `InvalidOwner` if new owner is the zero address
    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Self::Error>;

    // === Protocol Registration ===

    /// Register a protocol for circuit breaker protection
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol to register
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidProtocol` if protocol is zero address
    /// Returns `ProtocolAlreadyRegistered` if protocol is already registered
    fn register_protocol(&mut self, protocol: Address) -> Result<(), Self::Error>;

    /// Unregister a protocol from circuit breaker protection
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol to unregister
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `ProtocolNotRegistered` if protocol is not registered
    /// Returns `ProtocolAlreadyPaused` if protocol is currently paused
    fn unregister_protocol(&mut self, protocol: Address) -> Result<(), Self::Error>;

    /// Check if a protocol is registered
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol to check
    ///
    /// # Returns
    /// `true` if registered, `false` otherwise
    fn is_protocol_registered(&self, protocol: Address) -> bool;

    /// Get the total number of registered protocols
    ///
    /// # Returns
    /// The count of registered protocols
    fn get_protocol_count(&self) -> U256;

    /// Get the protocol address at a specific index
    ///
    /// # Arguments
    /// * `index` - Index in the protocol list
    ///
    /// # Returns
    /// The protocol address at the given index
    ///
    /// # Errors
    /// Returns `IndexOutOfBounds` if index is out of range
    /// Returns `ProtocolNotRegistered` if protocol at index is no longer registered
    fn get_protocol_at_index(&self, index: U256) -> Result<Address, Self::Error>;

    // === Protocol-Specific Circuit Operations ===

    /// Trip (pause) a specific protocol's circuit
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol to trip
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have OPERATOR role
    /// Returns `ProtocolNotRegistered` if protocol is not registered
    /// Returns `ProtocolAlreadyPaused` if protocol is already paused
    fn trip_protocol(&mut self, protocol: Address) -> Result<(), Self::Error>;

    /// Reset (unpause) a specific protocol's circuit
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol to reset
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have OPERATOR role
    /// Returns `ProtocolNotRegistered` if protocol is not registered
    /// Returns `ProtocolNotPaused` if protocol is not currently paused
    /// Returns `MinimumPauseDurationNotMet` if minimum pause duration has not elapsed
    /// Returns `CooldownPeriodActive` if cooldown period is still active
    fn reset_protocol(&mut self, protocol: Address) -> Result<(), Self::Error>;

    /// Check if a specific protocol is currently paused
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol to check
    ///
    /// # Returns
    /// `true` if paused, `false` otherwise
    fn is_protocol_paused(&self, protocol: Address) -> bool;

    /// Get the trip count for a specific protocol
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol
    ///
    /// # Returns
    /// The number of times the protocol has been tripped
    fn get_protocol_trip_count(&self, protocol: Address) -> U256;

    /// Get the pause timestamp for a specific protocol
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol
    ///
    /// # Returns
    /// The timestamp when the protocol was last paused (0 if never paused)
    fn get_protocol_pause_time(&self, protocol: Address) -> U256;

    // === Batch Operations ===

    /// Trip multiple protocols in a single transaction
    ///
    /// # Arguments
    /// * `protocols` - Array of protocol addresses to trip
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have OPERATOR role
    /// Returns `EmptyProtocolList` if protocols array is empty
    /// Returns `BatchSizeTooLarge` if protocols array exceeds maximum size
    fn trip_multiple_protocols(&mut self, protocols: Vec<Address>) -> Result<(), Self::Error>;

    /// Reset multiple protocols in a single transaction
    ///
    /// # Arguments
    /// * `protocols` - Array of protocol addresses to reset
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have OPERATOR role
    /// Returns `EmptyProtocolList` if protocols array is empty
    /// Returns `BatchSizeTooLarge` if protocols array exceeds maximum size
    fn reset_multiple_protocols(&mut self, protocols: Vec<Address>) -> Result<(), Self::Error>;

    // === Role-Based Access Control ===

    /// Grant a role to an account
    ///
    /// # Arguments
    /// * `account` - Address to grant the role to
    /// * `role` - Role bitmask to grant (ADMIN=1, OPERATOR=2)
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidRole` if role value is invalid
    /// Returns `InvalidProtocol` if account is zero address
    fn grant_role(&mut self, account: Address, role: u8) -> Result<(), Self::Error>;

    /// Revoke a role from an account
    ///
    /// # Arguments
    /// * `account` - Address to revoke the role from
    /// * `role` - Role bitmask to revoke (ADMIN=1, OPERATOR=2)
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidRole` if role value is invalid
    /// Returns `CannotRevokeOwnRole` if attempting to revoke own role
    /// Returns `InvalidOwner` if attempting to revoke owner's role
    fn revoke_role(&mut self, account: Address, role: u8) -> Result<(), Self::Error>;

    /// Check if an account has a specific role
    ///
    /// # Arguments
    /// * `account` - Address to check
    /// * `role` - Role bitmask to check (ADMIN=1, OPERATOR=2)
    ///
    /// # Returns
    /// `true` if account has the role, `false` otherwise
    fn has_role(&self, account: Address, role: u8) -> bool;

    /// Get all roles for an account
    ///
    /// # Arguments
    /// * `account` - Address to query
    ///
    /// # Returns
    /// Role bitmask for the account
    fn get_roles(&self, account: Address) -> u8;

    // === Time Lock Configuration ===

    /// Set the minimum pause duration
    ///
    /// # Arguments
    /// * `duration` - Minimum pause duration in seconds
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidDuration` if duration is zero or exceeds reasonable bounds
    /// Returns `MaxPauseDurationExceeded` if duration exceeds max pause duration
    fn set_min_pause_duration(&mut self, duration: U256) -> Result<(), Self::Error>;

    /// Set the maximum pause duration
    ///
    /// # Arguments
    /// * `duration` - Maximum pause duration in seconds
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidDuration` if duration is invalid or less than min pause duration
    fn set_max_pause_duration(&mut self, duration: U256) -> Result<(), Self::Error>;

    /// Set the cooldown period between trips
    ///
    /// # Arguments
    /// * `period` - Cooldown period in seconds
    ///
    /// # Errors
    /// Returns `InsufficientRole` if caller does not have ADMIN role
    /// Returns `InvalidDuration` if period exceeds reasonable bounds
    fn set_cooldown_period(&mut self, period: U256) -> Result<(), Self::Error>;

    /// Get the minimum pause duration
    ///
    /// # Returns
    /// The minimum pause duration in seconds
    fn get_min_pause_duration(&self) -> U256;

    /// Get the maximum pause duration
    ///
    /// # Returns
    /// The maximum pause duration in seconds
    fn get_max_pause_duration(&self) -> U256;

    /// Get the cooldown period
    ///
    /// # Returns
    /// The cooldown period in seconds
    fn get_cooldown_period(&self) -> U256;

    /// Check if a protocol can be reset (minimum duration has elapsed)
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol to check
    ///
    /// # Returns
    /// `true` if protocol can be reset, `false` otherwise
    fn can_reset(&self, protocol: Address) -> bool;

    /// Get the remaining pause time for a protocol
    ///
    /// # Arguments
    /// * `protocol` - Address of the protocol to check
    ///
    /// # Returns
    /// Remaining pause time in seconds (0 if can be reset)
    fn get_remaining_pause_time(&self, protocol: Address) -> U256;

    // === ERC-165 Interface Detection ===

    /// Query if a contract supports an interface (ERC-165)
    ///
    /// # Arguments
    /// * `interface_id` - 4-byte interface identifier
    ///
    /// # Returns
    /// `true` if the interface is supported, `false` otherwise
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool;
}
