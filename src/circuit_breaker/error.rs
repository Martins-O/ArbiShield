//! Error types for the CircuitBreaker contract

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256, FixedBytes};
use alloy_sol_types::{sol, SolError};

sol! {
    // === Existing Global Circuit Errors ===
    /// Emitted when attempting to trip an already tripped circuit
    error AlreadyTripped();

    /// Emitted when attempting to reset a circuit that is not tripped
    error NotTripped();

    /// Emitted when an unauthorized caller attempts to execute a privileged function
    error UnauthorizedCaller(address caller);

    /// Emitted when an invalid owner address is provided
    error InvalidOwner(address owner);

    // === Protocol Registration Errors ===
    /// Emitted when attempting to operate on an unregistered protocol
    error ProtocolNotRegistered(address protocol);

    /// Emitted when attempting to register an already registered protocol
    error ProtocolAlreadyRegistered(address protocol);

    /// Emitted when an invalid protocol address is provided (e.g., zero address)
    error InvalidProtocol(address protocol);

    // === Role-Based Access Control Errors ===
    /// Emitted when caller lacks the required role for an operation
    error InsufficientRole(address caller, uint8 required_role);

    /// Emitted when an invalid role value is provided
    error InvalidRole(uint8 role);

    /// Emitted when attempting to revoke one's own role (prevents lockout)
    error CannotRevokeOwnRole(address caller);

    // === Time Lock Errors ===
    /// Emitted when attempting to reset before minimum pause duration has elapsed
    error MinimumPauseDurationNotMet(uint256 elapsed, uint256 required);

    /// Emitted when attempting to trip during an active cooldown period
    error CooldownPeriodActive(uint256 remaining);

    /// Emitted when an invalid duration value is provided
    error InvalidDuration(uint256 duration);

    /// Emitted when a pause duration exceeds the maximum allowed
    error MaxPauseDurationExceeded(uint256 duration, uint256 max);

    // === Operational Errors ===
    /// Emitted when attempting to trip an already paused protocol
    error ProtocolAlreadyPaused(address protocol);

    /// Emitted when attempting to reset a protocol that is not paused
    error ProtocolNotPaused(address protocol);

    /// Emitted when a protocol operation conflicts with an active global circuit
    error GlobalCircuitActive();

    /// Emitted when an empty protocol list is provided to a batch operation
    error EmptyProtocolList();

    /// Emitted when a batch operation exceeds the maximum allowed size
    error BatchSizeTooLarge(uint256 size, uint256 max);

    /// Emitted when an index is out of bounds
    error IndexOutOfBounds(uint256 index, uint256 length);

    // === ERC-165 Errors ===
    /// Emitted when querying for an unsupported interface
    error UnsupportedInterface(bytes4 interface_id);
}

/// Error types for CircuitBreaker operations
#[derive(Debug)]
pub enum Error {
    // === Existing Errors ===
    /// Circuit is already tripped
    AlreadyTripped,
    /// Circuit is not tripped
    NotTripped,
    /// Unauthorized caller attempted a privileged operation
    UnauthorizedCaller(Address),
    /// Invalid owner address (zero address)
    InvalidOwner(Address),

    // === Protocol Management Errors ===
    /// Protocol is not registered
    ProtocolNotRegistered(Address),
    /// Protocol is already registered
    ProtocolAlreadyRegistered(Address),
    /// Invalid protocol address
    InvalidProtocol(Address),

    // === Role Management Errors ===
    /// Caller does not have sufficient role permissions
    InsufficientRole {
        caller: Address,
        required_role: u8,
    },
    /// Invalid role value
    InvalidRole(u8),
    /// Cannot revoke own role (prevents lockout)
    CannotRevokeOwnRole(Address),

    // === Time Lock Errors ===
    /// Minimum pause duration not met
    MinimumPauseDurationNotMet {
        elapsed: U256,
        required: U256,
    },
    /// Cooldown period is still active
    CooldownPeriodActive {
        remaining: U256,
    },
    /// Invalid duration value
    InvalidDuration(U256),
    /// Duration exceeds maximum allowed
    MaxPauseDurationExceeded {
        duration: U256,
        max: U256,
    },

    // === Operational Errors ===
    /// Protocol is already paused
    ProtocolAlreadyPaused(Address),
    /// Protocol is not currently paused
    ProtocolNotPaused(Address),
    /// Global circuit breaker is active
    GlobalCircuitActive,
    /// Empty protocol list provided
    EmptyProtocolList,
    /// Batch size exceeds maximum
    BatchSizeTooLarge {
        size: U256,
        max: U256,
    },
    /// Index out of bounds
    IndexOutOfBounds {
        index: U256,
        length: U256,
    },

    // === ERC-165 Errors ===
    /// Unsupported interface queried
    UnsupportedInterface(FixedBytes<4>),
}

impl From<Error> for Vec<u8> {
    fn from(err: Error) -> Vec<u8> {
        match err {
            // Existing errors
            Error::AlreadyTripped => AlreadyTripped {}.abi_encode(),
            Error::NotTripped => NotTripped {}.abi_encode(),
            Error::UnauthorizedCaller(caller) => UnauthorizedCaller { caller }.abi_encode(),
            Error::InvalidOwner(owner) => InvalidOwner { owner }.abi_encode(),
        }
    }
}
