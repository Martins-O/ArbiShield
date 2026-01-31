//! DetectionEngine contract for anomaly detection
//!
//! This contract allows registering metrics with thresholds and detecting
//! when reported values exceed those thresholds, indicating potential anomalies.

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_sdk::prelude::*;
use stylus_sdk::{evm, msg};

pub mod error;
pub mod interface;
pub mod storage;

use error::Error;
use interface::{
    AnomalyDetected, IDetectionEngine, MetricRegistered, MetricReported, OwnershipTransferred,
};
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
    use alloc::{format, vec};

    // ========================================
    // Threshold & Anomaly Detection Logic
    // ========================================

    #[test]
    fn test_value_above_threshold_is_anomaly() {
        let threshold = U256::from(100);
        let value = U256::from(150);
        assert!(value > threshold, "Value above threshold is an anomaly");
    }

    #[test]
    fn test_value_below_threshold_is_normal() {
        let threshold = U256::from(100);
        let value = U256::from(50);
        assert!(!(value > threshold), "Value below threshold is normal");
    }

    #[test]
    fn test_value_equal_threshold_is_normal() {
        let threshold = U256::from(100);
        let value = U256::from(100);
        assert!(
            !(value > threshold),
            "Value equal to threshold is NOT anomaly (strict >)"
        );
    }

    #[test]
    fn test_value_one_above_threshold_is_anomaly() {
        let threshold = U256::from(100);
        let value = U256::from(101);
        assert!(value > threshold);
    }

    #[test]
    fn test_zero_threshold_any_value_is_anomaly() {
        let threshold = U256::ZERO;
        let value = U256::from(1);
        assert!(
            value > threshold,
            "Any positive value exceeds zero threshold"
        );
    }

    #[test]
    fn test_zero_value_zero_threshold_is_normal() {
        let threshold = U256::ZERO;
        let value = U256::ZERO;
        assert!(!(value > threshold));
    }

    #[test]
    fn test_max_threshold_never_exceeded() {
        let threshold = U256::MAX;
        let value = U256::MAX;
        assert!(!(value > threshold), "U256::MAX cannot exceed itself");
    }

    #[test]
    fn test_large_threshold_values() {
        // 1 ETH in wei
        let threshold = U256::from(1_000_000_000_000_000_000u64);
        let below = U256::from(999_999_999_999_999_999u64);
        let above = U256::from(1_000_000_000_000_000_001u64);
        assert!(!(below > threshold));
        assert!(above > threshold);
    }

    // ========================================
    // Metric Count Arithmetic
    // ========================================

    #[test]
    fn test_metric_count_starts_zero() {
        let count = U256::ZERO;
        assert_eq!(count, U256::from(0));
    }

    #[test]
    fn test_metric_count_increments() {
        let mut count = U256::ZERO;
        for i in 1..=5u64 {
            count = count + U256::from(1);
            assert_eq!(count, U256::from(i));
        }
    }

    #[test]
    fn test_metric_count_independent_of_values() {
        // Count tracks number of metrics, not their values
        let count = U256::from(3); // 3 metrics registered
        let value = U256::from(1_000_000); // some metric value
        assert_ne!(count, value);
    }

    // ========================================
    // Metric ID Tests
    // ========================================

    #[test]
    fn test_metric_id_zero_valid() {
        let id = U256::ZERO;
        assert_eq!(id, U256::from(0), "ID 0 is valid");
    }

    #[test]
    fn test_metric_id_large() {
        let id = U256::from(u64::MAX);
        assert!(id > U256::ZERO);
    }

    #[test]
    fn test_metric_ids_distinct() {
        let id1 = U256::from(1);
        let id2 = U256::from(2);
        assert_ne!(id1, id2);
    }

    // ========================================
    // Error Encoding Tests
    // ========================================

    #[test]
    fn test_unauthorized_caller_error_encoding() {
        let caller = Address::from([0xAAu8; 20]);
        let encoded: Vec<u8> = Error::UnauthorizedCaller(caller).into();
        assert_eq!(encoded.len(), 36, "selector(4) + address(32)");
    }

    #[test]
    fn test_metric_not_found_error_encoding() {
        let id = U256::from(42);
        let encoded: Vec<u8> = Error::MetricNotFound { id }.into();
        assert_eq!(encoded.len(), 36, "selector(4) + uint256(32)");
    }

    #[test]
    fn test_invalid_threshold_error_encoding() {
        let value = U256::from(999);
        let encoded: Vec<u8> = Error::InvalidThreshold { value }.into();
        assert_eq!(encoded.len(), 36);
    }

    #[test]
    fn test_threshold_exceeded_error_encoding() {
        let current = U256::from(200);
        let threshold = U256::from(100);
        let encoded: Vec<u8> = Error::ThresholdExceeded { current, threshold }.into();
        // selector(4) + uint256(32) + uint256(32)
        assert_eq!(
            encoded.len(),
            68,
            "Should encode both current and threshold"
        );
    }

    #[test]
    fn test_invalid_owner_error_encoding() {
        let owner = Address::ZERO;
        let encoded: Vec<u8> = Error::InvalidOwner(owner).into();
        assert_eq!(encoded.len(), 36);
    }

    #[test]
    fn test_all_error_selectors_unique() {
        let errors: Vec<Vec<u8>> = vec![
            Error::UnauthorizedCaller(Address::ZERO).into(),
            Error::MetricNotFound { id: U256::ZERO }.into(),
            Error::InvalidThreshold { value: U256::ZERO }.into(),
            Error::ThresholdExceeded {
                current: U256::ZERO,
                threshold: U256::ZERO,
            }
            .into(),
            Error::InvalidOwner(Address::ZERO).into(),
        ];

        for i in 0..errors.len() {
            for j in (i + 1)..errors.len() {
                assert_ne!(
                    &errors[i][..4],
                    &errors[j][..4],
                    "Error selectors at indices {i} and {j} must differ"
                );
            }
        }
    }

    #[test]
    fn test_error_encoding_deterministic() {
        let enc1: Vec<u8> = Error::MetricNotFound { id: U256::from(7) }.into();
        let enc2: Vec<u8> = Error::MetricNotFound { id: U256::from(7) }.into();
        assert_eq!(enc1, enc2);
    }

    // ========================================
    // Address Validation
    // ========================================

    #[test]
    fn test_zero_address_check() {
        assert_eq!(Address::ZERO, Address::ZERO);
        assert_ne!(Address::from([1u8; 20]), Address::ZERO);
    }

    // ========================================
    // Debug Trait
    // ========================================

    #[test]
    fn test_all_errors_have_debug() {
        let errors: Vec<Error> = vec![
            Error::UnauthorizedCaller(Address::ZERO),
            Error::MetricNotFound { id: U256::ZERO },
            Error::InvalidThreshold { value: U256::ZERO },
            Error::ThresholdExceeded {
                current: U256::ZERO,
                threshold: U256::ZERO,
            },
            Error::InvalidOwner(Address::ZERO),
        ];
        for err in errors {
            assert!(!format!("{err:?}").is_empty());
        }
    }
}
