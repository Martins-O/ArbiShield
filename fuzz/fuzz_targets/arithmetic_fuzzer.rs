#![no_main]

use libfuzzer_sys::fuzz_target;
use alloy_primitives::U256;

extern crate alloc;

fuzz_target!(|data: &[u8]| {
    // Need at least 64 bytes for two U256 values
    if data.len() < 64 {
        return;
    }

    // Extract two U256 values
    let mut a_bytes = [0u8; 32];
    let mut b_bytes = [0u8; 32];
    a_bytes.copy_from_slice(&data[0..32]);
    b_bytes.copy_from_slice(&data[32..64]);

    let a = U256::from_be_bytes(a_bytes);
    let b = U256::from_be_bytes(b_bytes);

    // === Saturating Addition Tests ===

    let sum = a.saturating_add(b);

    // 1. Result is always >= both operands
    assert!(sum >= a, "saturating_add result {} < first operand {}", sum, a);
    assert!(sum >= b, "saturating_add result {} < second operand {}", sum, b);

    // 2. Determinism
    let sum2 = a.saturating_add(b);
    assert_eq!(sum, sum2, "Non-deterministic saturating_add");

    // 3. Commutativity: a + b == b + a
    let sum_rev = b.saturating_add(a);
    assert_eq!(sum, sum_rev, "saturating_add not commutative: {} + {} != {} + {}",
               a, b, b, a);

    // 4. Identity: a + 0 == a
    assert_eq!(a.saturating_add(U256::ZERO), a, "saturating_add identity failed");
    assert_eq!(U256::ZERO.saturating_add(a), a, "saturating_add identity failed (reversed)");

    // 5. At maximum: MAX + anything == MAX
    assert_eq!(U256::MAX.saturating_add(a), U256::MAX, "MAX + a should be MAX");
    assert_eq!(a.saturating_add(U256::MAX), U256::MAX, "a + MAX should be MAX");
    assert_eq!(U256::MAX.saturating_add(U256::MAX), U256::MAX, "MAX + MAX should be MAX");

    // 6. If neither is MAX and sum is MAX, we saturated
    if a != U256::MAX && b != U256::MAX && sum == U256::MAX {
        // Should have overflowed
        assert!(a.checked_add(b).is_none(), "Sum is MAX but no overflow detected");
    }

    // 7. Associativity (if we have third value)
    if data.len() >= 96 {
        let mut c_bytes = [0u8; 32];
        c_bytes.copy_from_slice(&data[64..96]);
        let c = U256::from_be_bytes(c_bytes);

        let ab_c = a.saturating_add(b).saturating_add(c);
        let a_bc = a.saturating_add(b.saturating_add(c));
        assert_eq!(ab_c, a_bc, "saturating_add not associative");
    }

    // === Saturating Subtraction Tests ===

    let diff = a.saturating_sub(b);

    // 8. Result is always <= first operand
    assert!(diff <= a, "saturating_sub result {} > first operand {}", diff, a);

    // 9. Determinism
    let diff2 = a.saturating_sub(b);
    assert_eq!(diff, diff2, "Non-deterministic saturating_sub");

    // 10. Identity: a - 0 == a
    assert_eq!(a.saturating_sub(U256::ZERO), a, "saturating_sub identity failed");

    // 11. Annihilation: a - a == 0
    assert_eq!(a.saturating_sub(a), U256::ZERO, "a - a should be zero");
    assert_eq!(b.saturating_sub(b), U256::ZERO, "b - b should be zero");

    // 12. At zero: 0 - anything == 0
    assert_eq!(U256::ZERO.saturating_sub(a), U256::ZERO, "0 - a should be 0");
    assert_eq!(U256::ZERO.saturating_sub(b), U256::ZERO, "0 - b should be 0");
    assert_eq!(U256::ZERO.saturating_sub(U256::MAX), U256::ZERO, "0 - MAX should be 0");

    // 13. If b > a, result should be zero
    if b > a {
        assert_eq!(diff, U256::ZERO, "saturating_sub underflow didn't saturate to zero");
    }

    // 14. If b <= a, result should be a - b (exact)
    if b <= a {
        match a.checked_sub(b) {
            Some(exact) => assert_eq!(diff, exact, "saturating_sub differs from checked_sub when no underflow"),
            None => panic!("checked_sub failed when b <= a"),
        }
    }

    // === Interaction between Add and Sub ===

    // 15. (a + b) - b == a (unless saturated)
    let sum_then_sub = sum.saturating_sub(b);
    if sum != U256::MAX {
        // Didn't saturate in addition
        if a.checked_add(b).is_some() {
            assert_eq!(sum_then_sub, a, "(a + b) - b should equal a");
        }
    }

    // 16. (a - b) + b == a (unless underflowed)
    let diff_then_add = diff.saturating_add(b);
    if diff != U256::ZERO || a == U256::ZERO {
        // Didn't underflow in subtraction (or a was zero)
        if b <= a {
            assert_eq!(diff_then_add, a, "(a - b) + b should equal a");
        }
    }

    // 17. Subtraction is anti-monotonic: if a >= c and b >= d, then (a-b) <= (c-d) doesn't hold
    // But: if a >= c, then (a-b) >= (c-b) for same b
    if data.len() >= 96 {
        let mut c_bytes = [0u8; 32];
        c_bytes.copy_from_slice(&data[64..96]);
        let c = U256::from_be_bytes(c_bytes);

        if a >= c {
            let a_minus_b = a.saturating_sub(b);
            let c_minus_b = c.saturating_sub(b);
            assert!(a_minus_b >= c_minus_b,
                    "Monotonicity violated: a >= c but (a-b) < (c-b)");
        }
    }

    // 18. Addition is monotonic: if a >= c, then (a+b) >= (c+b)
    if data.len() >= 96 {
        let mut c_bytes = [0u8; 32];
        c_bytes.copy_from_slice(&data[64..96]);
        let c = U256::from_be_bytes(c_bytes);

        if a >= c {
            let a_plus_b = a.saturating_add(b);
            let c_plus_b = c.saturating_add(b);
            assert!(a_plus_b >= c_plus_b,
                    "Monotonicity violated: a >= c but (a+b) < (c+b)");
        }
    }

    // === Special Value Tests ===

    // 19. Adding 1 to anything except MAX increases value
    if a < U256::MAX {
        let a_plus_one = a.saturating_add(U256::from(1));
        assert!(a_plus_one > a, "Adding 1 didn't increase value");
        assert_eq!(a_plus_one, a + U256::from(1), "saturating_add(1) differs from regular add");
    }

    // 20. Subtracting 1 from anything except 0 decreases value
    if a > U256::ZERO {
        let a_minus_one = a.saturating_sub(U256::from(1));
        assert!(a_minus_one < a, "Subtracting 1 didn't decrease value");
        assert_eq!(a_minus_one, a - U256::from(1), "saturating_sub(1) differs from regular sub");
    }

    // 21. Increment then decrement returns to original (if not at boundaries)
    if a < U256::MAX && a > U256::ZERO {
        let incremented = a.saturating_add(U256::from(1));
        let back = incremented.saturating_sub(U256::from(1));
        assert_eq!(back, a, "Increment+decrement didn't roundtrip");
    }

    // 22. Decrement then increment returns to original (if not at boundaries)
    if a > U256::ZERO && a < U256::MAX {
        let decremented = a.saturating_sub(U256::from(1));
        let back = decremented.saturating_add(U256::from(1));
        assert_eq!(back, a, "Decrement+increment didn't roundtrip");
    }

    // 23. Double addition: a + a == 2*a (if no overflow)
    let double = a.saturating_add(a);
    if a.checked_mul(U256::from(2)).is_some() {
        assert_eq!(double, a * U256::from(2), "a + a should equal 2*a");
    }

    // 24. No operation should panic
    let _ = a.saturating_add(U256::MAX);
    let _ = U256::MAX.saturating_add(U256::MAX);
    let _ = a.saturating_sub(U256::MAX);
    let _ = U256::ZERO.saturating_sub(U256::MAX);
    let _ = U256::MAX.saturating_sub(U256::ZERO);
});
