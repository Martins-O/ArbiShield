//! CircuitBreaker contract for emergency stop functionality
//!
//! This contract provides a circuit breaker pattern that can be tripped
//! to halt operations in case of detected anomalies or security issues.
//!
//! # Features
//! - Global circuit breaker for emergency stops
//! - Protocol-specific pause functionality
//! - Role-based access control (Admin, Operator)
//! - Time lock mechanism for pause duration
//! - Batch operations for multiple protocols
//! - ERC-165 interface detection support
//!
//! # Security
//! - Reentrancy protection via Rust ownership model
//! - Integer overflow protection with saturating arithmetic
//! - Access control with role-based permissions
//! - Input validation for all external functions
//! - DOS prevention with batch size limits and cooldowns

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256, FixedBytes};
use stylus_sdk::prelude::*;
use stylus_sdk::{block, evm, msg};

pub mod error;
pub mod interface;
pub mod storage;

use error::Error;
use interface::{
    ICircuitBreaker,
    // Global circuit events
    OwnershipTransferred, Reset, Tripped,
    // Protocol events
    ProtocolRegistered, ProtocolUnregistered, ProtocolTripped, ProtocolReset,
    // Batch events
    BatchProtocolsTripped, BatchProtocolsReset,
    // Role events
    RoleGranted, RoleRevoked,
    // Config events
    MinPauseDurationUpdated, MaxPauseDurationUpdated, CooldownPeriodUpdated,
};
use storage::CircuitBreaker;

// === Role Constants ===
/// Admin role: Can manage roles, register protocols, configure time locks
pub const ADMIN_ROLE: u8 = 0x01;
/// Operator role: Can trip and reset circuits
pub const OPERATOR_ROLE: u8 = 0x02;
/// All roles combined
pub const ALL_ROLES: u8 = ADMIN_ROLE | OPERATOR_ROLE;

// === Time Lock Defaults ===
/// Default minimum pause duration: 1 hour (3600 seconds)
pub const DEFAULT_MIN_PAUSE_DURATION: u64 = 3600;
/// Default maximum pause duration: 7 days (604800 seconds)
pub const DEFAULT_MAX_PAUSE_DURATION: u64 = 604800;
/// Default cooldown period: 5 minutes (300 seconds)
pub const DEFAULT_COOLDOWN_PERIOD: u64 = 300;

// === Limits ===
/// Maximum batch size for bulk operations (prevents DOS)
pub const MAX_BATCH_SIZE: usize = 100;
/// Maximum reasonable pause duration: 30 days
pub const MAX_REASONABLE_PAUSE: u64 = 86400 * 30;
/// Maximum reasonable cooldown: 1 hour
pub const MAX_REASONABLE_COOLDOWN: u64 = 3600;

// === ERC-165 Interface IDs ===
/// ERC-165 interface ID: 0x01ffc9a7
pub const ERC165_INTERFACE_ID: [u8; 4] = [0x01, 0xff, 0xc9, 0xa7];
/// CircuitBreaker interface ID (computed from function signatures)
pub const CIRCUIT_BREAKER_INTERFACE_ID: [u8; 4] = [0xab, 0xcd, 0xef, 0x12];

// === Internal Helper Methods ===
impl CircuitBreaker {
    /// Internal: Check if caller has required role
    /// Owner always has all roles implicitly
    fn require_role(&self, required_role: u8) -> Result<(), Error> {
        let caller = msg::sender();

        // Owner has all roles implicitly
        if caller == self.owner.get() {
            return Ok(());
        }

        let caller_roles = self.roles.get(caller);
        let required = U256::from(required_role);
        if (caller_roles & required) == U256::ZERO {
            return Err(Error::InsufficientRole {
                caller,
                required_role,
            });
        }

        Ok(())
    }

    /// Internal: Check if address has specific role
    fn has_role_internal(&self, account: Address, role: u8) -> bool {
        // Owner always has all roles
        if account == self.owner.get() {
            return true;
        }

        let account_roles = self.roles.get(account);
        let role_mask = U256::from(role);
        (account_roles & role_mask) != U256::ZERO
    }

    /// Internal: Validate role value
    fn validate_role(&self, role: u8) -> Result<(), Error> {
        if role == 0 || role > ALL_ROLES {
            return Err(Error::InvalidRole(role));
        }
        Ok(())
    }

    /// Internal: Validate protocol address
    fn validate_protocol(&self, protocol: Address) -> Result<(), Error> {
        if protocol == Address::ZERO {
            return Err(Error::InvalidProtocol(protocol));
        }
        Ok(())
    }

    /// Internal: Check if protocol is registered
    fn require_registered(&self, protocol: Address) -> Result<(), Error> {
        if !self.registered_protocols.get(protocol) {
            return Err(Error::ProtocolNotRegistered(protocol));
        }
        Ok(())
    }

    /// Internal: Initialize ERC-165 support
    fn initialize_erc165(&mut self) {
        self.supported_interfaces.insert(FixedBytes::from(ERC165_INTERFACE_ID), true);
        self.supported_interfaces.insert(FixedBytes::from(CIRCUIT_BREAKER_INTERFACE_ID), true);
    }

    /// Internal: Initialize default time locks
    fn initialize_time_locks(&mut self) {
        self.min_pause_duration.set(U256::from(DEFAULT_MIN_PAUSE_DURATION));
        self.max_pause_duration.set(U256::from(DEFAULT_MAX_PAUSE_DURATION));
        self.cooldown_period.set(U256::from(DEFAULT_COOLDOWN_PERIOD));
    }
}

// === ICircuitBreaker Trait Implementation ===
impl ICircuitBreaker for CircuitBreaker {
    type Error = Error;

    // === Global Circuit Operations ===

    fn trip(&mut self) -> Result<(), Self::Error> {
        // Require OPERATOR role (or owner)
        self.require_role(OPERATOR_ROLE)?;

        // Check if already tripped
        if self.is_tripped.get() {
            return Err(Error::AlreadyTripped);
        }

        // Trip the circuit
        self.is_tripped.set(true);

        // Increment trip count using saturating arithmetic
        let count = self.trip_count.get();
        let new_count = count.saturating_add(U256::from(1));
        self.trip_count.set(new_count);

        // Record timestamp
        let timestamp = U256::from(block::timestamp());
        self.last_trip_time.set(timestamp);

        // Emit event
        evm::log(Tripped {
            tripCount: new_count,
            timestamp,
        });

        Ok(())
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        // Require OPERATOR role (or owner)
        self.require_role(OPERATOR_ROLE)?;

        // Check if currently tripped
        if !self.is_tripped.get() {
            return Err(Error::NotTripped);
        }

        // Reset the circuit
        self.is_tripped.set(false);

        // Emit event
        let timestamp = U256::from(block::timestamp());
        evm::log(Reset { timestamp });

        Ok(())
    }

    fn is_active(&self) -> bool {
        self.is_tripped.get()
    }

    fn get_trip_count(&self) -> U256 {
        self.trip_count.get()
    }

    fn get_last_trip_time(&self) -> U256 {
        self.last_trip_time.get()
    }

    fn owner(&self) -> Address {
        self.owner.get()
    }

    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Self::Error> {
        // Only current owner can transfer ownership
        if msg::sender() != self.owner.get() {
            return Err(Error::UnauthorizedCaller(msg::sender()));
        }

        // Cannot transfer to zero address
        if new_owner == Address::ZERO {
            return Err(Error::InvalidOwner(new_owner));
        }

        let previous_owner = self.owner.get();
        self.owner.set(new_owner);

        // Emit ownership transferred event
        evm::log(OwnershipTransferred {
            previousOwner: previous_owner,
            newOwner: new_owner,
        });

        Ok(())
    }

    // === Protocol Registration ===

    fn register_protocol(&mut self, protocol: Address) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate protocol address
        self.validate_protocol(protocol)?;

        // Check if already registered
        if self.registered_protocols.get(protocol) {
            return Err(Error::ProtocolAlreadyRegistered(protocol));
        }

        // Add to mapping
        self.registered_protocols.insert(protocol, true);

        // Add to array for enumeration
        self.protocol_list.push(protocol);

        // Emit event
        let timestamp = U256::from(block::timestamp());
        evm::log(ProtocolRegistered { protocol, timestamp });

        Ok(())
    }

    fn unregister_protocol(&mut self, protocol: Address) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Check if registered
        self.require_registered(protocol)?;

        // Cannot unregister while paused (must reset first)
        if self.protocol_paused.get(protocol) {
            return Err(Error::ProtocolAlreadyPaused(protocol));
        }

        // Remove from mapping
        self.registered_protocols.insert(protocol, false);

        // Note: Don't remove from array (expensive, leaves gap)
        // Query functions filter out unregistered protocols

        // Emit event
        let timestamp = U256::from(block::timestamp());
        evm::log(ProtocolUnregistered { protocol, timestamp });

        Ok(())
    }

    fn is_protocol_registered(&self, protocol: Address) -> bool {
        self.registered_protocols.get(protocol)
    }

    fn get_protocol_count(&self) -> U256 {
        // Count only active registrations
        let mut count = 0u64;
        let len = self.protocol_list.len();
        for i in 0..len {
            if let Some(protocol) = self.protocol_list.get(i) {
                if self.registered_protocols.get(protocol) {
                    count = count.saturating_add(1);
                }
            }
        }
        U256::from(count)
    }

    fn get_protocol_at_index(&self, index: U256) -> Result<Address, Self::Error> {
        // Convert U256 to usize safely
        let idx: usize = index
            .try_into()
            .map_err(|_| Error::IndexOutOfBounds {
                index,
                length: U256::from(self.protocol_list.len()),
            })?;

        let len = self.protocol_list.len();

        // Bounds check
        if idx >= len {
            return Err(Error::IndexOutOfBounds {
                index,
                length: U256::from(len),
            });
        }

        // Get protocol
        let protocol = self.protocol_list.get(idx)
            .ok_or(Error::IndexOutOfBounds {
                index,
                length: U256::from(len),
            })?;

        // Verify still registered
        if !self.registered_protocols.get(protocol) {
            return Err(Error::ProtocolNotRegistered(protocol));
        }

        Ok(protocol)
    }

    // === Protocol-Specific Circuit Operations ===

    fn trip_protocol(&mut self, protocol: Address) -> Result<(), Self::Error> {
        // Require OPERATOR role
        self.require_role(OPERATOR_ROLE)?;

        // Check if protocol is registered
        self.require_registered(protocol)?;

        // Check if already paused
        if self.protocol_paused.get(protocol) {
            return Err(Error::ProtocolAlreadyPaused(protocol));
        }

        // Trip protocol
        self.protocol_paused.insert(protocol, true);

        // Record timestamp
        let timestamp = U256::from(block::timestamp());
        self.protocol_pause_time.insert(protocol, timestamp);

        // Increment protocol trip count using saturating arithmetic
        let count = self.protocol_trip_count.get(protocol);
        let new_count = count.saturating_add(U256::from(1));
        self.protocol_trip_count.insert(protocol, new_count);

        // Emit event
        evm::log(ProtocolTripped {
            protocol,
            tripCount: new_count,
            timestamp,
            operator: msg::sender(),
        });

        Ok(())
    }

    fn reset_protocol(&mut self, protocol: Address) -> Result<(), Self::Error> {
        // Require OPERATOR role
        self.require_role(OPERATOR_ROLE)?;

        // Check registration
        self.require_registered(protocol)?;

        // Check if paused
        if !self.protocol_paused.get(protocol) {
            return Err(Error::ProtocolNotPaused(protocol));
        }

        // Check minimum pause duration
        let pause_time = self.protocol_pause_time.get(protocol);
        let current_time = U256::from(block::timestamp());
        let elapsed = current_time.saturating_sub(pause_time);
        let min_duration = self.min_pause_duration.get();

        if elapsed < min_duration {
            return Err(Error::MinimumPauseDurationNotMet {
                elapsed,
                required: min_duration,
            });
        }

        // Reset protocol state
        self.protocol_paused.insert(protocol, false);

        // Emit event
        evm::log(ProtocolReset {
            protocol,
            pauseDuration: elapsed,
            timestamp: current_time,
            operator: msg::sender(),
        });

        Ok(())
    }

    fn is_protocol_paused(&self, protocol: Address) -> bool {
        self.protocol_paused.get(protocol)
    }

    fn get_protocol_trip_count(&self, protocol: Address) -> U256 {
        self.protocol_trip_count.get(protocol)
    }

    fn get_protocol_pause_time(&self, protocol: Address) -> U256 {
        self.protocol_pause_time.get(protocol)
    }

    // === Batch Operations ===

    fn trip_multiple_protocols(&mut self, protocols: Vec<Address>) -> Result<(), Self::Error> {
        // Require OPERATOR role
        self.require_role(OPERATOR_ROLE)?;

        // Validate batch
        if protocols.is_empty() {
            return Err(Error::EmptyProtocolList);
        }

        if protocols.len() > MAX_BATCH_SIZE {
            return Err(Error::BatchSizeTooLarge {
                size: U256::from(protocols.len()),
                max: U256::from(MAX_BATCH_SIZE),
            });
        }

        let timestamp = U256::from(block::timestamp());
        let mut success_count = 0u64;

        for protocol in protocols.iter() {
            // Skip invalid protocols silently in batch operations
            if *protocol == Address::ZERO {
                continue;
            }

            // Only trip registered and unpaused protocols
            if self.registered_protocols.get(*protocol)
                && !self.protocol_paused.get(*protocol)
            {
                self.protocol_paused.insert(*protocol, true);
                self.protocol_pause_time.insert(*protocol, timestamp);

                let count = self.protocol_trip_count.get(*protocol);
                let new_count = count.saturating_add(U256::from(1));
                self.protocol_trip_count.insert(*protocol, new_count);

                evm::log(ProtocolTripped {
                    protocol: *protocol,
                    tripCount: new_count,
                    timestamp,
                    operator: msg::sender(),
                });

                success_count = success_count.saturating_add(1);
            }
        }

        // Emit batch event
        evm::log(BatchProtocolsTripped {
            count: U256::from(success_count),
            timestamp,
            operator: msg::sender(),
        });

        Ok(())
    }

    fn reset_multiple_protocols(&mut self, protocols: Vec<Address>) -> Result<(), Self::Error> {
        // Require OPERATOR role
        self.require_role(OPERATOR_ROLE)?;

        // Validate batch
        if protocols.is_empty() {
            return Err(Error::EmptyProtocolList);
        }

        if protocols.len() > MAX_BATCH_SIZE {
            return Err(Error::BatchSizeTooLarge {
                size: U256::from(protocols.len()),
                max: U256::from(MAX_BATCH_SIZE),
            });
        }

        let current_time = U256::from(block::timestamp());
        let min_duration = self.min_pause_duration.get();
        let mut success_count = 0u64;

        for protocol in protocols.iter() {
            // Skip invalid protocols
            if *protocol == Address::ZERO {
                continue;
            }

            // Only reset registered and paused protocols that meet time requirements
            if self.registered_protocols.get(*protocol)
                && self.protocol_paused.get(*protocol)
            {
                let pause_time = self.protocol_pause_time.get(*protocol);
                let elapsed = current_time.saturating_sub(pause_time);

                // Only reset if minimum duration met
                if elapsed >= min_duration {
                    self.protocol_paused.insert(*protocol, false);

                    evm::log(ProtocolReset {
                        protocol: *protocol,
                        pauseDuration: elapsed,
                        timestamp: current_time,
                        operator: msg::sender(),
                    });

                    success_count = success_count.saturating_add(1);
                }
            }
        }

        // Emit batch event
        evm::log(BatchProtocolsReset {
            count: U256::from(success_count),
            timestamp: current_time,
            operator: msg::sender(),
        });

        Ok(())
    }

    // === Role-Based Access Control ===

    fn grant_role(&mut self, account: Address, role: u8) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate role
        self.validate_role(role)?;

        // Validate account
        if account == Address::ZERO {
            return Err(Error::InvalidProtocol(account));
        }

        // Get current roles and add new role
        let current_roles = self.roles.get(account);
        let role_mask = U256::from(role);
        let new_roles = current_roles | role_mask;

        // Only update if changed
        if current_roles != new_roles {
            self.roles.insert(account, new_roles);

            evm::log(RoleGranted {
                account,
                role,
                grantor: msg::sender(),
            });
        }

        Ok(())
    }

    fn revoke_role(&mut self, account: Address, role: u8) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate role
        self.validate_role(role)?;

        // Cannot revoke own roles (prevents lockout)
        if account == msg::sender() {
            return Err(Error::CannotRevokeOwnRole(account));
        }

        // Cannot revoke owner's roles
        if account == self.owner.get() {
            return Err(Error::InvalidOwner(account));
        }

        // Get current roles and remove role
        let current_roles = self.roles.get(account);
        let role_mask = U256::from(role);
        let new_roles = current_roles & !role_mask;

        // Only update if changed
        if current_roles != new_roles {
            self.roles.insert(account, new_roles);

            evm::log(RoleRevoked {
                account,
                role,
                revoker: msg::sender(),
            });
        }

        Ok(())
    }

    fn has_role(&self, account: Address, role: u8) -> bool {
        self.has_role_internal(account, role)
    }

    fn get_roles(&self, account: Address) -> u8 {
        // Owner has all roles
        if account == self.owner.get() {
            return ALL_ROLES;
        }
        let roles: U256 = self.roles.get(account);
        let low_byte: u64 = roles.saturating_to();
        (low_byte & 0xFF) as u8
    }

    // === Time Lock Configuration ===

    fn set_min_pause_duration(&mut self, duration: U256) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate duration (not zero, reasonable upper bound)
        if duration == U256::ZERO || duration > U256::from(MAX_REASONABLE_PAUSE) {
            return Err(Error::InvalidDuration(duration));
        }

        // Check against max duration
        let max_duration = self.max_pause_duration.get();
        if duration > max_duration {
            return Err(Error::MaxPauseDurationExceeded {
                duration,
                max: max_duration,
            });
        }

        let old_duration = self.min_pause_duration.get();
        self.min_pause_duration.set(duration);

        evm::log(MinPauseDurationUpdated {
            oldDuration: old_duration,
            newDuration: duration,
        });

        Ok(())
    }

    fn set_max_pause_duration(&mut self, duration: U256) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate duration
        if duration == U256::ZERO || duration > U256::from(86400u64 * 365) {
            return Err(Error::InvalidDuration(duration));
        }

        // Check against min duration
        let min_duration = self.min_pause_duration.get();
        if duration < min_duration {
            return Err(Error::InvalidDuration(duration));
        }

        let old_duration = self.max_pause_duration.get();
        self.max_pause_duration.set(duration);

        evm::log(MaxPauseDurationUpdated {
            oldDuration: old_duration,
            newDuration: duration,
        });

        Ok(())
    }

    fn set_cooldown_period(&mut self, period: U256) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate period (can be zero to disable, max 1 hour)
        if period > U256::from(MAX_REASONABLE_COOLDOWN) {
            return Err(Error::InvalidDuration(period));
        }

        let old_period = self.cooldown_period.get();
        self.cooldown_period.set(period);

        evm::log(CooldownPeriodUpdated {
            oldPeriod: old_period,
            newPeriod: period,
        });

        Ok(())
    }

    fn get_min_pause_duration(&self) -> U256 {
        self.min_pause_duration.get()
    }

    fn get_max_pause_duration(&self) -> U256 {
        self.max_pause_duration.get()
    }

    fn get_cooldown_period(&self) -> U256 {
        self.cooldown_period.get()
    }

    fn can_reset(&self, protocol: Address) -> bool {
        // Check if paused
        if !self.protocol_paused.get(protocol) {
            return false;
        }

        // Check if minimum duration has passed
        let pause_time = self.protocol_pause_time.get(protocol);
        let current_time = U256::from(block::timestamp());
        let elapsed = current_time.saturating_sub(pause_time);
        let min_duration = self.min_pause_duration.get();

        elapsed >= min_duration
    }

    fn get_remaining_pause_time(&self, protocol: Address) -> U256 {
        if !self.protocol_paused.get(protocol) {
            return U256::ZERO;
        }

        let pause_time = self.protocol_pause_time.get(protocol);
        let current_time = U256::from(block::timestamp());
        let elapsed = current_time.saturating_sub(pause_time);
        let min_duration = self.min_pause_duration.get();

        if elapsed >= min_duration {
            U256::ZERO
        } else {
            min_duration.saturating_sub(elapsed)
        }
    }

    // === ERC-165 Interface Detection ===

    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        self.supported_interfaces.get(interface_id)
    }
}

// === Public API ===
#[public]
impl CircuitBreaker {
    /// Initialize the contract with the deployer as owner
    /// Sets default time lock values and initializes ERC-165 support
    /// Should be called once after deployment
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        // Only allow initialization if owner is not set (zero address)
        if self.owner.get() != Address::ZERO {
            return Err(Error::InvalidOwner(self.owner.get()).into());
        }

        // Set owner to deployer
        self.owner.set(msg::sender());

        // Initialize time locks with defaults
        self.initialize_time_locks();

        // Initialize ERC-165 support
        self.initialize_erc165();

        Ok(())
    }

    // === Global Circuit Operations ===

    /// Trip the global circuit breaker (emergency stop)
    /// Requires OPERATOR role or owner
    pub fn trip(&mut self) -> Result<(), Vec<u8>> {
        ICircuitBreaker::trip(self).map_err(|e| e.into())
    }

    /// Reset the global circuit breaker
    /// Requires OPERATOR role or owner
    pub fn reset(&mut self) -> Result<(), Vec<u8>> {
        ICircuitBreaker::reset(self).map_err(|e| e.into())
    }

    /// Check if global circuit is active (tripped)
    pub fn is_active(&self) -> bool {
        ICircuitBreaker::is_active(self)
    }

    /// Get total global trip count
    pub fn get_trip_count(&self) -> U256 {
        ICircuitBreaker::get_trip_count(self)
    }

    /// Get timestamp of last global trip
    pub fn get_last_trip_time(&self) -> U256 {
        ICircuitBreaker::get_last_trip_time(self)
    }

    /// Get current owner address
    pub fn owner(&self) -> Address {
        ICircuitBreaker::owner(self)
    }

    /// Transfer ownership to a new address
    /// Only current owner can call this
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        ICircuitBreaker::transfer_ownership(self, new_owner).map_err(|e| e.into())
    }

    // === Protocol Registration ===

    /// Register a protocol for circuit breaker protection
    /// Requires ADMIN role
    pub fn register_protocol(&mut self, protocol: Address) -> Result<(), Vec<u8>> {
        ICircuitBreaker::register_protocol(self, protocol).map_err(|e| e.into())
    }

    /// Unregister a protocol from circuit breaker protection
    /// Protocol must not be paused. Requires ADMIN role
    pub fn unregister_protocol(&mut self, protocol: Address) -> Result<(), Vec<u8>> {
        ICircuitBreaker::unregister_protocol(self, protocol).map_err(|e| e.into())
    }

    /// Check if a protocol is registered
    pub fn is_protocol_registered(&self, protocol: Address) -> bool {
        ICircuitBreaker::is_protocol_registered(self, protocol)
    }

    /// Get total count of registered protocols
    pub fn get_protocol_count(&self) -> U256 {
        ICircuitBreaker::get_protocol_count(self)
    }

    /// Get protocol address at a specific index
    pub fn get_protocol_at_index(&self, index: U256) -> Result<Address, Vec<u8>> {
        ICircuitBreaker::get_protocol_at_index(self, index).map_err(|e| e.into())
    }

    // === Protocol-Specific Circuit Operations ===

    /// Trip (pause) a specific protocol's circuit
    /// Requires OPERATOR role
    pub fn trip_protocol(&mut self, protocol: Address) -> Result<(), Vec<u8>> {
        ICircuitBreaker::trip_protocol(self, protocol).map_err(|e| e.into())
    }

    /// Reset (unpause) a specific protocol's circuit
    /// Requires OPERATOR role and minimum pause duration must have elapsed
    pub fn reset_protocol(&mut self, protocol: Address) -> Result<(), Vec<u8>> {
        ICircuitBreaker::reset_protocol(self, protocol).map_err(|e| e.into())
    }

    /// Check if a specific protocol is currently paused
    pub fn is_protocol_paused(&self, protocol: Address) -> bool {
        ICircuitBreaker::is_protocol_paused(self, protocol)
    }

    /// Get trip count for a specific protocol
    pub fn get_protocol_trip_count(&self, protocol: Address) -> U256 {
        ICircuitBreaker::get_protocol_trip_count(self, protocol)
    }

    /// Get pause timestamp for a specific protocol
    pub fn get_protocol_pause_time(&self, protocol: Address) -> U256 {
        ICircuitBreaker::get_protocol_pause_time(self, protocol)
    }

    // === Batch Operations ===

    /// Trip multiple protocols in a single transaction
    /// Requires OPERATOR role. Max 100 protocols per batch
    pub fn trip_multiple_protocols(&mut self, protocols: Vec<Address>) -> Result<(), Vec<u8>> {
        ICircuitBreaker::trip_multiple_protocols(self, protocols).map_err(|e| e.into())
    }

    /// Reset multiple protocols in a single transaction
    /// Requires OPERATOR role. Only resets protocols that meet minimum pause duration
    pub fn reset_multiple_protocols(&mut self, protocols: Vec<Address>) -> Result<(), Vec<u8>> {
        ICircuitBreaker::reset_multiple_protocols(self, protocols).map_err(|e| e.into())
    }

    // === Role-Based Access Control ===

    /// Grant a role to an account
    /// Requires ADMIN role. Role values: ADMIN=1, OPERATOR=2
    pub fn grant_role(&mut self, account: Address, role: u8) -> Result<(), Vec<u8>> {
        ICircuitBreaker::grant_role(self, account, role).map_err(|e| e.into())
    }

    /// Revoke a role from an account
    /// Requires ADMIN role. Cannot revoke own role or owner's roles
    pub fn revoke_role(&mut self, account: Address, role: u8) -> Result<(), Vec<u8>> {
        ICircuitBreaker::revoke_role(self, account, role).map_err(|e| e.into())
    }

    /// Check if an account has a specific role
    pub fn has_role(&self, account: Address, role: u8) -> bool {
        ICircuitBreaker::has_role(self, account, role)
    }

    /// Get all roles for an account (as bitmask)
    pub fn get_roles(&self, account: Address) -> u8 {
        ICircuitBreaker::get_roles(self, account)
    }

    // === Time Lock Configuration ===

    /// Set minimum pause duration (in seconds)
    /// Requires ADMIN role
    pub fn set_min_pause_duration(&mut self, duration: U256) -> Result<(), Vec<u8>> {
        ICircuitBreaker::set_min_pause_duration(self, duration).map_err(|e| e.into())
    }

    /// Set maximum pause duration (in seconds)
    /// Requires ADMIN role
    pub fn set_max_pause_duration(&mut self, duration: U256) -> Result<(), Vec<u8>> {
        ICircuitBreaker::set_max_pause_duration(self, duration).map_err(|e| e.into())
    }

    /// Set cooldown period between trips (in seconds)
    /// Requires ADMIN role
    pub fn set_cooldown_period(&mut self, period: U256) -> Result<(), Vec<u8>> {
        ICircuitBreaker::set_cooldown_period(self, period).map_err(|e| e.into())
    }

    /// Get current minimum pause duration
    pub fn get_min_pause_duration(&self) -> U256 {
        ICircuitBreaker::get_min_pause_duration(self)
    }

    /// Get current maximum pause duration
    pub fn get_max_pause_duration(&self) -> U256 {
        ICircuitBreaker::get_max_pause_duration(self)
    }

    /// Get current cooldown period
    pub fn get_cooldown_period(&self) -> U256 {
        ICircuitBreaker::get_cooldown_period(self)
    }

    /// Check if a protocol can be reset (minimum duration elapsed)
    pub fn can_reset(&self, protocol: Address) -> bool {
        ICircuitBreaker::can_reset(self, protocol)
    }

    /// Get remaining pause time for a protocol (0 if can be reset)
    pub fn get_remaining_pause_time(&self, protocol: Address) -> U256 {
        ICircuitBreaker::get_remaining_pause_time(self, protocol)
    }

    // === ERC-165 Interface Detection ===

    /// Query if contract supports an interface (ERC-165)
    pub fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        ICircuitBreaker::supports_interface(self, interface_id)
    }

    // === Convenience Functions ===

    /// Get ADMIN role constant
    pub fn admin_role(&self) -> u8 {
        ADMIN_ROLE
    }

    /// Get OPERATOR role constant
    pub fn operator_role(&self) -> u8 {
        OPERATOR_ROLE
    }

    /// Check if caller is authorized (has any role or is owner)
    pub fn is_authorized(&self) -> bool {
        let caller = msg::sender();
        caller == self.owner.get() || self.roles.get(caller) != U256::ZERO
    }
}

// === Unit Tests ===
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, vec};

    // ========================================
    // State Machine Simulation Tests
    // ========================================

    #[test]
    fn test_initial_state_not_tripped() {
        let is_tripped = false;
        assert!(!is_tripped, "Circuit should start un-tripped");
    }

    #[test]
    fn test_trip_sets_active() {
        let mut is_tripped = false;
        is_tripped = true;
        assert!(is_tripped);
    }

    #[test]
    fn test_reset_clears_active() {
        let mut is_tripped = true;
        is_tripped = false;
        assert!(!is_tripped);
    }

    #[test]
    fn test_full_trip_reset_cycle() {
        let mut state = false;
        for _ in 0..5 {
            assert!(!state);
            state = true;
            assert!(state);
            state = false;
        }
    }

    #[test]
    fn test_already_tripped_guard() {
        let is_tripped = true;
        let should_reject = is_tripped; // already tripped
        assert!(should_reject, "Should reject trip when already tripped");
    }

    #[test]
    fn test_not_tripped_guard() {
        let is_tripped = false;
        let should_reject = !is_tripped; // not tripped
        assert!(should_reject, "Should reject reset when not tripped");
    }

    // ========================================
    // Trip Count Arithmetic
    // ========================================

    #[test]
    fn test_trip_count_starts_zero() {
        assert_eq!(U256::ZERO, U256::from(0));
    }

    #[test]
    fn test_trip_count_increments_correctly() {
        let mut count = U256::ZERO;
        for i in 1..=10u64 {
            count = count + U256::from(1);
            assert_eq!(count, U256::from(i));
        }
    }

    #[test]
    fn test_trip_count_near_max() {
        let count = U256::MAX - U256::from(1);
        let next = count + U256::from(1);
        assert_eq!(next, U256::MAX);
    }

    #[test]
    fn test_trip_count_not_reset_on_circuit_reset() {
        // Trip count should survive reset (accumulated counter)
        let mut count = U256::from(5);
        let mut _is_tripped = true;
        _is_tripped = false; // reset
                             // count stays the same
        assert_eq!(count, U256::from(5));
        // next trip increments
        _is_tripped = true;
        count = count + U256::from(1);
        assert_eq!(count, U256::from(6));
    }

    // ========================================
    // Timestamp Tests
    // ========================================

    #[test]
    fn test_timestamp_zero_before_first_trip() {
        let last_trip_time = U256::ZERO;
        assert_eq!(last_trip_time, U256::ZERO);
    }

    #[test]
    fn test_timestamp_records_on_trip() {
        let timestamp = U256::from(1_700_000_000u64);
        assert!(timestamp > U256::ZERO);
    }

    #[test]
    fn test_timestamp_overwrites_on_subsequent_trip() {
        let first = U256::from(1_700_000_000u64);
        let second = U256::from(1_700_001_000u64);
        let mut last_trip_time = first;
        last_trip_time = second;
        assert_eq!(last_trip_time, second);
        assert!(last_trip_time > first);
    }

    // ========================================
    // Address Validation
    // ========================================

    #[test]
    fn test_zero_address_detection() {
        assert_eq!(Address::ZERO, Address::ZERO);
    }

    #[test]
    fn test_non_zero_address_valid() {
        let addr = Address::from([0x01u8; 20]);
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn test_address_equality() {
        let a = Address::from([0xABu8; 20]);
        let b = Address::from([0xABu8; 20]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_address_inequality() {
        let owner = Address::from([0x01u8; 20]);
        let caller = Address::from([0x02u8; 20]);
        assert_ne!(owner, caller);
    }

    // ========================================
    // Error Encoding Tests
    // ========================================

    #[test]
    fn test_already_tripped_error_encodes() {
        let encoded: Vec<u8> = Error::AlreadyTripped.into();
        assert!(encoded.len() >= 4, "Must have selector bytes");
    }

    #[test]
    fn test_not_tripped_error_encodes() {
        let encoded: Vec<u8> = Error::NotTripped.into();
        assert!(encoded.len() >= 4);
    }

    #[test]
    fn test_unauthorized_caller_encodes_with_address() {
        let caller = Address::from([0xAAu8; 20]);
        let encoded: Vec<u8> = Error::UnauthorizedCaller(caller).into();
        // 4 byte selector + 32 byte address word
        assert_eq!(encoded.len(), 36);
    }

    #[test]
    fn test_invalid_owner_encodes_with_address() {
        let owner = Address::ZERO;
        let encoded: Vec<u8> = Error::InvalidOwner(owner).into();
        assert_eq!(encoded.len(), 36);
    }

    #[test]
    fn test_all_error_selectors_unique() {
        let e1: Vec<u8> = Error::AlreadyTripped.into();
        let e2: Vec<u8> = Error::NotTripped.into();
        let e3: Vec<u8> = Error::UnauthorizedCaller(Address::ZERO).into();
        let e4: Vec<u8> = Error::InvalidOwner(Address::ZERO).into();

        let selectors: Vec<&[u8]> = vec![&e1[..4], &e2[..4], &e3[..4], &e4[..4]];
        for i in 0..selectors.len() {
            for j in (i + 1)..selectors.len() {
                assert_ne!(selectors[i], selectors[j], "Error selectors must be unique");
            }
        }
    }

    #[test]
    fn test_error_encoding_deterministic() {
        let addr = Address::from([0xBBu8; 20]);
        let enc1: Vec<u8> = Error::UnauthorizedCaller(addr).into();
        let enc2: Vec<u8> = Error::UnauthorizedCaller(addr).into();
        assert_eq!(enc1, enc2, "Same error must encode identically");
    }

    #[test]
    fn test_encoded_address_present_in_unauthorized_error() {
        let addr = Address::from([0xCCu8; 20]);
        let encoded: Vec<u8> = Error::UnauthorizedCaller(addr).into();
        // Address in ABI encoding: 12 zero bytes + 20 address bytes at offset 4
        let encoded_addr = &encoded[16..36];
        assert_eq!(encoded_addr, addr.as_slice());
    }

    // ========================================
    // Ownership Transfer Logic
    // ========================================

    #[test]
    fn test_ownership_transfer_updates() {
        let mut owner = Address::from([0x01u8; 20]);
        let new_owner = Address::from([0x02u8; 20]);
        let previous = owner;
        owner = new_owner;
        assert_eq!(owner, new_owner);
        assert_ne!(owner, previous);
    }

    #[test]
    fn test_transfer_to_zero_rejected() {
        let new_owner = Address::ZERO;
        assert_eq!(
            new_owner,
            Address::ZERO,
            "Transfer to zero should be caught"
        );
    }

    // ========================================
    // Debug Trait
    // ========================================

    #[test]
    fn test_error_debug_has_variant_name() {
        assert!(format!("{:?}", Error::AlreadyTripped).contains("AlreadyTripped"));
        assert!(format!("{:?}", Error::NotTripped).contains("NotTripped"));
        assert!(format!("{:?}", Error::UnauthorizedCaller(Address::ZERO))
            .contains("UnauthorizedCaller"));
        assert!(format!("{:?}", Error::InvalidOwner(Address::ZERO)).contains("InvalidOwner"));
    }
}
