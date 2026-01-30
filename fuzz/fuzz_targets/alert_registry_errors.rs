#![no_main]

use libfuzzer_sys::fuzz_target;
use alloy_primitives::{Address, U256, FixedBytes};
use arbishield::alert_registry::error::Error;
use alloc::vec::Vec;

extern crate alloc;

fuzz_target!(|data: &[u8]| {
    // Need at least 1 byte for error type selection
    if data.is_empty() {
        return;
    }

    let error_type = data[0] % 17; // 17 error types total

    // Helper to extract U256 from data
    let extract_u256 = |offset: usize| -> U256 {
        if data.len() < offset + 32 {
            return U256::ZERO;
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&data[offset..offset + 32]);
        U256::from_be_bytes(bytes)
    };

    // Helper to extract Address from data
    let extract_address = |offset: usize| -> Address {
        if data.len() < offset + 20 {
            return Address::ZERO;
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&data[offset..offset + 20]);
        Address::from(bytes)
    };

    // Helper to extract u8 from data
    let extract_u8 = |offset: usize| -> u8 {
        data.get(offset).copied().unwrap_or(0)
    };

    // Helper to extract FixedBytes<4> from data
    let extract_bytes4 = |offset: usize| -> FixedBytes<4> {
        if data.len() < offset + 4 {
            return FixedBytes::<4>::ZERO;
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&data[offset..offset + 4]);
        FixedBytes::from(bytes)
    };

    // Create error based on type
    let error: Error = match error_type {
        0 => Error::AlertNotFound { id: extract_u256(1) },
        1 => Error::InvalidAlert,
        2 => Error::UnauthorizedCaller(extract_address(1)),
        3 => Error::InvalidOwner(extract_address(1)),
        4 => Error::InsufficientRole {
            caller: extract_address(1),
            required_role: extract_u8(21),
        },
        5 => Error::InvalidRole(extract_u8(1)),
        6 => Error::CannotRevokeOwnRole(extract_address(1)),
        7 => Error::AlreadySubscribed(extract_address(1)),
        8 => Error::NotSubscribed(extract_address(1)),
        9 => Error::InvalidSubscriber(extract_address(1)),
        10 => Error::AlertAlreadyAcknowledged { id: extract_u256(1) },
        11 => Error::NotAlertProtocol {
            caller: extract_address(1),
            id: extract_u256(21),
        },
        12 => Error::InvalidPriorityLevel { level: extract_u256(1) },
        13 => Error::BatchSizeTooLarge {
            size: extract_u256(1),
            max: extract_u256(33),
        },
        14 => Error::InvalidExpirationDuration { duration: extract_u256(1) },
        15 => Error::InvalidAddress(extract_address(1)),
        16 => Error::UnsupportedInterface(extract_bytes4(1)),
        _ => unreachable!(),
    };

    // Encode error
    let encoded: Vec<u8> = error.clone().into();

    // === Invariant Checks ===

    // 1. All errors must encode to at least 4 bytes (selector)
    assert!(encoded.len() >= 4, "Error encoded to {} bytes, minimum is 4", encoded.len());

    // 2. Determinism: encoding same error twice produces same result
    let encoded2: Vec<u8> = error.clone().into();
    assert_eq!(encoded, encoded2, "Non-deterministic error encoding");

    // 3. Selector (first 4 bytes) must be non-zero for most errors
    let selector = &encoded[0..4];
    // At least one byte should be non-zero in selector
    let has_nonzero = selector.iter().any(|&b| b != 0);
    assert!(has_nonzero, "Error selector is all zeros");

    // 4. Expected sizes for specific error types
    match error {
        Error::InvalidAlert => {
            // Simple errors: selector only (4 bytes) + potential padding
            assert!(encoded.len() >= 4, "InvalidAlert too short");
        }
        Error::UnauthorizedCaller(_) | Error::InvalidOwner(_) |
        Error::AlreadySubscribed(_) | Error::NotSubscribed(_) |
        Error::InvalidSubscriber(_) | Error::CannotRevokeOwnRole(_) |
        Error::InvalidAddress(_) => {
            // Error with single Address: selector(4) + address(32) = 36 bytes
            assert_eq!(encoded.len(), 36, "Address error should be 36 bytes, got {}", encoded.len());
        }
        Error::AlertNotFound { .. } | Error::AlertAlreadyAcknowledged { .. } |
        Error::InvalidPriorityLevel { .. } | Error::InvalidExpirationDuration { .. } |
        Error::UnsupportedInterface(_) => {
            // Error with single U256/bytes4: selector(4) + param(32) = 36 bytes
            assert_eq!(encoded.len(), 36, "Single-param error should be 36 bytes, got {}", encoded.len());
        }
        Error::InvalidRole(_) => {
            // Error with single u8: selector(4) + uint8(32) = 36 bytes
            assert_eq!(encoded.len(), 36, "InvalidRole should be 36 bytes, got {}", encoded.len());
        }
        Error::InsufficientRole { .. } | Error::NotAlertProtocol { .. } => {
            // Error with Address + u8/U256: selector(4) + address(32) + param(32) = 68 bytes
            assert_eq!(encoded.len(), 68, "Two-param error should be 68 bytes, got {}", encoded.len());
        }
        Error::BatchSizeTooLarge { .. } => {
            // Error with two U256: selector(4) + U256(32) + U256(32) = 68 bytes
            assert_eq!(encoded.len(), 68, "BatchSizeTooLarge should be 68 bytes, got {}", encoded.len());
        }
    }

    // 5. Encoded data should not contain uninitialized memory patterns
    // (all bytes should be deterministic based on input)
    let encoded3: Vec<u8> = error.into();
    assert_eq!(encoded, encoded3, "Encoding contains uninitialized data");
});
