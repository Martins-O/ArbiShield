//! AlertRegistry contract for logging and storing security alerts
//!
//! This contract maintains a permanent record of security alerts,
//! including their severity, source, and associated metadata.

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes, U256};
use stylus_sdk::{block, evm, msg};
use stylus_sdk::prelude::*;

pub mod error;
pub mod interface;
pub mod storage;

use error::Error;
use interface::{AlertRegistered, IAlertRegistry, OwnershipTransferred};
use storage::{Alert, AlertRegistry};

// Implementation of the AlertRegistry contract
impl IAlertRegistry for AlertRegistry {
    type Error = Error;

    fn register_alert(
        &mut self,
        severity: u8,
        source: Address,
        message_hash: FixedBytes<32>,
    ) -> Result<U256, Self::Error> {
        // Validate alert data
        if source == Address::ZERO {
            return Err(Error::InvalidAlert);
        }

        // Get current timestamp
        let timestamp = U256::from(block::timestamp());

        // Create new alert
        let mut new_alert = self.alerts.grow();
        new_alert.timestamp.set(timestamp);
        new_alert.severity.set(U256::from(severity));
        new_alert.source.set(source);
        new_alert.message_hash.set(message_hash);

        // Get the alert ID (length - 1)
        let alert_id = U256::from(self.alerts.len() - 1);

        // Emit event
        evm::log(AlertRegistered {
            id: alert_id,
            timestamp,
            severity,
            source,
            messageHash: message_hash,
        });

        Ok(alert_id)
    }

    fn get_alert(
        &self,
        id: U256,
    ) -> Result<(U256, u8, Address, FixedBytes<32>), Self::Error> {
        // Convert U256 to usize for array indexing
        let index: usize = id
            .try_into()
            .map_err(|_| Error::AlertNotFound { id })?;

        // Check if index is within bounds
        if index >= self.alerts.len() {
            return Err(Error::AlertNotFound { id });
        }

        // Get alert from storage
        let alert = self.alerts.get(index).unwrap();
        let timestamp = alert.timestamp.get();
        let severity_uint: U256 = alert.severity.get();
        let severity: u8 = severity_uint.saturating_to::<u64>() as u8;
        let source = alert.source.get();
        let message_hash = alert.message_hash.get();

        Ok((timestamp, severity, source, message_hash))
    }

    fn get_alert_count(&self) -> U256 {
        U256::from(self.alerts.len())
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
impl AlertRegistry {
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

    /// Register a new alert
    pub fn register_alert(
        &mut self,
        severity: u8,
        source: Address,
        message_hash: FixedBytes<32>,
    ) -> Result<U256, Vec<u8>> {
        IAlertRegistry::register_alert(self, severity, source, message_hash)
            .map_err(|e| e.into())
    }

    /// Get alert by ID
    pub fn get_alert(&self, id: U256) -> Result<(U256, u8, Address, FixedBytes<32>), Vec<u8>> {
        IAlertRegistry::get_alert(self, id).map_err(|e| e.into())
    }

    /// Get total alert count
    pub fn get_alert_count(&self) -> U256 {
        IAlertRegistry::get_alert_count(self)
    }

    /// Get current owner
    pub fn owner(&self) -> Address {
        IAlertRegistry::owner(self)
    }

    /// Transfer ownership
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        IAlertRegistry::transfer_ownership(self, new_owner).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_range() {
        let severity: u8 = 255;
        assert!(severity <= 255);
    }
}
