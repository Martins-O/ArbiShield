//! Interface and event definitions for the AlertRegistry contract

use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::sol;

sol! {
    /// Emitted when a new alert is registered
    event AlertRegistered(
        uint256 indexed id,
        uint256 timestamp,
        uint8 severity,
        address indexed source,
        bytes32 messageHash
    );

    /// Emitted when ownership is transferred
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
}

/// AlertRegistry interface trait
pub trait IAlertRegistry {
    /// Error type for the interface
    type Error;

    /// Register a new alert
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

    /// Get alert details by ID
    ///
    /// # Arguments
    /// * `id` - Alert identifier
    ///
    /// # Returns
    /// Tuple of (timestamp, severity, source, message_hash)
    ///
    /// # Errors
    /// Returns `AlertNotFound` if alert ID is invalid
    fn get_alert(
        &self,
        id: U256,
    ) -> Result<(U256, u8, Address, FixedBytes<32>), Self::Error>;

    /// Get the total number of registered alerts
    ///
    /// # Returns
    /// The alert count
    fn get_alert_count(&self) -> U256;

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
}
