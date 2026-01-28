//! Error types for the CircuitBreaker contract

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::Address;
use alloy_sol_types::{sol, SolError};

sol! {
    /// Emitted when attempting to trip an already tripped circuit
    error AlreadyTripped();

    /// Emitted when attempting to reset a circuit that is not tripped
    error NotTripped();

    /// Emitted when an unauthorized caller attempts to execute a privileged function
    error UnauthorizedCaller(address caller);

    /// Emitted when an invalid owner address is provided
    error InvalidOwner(address owner);
}

/// Error types for CircuitBreaker operations
#[derive(Debug)]
pub enum Error {
    /// Circuit is already tripped
    AlreadyTripped,
    /// Circuit is not tripped
    NotTripped,
    /// Unauthorized caller attempted a privileged operation
    UnauthorizedCaller(Address),
    /// Invalid owner address (zero address)
    InvalidOwner(Address),
}

impl From<Error> for Vec<u8> {
    fn from(err: Error) -> Vec<u8> {
        match err {
            Error::AlreadyTripped => AlreadyTripped {}.abi_encode(),
            Error::NotTripped => NotTripped {}.abi_encode(),
            Error::UnauthorizedCaller(caller) => {
                UnauthorizedCaller { caller }.abi_encode()
            }
            Error::InvalidOwner(owner) => {
                InvalidOwner { owner }.abi_encode()
            }
        }
    }
}
