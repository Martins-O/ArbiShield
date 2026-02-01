//! Error types for the DetectionEngine contract

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::{sol, SolError};

sol! {
    // === V1 Errors (preserved) ===
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

    // === V2: Role-Based Access Control Errors ===
    /// Emitted when caller lacks the required role for an operation
    error InsufficientRole(address caller, uint8 required_role);

    /// Emitted when an invalid role value is provided
    error InvalidRole(uint8 role);

    /// Emitted when attempting to revoke one's own role (prevents lockout)
    error CannotRevokeOwnRole(address caller);

    // === V2: Pattern Registration Errors ===
    /// Emitted when a pattern ID is not found
    error PatternNotFound(uint256 id);

    /// Emitted when attempting to register a pattern with an existing ID
    error PatternAlreadyExists(uint256 id);

    /// Emitted when an invalid pattern type bitmask is provided
    error InvalidPatternType(uint256 pattern_type);

    /// Emitted when a severity value exceeds the maximum allowed
    error InvalidSeverity(uint256 severity);

    /// Emitted when operating on an inactive pattern
    error PatternInactive(uint256 id);

    // === V2: Analysis Errors ===
    /// Emitted when an analysis ID is not found
    error AnalysisNotFound(uint256 id);

    /// Emitted when invalid inputs are provided for analysis
    error InvalidAnalysisInput();

    // === V2: Whitelist Errors ===
    /// Emitted when attempting to whitelist an already whitelisted address
    error AddressAlreadyWhitelisted(address addr);

    /// Emitted when attempting to remove a non-whitelisted address
    error AddressNotWhitelisted(address addr);

    /// Emitted when an invalid address (zero) is provided
    error InvalidAddress(address addr);

    // === V2: Configuration Errors ===
    /// Emitted when an invalid threshold configuration value is provided
    error InvalidThresholdValue(uint256 value);

    // === V2: Limit Errors ===
    /// Emitted when a batch operation exceeds the maximum allowed size
    error BatchSizeTooLarge(uint256 size, uint256 max);

    // === V2: ERC-165 Errors ===
    /// Emitted when querying for an unsupported interface
    error UnsupportedInterface(bytes4 interface_id);
}

/// Error types for DetectionEngine operations
#[derive(Debug)]
pub enum Error {
    // === V1 Errors (preserved) ===
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

    // === V2: Role Management Errors ===
    /// Caller does not have sufficient role permissions
    InsufficientRole {
        caller: Address,
        required_role: u8,
    },
    /// Invalid role value
    InvalidRole(u8),
    /// Cannot revoke own role (prevents lockout)
    CannotRevokeOwnRole(Address),

    // === V2: Pattern Errors ===
    /// Pattern ID not found
    PatternNotFound { id: U256 },
    /// Pattern with this ID already exists
    PatternAlreadyExists { id: U256 },
    /// Invalid pattern type bitmask
    InvalidPatternType { pattern_type: U256 },
    /// Severity exceeds maximum allowed
    InvalidSeverity { severity: U256 },
    /// Pattern is inactive
    PatternInactive { id: U256 },

    // === V2: Analysis Errors ===
    /// Analysis ID not found
    AnalysisNotFound { id: U256 },
    /// Invalid analysis input (zero addresses)
    InvalidAnalysisInput,

    // === V2: Whitelist Errors ===
    /// Address is already whitelisted
    AddressAlreadyWhitelisted(Address),
    /// Address is not whitelisted
    AddressNotWhitelisted(Address),
    /// Invalid address (zero address)
    InvalidAddress(Address),

    // === V2: Configuration Errors ===
    /// Invalid threshold configuration value
    InvalidThresholdValue { value: U256 },

    // === V2: Limit Errors ===
    /// Batch size exceeds maximum
    BatchSizeTooLarge {
        size: U256,
        max: U256,
    },

    // === V2: ERC-165 Errors ===
    /// Unsupported interface queried
    UnsupportedInterface(FixedBytes<4>),
}

impl From<Error> for Vec<u8> {
    fn from(err: Error) -> Vec<u8> {
        match err {
            Error::UnauthorizedCaller(caller) => UnauthorizedCaller { caller }.abi_encode(),
            Error::MetricNotFound { id } => MetricNotFound { id }.abi_encode(),
            Error::InvalidThreshold { value } => InvalidThreshold { value }.abi_encode(),
            Error::ThresholdExceeded { current, threshold } => {
                ThresholdExceeded { current, threshold }.abi_encode()
            }
            Error::InvalidOwner(owner) => InvalidOwner { owner }.abi_encode(),
        }
    }
}
