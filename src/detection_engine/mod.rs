//! DetectionEngine contract for anomaly detection
//!
//! This contract allows registering metrics with thresholds and detecting
//! when reported values exceed those thresholds, indicating potential anomalies.

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_sdk::{evm, msg};
use stylus_sdk::prelude::*;

pub mod error;
pub mod interface;
pub mod storage;

use error::Error;
use interface::{AnomalyDetected, IDetectionEngine, MetricRegistered, MetricReported, OwnershipTransferred};
use storage::DetectionEngine;

// Implementation of the DetectionEngine contract
impl IDetectionEngine for DetectionEngine {
    type Error = Error;

    fn register_metric(&mut self, id: U256, threshold: U256) -> Result<(), Self::Error> {
        // Only owner can register metrics
        if msg::sender() != self.owner.get() {
            return Err(Error::UnauthorizedCaller(msg::sender()));
        }

        // Store the threshold
        self.thresholds.setter(id).set(threshold);

        // Increment metric count
        let count = self.metric_count.get();
        self.metric_count.set(count + U256::from(1));

        // Emit event
        evm::log(MetricRegistered { id, threshold });

        Ok(())
    }

    fn report_metric(&mut self, id: U256, value: U256) -> Result<(), Self::Error> {
        // Check if metric exists (has a threshold set)
        let threshold = self.thresholds.get(id);
        if threshold == U256::ZERO && !self.thresholds.setter(id).get().is_zero() {
            return Err(Error::MetricNotFound { id });
        }

        // Store the current value
        self.current_values.setter(id).set(value);

        // Emit metric reported event
        evm::log(MetricReported { id, value });

        // Check if anomaly detected and emit event if so
        if value > threshold {
            evm::log(AnomalyDetected {
                id,
                current: value,
                threshold,
            });
        }

        Ok(())
    }

    fn check_anomaly(&self, id: U256) -> Result<bool, Self::Error> {
        let threshold = self.thresholds.get(id);
        let current = self.current_values.get(id);

        // If threshold is zero and no value set, metric doesn't exist
        if threshold == U256::ZERO && current == U256::ZERO {
            return Err(Error::MetricNotFound { id });
        }

        Ok(current > threshold)
    }

    fn get_threshold(&self, id: U256) -> U256 {
        self.thresholds.get(id)
    }

    fn get_current_value(&self, id: U256) -> U256 {
        self.current_values.get(id)
    }

    fn get_metric_count(&self) -> U256 {
        self.metric_count.get()
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
impl DetectionEngine {
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

    /// Register a new metric with a threshold
    pub fn register_metric(&mut self, id: U256, threshold: U256) -> Result<(), Vec<u8>> {
        IDetectionEngine::register_metric(self, id, threshold).map_err(|e| e.into())
    }

    /// Report a metric value
    pub fn report_metric(&mut self, id: U256, value: U256) -> Result<(), Vec<u8>> {
        IDetectionEngine::report_metric(self, id, value).map_err(|e| e.into())
    }

    /// Check if a metric indicates an anomaly
    pub fn check_anomaly(&self, id: U256) -> Result<bool, Vec<u8>> {
        IDetectionEngine::check_anomaly(self, id).map_err(|e| e.into())
    }

    /// Get threshold for a metric
    pub fn get_threshold(&self, id: U256) -> U256 {
        IDetectionEngine::get_threshold(self, id)
    }

    /// Get current value for a metric
    pub fn get_current_value(&self, id: U256) -> U256 {
        IDetectionEngine::get_current_value(self, id)
    }

    /// Get total metric count
    pub fn get_metric_count(&self) -> U256 {
        IDetectionEngine::get_metric_count(self)
    }

    /// Get current owner
    pub fn owner(&self) -> Address {
        IDetectionEngine::owner(self)
    }

    /// Transfer ownership
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        IDetectionEngine::transfer_ownership(self, new_owner).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_comparison() {
        let threshold = U256::from(100);
        let value = U256::from(150);
        assert!(value > threshold);
    }
}
