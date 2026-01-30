#![no_main]

use libfuzzer_sys::fuzz_target;

extern crate alloc;

const ADMIN_ROLE: u8 = 0x01;
const MONITOR_ROLE: u8 = 0x02;
const ALL_ROLES: u8 = ADMIN_ROLE | MONITOR_ROLE;

/// Simulate RBAC operations
#[derive(Clone, Debug)]
struct RBACState {
    roles: u8,
}

impl RBACState {
    fn new() -> Self {
        Self { roles: 0 }
    }

    fn grant_role(&mut self, role: u8) {
        self.roles |= role;
    }

    fn revoke_role(&mut self, role: u8) {
        self.roles &= !role;
    }

    fn has_role(&self, role: u8) -> bool {
        (self.roles & role) == role
    }

    fn has_any_role(&self, role: u8) -> bool {
        (self.roles & role) != 0
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let mut rbac = RBACState::new();

    // Process random RBAC operations
    for &byte in data {
        let operation = byte % 4; // 0 = grant admin, 1 = grant monitor, 2 = revoke admin, 3 = revoke monitor
        let before_roles = rbac.roles;

        match operation {
            0 => {
                // Grant admin role
                rbac.grant_role(ADMIN_ROLE);

                // === Invariant Checks ===
                assert!(rbac.has_role(ADMIN_ROLE), "Admin role not granted");
                assert!(rbac.roles >= before_roles, "Roles decreased after grant");

                // If had monitor before, should still have it
                if (before_roles & MONITOR_ROLE) == MONITOR_ROLE {
                    assert!(rbac.has_role(MONITOR_ROLE), "Monitor role lost during admin grant");
                }
            }
            1 => {
                // Grant monitor role
                rbac.grant_role(MONITOR_ROLE);

                // === Invariant Checks ===
                assert!(rbac.has_role(MONITOR_ROLE), "Monitor role not granted");
                assert!(rbac.roles >= before_roles, "Roles decreased after grant");

                // If had admin before, should still have it
                if (before_roles & ADMIN_ROLE) == ADMIN_ROLE {
                    assert!(rbac.has_role(ADMIN_ROLE), "Admin role lost during monitor grant");
                }
            }
            2 => {
                // Revoke admin role
                let had_admin = rbac.has_role(ADMIN_ROLE);
                rbac.revoke_role(ADMIN_ROLE);

                // === Invariant Checks ===
                assert!(!rbac.has_role(ADMIN_ROLE), "Admin role not revoked");

                // If didn't have admin before, roles unchanged
                if !had_admin {
                    assert_eq!(rbac.roles, before_roles, "Roles changed when revoking absent role");
                }

                // Monitor role should be unaffected
                if (before_roles & MONITOR_ROLE) == MONITOR_ROLE {
                    assert!(rbac.has_role(MONITOR_ROLE), "Monitor role lost during admin revoke");
                }
            }
            3 => {
                // Revoke monitor role
                let had_monitor = rbac.has_role(MONITOR_ROLE);
                rbac.revoke_role(MONITOR_ROLE);

                // === Invariant Checks ===
                assert!(!rbac.has_role(MONITOR_ROLE), "Monitor role not revoked");

                // If didn't have monitor before, roles unchanged
                if !had_monitor {
                    assert_eq!(rbac.roles, before_roles, "Roles changed when revoking absent role");
                }

                // Admin role should be unaffected
                if (before_roles & ADMIN_ROLE) == ADMIN_ROLE {
                    assert!(rbac.has_role(ADMIN_ROLE), "Admin role lost during monitor revoke");
                }
            }
            _ => unreachable!(),
        }

        // === Global Invariants (after every operation) ===

        // 1. Roles should only contain valid bits
        assert_eq!(rbac.roles & !ALL_ROLES, 0,
                   "Invalid role bits set: {:#04x}", rbac.roles);

        // 2. If has role, has_role should return true
        if (rbac.roles & ADMIN_ROLE) == ADMIN_ROLE {
            assert!(rbac.has_role(ADMIN_ROLE), "has_role inconsistent with internal state");
        }
        if (rbac.roles & MONITOR_ROLE) == MONITOR_ROLE {
            assert!(rbac.has_role(MONITOR_ROLE), "has_role inconsistent with internal state");
        }

        // 3. If doesn't have role, has_role should return false
        if (rbac.roles & ADMIN_ROLE) != ADMIN_ROLE {
            assert!(!rbac.has_role(ADMIN_ROLE), "has_role false positive for admin");
        }
        if (rbac.roles & MONITOR_ROLE) != MONITOR_ROLE {
            assert!(!rbac.has_role(MONITOR_ROLE), "has_role false positive for monitor");
        }

        // 4. has_any_role semantics
        if rbac.roles != 0 {
            assert!(rbac.has_any_role(ALL_ROLES), "has_any_role should be true when roles != 0");
        }

        // 5. Granting all roles gives maximum roles
        let mut test_all = RBACState::new();
        test_all.grant_role(ADMIN_ROLE);
        test_all.grant_role(MONITOR_ROLE);
        assert_eq!(test_all.roles, ALL_ROLES, "Granting all roles doesn't equal ALL_ROLES");

        // 6. Revoking all roles gives zero roles
        let mut test_none = RBACState { roles: ALL_ROLES };
        test_none.revoke_role(ADMIN_ROLE);
        test_none.revoke_role(MONITOR_ROLE);
        assert_eq!(test_none.roles, 0, "Revoking all roles doesn't equal zero");

        // 7. Idempotence: granting twice is same as granting once
        let mut test_idempotent = RBACState { roles: before_roles };
        test_idempotent.grant_role(ADMIN_ROLE);
        test_idempotent.grant_role(ADMIN_ROLE);
        let mut test_single = RBACState { roles: before_roles };
        test_single.grant_role(ADMIN_ROLE);
        assert_eq!(test_idempotent.roles, test_single.roles,
                   "Double grant differs from single grant");

        // 8. Idempotence: revoking twice is same as revoking once
        let mut test_idempotent2 = RBACState { roles: before_roles };
        test_idempotent2.revoke_role(MONITOR_ROLE);
        test_idempotent2.revoke_role(MONITOR_ROLE);
        let mut test_single2 = RBACState { roles: before_roles };
        test_single2.revoke_role(MONITOR_ROLE);
        assert_eq!(test_idempotent2.roles, test_single2.roles,
                   "Double revoke differs from single revoke");

        // 9. Commutativity: grant order doesn't matter
        let mut test_ab = RBACState::new();
        test_ab.grant_role(ADMIN_ROLE);
        test_ab.grant_role(MONITOR_ROLE);
        let mut test_ba = RBACState::new();
        test_ba.grant_role(MONITOR_ROLE);
        test_ba.grant_role(ADMIN_ROLE);
        assert_eq!(test_ab.roles, test_ba.roles, "Grant order affects result");

        // 10. Inverse property: grant then revoke same role returns to original
        let mut test_roundtrip = RBACState { roles: before_roles };
        test_roundtrip.grant_role(ADMIN_ROLE);
        test_roundtrip.revoke_role(ADMIN_ROLE);
        // Should be back to original (if didn't have admin) or still have other roles
        let had_admin = (before_roles & ADMIN_ROLE) == ADMIN_ROLE;
        if !had_admin {
            assert_eq!(test_roundtrip.roles, before_roles,
                       "Grant+revoke didn't return to original");
        }
    }

    // === Final State Checks ===

    // Combined role check
    if rbac.has_role(ADMIN_ROLE) && rbac.has_role(MONITOR_ROLE) {
        assert_eq!(rbac.roles, ALL_ROLES, "Has both roles but doesn't equal ALL_ROLES");
    }

    // Empty role check
    if !rbac.has_role(ADMIN_ROLE) && !rbac.has_role(MONITOR_ROLE) {
        assert_eq!(rbac.roles, 0, "Has no roles but internal state is non-zero");
    }

    // Partial role check
    if rbac.has_role(ADMIN_ROLE) && !rbac.has_role(MONITOR_ROLE) {
        assert_eq!(rbac.roles, ADMIN_ROLE, "Has only admin but state doesn't match");
    }
    if !rbac.has_role(ADMIN_ROLE) && rbac.has_role(MONITOR_ROLE) {
        assert_eq!(rbac.roles, MONITOR_ROLE, "Has only monitor but state doesn't match");
    }
});
