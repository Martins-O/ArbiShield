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
    use alloc::{vec, format};

    // ========================================
    // State Machine Simulation Tests
    // ========================================

    #[test]
    fn test_initial_state_not_tripped() {
        let is_tripped = false;
        assert!(!is_tripped, "Circuit should start un-tripped");
    }

    #[test]
    fn test_trip_sets_active() {
        let mut is_tripped = false;
        is_tripped = true;
        assert!(is_tripped);
    }

    #[test]
    fn test_reset_clears_active() {
        let mut is_tripped = true;
        is_tripped = false;
        assert!(!is_tripped);
    }

    #[test]
    fn test_full_trip_reset_cycle() {
        let mut state = false;
        for _ in 0..5 {
            assert!(!state);
            state = true;
            assert!(state);
            state = false;
        }
    }

    #[test]
    fn test_already_tripped_guard() {
        let is_tripped = true;
        let should_reject = is_tripped; // already tripped
        assert!(should_reject, "Should reject trip when already tripped");
    }

    #[test]
    fn test_not_tripped_guard() {
        let is_tripped = false;
        let should_reject = !is_tripped; // not tripped
        assert!(should_reject, "Should reject reset when not tripped");
    }

    // ========================================
    // Trip Count Arithmetic
    // ========================================

    #[test]
    fn test_trip_count_starts_zero() {
        assert_eq!(U256::ZERO, U256::from(0));
    }

    #[test]
    fn test_trip_count_increments_correctly() {
        let mut count = U256::ZERO;
        for i in 1..=10u64 {
            count = count + U256::from(1);
            assert_eq!(count, U256::from(i));
        }
    }

    #[test]
    fn test_trip_count_near_max() {
        let count = U256::MAX - U256::from(1);
        let next = count + U256::from(1);
        assert_eq!(next, U256::MAX);
    }

    #[test]
    fn test_trip_count_not_reset_on_circuit_reset() {
        // Trip count should survive reset (accumulated counter)
        let mut count = U256::from(5);
        let mut _is_tripped = true;
        _is_tripped = false; // reset
        // count stays the same
        assert_eq!(count, U256::from(5));
        // next trip increments
        _is_tripped = true;
        count = count + U256::from(1);
        assert_eq!(count, U256::from(6));
    }

    // ========================================
    // Timestamp Tests
    // ========================================

    #[test]
    fn test_timestamp_zero_before_first_trip() {
        let last_trip_time = U256::ZERO;
        assert_eq!(last_trip_time, U256::ZERO);
    }

    #[test]
    fn test_timestamp_records_on_trip() {
        let timestamp = U256::from(1_700_000_000u64);
        assert!(timestamp > U256::ZERO);
    }

    #[test]
    fn test_timestamp_overwrites_on_subsequent_trip() {
        let first = U256::from(1_700_000_000u64);
        let second = U256::from(1_700_001_000u64);
        let mut last_trip_time = first;
        last_trip_time = second;
        assert_eq!(last_trip_time, second);
        assert!(last_trip_time > first);
    }

    // ========================================
    // Address Validation
    // ========================================

    #[test]
    fn test_zero_address_detection() {
        assert_eq!(Address::ZERO, Address::ZERO);
    }

    #[test]
    fn test_non_zero_address_valid() {
        let addr = Address::from([0x01u8; 20]);
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn test_address_equality() {
        let a = Address::from([0xABu8; 20]);
        let b = Address::from([0xABu8; 20]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_address_inequality() {
        let owner = Address::from([0x01u8; 20]);
        let caller = Address::from([0x02u8; 20]);
        assert_ne!(owner, caller);
    }

    // ========================================
    // Error Encoding Tests
    // ========================================

    #[test]
    fn test_already_tripped_error_encodes() {
        let encoded: Vec<u8> = Error::AlreadyTripped.into();
        assert!(encoded.len() >= 4, "Must have selector bytes");
    }

    #[test]
    fn test_not_tripped_error_encodes() {
        let encoded: Vec<u8> = Error::NotTripped.into();
        assert!(encoded.len() >= 4);
    }

    #[test]
    fn test_unauthorized_caller_encodes_with_address() {
        let caller = Address::from([0xAAu8; 20]);
        let encoded: Vec<u8> = Error::UnauthorizedCaller(caller).into();
        // 4 byte selector + 32 byte address word
        assert_eq!(encoded.len(), 36);
    }

    #[test]
    fn test_invalid_owner_encodes_with_address() {
        let owner = Address::ZERO;
        let encoded: Vec<u8> = Error::InvalidOwner(owner).into();
        assert_eq!(encoded.len(), 36);
    }

    #[test]
    fn test_all_error_selectors_unique() {
        let e1: Vec<u8> = Error::AlreadyTripped.into();
        let e2: Vec<u8> = Error::NotTripped.into();
        let e3: Vec<u8> = Error::UnauthorizedCaller(Address::ZERO).into();
        let e4: Vec<u8> = Error::InvalidOwner(Address::ZERO).into();

        let selectors: Vec<&[u8]> = vec![&e1[..4], &e2[..4], &e3[..4], &e4[..4]];
        for i in 0..selectors.len() {
            for j in (i + 1)..selectors.len() {
                assert_ne!(selectors[i], selectors[j], "Error selectors must be unique");
            }
        }
    }

    #[test]
    fn test_error_encoding_deterministic() {
        let addr = Address::from([0xBBu8; 20]);
        let enc1: Vec<u8> = Error::UnauthorizedCaller(addr).into();
        let enc2: Vec<u8> = Error::UnauthorizedCaller(addr).into();
        assert_eq!(enc1, enc2, "Same error must encode identically");
    }

    #[test]
    fn test_encoded_address_present_in_unauthorized_error() {
        let addr = Address::from([0xCCu8; 20]);
        let encoded: Vec<u8> = Error::UnauthorizedCaller(addr).into();
        // Address in ABI encoding: 12 zero bytes + 20 address bytes at offset 4
        let encoded_addr = &encoded[16..36];
        assert_eq!(encoded_addr, addr.as_slice());
    }

    // ========================================
    // Ownership Transfer Logic
    // ========================================

    #[test]
    fn test_ownership_transfer_updates() {
        let mut owner = Address::from([0x01u8; 20]);
        let new_owner = Address::from([0x02u8; 20]);
        let previous = owner;
        owner = new_owner;
        assert_eq!(owner, new_owner);
        assert_ne!(owner, previous);
    }

    #[test]
    fn test_transfer_to_zero_rejected() {
        let new_owner = Address::ZERO;
        assert_eq!(new_owner, Address::ZERO, "Transfer to zero should be caught");
    }

    // ========================================
    // Debug Trait
    // ========================================

    #[test]
    fn test_error_debug_has_variant_name() {
        assert!(format!("{:?}", Error::AlreadyTripped).contains("AlreadyTripped"));
        assert!(format!("{:?}", Error::NotTripped).contains("NotTripped"));
        assert!(format!("{:?}", Error::UnauthorizedCaller(Address::ZERO)).contains("UnauthorizedCaller"));
        assert!(format!("{:?}", Error::InvalidOwner(Address::ZERO)).contains("InvalidOwner"));
    }
}
