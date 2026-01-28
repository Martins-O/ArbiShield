//! CircuitBreaker contract for emergency stop functionality
//!
//! This contract provides a circuit breaker pattern that can be tripped
//! to halt operations in case of detected anomalies or security issues.

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_sdk::{block, evm, msg};
use stylus_sdk::prelude::*;

pub mod error;
pub mod interface;
pub mod storage;

use error::Error;
use interface::{ICircuitBreaker, OwnershipTransferred, Reset, Tripped};
use storage::CircuitBreaker;

// Implementation of the CircuitBreaker contract
impl ICircuitBreaker for CircuitBreaker {
    type Error = Error;

    fn trip(&mut self) -> Result<(), Self::Error> {
        // Only owner can trip the circuit
        if msg::sender() != self.owner.get() {
            return Err(Error::UnauthorizedCaller(msg::sender()));
        }

        // Check if already tripped
        if self.is_tripped.get() {
            return Err(Error::AlreadyTripped);
        }

        // Trip the circuit
        self.is_tripped.set(true);

        // Increment trip count
        let count = self.trip_count.get();
        self.trip_count.set(count + U256::from(1));

        // Record timestamp
        let timestamp = U256::from(block::timestamp());
        self.last_trip_time.set(timestamp);

        // Emit event
        evm::log(Tripped {
            tripCount: count + U256::from(1),
            timestamp,
        });

        Ok(())
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        // Only owner can reset the circuit
        if msg::sender() != self.owner.get() {
            return Err(Error::UnauthorizedCaller(msg::sender()));
        }

        // Check if currently tripped
        if !self.is_tripped.get() {
            return Err(Error::NotTripped);
        }

        // Reset the circuit
        self.is_tripped.set(false);

        // Emit event
        let timestamp = U256::from(block::timestamp());
        evm::log(Reset { timestamp });

        Ok(())
    }

    fn is_active(&self) -> bool {
        self.is_tripped.get()
    }

    fn get_trip_count(&self) -> U256 {
        self.trip_count.get()
    }

    fn get_last_trip_time(&self) -> U256 {
        self.last_trip_time.get()
    }

    fn owner(&self) -> Address {
        self.owner.get()
    }

    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Self::Error> {
        // Only current owner can transfer ownership
        if msg::sender() != self.owner.get() {
            return Err(Error::UnauthorizedCaller(msg::sender()));
        }

        // Cannot transfer to zero address
        if new_owner == Address::ZERO {
            return Err(Error::InvalidOwner(new_owner));
        }

        let previous_owner = self.owner.get();
        self.owner.set(new_owner);

        // Emit ownership transferred event
        evm::log(OwnershipTransferred {
            previousOwner: previous_owner,
            newOwner: new_owner,
        });

        Ok(())
    }
}

#[public]
impl CircuitBreaker {
    /// Initialize the contract with the deployer as owner
    /// Should be called once after deployment
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        // Only allow initialization if owner is not set
        if self.owner.get() != Address::ZERO {
            return Err(Error::InvalidOwner(self.owner.get()).into());
        }
        self.owner.set(msg::sender());
        Ok(())
    }

    /// Trip the circuit breaker
    pub fn trip(&mut self) -> Result<(), Vec<u8>> {
        ICircuitBreaker::trip(self).map_err(|e| e.into())
    }

    /// Reset the circuit breaker
    pub fn reset(&mut self) -> Result<(), Vec<u8>> {
        ICircuitBreaker::reset(self).map_err(|e| e.into())
    }

    /// Check if circuit is active
    pub fn is_active(&self) -> bool {
        ICircuitBreaker::is_active(self)
    }

    /// Get trip count
    pub fn get_trip_count(&self) -> U256 {
        ICircuitBreaker::get_trip_count(self)
    }

    /// Get last trip time
    pub fn get_last_trip_time(&self) -> U256 {
        ICircuitBreaker::get_last_trip_time(self)
    }

    /// Get current owner
    pub fn owner(&self) -> Address {
        ICircuitBreaker::owner(self)
    }

    /// Transfer ownership
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        ICircuitBreaker::transfer_ownership(self, new_owner).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_state() {
        let is_tripped = false;
        assert!(!is_tripped);
    }
}
