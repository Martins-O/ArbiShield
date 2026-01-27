//! Error types for the DetectionEngine contract

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

sol! {
    /// Emitted when an unauthorized caller attempts to execute a privileged function
    error UnauthorizedCaller(address caller);

    /// Emitted when a metric ID is not found
    error MetricNotFound(uint256 id);

    /// Emitted when an invalid threshold value is provided
    error InvalidThreshold(uint256 value);

    /// Emitted when a metric value exceeds its threshold
    error ThresholdExceeded(uint256 current, uint256 threshold);

    /// Emitted when an invalid owner address is provided
    error InvalidOwner(address owner);
}

/// Error types for DetectionEngine operations
#[derive(Debug)]
pub enum Error {
    /// Unauthorized caller attempted a privileged operation
    UnauthorizedCaller(Address),
    /// Metric ID not found in registry
    MetricNotFound { id: U256 },
    /// Invalid threshold value
    InvalidThreshold { value: U256 },
    /// Threshold exceeded
    ThresholdExceeded { current: U256, threshold: U256 },
    /// Invalid owner address (zero address)
    InvalidOwner(Address),
}

impl From<Error> for Vec<u8> {
    fn from(err: Error) -> Vec<u8> {
        match err {
            Error::UnauthorizedCaller(caller) => {
                UnauthorizedCaller { caller }.encode()
            }
            Error::MetricNotFound { id } => {
                MetricNotFound { id }.encode()
            }
            Error::InvalidThreshold { value } => {
                InvalidThreshold { value }.encode()
            }
            Error::ThresholdExceeded { current, threshold } => {
                ThresholdExceeded { current, threshold }.encode()
            }
            Error::InvalidOwner(owner) => {
                InvalidOwner { owner }.encode()
            }
        }
    }
}
