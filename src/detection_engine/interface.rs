//! Interface and event definitions for the DetectionEngine contract

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

sol! {
    /// Emitted when a new metric is registered
    event MetricRegistered(uint256 indexed id, uint256 threshold);

    /// Emitted when a metric value is reported
    event MetricReported(uint256 indexed id, uint256 value);

    /// Emitted when an anomaly is detected
    event AnomalyDetected(uint256 indexed id, uint256 current, uint256 threshold);

    /// Emitted when ownership is transferred
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
}

/// DetectionEngine interface trait
pub trait IDetectionEngine {
    /// Error type for the interface
    type Error;

    /// Register a new metric with a threshold
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the metric
    /// * `threshold` - Maximum allowed value before anomaly is detected
    ///
    /// # Errors
    /// Returns `UnauthorizedCaller` if caller is not the owner
    fn register_metric(&mut self, id: U256, threshold: U256) -> Result<(), Self::Error>;

    /// Report a new value for a registered metric
    ///
    /// # Arguments
    /// * `id` - Metric identifier
    /// * `value` - Current value to report
    ///
    /// # Errors
    /// Returns `MetricNotFound` if metric is not registered
    fn report_metric(&mut self, id: U256, value: U256) -> Result<(), Self::Error>;

    /// Check if a metric value indicates an anomaly
    ///
    /// # Arguments
    /// * `id` - Metric identifier
    ///
    /// # Returns
    /// `true` if current value exceeds threshold, `false` otherwise
    ///
    /// # Errors
    /// Returns `MetricNotFound` if metric is not registered
    fn check_anomaly(&self, id: U256) -> Result<bool, Self::Error>;

    /// Get the threshold for a metric
    ///
    /// # Arguments
    /// * `id` - Metric identifier
    ///
    /// # Returns
    /// The threshold value
    fn get_threshold(&self, id: U256) -> U256;

    /// Get the current value for a metric
    ///
    /// # Arguments
    /// * `id` - Metric identifier
    ///
    /// # Returns
    /// The current value
    fn get_current_value(&self, id: U256) -> U256;

    /// Get the total number of registered metrics
    ///
    /// # Returns
    /// The metric count
    fn get_metric_count(&self) -> U256;

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
