#![no_main]

use libfuzzer_sys::fuzz_target;
use alloy_primitives::U256;
use arbishield::alert_registry::storage::AlertRegistry;

fuzz_target!(|data: &[u8]| {
    // Skip if input is too small
    if data.len() < 32 {
        return;
    }

    // Parse bytes into U256 threat level
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&data[0..32]);
    let threat_level = U256::from_be_bytes(bytes);

    // Compute priority
    let priority = AlertRegistry::compute_priority(threat_level);
    let priority_u64 = priority.saturating_to::<u64>();

    // === Invariant Checks ===

    // 1. Priority must always be 0, 1, 2, or 3
    assert!(priority_u64 <= 3, "Priority {} exceeds maximum 3", priority_u64);

    // 2. Monotonicity: higher threat should never produce lower priority
    let threat_u64 = threat_level.saturating_to::<u64>();
    if threat_u64 < 40 {
        assert_eq!(priority_u64, 0, "Threat {} should be LOW (0), got {}", threat_u64, priority_u64);
    } else if threat_u64 < 70 {
        assert_eq!(priority_u64, 1, "Threat {} should be MEDIUM (1), got {}", threat_u64, priority_u64);
    } else if threat_u64 < 90 {
        assert_eq!(priority_u64, 2, "Threat {} should be HIGH (2), got {}", threat_u64, priority_u64);
    } else {
        assert_eq!(priority_u64, 3, "Threat {} should be CRITICAL (3), got {}", threat_u64, priority_u64);
    }

    // 3. Determinism: same input always produces same output
    let priority2 = AlertRegistry::compute_priority(threat_level);
    assert_eq!(priority, priority2, "Non-deterministic priority computation");

    // 4. Very large threat levels should be CRITICAL
    if threat_level > U256::from(100u64) {
        assert_eq!(priority_u64, 3, "Large threat {} should be CRITICAL", threat_level);
    }

    // 5. Zero threat should be LOW
    if threat_level.is_zero() {
        assert_eq!(priority_u64, 0, "Zero threat should be LOW");
    }

    // 6. U256::MAX should be CRITICAL
    if threat_level == U256::MAX {
        assert_eq!(priority_u64, 3, "MAX threat should be CRITICAL");
    }
});
