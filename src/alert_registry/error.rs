//! Error types for the AlertRegistry contract

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::{sol, SolError};

sol! {
    // === V1 Errors (preserved) ===

    /// Emitted when an alert ID is not found
    error AlertNotFound(uint256 id);

    /// Emitted when an invalid alert is provided
    error InvalidAlert();

    /// Emitted when an unauthorized caller attempts to execute a privileged function
    error UnauthorizedCaller(address caller);

    /// Emitted when an invalid owner address is provided
    error InvalidOwner(address owner);

    // === V2: Role-Based Access Control Errors ===

    /// Emitted when caller lacks the required role for an operation
    error InsufficientRole(address caller, uint8 required_role);

    /// Emitted when an invalid role value is provided
    error InvalidRole(uint8 role);

    /// Emitted when attempting to revoke one's own role (prevents lockout)
    error CannotRevokeOwnRole(address caller);

    // === V2: Subscriber Errors ===

    /// Emitted when attempting to add an already subscribed address
    error AlreadySubscribed(address subscriber);

    /// Emitted when attempting to remove a non-subscribed address
    error NotSubscribed(address subscriber);

    /// Emitted when an invalid subscriber address is provided
    error InvalidSubscriber(address subscriber);

    // === V2: Acknowledgment Errors ===

    /// Emitted when attempting to acknowledge an already acknowledged alert
    error AlertAlreadyAcknowledged(uint256 id);

    /// Emitted when caller is not the protocol associated with the alert
    error NotAlertProtocol(address caller, uint256 id);

    // === V2: Priority Errors ===

    /// Emitted when an invalid priority level is provided
    error InvalidPriorityLevel(uint256 level);

    // === V2: Limit Errors ===

    /// Emitted when a batch operation exceeds the maximum allowed size
    error BatchSizeTooLarge(uint256 size, uint256 max);

    // === V2: Configuration Errors ===

    /// Emitted when an invalid expiration duration is provided
    error InvalidExpirationDuration(uint256 duration);

    /// Emitted when an invalid address (zero) is provided
    error InvalidAddress(address addr);

    // === V2: ERC-165 Errors ===

    /// Emitted when querying for an unsupported interface
    error UnsupportedInterface(bytes4 interface_id);
}

/// Error types for AlertRegistry operations
#[derive(Debug)]
pub enum Error {
    // === V1 Errors (preserved) ===
    /// Alert ID not found in registry
    AlertNotFound { id: U256 },
    /// Invalid alert data provided
    InvalidAlert,
    /// Unauthorized caller attempted a privileged operation
    UnauthorizedCaller(Address),
    /// Invalid owner address (zero address)
    InvalidOwner(Address),

    // === V2: Role Management Errors ===
    /// Caller does not have sufficient role permissions
    InsufficientRole { caller: Address, required_role: u8 },
    /// Invalid role value
    InvalidRole(u8),
    /// Cannot revoke own role (prevents lockout)
    CannotRevokeOwnRole(Address),

    // === V2: Subscriber Errors ===
    /// Address is already subscribed
    AlreadySubscribed(Address),
    /// Address is not subscribed
    NotSubscribed(Address),
    /// Invalid subscriber address
    InvalidSubscriber(Address),

    // === V2: Acknowledgment Errors ===
    /// Alert is already acknowledged
    AlertAlreadyAcknowledged { id: U256 },
    /// Caller is not the protocol for this alert
    NotAlertProtocol { caller: Address, id: U256 },

    // === V2: Priority Errors ===
    /// Invalid priority level
    InvalidPriorityLevel { level: U256 },

    // === V2: Limit Errors ===
    /// Batch size exceeds maximum
    BatchSizeTooLarge { size: U256, max: U256 },

    // === V2: Configuration Errors ===
    /// Invalid expiration duration
    InvalidExpirationDuration { duration: U256 },
    /// Invalid address (zero address)
    InvalidAddress(Address),

    // === V2: ERC-165 Errors ===
    /// Unsupported interface queried
    UnsupportedInterface(FixedBytes<4>),
}

impl From<Error> for Vec<u8> {
    fn from(err: Error) -> Vec<u8> {
        match err {
            // V1 errors
            Error::AlertNotFound { id } => AlertNotFound { id }.abi_encode(),
            Error::InvalidAlert => InvalidAlert {}.abi_encode(),
            Error::UnauthorizedCaller(caller) => UnauthorizedCaller { caller }.abi_encode(),
            Error::InvalidOwner(owner) => InvalidOwner { owner }.abi_encode(),

            // V2: Role management errors
            Error::InsufficientRole {
                caller,
                required_role,
            } => InsufficientRole {
                caller,
                required_role,
            }
            .abi_encode(),
            Error::InvalidRole(role) => InvalidRole { role }.abi_encode(),
            Error::CannotRevokeOwnRole(caller) => CannotRevokeOwnRole { caller }.abi_encode(),

            // V2: Subscriber errors
            Error::AlreadySubscribed(subscriber) => AlreadySubscribed { subscriber }.abi_encode(),
            Error::NotSubscribed(subscriber) => NotSubscribed { subscriber }.abi_encode(),
            Error::InvalidSubscriber(subscriber) => InvalidSubscriber { subscriber }.abi_encode(),

            // V2: Acknowledgment errors
            Error::AlertAlreadyAcknowledged { id } => AlertAlreadyAcknowledged { id }.abi_encode(),
            Error::NotAlertProtocol { caller, id } => NotAlertProtocol { caller, id }.abi_encode(),

            // V2: Priority errors
            Error::InvalidPriorityLevel { level } => InvalidPriorityLevel { level }.abi_encode(),

            // V2: Limit errors
            Error::BatchSizeTooLarge { size, max } => BatchSizeTooLarge { size, max }.abi_encode(),

            // V2: Configuration errors
            Error::InvalidExpirationDuration { duration } => {
                InvalidExpirationDuration { duration }.abi_encode()
            }
            Error::InvalidAddress(addr) => InvalidAddress { addr }.abi_encode(),

            // V2: ERC-165 errors
            Error::UnsupportedInterface(interface_id) => {
                UnsupportedInterface { interface_id }.abi_encode()
            }
        }
    }
}
