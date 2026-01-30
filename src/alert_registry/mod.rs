//! AlertRegistry contract for logging and storing security alerts
//!
//! This contract maintains a permanent record of security alerts,
//! including their severity, source, and associated metadata.
//! V2 adds enhanced alerts with prioritization, subscriber management,
//! acknowledgment, expiration, RBAC, and DetectionEngine integration.

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes, U256};
use stylus_sdk::{block, evm, msg};
use stylus_sdk::prelude::*;

pub mod error;
pub mod interface;
pub mod storage;

use error::Error;
use interface::{
    // V1 events
    AlertRegistered, OwnershipTransferred,
    // V2 events
    EnhancedAlertRegistered, AlertAcknowledged,
    SubscriberAdded, SubscriberRemoved,
    AlertExpirationUpdated, DetectionEngineUpdated,
    RoleGranted, RoleRevoked,
    HighThreatAlertEmitted, CriticalAlertEmitted,
    // Trait
    IAlertRegistry,
};
use storage::AlertRegistry;

// === Constants ===

/// Admin role bitmask - can manage roles, subscribers, configuration
const ADMIN_ROLE: u8 = 0x01;
/// Monitor role bitmask - can register enhanced alerts
const MONITOR_ROLE: u8 = 0x02;
/// All valid roles combined
const ALL_ROLES: u8 = ADMIN_ROLE | MONITOR_ROLE;

/// Low priority level (threat_level < 40)
const PRIORITY_LOW: u64 = 0;
/// Medium priority level (40 <= threat_level < 70)
const PRIORITY_MEDIUM: u64 = 1;
/// High priority level (70 <= threat_level < 90)
const PRIORITY_HIGH: u64 = 2;
/// Critical priority level (threat_level >= 90)
const PRIORITY_CRITICAL: u64 = 3;

/// Threshold for medium priority
const THRESHOLD_MEDIUM: u64 = 40;
/// Threshold for high priority
const THRESHOLD_HIGH: u64 = 70;
/// Threshold for critical priority
const THRESHOLD_CRITICAL: u64 = 90;
/// Maximum threat level
const MAX_THREAT_LEVEL: u64 = 100;

/// Default alert expiration duration: 30 days in seconds
const DEFAULT_EXPIRATION: u64 = 2_592_000;
/// Maximum batch size for DOS prevention
#[allow(dead_code)]
const MAX_BATCH_SIZE: u64 = 100;

// === Internal Helpers ===

impl AlertRegistry {
    /// Check if the caller has the required role.
    /// Owner implicitly has all roles.
    /// DetectionEngine address implicitly has MONITOR_ROLE.
    fn require_role(&self, role: u8) -> Result<(), Error> {
        let caller = msg::sender();

        // Owner has all roles
        if caller == self.owner.get() {
            return Ok(());
        }

        // DetectionEngine has MONITOR_ROLE
        if role == MONITOR_ROLE {
            let de_addr = self.detection_engine.get();
            if de_addr != Address::ZERO && caller == de_addr {
                return Ok(());
            }
        }

        // Check role bitmask
        let caller_roles: U256 = self.roles.get(caller);
        let role_bit = U256::from(role);
        if (caller_roles & role_bit) == role_bit {
            return Ok(());
        }

        Err(Error::InsufficientRole {
            caller,
            required_role: role,
        })
    }

    /// Compute priority level from threat level
    fn compute_priority(threat_level: U256) -> U256 {
        let level = threat_level.saturating_to::<u64>();
        if level >= THRESHOLD_CRITICAL {
            U256::from(PRIORITY_CRITICAL)
        } else if level >= THRESHOLD_HIGH {
            U256::from(PRIORITY_HIGH)
        } else if level >= THRESHOLD_MEDIUM {
            U256::from(PRIORITY_MEDIUM)
        } else {
            U256::from(PRIORITY_LOW)
        }
    }

    /// Check if an enhanced alert is expired based on current timestamp
    fn is_enhanced_alert_expired_internal(&self, alert_timestamp: U256) -> bool {
        let expiration = self.alert_expiration_duration.get();
        if expiration.is_zero() {
            return false; // No expiration set
        }
        let now = U256::from(block::timestamp());
        let elapsed = now.saturating_sub(alert_timestamp);
        elapsed > expiration
    }

    /// Initialize default configuration values
    fn initialize_defaults(&mut self) {
        self.alert_expiration_duration.set(U256::from(DEFAULT_EXPIRATION));
    }

    /// Initialize ERC-165 supported interfaces
    fn initialize_erc165(&mut self) {
        // ERC-165 interface ID: 0x01ffc9a7
        let erc165_id = FixedBytes::<4>::from([0x01, 0xff, 0xc9, 0xa7]);
        self.supported_interfaces.setter(erc165_id).set(true);
    }
}

// === Trait Implementation ===

impl IAlertRegistry for AlertRegistry {
    type Error = Error;

    // === V1 Methods (preserved) ===

    fn register_alert(
        &mut self,
        severity: u8,
        source: Address,
        message_hash: FixedBytes<32>,
    ) -> Result<U256, Self::Error> {
        // Validate alert data
        if source == Address::ZERO {
            return Err(Error::InvalidAlert);
        }

        // Get current timestamp
        let timestamp = U256::from(block::timestamp());

        // Create new alert in V1 array
        let mut new_alert = self.alerts.grow();
        new_alert.timestamp.set(timestamp);
        new_alert.severity.set(U256::from(severity));
        new_alert.source.set(source);
        new_alert.message_hash.set(message_hash);

        // Get the alert ID (length - 1)
        let alert_id = U256::from(self.alerts.len() - 1);

        // Emit event
        evm::log(AlertRegistered {
            id: alert_id,
            timestamp,
            severity,
            source,
            messageHash: message_hash,
        });

        Ok(alert_id)
    }

    fn get_alert(
        &self,
        id: U256,
    ) -> Result<(U256, u8, Address, FixedBytes<32>), Self::Error> {
        let index: usize = id
            .try_into()
            .map_err(|_| Error::AlertNotFound { id })?;

        if index >= self.alerts.len() {
            return Err(Error::AlertNotFound { id });
        }

        let alert = self.alerts.get(index).unwrap();
        let timestamp = alert.timestamp.get();
        let severity_uint: U256 = alert.severity.get();
        let severity: u8 = severity_uint.saturating_to::<u64>() as u8;
        let source = alert.source.get();
        let message_hash = alert.message_hash.get();

        Ok((timestamp, severity, source, message_hash))
    }

    fn get_alert_count(&self) -> U256 {
        U256::from(self.alerts.len())
    }

    fn owner(&self) -> Address {
        self.owner.get()
    }

    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Self::Error> {
        if msg::sender() != self.owner.get() {
            return Err(Error::UnauthorizedCaller(msg::sender()));
        }

        if new_owner == Address::ZERO {
            return Err(Error::InvalidOwner(new_owner));
        }

        let previous_owner = self.owner.get();
        self.owner.set(new_owner);

        evm::log(OwnershipTransferred {
            previousOwner: previous_owner,
            newOwner: new_owner,
        });

        Ok(())
    }

    // === V2: Enhanced Alert Registration ===

    fn register_enhanced_alert(
        &mut self,
        protocol: Address,
        threat_level: U256,
        pattern_matched: U256,
        message_hash: FixedBytes<32>,
        source: Address,
    ) -> Result<U256, Self::Error> {
        // Require MONITOR_ROLE (owner, detection engine, or explicit role)
        self.require_role(MONITOR_ROLE)?;

        // Validate inputs
        if protocol == Address::ZERO {
            return Err(Error::InvalidAddress(protocol));
        }

        let max_threat = U256::from(MAX_THREAT_LEVEL);
        if threat_level > max_threat {
            return Err(Error::InvalidAlert);
        }

        // Compute priority
        let priority_level = Self::compute_priority(threat_level);

        // Get timestamp
        let timestamp = U256::from(block::timestamp());

        // Increment alert count and get ID (1-indexed)
        let count = self.enhanced_alert_count.get();
        let alert_id = count.saturating_add(U256::from(1));
        self.enhanced_alert_count.set(alert_id);

        // Store enhanced alert
        {
            let mut alert = self.enhanced_alerts.setter(alert_id);
            alert.protocol.set(protocol);
            alert.threat_level.set(threat_level);
            alert.pattern_matched.set(pattern_matched);
            alert.timestamp.set(timestamp);
            alert.priority_level.set(priority_level);
            alert.acknowledged.set(false);
            alert.acknowledged_by.set(Address::ZERO);
            alert.acknowledged_at.set(U256::ZERO);
            alert.message_hash.set(message_hash);
            alert.source.set(source);
        }

        // Update per-protocol alert count
        let proto_count = self.protocol_alert_count.get(protocol);
        self.protocol_alert_count.setter(protocol).set(proto_count.saturating_add(U256::from(1)));

        // Update priority-based count
        let prio_count = self.priority_alert_counts.get(priority_level);
        self.priority_alert_counts.setter(priority_level).set(prio_count.saturating_add(U256::from(1)));

        // Emit enhanced alert event
        evm::log(EnhancedAlertRegistered {
            id: alert_id,
            protocol,
            threat_level,
            priority_level,
            pattern_matched,
        });

        // Off-chain indexing events for high/critical
        let high_prio = U256::from(PRIORITY_HIGH);
        let critical_prio = U256::from(PRIORITY_CRITICAL);

        if priority_level >= high_prio {
            evm::log(HighThreatAlertEmitted {
                id: alert_id,
                protocol,
                threat_level,
            });
        }

        if priority_level >= critical_prio {
            evm::log(CriticalAlertEmitted {
                id: alert_id,
                protocol,
                threat_level,
            });
        }

        Ok(alert_id)
    }

    fn get_enhanced_alert(
        &self,
        id: U256,
    ) -> Result<(Address, U256, U256, U256, U256, bool, FixedBytes<32>, Address), Self::Error> {
        // Check if alert exists (ID must be >= 1 and <= enhanced_alert_count)
        let count = self.enhanced_alert_count.get();
        if id.is_zero() || id > count {
            return Err(Error::AlertNotFound { id });
        }

        let alert = self.enhanced_alerts.get(id);
        let protocol = alert.protocol.get();
        let threat_level = alert.threat_level.get();
        let pattern_matched = alert.pattern_matched.get();
        let timestamp = alert.timestamp.get();
        let priority_level = alert.priority_level.get();
        let acknowledged = alert.acknowledged.get();
        let message_hash = alert.message_hash.get();
        let source = alert.source.get();

        Ok((protocol, threat_level, pattern_matched, timestamp, priority_level, acknowledged, message_hash, source))
    }

    fn get_enhanced_alert_count(&self) -> U256 {
        self.enhanced_alert_count.get()
    }

    // === V2: Acknowledgment ===

    fn acknowledge_alert(&mut self, id: U256) -> Result<(), Self::Error> {
        // Check if alert exists
        let count = self.enhanced_alert_count.get();
        if id.is_zero() || id > count {
            return Err(Error::AlertNotFound { id });
        }

        // Check not already acknowledged
        let already_ack = self.enhanced_alerts.get(id).acknowledged.get();
        if already_ack {
            return Err(Error::AlertAlreadyAcknowledged { id });
        }

        // Get the alert's protocol
        let alert_protocol = self.enhanced_alerts.get(id).protocol.get();
        let caller = msg::sender();

        // Caller must be the alert's protocol OR have ADMIN_ROLE
        let is_protocol = caller == alert_protocol;
        let is_admin = self.has_role(caller, ADMIN_ROLE);
        if !is_protocol && !is_admin {
            return Err(Error::NotAlertProtocol { caller, id });
        }

        // Set acknowledgment fields
        {
            let mut alert = self.enhanced_alerts.setter(id);
            alert.acknowledged.set(true);
            alert.acknowledged_by.set(caller);
            alert.acknowledged_at.set(U256::from(block::timestamp()));
        }

        // Emit event
        evm::log(AlertAcknowledged {
            id,
            protocol: alert_protocol,
            acknowledger: caller,
        });

        Ok(())
    }

    fn is_alert_acknowledged(&self, id: U256) -> bool {
        let count = self.enhanced_alert_count.get();
        if id.is_zero() || id > count {
            return false;
        }
        self.enhanced_alerts.get(id).acknowledged.get()
    }

    // === V2: Subscriber Management ===

    fn add_subscriber(&mut self, addr: Address) -> Result<(), Self::Error> {
        self.require_role(ADMIN_ROLE)?;

        if addr == Address::ZERO {
            return Err(Error::InvalidSubscriber(addr));
        }

        if self.subscribers.get(addr) {
            return Err(Error::AlreadySubscribed(addr));
        }

        self.subscribers.setter(addr).set(true);
        let count = self.subscriber_count.get();
        self.subscriber_count.set(count.saturating_add(U256::from(1)));

        evm::log(SubscriberAdded {
            subscriber: addr,
            grantor: msg::sender(),
        });

        Ok(())
    }

    fn remove_subscriber(&mut self, addr: Address) -> Result<(), Self::Error> {
        self.require_role(ADMIN_ROLE)?;

        if !self.subscribers.get(addr) {
            return Err(Error::NotSubscribed(addr));
        }

        self.subscribers.setter(addr).set(false);
        let count = self.subscriber_count.get();
        self.subscriber_count.set(count.saturating_sub(U256::from(1)));

        evm::log(SubscriberRemoved {
            subscriber: addr,
            revoker: msg::sender(),
        });

        Ok(())
    }

    fn is_subscriber(&self, addr: Address) -> bool {
        self.subscribers.get(addr)
    }

    fn get_subscriber_count(&self) -> U256 {
        self.subscriber_count.get()
    }

    // === V2: Queries ===

    fn get_protocol_alert_count(&self, protocol: Address) -> U256 {
        self.protocol_alert_count.get(protocol)
    }

    fn get_priority_alert_count(&self, priority_level: U256) -> U256 {
        self.priority_alert_counts.get(priority_level)
    }

    fn is_alert_expired(&self, id: U256) -> Result<bool, Self::Error> {
        let count = self.enhanced_alert_count.get();
        if id.is_zero() || id > count {
            return Err(Error::AlertNotFound { id });
        }

        let alert_timestamp = self.enhanced_alerts.get(id).timestamp.get();
        Ok(self.is_enhanced_alert_expired_internal(alert_timestamp))
    }

    // === V2: Role-Based Access Control ===

    fn grant_role(&mut self, account: Address, role: u8) -> Result<(), Self::Error> {
        self.require_role(ADMIN_ROLE)?;

        // Validate role
        if role == 0 || (role & !ALL_ROLES) != 0 {
            return Err(Error::InvalidRole(role));
        }

        // Validate address
        if account == Address::ZERO {
            return Err(Error::InvalidAddress(account));
        }

        // Set role bits
        let current = self.roles.get(account);
        let role_bits = U256::from(role);
        self.roles.setter(account).set(current | role_bits);

        evm::log(RoleGranted {
            account,
            role,
            grantor: msg::sender(),
        });

        Ok(())
    }

    fn revoke_role(&mut self, account: Address, role: u8) -> Result<(), Self::Error> {
        self.require_role(ADMIN_ROLE)?;

        // Validate role
        if role == 0 || (role & !ALL_ROLES) != 0 {
            return Err(Error::InvalidRole(role));
        }

        // Cannot revoke own role
        if account == msg::sender() {
            return Err(Error::CannotRevokeOwnRole(account));
        }

        // Clear role bits
        let current = self.roles.get(account);
        let role_bits = U256::from(role);
        self.roles.setter(account).set(current & !role_bits);

        evm::log(RoleRevoked {
            account,
            role,
            revoker: msg::sender(),
        });

        Ok(())
    }

    fn has_role(&self, account: Address, role: u8) -> bool {
        // Owner has all roles
        if account == self.owner.get() {
            return true;
        }
        let current = self.roles.get(account);
        let role_bits = U256::from(role);
        (current & role_bits) == role_bits
    }

    fn get_roles(&self, account: Address) -> u8 {
        let roles: U256 = self.roles.get(account);
        roles.saturating_to::<u64>() as u8
    }

    // === V2: Configuration ===

    fn set_alert_expiration_duration(&mut self, duration: U256) -> Result<(), Self::Error> {
        self.require_role(ADMIN_ROLE)?;

        if duration.is_zero() {
            return Err(Error::InvalidExpirationDuration { duration });
        }

        let old_duration = self.alert_expiration_duration.get();
        self.alert_expiration_duration.set(duration);

        evm::log(AlertExpirationUpdated {
            old_duration,
            new_duration: duration,
        });

        Ok(())
    }

    fn get_alert_expiration_duration(&self) -> U256 {
        self.alert_expiration_duration.get()
    }

    fn set_detection_engine(&mut self, addr: Address) -> Result<(), Self::Error> {
        self.require_role(ADMIN_ROLE)?;

        if addr == Address::ZERO {
            return Err(Error::InvalidAddress(addr));
        }

        let old_engine = self.detection_engine.get();
        self.detection_engine.set(addr);

        evm::log(DetectionEngineUpdated {
            old_engine,
            new_engine: addr,
        });

        Ok(())
    }

    fn get_detection_engine(&self) -> Address {
        self.detection_engine.get()
    }

    // === V2: ERC-165 Interface Detection ===

    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        self.supported_interfaces.get(interface_id)
    }
}

// === Public API ===

#[public]
impl AlertRegistry {
    /// Initialize the contract with the deployer as owner
    /// Should be called once after deployment
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        // Only allow initialization if owner is not set
        if self.owner.get() != Address::ZERO {
            return Err(Error::InvalidOwner(self.owner.get()).into());
        }
        let caller = msg::sender();
        self.owner.set(caller);

        // Grant owner all roles
        self.roles.setter(caller).set(U256::from(ALL_ROLES));

        // Set default configuration
        self.initialize_defaults();

        // Initialize ERC-165
        self.initialize_erc165();

        Ok(())
    }

    // --- V1 Public Wrappers ---

    /// Register a new alert (V1)
    pub fn register_alert(
        &mut self,
        severity: u8,
        source: Address,
        message_hash: FixedBytes<32>,
    ) -> Result<U256, Vec<u8>> {
        IAlertRegistry::register_alert(self, severity, source, message_hash)
            .map_err(|e| e.into())
    }

    /// Get alert by ID (V1)
    pub fn get_alert(&self, id: U256) -> Result<(U256, u8, Address, FixedBytes<32>), Vec<u8>> {
        IAlertRegistry::get_alert(self, id).map_err(|e| e.into())
    }

    /// Get total alert count (V1)
    pub fn get_alert_count(&self) -> U256 {
        IAlertRegistry::get_alert_count(self)
    }

    /// Get current owner
    pub fn owner(&self) -> Address {
        IAlertRegistry::owner(self)
    }

    /// Transfer ownership
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        IAlertRegistry::transfer_ownership(self, new_owner).map_err(|e| e.into())
    }

    // --- V2 Public Wrappers ---

    /// Register a new enhanced alert with full metadata
    pub fn register_enhanced_alert(
        &mut self,
        protocol: Address,
        threat_level: U256,
        pattern_matched: U256,
        message_hash: FixedBytes<32>,
        source: Address,
    ) -> Result<U256, Vec<u8>> {
        IAlertRegistry::register_enhanced_alert(self, protocol, threat_level, pattern_matched, message_hash, source)
            .map_err(|e| e.into())
    }

    /// Get enhanced alert details by ID
    pub fn get_enhanced_alert(
        &self,
        id: U256,
    ) -> Result<(Address, U256, U256, U256, U256, bool, FixedBytes<32>, Address), Vec<u8>> {
        IAlertRegistry::get_enhanced_alert(self, id).map_err(|e| e.into())
    }

    /// Get total enhanced alert count
    pub fn get_enhanced_alert_count(&self) -> U256 {
        IAlertRegistry::get_enhanced_alert_count(self)
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, id: U256) -> Result<(), Vec<u8>> {
        IAlertRegistry::acknowledge_alert(self, id).map_err(|e| e.into())
    }

    /// Check if an alert is acknowledged
    pub fn is_alert_acknowledged(&self, id: U256) -> bool {
        IAlertRegistry::is_alert_acknowledged(self, id)
    }

    /// Add a subscriber
    pub fn add_subscriber(&mut self, addr: Address) -> Result<(), Vec<u8>> {
        IAlertRegistry::add_subscriber(self, addr).map_err(|e| e.into())
    }

    /// Remove a subscriber
    pub fn remove_subscriber(&mut self, addr: Address) -> Result<(), Vec<u8>> {
        IAlertRegistry::remove_subscriber(self, addr).map_err(|e| e.into())
    }

    /// Check if an address is subscribed
    pub fn is_subscriber(&self, addr: Address) -> bool {
        IAlertRegistry::is_subscriber(self, addr)
    }

    /// Get subscriber count
    pub fn get_subscriber_count(&self) -> U256 {
        IAlertRegistry::get_subscriber_count(self)
    }

    /// Get alert count for a protocol
    pub fn get_protocol_alert_count(&self, protocol: Address) -> U256 {
        IAlertRegistry::get_protocol_alert_count(self, protocol)
    }

    /// Get alert count for a priority level
    pub fn get_priority_alert_count(&self, priority_level: U256) -> U256 {
        IAlertRegistry::get_priority_alert_count(self, priority_level)
    }

    /// Check if an alert is expired
    pub fn is_alert_expired(&self, id: U256) -> Result<bool, Vec<u8>> {
        IAlertRegistry::is_alert_expired(self, id).map_err(|e| e.into())
    }

    /// Grant a role to an account
    pub fn grant_role(&mut self, account: Address, role: u8) -> Result<(), Vec<u8>> {
        IAlertRegistry::grant_role(self, account, role).map_err(|e| e.into())
    }

    /// Revoke a role from an account
    pub fn revoke_role(&mut self, account: Address, role: u8) -> Result<(), Vec<u8>> {
        IAlertRegistry::revoke_role(self, account, role).map_err(|e| e.into())
    }

    /// Check if an account has a role
    pub fn has_role(&self, account: Address, role: u8) -> bool {
        IAlertRegistry::has_role(self, account, role)
    }

    /// Get all roles for an account
    pub fn get_roles(&self, account: Address) -> u8 {
        IAlertRegistry::get_roles(self, account)
    }

    /// Set alert expiration duration
    pub fn set_alert_expiration_duration(&mut self, duration: U256) -> Result<(), Vec<u8>> {
        IAlertRegistry::set_alert_expiration_duration(self, duration).map_err(|e| e.into())
    }

    /// Get alert expiration duration
    pub fn get_alert_expiration_duration(&self) -> U256 {
        IAlertRegistry::get_alert_expiration_duration(self)
    }

    /// Set detection engine address
    pub fn set_detection_engine(&mut self, addr: Address) -> Result<(), Vec<u8>> {
        IAlertRegistry::set_detection_engine(self, addr).map_err(|e| e.into())
    }

    /// Get detection engine address
    pub fn get_detection_engine(&self) -> Address {
        IAlertRegistry::get_detection_engine(self)
    }

    /// Check if an interface is supported (ERC-165)
    pub fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        IAlertRegistry::supports_interface(self, interface_id)
    }
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;

    // --- Role Constants ---

    #[test]
    fn test_admin_role_constant() {
        assert_eq!(ADMIN_ROLE, 0x01);
    }

    #[test]
    fn test_monitor_role_constant() {
        assert_eq!(MONITOR_ROLE, 0x02);
    }

    #[test]
    fn test_all_roles_is_combined_bitmask() {
        assert_eq!(ALL_ROLES, ADMIN_ROLE | MONITOR_ROLE);
        assert_eq!(ALL_ROLES, 0x03);
    }

    #[test]
    fn test_roles_are_distinct_bits() {
        assert_eq!(ADMIN_ROLE & MONITOR_ROLE, 0);
    }

    #[test]
    fn test_role_bitmask_operations() {
        let mut roles: u8 = 0;

        // Grant admin
        roles |= ADMIN_ROLE;
        assert_eq!(roles & ADMIN_ROLE, ADMIN_ROLE);
        assert_eq!(roles & MONITOR_ROLE, 0);

        // Grant monitor
        roles |= MONITOR_ROLE;
        assert_eq!(roles & ADMIN_ROLE, ADMIN_ROLE);
        assert_eq!(roles & MONITOR_ROLE, MONITOR_ROLE);
        assert_eq!(roles, ALL_ROLES);

        // Revoke admin
        roles &= !ADMIN_ROLE;
        assert_eq!(roles & ADMIN_ROLE, 0);
        assert_eq!(roles & MONITOR_ROLE, MONITOR_ROLE);
    }

    #[test]
    fn test_invalid_role_detection() {
        let invalid_role: u8 = 0x04; // Not in ALL_ROLES
        assert_ne!(invalid_role & !ALL_ROLES, 0);

        let valid_role: u8 = ADMIN_ROLE;
        assert_eq!(valid_role & !ALL_ROLES, 0);
    }

    // --- Priority Constants ---

    #[test]
    fn test_priority_level_ordering() {
        assert!(PRIORITY_LOW < PRIORITY_MEDIUM);
        assert!(PRIORITY_MEDIUM < PRIORITY_HIGH);
        assert!(PRIORITY_HIGH < PRIORITY_CRITICAL);
    }

    #[test]
    fn test_priority_level_values() {
        assert_eq!(PRIORITY_LOW, 0);
        assert_eq!(PRIORITY_MEDIUM, 1);
        assert_eq!(PRIORITY_HIGH, 2);
        assert_eq!(PRIORITY_CRITICAL, 3);
    }

    #[test]
    fn test_threshold_ordering() {
        assert!(THRESHOLD_MEDIUM < THRESHOLD_HIGH);
        assert!(THRESHOLD_HIGH < THRESHOLD_CRITICAL);
        assert!(THRESHOLD_CRITICAL <= MAX_THREAT_LEVEL);
    }

    #[test]
    fn test_threshold_values() {
        assert_eq!(THRESHOLD_MEDIUM, 40);
        assert_eq!(THRESHOLD_HIGH, 70);
        assert_eq!(THRESHOLD_CRITICAL, 90);
        assert_eq!(MAX_THREAT_LEVEL, 100);
    }

    // --- Priority Computation ---

    #[test]
    fn test_compute_priority_low_zero() {
        let result = AlertRegistry::compute_priority(U256::from(0));
        assert_eq!(result, U256::from(PRIORITY_LOW));
    }

    #[test]
    fn test_compute_priority_low_below_medium() {
        let result = AlertRegistry::compute_priority(U256::from(39));
        assert_eq!(result, U256::from(PRIORITY_LOW));
    }

    #[test]
    fn test_compute_priority_medium_at_threshold() {
        let result = AlertRegistry::compute_priority(U256::from(THRESHOLD_MEDIUM));
        assert_eq!(result, U256::from(PRIORITY_MEDIUM));
    }

    #[test]
    fn test_compute_priority_medium_below_high() {
        let result = AlertRegistry::compute_priority(U256::from(69));
        assert_eq!(result, U256::from(PRIORITY_MEDIUM));
    }

    #[test]
    fn test_compute_priority_high_at_threshold() {
        let result = AlertRegistry::compute_priority(U256::from(THRESHOLD_HIGH));
        assert_eq!(result, U256::from(PRIORITY_HIGH));
    }

    #[test]
    fn test_compute_priority_high_below_critical() {
        let result = AlertRegistry::compute_priority(U256::from(89));
        assert_eq!(result, U256::from(PRIORITY_HIGH));
    }

    #[test]
    fn test_compute_priority_critical_at_threshold() {
        let result = AlertRegistry::compute_priority(U256::from(THRESHOLD_CRITICAL));
        assert_eq!(result, U256::from(PRIORITY_CRITICAL));
    }

    #[test]
    fn test_compute_priority_critical_at_max() {
        let result = AlertRegistry::compute_priority(U256::from(MAX_THREAT_LEVEL));
        assert_eq!(result, U256::from(PRIORITY_CRITICAL));
    }

    // --- Configuration Constants ---

    #[test]
    fn test_default_expiration_is_30_days() {
        assert_eq!(DEFAULT_EXPIRATION, 30 * 24 * 60 * 60);
        assert_eq!(DEFAULT_EXPIRATION, 2_592_000);
    }

    #[test]
    fn test_max_batch_size() {
        assert_eq!(MAX_BATCH_SIZE, 100);
    }

    // --- Saturating Arithmetic ---

    #[test]
    fn test_saturating_add_no_overflow() {
        let a = U256::from(50);
        let b = U256::from(30);
        assert_eq!(a.saturating_add(b), U256::from(80));
    }

    #[test]
    fn test_saturating_add_at_max() {
        let max = U256::MAX;
        let result = max.saturating_add(U256::from(1));
        assert_eq!(result, U256::MAX);
    }

    #[test]
    fn test_saturating_sub_no_underflow() {
        let a = U256::from(100);
        let b = U256::from(200);
        assert_eq!(a.saturating_sub(b), U256::ZERO);
    }

    // --- Threat Level Validation ---

    #[test]
    fn test_threat_level_max_is_100() {
        assert_eq!(MAX_THREAT_LEVEL, 100);
    }

    #[test]
    fn test_threat_level_boundaries() {
        // At maximum
        let at_max = U256::from(MAX_THREAT_LEVEL);
        let max = U256::from(MAX_THREAT_LEVEL);
        assert!(at_max <= max);

        // Above maximum
        let above_max = U256::from(MAX_THREAT_LEVEL + 1);
        assert!(above_max > max);
    }

    // --- Batch Size Limits ---

    #[test]
    fn test_batch_size_limit() {
        let batch = U256::from(MAX_BATCH_SIZE);
        let max = U256::from(MAX_BATCH_SIZE);
        assert!(batch <= max);

        let too_large = U256::from(MAX_BATCH_SIZE + 1);
        assert!(too_large > max);
    }

    // --- Address Zero Checks ---

    #[test]
    fn test_zero_address_detection() {
        let zero = Address::ZERO;
        assert_eq!(zero, Address::ZERO);

        let non_zero = Address::from([1u8; 20]);
        assert_ne!(non_zero, Address::ZERO);
    }

    // --- ERC-165 Interface IDs ---

    #[test]
    fn test_erc165_interface_id() {
        let erc165_id = FixedBytes::<4>::from([0x01, 0xff, 0xc9, 0xa7]);
        assert_eq!(erc165_id[0], 0x01);
        assert_eq!(erc165_id[1], 0xff);
        assert_eq!(erc165_id[2], 0xc9);
        assert_eq!(erc165_id[3], 0xa7);
    }

    // --- Priority Count Tracking ---

    #[test]
    fn test_priority_count_increments() {
        let mut counts = [U256::ZERO; 4];

        // Simulate registering alerts at different priorities
        counts[PRIORITY_LOW as usize] = counts[PRIORITY_LOW as usize].saturating_add(U256::from(1));
        counts[PRIORITY_MEDIUM as usize] = counts[PRIORITY_MEDIUM as usize].saturating_add(U256::from(1));
        counts[PRIORITY_HIGH as usize] = counts[PRIORITY_HIGH as usize].saturating_add(U256::from(1));
        counts[PRIORITY_CRITICAL as usize] = counts[PRIORITY_CRITICAL as usize].saturating_add(U256::from(1));

        assert_eq!(counts[0], U256::from(1));
        assert_eq!(counts[1], U256::from(1));
        assert_eq!(counts[2], U256::from(1));
        assert_eq!(counts[3], U256::from(1));
    }

    // --- Expiration Logic ---

    #[test]
    fn test_expiration_calculation() {
        let created_at: u64 = 1_000_000;
        let expiration_duration: u64 = DEFAULT_EXPIRATION;
        let now_not_expired: u64 = created_at + expiration_duration - 1;
        let now_expired: u64 = created_at + expiration_duration + 1;

        let elapsed_not = now_not_expired - created_at;
        assert!(elapsed_not <= expiration_duration);

        let elapsed_yes = now_expired - created_at;
        assert!(elapsed_yes > expiration_duration);
    }

    #[test]
    fn test_expiration_at_boundary() {
        let created_at = U256::from(1_000_000u64);
        let expiration = U256::from(DEFAULT_EXPIRATION);
        let now_exact = created_at.saturating_add(expiration);

        // At exactly expiration time, NOT expired (> not >=)
        let elapsed = now_exact.saturating_sub(created_at);
        assert!(!(elapsed > expiration));

        // One second after, expired
        let now_after = now_exact.saturating_add(U256::from(1));
        let elapsed_after = now_after.saturating_sub(created_at);
        assert!(elapsed_after > expiration);
    }

    // --- Acknowledgment State ---

    #[test]
    fn test_acknowledgment_state_transition() {
        let mut acknowledged = false;
        assert!(!acknowledged);

        acknowledged = true;
        assert!(acknowledged);

        // Cannot un-acknowledge (one-way transition)
        // This is enforced by the contract logic
    }
}
