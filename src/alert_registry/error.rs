//! Error types for the AlertRegistry contract

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

sol! {
    /// Emitted when an alert ID is not found
    error AlertNotFound(uint256 id);

    /// Emitted when an invalid alert is provided
    error InvalidAlert();

    /// Emitted when an unauthorized caller attempts to execute a privileged function
    error UnauthorizedCaller(address caller);

    /// Emitted when an invalid owner address is provided
    error InvalidOwner(address owner);
}

/// Error types for AlertRegistry operations
#[derive(Debug)]
pub enum Error {
    /// Alert ID not found in registry
    AlertNotFound { id: U256 },
    /// Invalid alert data provided
    InvalidAlert,
    /// Unauthorized caller attempted a privileged operation
    UnauthorizedCaller(Address),
    /// Invalid owner address (zero address)
    InvalidOwner(Address),
}

impl From<Error> for Vec<u8> {
    fn from(err: Error) -> Vec<u8> {
        match err {
            Error::AlertNotFound { id } => {
                AlertNotFound { id }.encode()
            }
            Error::InvalidAlert => {
                InvalidAlert {}.encode()
            }
            Error::UnauthorizedCaller(caller) => {
                UnauthorizedCaller { caller }.encode()
            }
            Error::InvalidOwner(owner) => {
                InvalidOwner { owner }.encode()
            }
        }
    }
}
