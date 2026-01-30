#![no_main]

use libfuzzer_sys::fuzz_target;
use alloy_primitives::{Address, U256};
use arbishield::alert_registry::error::Error as ARError;
use arbishield::circuit_breaker::error::Error as CBError;
use arbishield::detection_engine::error::Error as DEError;
use alloc::vec::Vec;

extern crate alloc;

fuzz_target!(|data: &[u8]| {
    // Need at least 20 bytes for an address
    if data.len() < 20 {
        return;
    }

    // Extract address from data
    let mut addr_bytes = [0u8; 20];
    addr_bytes.copy_from_slice(&data[0..20]);
    let addr = Address::from(addr_bytes);

    // Create shared error type across contracts: UnauthorizedCaller
    let cb_error: Vec<u8> = CBError::UnauthorizedCaller(addr).into();
    let de_error: Vec<u8> = DEError::UnauthorizedCaller(addr).into();
    let ar_error: Vec<u8> = ARError::UnauthorizedCaller(addr).into();

    // === Cross-Contract Invariants ===

    // 1. Same error signature produces same selector across contracts
    assert_eq!(&cb_error[0..4], &de_error[0..4],
               "CB and DE UnauthorizedCaller selectors differ");
    assert_eq!(&de_error[0..4], &ar_error[0..4],
               "DE and AR UnauthorizedCaller selectors differ");

    // 2. Same error with same data produces identical full encoding
    assert_eq!(cb_error, de_error,
               "CB and DE UnauthorizedCaller full encoding differs");
    assert_eq!(de_error, ar_error,
               "DE and AR UnauthorizedCaller full encoding differs");

    // 3. All encodings should be 36 bytes (selector + address)
    assert_eq!(cb_error.len(), 36, "CB error wrong size");
    assert_eq!(de_error.len(), 36, "DE error wrong size");
    assert_eq!(ar_error.len(), 36, "AR error wrong size");

    // 4. Address should be preserved in encoding (at offset 16-36)
    let encoded_addr = &cb_error[16..36];
    assert_eq!(encoded_addr, addr.as_slice(),
               "Address not preserved in encoding");

    // Test InvalidOwner as well if we have data
    if data.len() >= 40 {
        let mut owner_bytes = [0u8; 20];
        owner_bytes.copy_from_slice(&data[20..40]);
        let owner = Address::from(owner_bytes);

        let cb_owner: Vec<u8> = CBError::InvalidOwner(owner).into();
        let de_owner: Vec<u8> = DEError::InvalidOwner(owner).into();
        let ar_owner: Vec<u8> = ARError::InvalidOwner(owner).into();

        // 5. InvalidOwner should also have consistent selectors
        assert_eq!(&cb_owner[0..4], &de_owner[0..4],
                   "InvalidOwner selectors differ");
        assert_eq!(&de_owner[0..4], &ar_owner[0..4],
                   "InvalidOwner selectors differ");

        // 6. InvalidOwner and UnauthorizedCaller should have different selectors
        assert_ne!(&cb_error[0..4], &cb_owner[0..4],
                   "Different errors have same selector");

        // 7. Same size for InvalidOwner
        assert_eq!(cb_owner.len(), 36, "InvalidOwner wrong size");
        assert_eq!(de_owner.len(), 36, "InvalidOwner wrong size");
        assert_eq!(ar_owner.len(), 36, "InvalidOwner wrong size");
    }

    // Test with U256 errors if we have data
    if data.len() >= 52 {
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(&data[20..52]);
        let id = U256::from_be_bytes(id_bytes);

        // DE has MetricNotFound, AR has AlertNotFound
        let de_not_found: Vec<u8> = DEError::MetricNotFound { id }.into();
        let ar_not_found: Vec<u8> = ARError::AlertNotFound { id }.into();

        // 8. Different "NotFound" errors should have different selectors
        assert_ne!(&de_not_found[0..4], &ar_not_found[0..4],
                   "MetricNotFound and AlertNotFound have same selector");

        // 9. Both should be 36 bytes
        assert_eq!(de_not_found.len(), 36, "MetricNotFound wrong size");
        assert_eq!(ar_not_found.len(), 36, "AlertNotFound wrong size");

        // 10. Determinism check
        let de_not_found2: Vec<u8> = DEError::MetricNotFound { id }.into();
        let ar_not_found2: Vec<u8> = ARError::AlertNotFound { id }.into();
        assert_eq!(de_not_found, de_not_found2, "DE MetricNotFound non-deterministic");
        assert_eq!(ar_not_found, ar_not_found2, "AR AlertNotFound non-deterministic");
    }

    // Test selector collision resistance
    // With random data, selectors should be distributed
    let selector_u32 = u32::from_be_bytes([
        cb_error[0], cb_error[1], cb_error[2], cb_error[3]
    ]);

    // 11. Selector should not be all zeros (very unlikely for Solidity errors)
    assert_ne!(selector_u32, 0, "Selector is all zeros");

    // 12. Selector should not be all ones (very unlikely)
    assert_ne!(selector_u32, 0xFFFFFFFF, "Selector is all ones");
});
