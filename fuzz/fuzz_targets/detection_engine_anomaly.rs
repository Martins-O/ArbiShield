#![no_main]

use libfuzzer_sys::fuzz_target;
use alloy_primitives::U256;

extern crate alloc;

/// Check if value exceeds threshold (anomaly detection logic)
fn is_anomaly(value: U256, threshold: U256) -> bool {
    value > threshold
}

fuzz_target!(|data: &[u8]| {
    // Need at least 64 bytes for value and threshold
    if data.len() < 64 {
        return;
    }

    // Extract value and threshold
    let mut value_bytes = [0u8; 32];
    let mut threshold_bytes = [0u8; 32];
    value_bytes.copy_from_slice(&data[0..32]);
    threshold_bytes.copy_from_slice(&data[32..64]);

    let value = U256::from_be_bytes(value_bytes);
    let threshold = U256::from_be_bytes(threshold_bytes);

    let result = is_anomaly(value, threshold);

    // === Invariant Checks ===

    // 1. Determinism: same inputs produce same output
    let result2 = is_anomaly(value, threshold);
    assert_eq!(result, result2, "Non-deterministic anomaly detection");

    // 2. Antisymmetry: if value > threshold, then threshold !> value
    if result {
        assert!(!is_anomaly(threshold, value) || threshold == value,
                "Antisymmetry violated: both value > threshold and threshold > value");
    }

    // 3. Transitivity: if a > b and b > c, then a > c
    if data.len() >= 96 {
        let mut c_bytes = [0u8; 32];
        c_bytes.copy_from_slice(&data[64..96]);
        let c = U256::from_be_bytes(c_bytes);

        if is_anomaly(value, threshold) && is_anomaly(threshold, c) {
            assert!(is_anomaly(value, c),
                    "Transitivity violated: value > threshold > c but value !> c");
        }
    }

    // 4. Boundary: value == threshold should NOT be anomaly
    if value == threshold {
        assert!(!result, "Equal values should not be anomaly");
    }

    // 5. Boundary: value == threshold + 1 SHOULD be anomaly
    if threshold < U256::MAX {
        let just_over = threshold.saturating_add(U256::from(1));
        assert!(is_anomaly(just_over, threshold),
                "Value exactly 1 over threshold should be anomaly");
    }

    // 6. Zero threshold: any positive value is anomaly
    if threshold.is_zero() && !value.is_zero() {
        assert!(result, "Positive value with zero threshold should be anomaly");
    }

    // 7. Zero threshold: zero value is NOT anomaly
    if threshold.is_zero() && value.is_zero() {
        assert!(!result, "Zero value with zero threshold should not be anomaly");
    }

    // 8. MAX threshold: nothing can exceed it
    if threshold == U256::MAX {
        assert!(!result, "Nothing can exceed U256::MAX");
    }

    // 9. MAX value: exceeds everything except MAX
    if value == U256::MAX && threshold < U256::MAX {
        assert!(result, "U256::MAX should exceed any smaller threshold");
    }

    // 10. Irreflexivity: value > value is always false
    assert!(!is_anomaly(value, value), "Value cannot be anomaly against itself");
    assert!(!is_anomaly(threshold, threshold), "Threshold cannot be anomaly against itself");

    // 11. Monotonicity: if value increases, anomaly likelihood increases
    if data.len() >= 96 {
        let mut larger_bytes = [0u8; 32];
        larger_bytes.copy_from_slice(&data[64..96]);
        let mut larger_value = U256::from_be_bytes(larger_bytes);

        // Ensure larger_value > value
        if larger_value <= value {
            larger_value = value.saturating_add(U256::from(1));
        }

        if larger_value > value {
            // If value is anomaly, larger_value must also be anomaly
            if is_anomaly(value, threshold) {
                assert!(is_anomaly(larger_value, threshold),
                        "Monotonicity violated: {} is anomaly but larger {} is not",
                        value, larger_value);
            }
        }
    }

    // 12. Commutativity with constant: (v + c) > (t + c) iff v > t
    // (but only if no overflow)
    if data.len() >= 96 {
        let mut constant_bytes = [0u8; 32];
        constant_bytes.copy_from_slice(&data[64..96]);
        let constant = U256::from_be_bytes(constant_bytes);

        // Check if adding constant would overflow
        let value_overflow = value.checked_add(constant).is_none();
        let threshold_overflow = threshold.checked_add(constant).is_none();

        if !value_overflow && !threshold_overflow {
            let shifted_value = value.saturating_add(constant);
            let shifted_threshold = threshold.saturating_add(constant);

            let original_result = is_anomaly(value, threshold);
            let shifted_result = is_anomaly(shifted_value, shifted_threshold);

            assert_eq!(original_result, shifted_result,
                       "Shifting both value and threshold by same amount changed result");
        }
    }

    // 13. Boolean result: must be exactly true or false
    assert!(result == true || result == false, "Result must be boolean");

    // 14. Equivalence: result matches direct comparison
    assert_eq!(result, value > threshold,
               "Anomaly detection result doesn't match direct comparison");
});
