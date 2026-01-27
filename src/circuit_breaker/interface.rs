//! Interface and event definitions for the CircuitBreaker contract

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

sol! {
    /// Emitted when the circuit breaker is tripped
    event Tripped(uint256 indexed tripCount, uint256 timestamp);

    /// Emitted when the circuit breaker is reset
    event Reset(uint256 timestamp);

    /// Emitted when ownership is transferred
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
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
}
