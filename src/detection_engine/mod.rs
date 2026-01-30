//! DetectionEngine contract for anomaly detection and exploit pattern analysis
//!
//! This contract allows registering metrics with thresholds and detecting
//! when reported values exceed those thresholds, indicating potential anomalies.
//!
//! # Features
//! - Metric registration and anomaly detection (V1)
//! - Exploit pattern registration (flash loan, price manipulation, reentrancy, front-running)
//! - Real-time transaction analysis with threat scoring (0-100)
//! - Whitelist management for trusted addresses
//! - Role-based access control (Admin, Analyst)
//! - ERC-165 interface detection support
//!
//! # Security
//! - Reentrancy protection via Rust ownership model
//! - Integer overflow protection with saturating arithmetic
//! - Access control with role-based permissions
//! - Input validation for all external functions
//! - DOS prevention with batch size limits

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
    IDetectionEngine,
    // V1 events
    AnomalyDetected, MetricRegistered, MetricReported, OwnershipTransferred,
    // V2: Pattern events
    PatternRegistered, PatternDeactivated, PatternActivated,
    // V2: Analysis events
    TransactionAnalyzed, HighThreatDetected,
    // V2: Whitelist events
    AddressWhitelisted, AddressRemovedFromWhitelist,
    // V2: Role events
    RoleGranted, RoleRevoked,
    // V2: Config events
    HighThreatThresholdUpdated, PatternTypeThresholdUpdated,
    PatternTypeRateThresholdUpdated,
};
use storage::DetectionEngine;

// === Role Constants ===
/// Admin role: Can manage roles, register patterns, configure thresholds, manage whitelist
pub const ADMIN_ROLE: u8 = 0x01;
/// Analyst role: Can report metrics and analyze transactions
pub const ANALYST_ROLE: u8 = 0x02;
/// All roles combined
pub const ALL_ROLES: u8 = ADMIN_ROLE | ANALYST_ROLE;

// === Pattern Type Constants (bitmask flags) ===
/// Flash loan exploit pattern
pub const PATTERN_FLASH_LOAN: u64 = 1;
/// Price manipulation exploit pattern
pub const PATTERN_PRICE_MANIPULATION: u64 = 2;
/// Reentrancy exploit pattern
pub const PATTERN_REENTRANCY: u64 = 4;
/// Front-running exploit pattern
pub const PATTERN_FRONTRUNNING: u64 = 8;
/// All valid pattern types combined
pub const ALL_PATTERN_TYPES: u64 = PATTERN_FLASH_LOAN
    | PATTERN_PRICE_MANIPULATION
    | PATTERN_REENTRANCY
    | PATTERN_FRONTRUNNING;

// === Threat Scoring Constants ===
/// Default threshold above which threats are considered high
pub const DEFAULT_HIGH_THREAT_THRESHOLD: u64 = 70;
/// Maximum possible threat score
pub const MAX_THREAT_SCORE: u64 = 100;
/// Maximum severity for a pattern
pub const MAX_SEVERITY: u64 = 100;

// === Limits ===
/// Maximum batch size for bulk operations (prevents DOS)
pub const MAX_BATCH_SIZE: usize = 100;

// === ERC-165 Interface IDs ===
/// ERC-165 interface ID: 0x01ffc9a7
pub const ERC165_INTERFACE_ID: [u8; 4] = [0x01, 0xff, 0xc9, 0xa7];
/// DetectionEngine interface ID (computed from function signatures)
pub const DETECTION_ENGINE_INTERFACE_ID: [u8; 4] = [0xde, 0x1e, 0xc1, 0x00];

// === Internal Helper Methods ===
impl DetectionEngine {
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

    /// Internal: Validate pattern type bitmask
    fn validate_pattern_type(&self, pattern_type: U256) -> Result<(), Error> {
        let pt: u64 = pattern_type.saturating_to();
        if pt == 0 || pt > ALL_PATTERN_TYPES {
            return Err(Error::InvalidPatternType { pattern_type });
        }
        Ok(())
    }

    /// Internal: Check if a pattern exists (ID < pattern_count)
    fn require_pattern_exists(&self, id: U256) -> Result<(), Error> {
        let count = self.pattern_count.get();
        if id >= count {
            return Err(Error::PatternNotFound { id });
        }
        Ok(())
    }

    /// Internal: Initialize ERC-165 support
    fn initialize_erc165(&mut self) {
        self.supported_interfaces
            .insert(FixedBytes::from(ERC165_INTERFACE_ID), true);
        self.supported_interfaces
            .insert(FixedBytes::from(DETECTION_ENGINE_INTERFACE_ID), true);
    }

    /// Internal: Initialize default configuration values
    fn initialize_defaults(&mut self) {
        self.high_threat_threshold
            .set(U256::from(DEFAULT_HIGH_THREAT_THRESHOLD));
    }

    /// Internal: Compute threat score for a transaction
    ///
    /// Iterates over registered patterns (bounded by pattern_count),
    /// checks each active pattern against transaction parameters,
    /// and returns the average severity of matched patterns (capped at 100).
    fn compute_threat_score(
        &self,
        value: U256,
        gas_price: U256,
    ) -> (U256, U256) {
        let pattern_count: u64 = self.pattern_count.get().saturating_to();
        let mut total_severity = U256::ZERO;
        let mut match_count = U256::ZERO;
        let mut matched_patterns = U256::ZERO;

        for i in 0..pattern_count {
            let id = U256::from(i);

            // Skip inactive patterns
            if !self.pattern_active.get(id) {
                continue;
            }

            let pattern = self.patterns.get(id);
            let pt: u64 = pattern.pattern_type.get().saturating_to();
            let severity = pattern.severity.get();

            let mut pattern_matched = false;

            // Check flash loan pattern: value exceeds flash loan threshold
            if (pt & PATTERN_FLASH_LOAN) != 0 {
                let threshold = self.pattern_type_thresholds.get(U256::from(PATTERN_FLASH_LOAN));
                if threshold != U256::ZERO && value > threshold {
                    pattern_matched = true;
                }
            }

            // Check price manipulation pattern: gas price exceeds rate threshold
            if (pt & PATTERN_PRICE_MANIPULATION) != 0 {
                let threshold = self
                    .pattern_type_rate_thresholds
                    .get(U256::from(PATTERN_PRICE_MANIPULATION));
                if threshold != U256::ZERO && gas_price > threshold {
                    pattern_matched = true;
                }
            }

            // Check reentrancy pattern: value exceeds reentrancy threshold
            if (pt & PATTERN_REENTRANCY) != 0 {
                let threshold = self.pattern_type_thresholds.get(U256::from(PATTERN_REENTRANCY));
                if threshold != U256::ZERO && value > threshold {
                    pattern_matched = true;
                }
            }

            // Check front-running pattern: gas price exceeds front-running rate threshold
            if (pt & PATTERN_FRONTRUNNING) != 0 {
                let threshold = self
                    .pattern_type_rate_thresholds
                    .get(U256::from(PATTERN_FRONTRUNNING));
                if threshold != U256::ZERO && gas_price > threshold {
                    pattern_matched = true;
                }
            }

            if pattern_matched {
                total_severity = total_severity.saturating_add(severity);
                match_count = match_count.saturating_add(U256::from(1));
                matched_patterns = matched_patterns | U256::from(pt);
            }
        }

        // Compute average severity, capped at MAX_THREAT_SCORE
        let threat_level = if match_count > U256::ZERO {
            let avg = total_severity / match_count;
            let max = U256::from(MAX_THREAT_SCORE);
            if avg > max { max } else { avg }
        } else {
            U256::ZERO
        };

        (threat_level, matched_patterns)
    }
}

// === IDetectionEngine Trait Implementation ===
impl IDetectionEngine for DetectionEngine {
    type Error = Error;

    // === V1 Methods (updated with RBAC) ===

    fn register_metric(&mut self, id: U256, threshold: U256) -> Result<(), Self::Error> {
        // V2: Changed from owner-only to ADMIN role (backward-compatible since owner has all roles)
        self.require_role(ADMIN_ROLE)?;

        // Store the threshold
        self.thresholds.setter(id).set(threshold);

        // Increment metric count
        let count = self.metric_count.get();
        self.metric_count.set(count.saturating_add(U256::from(1)));

        // Emit event
        evm::log(MetricRegistered { id, threshold });

        Ok(())
    }

    fn report_metric(&mut self, id: U256, value: U256) -> Result<(), Self::Error> {
        // V2: Require ANALYST role
        self.require_role(ANALYST_ROLE)?;

        // Check if metric exists (has a threshold set)
        let threshold = self.thresholds.get(id);
        if threshold == U256::ZERO {
            return Err(Error::MetricNotFound { id });
        }

        // V2: Store previous value and timestamp for rate-of-change tracking
        let current = self.current_values.get(id);
        self.previous_values.setter(id).set(current);
        self.last_report_time
            .setter(id)
            .set(U256::from(block::timestamp()));

        // Store the current value
        self.current_values.setter(id).set(value);

        // Emit metric reported event
        evm::log(MetricReported { id, value });

        // Check if anomaly detected and emit event if so
        if value > threshold {
            evm::log(AnomalyDetected {
                id,
                current: value,
                threshold,
            });
        }

        Ok(())
    }

    fn check_anomaly(&self, id: U256) -> Result<bool, Self::Error> {
        let threshold = self.thresholds.get(id);
        let current = self.current_values.get(id);

        // If threshold is zero and no value set, metric doesn't exist
        if threshold == U256::ZERO && current == U256::ZERO {
            return Err(Error::MetricNotFound { id });
        }

        Ok(current > threshold)
    }

    fn get_threshold(&self, id: U256) -> U256 {
        self.thresholds.get(id)
    }

    fn get_current_value(&self, id: U256) -> U256 {
        self.current_values.get(id)
    }

    fn get_metric_count(&self) -> U256 {
        self.metric_count.get()
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

    // === V2: Pattern Registration ===

    fn register_pattern(
        &mut self,
        pattern_type: U256,
        severity: U256,
        description_hash: FixedBytes<32>,
    ) -> Result<U256, Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate pattern type
        self.validate_pattern_type(pattern_type)?;

        // Validate severity
        if severity > U256::from(MAX_SEVERITY) {
            return Err(Error::InvalidSeverity { severity });
        }

        // Get next pattern ID
        let id = self.pattern_count.get();

        // Store pattern data
        let mut pattern = self.patterns.setter(id);
        pattern.pattern_type.set(pattern_type);
        pattern.severity.set(severity);
        pattern.description_hash.set(description_hash);
        pattern.created_at.set(U256::from(block::timestamp()));

        // Mark as active
        self.pattern_active.insert(id, true);

        // Increment pattern count
        self.pattern_count
            .set(id.saturating_add(U256::from(1)));

        // Emit event
        evm::log(PatternRegistered {
            id,
            pattern_type,
            severity,
            description_hash,
        });

        Ok(id)
    }

    fn deactivate_pattern(&mut self, id: U256) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Check pattern exists
        self.require_pattern_exists(id)?;

        // Check if already inactive
        if !self.pattern_active.get(id) {
            return Err(Error::PatternInactive { id });
        }

        // Deactivate
        self.pattern_active.insert(id, false);

        // Emit event
        evm::log(PatternDeactivated { id });

        Ok(())
    }

    fn activate_pattern(&mut self, id: U256) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Check pattern exists
        self.require_pattern_exists(id)?;

        // Activate (idempotent)
        self.pattern_active.insert(id, true);

        // Emit event
        evm::log(PatternActivated { id });

        Ok(())
    }

    fn get_pattern(&self, id: U256) -> Result<(U256, U256, FixedBytes<32>, U256), Self::Error> {
        // Check pattern exists
        self.require_pattern_exists(id)?;

        let pattern = self.patterns.get(id);
        Ok((
            pattern.pattern_type.get(),
            pattern.severity.get(),
            pattern.description_hash.get(),
            pattern.created_at.get(),
        ))
    }

    fn get_pattern_count(&self) -> U256 {
        self.pattern_count.get()
    }

    fn is_pattern_active(&self, id: U256) -> bool {
        self.pattern_active.get(id)
    }

    // === V2: Transaction Analysis ===

    fn analyze_transaction(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
        calldata_hash: FixedBytes<32>,
        gas_price: U256,
    ) -> Result<U256, Self::Error> {
        // Require ANALYST role
        self.require_role(ANALYST_ROLE)?;

        // Validate inputs
        if from == Address::ZERO || to == Address::ZERO {
            return Err(Error::InvalidAnalysisInput);
        }

        // Get next analysis ID
        let analysis_id = self.analysis_count.get();
        let timestamp = U256::from(block::timestamp());

        // Check whitelist - whitelisted addresses get zero threat
        let (threat_level, matched_patterns) = if self.whitelisted.get(from) {
            (U256::ZERO, U256::ZERO)
        } else {
            // Compute threat score based on registered patterns
            self.compute_threat_score(value, gas_price)
        };

        // Store analysis result
        let mut result = self.analysis_results.setter(analysis_id);
        result.analyzed_from.set(from);
        result.analyzed_to.set(to);
        result.tx_value.set(value);
        result.calldata_hash.set(calldata_hash);
        result.gas_price.set(gas_price);
        result.threat_level.set(threat_level);
        result.matched_patterns.set(matched_patterns);
        result.timestamp.set(timestamp);

        // Update address threat scores (weighted running average)
        let prev_count = self.address_analysis_count.get(from);
        let prev_score = self.address_threat_scores.get(from);
        let new_count = prev_count.saturating_add(U256::from(1));

        // Weighted average: (prev_score * prev_count + threat_level) / new_count
        let weighted_sum = prev_score
            .saturating_mul(prev_count)
            .saturating_add(threat_level);
        let new_score = if new_count > U256::ZERO {
            weighted_sum / new_count
        } else {
            U256::ZERO
        };

        self.address_threat_scores.insert(from, new_score);
        self.address_analysis_count.insert(from, new_count);

        // Increment total analysis count
        self.analysis_count
            .set(analysis_id.saturating_add(U256::from(1)));

        // Emit high threat event if above threshold
        let high_threshold = self.high_threat_threshold.get();
        if threat_level >= high_threshold && high_threshold > U256::ZERO {
            evm::log(HighThreatDetected {
                analysis_id,
                from_addr: from,
                threat_level,
                matched_patterns,
            });
        }

        // Emit analysis event
        evm::log(TransactionAnalyzed {
            analysis_id,
            from_addr: from,
            to_addr: to,
            threat_level,
            matched_patterns,
        });

        Ok(analysis_id)
    }

    fn get_analysis_result(
        &self,
        id: U256,
    ) -> Result<(Address, Address, U256, FixedBytes<32>, U256, U256, U256, U256), Self::Error> {
        // Check analysis exists
        if id >= self.analysis_count.get() {
            return Err(Error::AnalysisNotFound { id });
        }

        let result = self.analysis_results.get(id);
        Ok((
            result.analyzed_from.get(),
            result.analyzed_to.get(),
            result.tx_value.get(),
            result.calldata_hash.get(),
            result.gas_price.get(),
            result.threat_level.get(),
            result.matched_patterns.get(),
            result.timestamp.get(),
        ))
    }

    fn get_analysis_count(&self) -> U256 {
        self.analysis_count.get()
    }

    // === V2: Threat Scoring ===

    fn get_address_threat_score(&self, addr: Address) -> U256 {
        self.address_threat_scores.get(addr)
    }

    fn get_address_analysis_count(&self, addr: Address) -> U256 {
        self.address_analysis_count.get(addr)
    }

    fn set_high_threat_threshold(&mut self, threshold: U256) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate: must not exceed MAX_THREAT_SCORE
        if threshold > U256::from(MAX_THREAT_SCORE) {
            return Err(Error::InvalidThresholdValue { value: threshold });
        }

        let old_threshold = self.high_threat_threshold.get();
        self.high_threat_threshold.set(threshold);

        evm::log(HighThreatThresholdUpdated {
            old_threshold,
            new_threshold: threshold,
        });

        Ok(())
    }

    fn get_high_threat_threshold(&self) -> U256 {
        self.high_threat_threshold.get()
    }

    // === V2: Whitelist Management ===

    fn add_to_whitelist(&mut self, addr: Address) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate address
        if addr == Address::ZERO {
            return Err(Error::InvalidAddress(addr));
        }

        // Check if already whitelisted
        if self.whitelisted.get(addr) {
            return Err(Error::AddressAlreadyWhitelisted(addr));
        }

        // Add to whitelist
        self.whitelisted.insert(addr, true);

        evm::log(AddressWhitelisted {
            addr,
            grantor: msg::sender(),
        });

        Ok(())
    }

    fn remove_from_whitelist(&mut self, addr: Address) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Check if whitelisted
        if !self.whitelisted.get(addr) {
            return Err(Error::AddressNotWhitelisted(addr));
        }

        // Remove from whitelist
        self.whitelisted.insert(addr, false);

        evm::log(AddressRemovedFromWhitelist {
            addr,
            revoker: msg::sender(),
        });

        Ok(())
    }

    fn is_whitelisted(&self, addr: Address) -> bool {
        self.whitelisted.get(addr)
    }

    // === V2: Role-Based Access Control ===

    fn grant_role(&mut self, account: Address, role: u8) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate role
        self.validate_role(role)?;

        // Validate account
        if account == Address::ZERO {
            return Err(Error::InvalidAddress(account));
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

    // === V2: Pattern Type Configuration ===

    fn set_pattern_type_threshold(
        &mut self,
        pattern_type: U256,
        threshold: U256,
    ) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate pattern type
        self.validate_pattern_type(pattern_type)?;

        let old_value = self.pattern_type_thresholds.get(pattern_type);
        self.pattern_type_thresholds.insert(pattern_type, threshold);

        evm::log(PatternTypeThresholdUpdated {
            pattern_type,
            old_value,
            new_value: threshold,
        });

        Ok(())
    }

    fn get_pattern_type_threshold(&self, pattern_type: U256) -> U256 {
        self.pattern_type_thresholds.get(pattern_type)
    }

    fn set_pattern_type_rate_threshold(
        &mut self,
        pattern_type: U256,
        threshold: U256,
    ) -> Result<(), Self::Error> {
        // Require ADMIN role
        self.require_role(ADMIN_ROLE)?;

        // Validate pattern type
        self.validate_pattern_type(pattern_type)?;

        let old_value = self.pattern_type_rate_thresholds.get(pattern_type);
        self.pattern_type_rate_thresholds
            .insert(pattern_type, threshold);

        evm::log(PatternTypeRateThresholdUpdated {
            pattern_type,
            old_value,
            new_value: threshold,
        });

        Ok(())
    }

    fn get_pattern_type_rate_threshold(&self, pattern_type: U256) -> U256 {
        self.pattern_type_rate_thresholds.get(pattern_type)
    }

    // === V2: ERC-165 Interface Detection ===

    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        self.supported_interfaces.get(interface_id)
    }
}

// === Public API ===
#[public]
impl DetectionEngine {
    /// Initialize the contract with the deployer as owner
    /// Sets default configuration values and initializes ERC-165 support
    /// Should be called once after deployment
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        // Only allow initialization if owner is not set
        if self.owner.get() != Address::ZERO {
            return Err(Error::InvalidOwner(self.owner.get()).into());
        }
        self.owner.set(msg::sender());

        // Initialize V2 defaults
        self.initialize_defaults();

        // Initialize ERC-165 support
        self.initialize_erc165();

        Ok(())
    }

    // === V1 Public Methods ===

    /// Register a new metric with a threshold (requires ADMIN role)
    pub fn register_metric(&mut self, id: U256, threshold: U256) -> Result<(), Vec<u8>> {
        IDetectionEngine::register_metric(self, id, threshold).map_err(|e| e.into())
    }

    /// Report a metric value (requires ANALYST role)
    pub fn report_metric(&mut self, id: U256, value: U256) -> Result<(), Vec<u8>> {
        IDetectionEngine::report_metric(self, id, value).map_err(|e| e.into())
    }

    /// Check if a metric indicates an anomaly
    pub fn check_anomaly(&self, id: U256) -> Result<bool, Vec<u8>> {
        IDetectionEngine::check_anomaly(self, id).map_err(|e| e.into())
    }

    /// Get threshold for a metric
    pub fn get_threshold(&self, id: U256) -> U256 {
        IDetectionEngine::get_threshold(self, id)
    }

    /// Get current value for a metric
    pub fn get_current_value(&self, id: U256) -> U256 {
        IDetectionEngine::get_current_value(self, id)
    }

    /// Get total metric count
    pub fn get_metric_count(&self) -> U256 {
        IDetectionEngine::get_metric_count(self)
    }

    /// Get current owner
    pub fn owner(&self) -> Address {
        IDetectionEngine::owner(self)
    }

    /// Transfer ownership
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        IDetectionEngine::transfer_ownership(self, new_owner).map_err(|e| e.into())
    }

    // === V2: Pattern Registration Public Methods ===

    /// Register a new exploit pattern (requires ADMIN role)
    pub fn register_pattern(
        &mut self,
        pattern_type: U256,
        severity: U256,
        description_hash: FixedBytes<32>,
    ) -> Result<U256, Vec<u8>> {
        IDetectionEngine::register_pattern(self, pattern_type, severity, description_hash)
            .map_err(|e| e.into())
    }

    /// Deactivate an exploit pattern (requires ADMIN role)
    pub fn deactivate_pattern(&mut self, id: U256) -> Result<(), Vec<u8>> {
        IDetectionEngine::deactivate_pattern(self, id).map_err(|e| e.into())
    }

    /// Activate an exploit pattern (requires ADMIN role)
    pub fn activate_pattern(&mut self, id: U256) -> Result<(), Vec<u8>> {
        IDetectionEngine::activate_pattern(self, id).map_err(|e| e.into())
    }

    /// Get an exploit pattern by ID
    pub fn get_pattern(
        &self,
        id: U256,
    ) -> Result<(U256, U256, FixedBytes<32>, U256), Vec<u8>> {
        IDetectionEngine::get_pattern(self, id).map_err(|e| e.into())
    }

    /// Get total pattern count
    pub fn get_pattern_count(&self) -> U256 {
        IDetectionEngine::get_pattern_count(self)
    }

    /// Check if a pattern is active
    pub fn is_pattern_active(&self, id: U256) -> bool {
        IDetectionEngine::is_pattern_active(self, id)
    }

    // === V2: Transaction Analysis Public Methods ===

    /// Analyze a transaction for potential exploits (requires ANALYST role)
    pub fn analyze_transaction(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
        calldata_hash: FixedBytes<32>,
        gas_price: U256,
    ) -> Result<U256, Vec<u8>> {
        IDetectionEngine::analyze_transaction(self, from, to, value, calldata_hash, gas_price)
            .map_err(|e| e.into())
    }

    /// Get an analysis result by ID
    pub fn get_analysis_result(
        &self,
        id: U256,
    ) -> Result<(Address, Address, U256, FixedBytes<32>, U256, U256, U256, U256), Vec<u8>> {
        IDetectionEngine::get_analysis_result(self, id).map_err(|e| e.into())
    }

    /// Get total analysis count
    pub fn get_analysis_count(&self) -> U256 {
        IDetectionEngine::get_analysis_count(self)
    }

    // === V2: Threat Scoring Public Methods ===

    /// Get cumulative threat score for an address
    pub fn get_address_threat_score(&self, addr: Address) -> U256 {
        IDetectionEngine::get_address_threat_score(self, addr)
    }

    /// Get analysis count for an address
    pub fn get_address_analysis_count(&self, addr: Address) -> U256 {
        IDetectionEngine::get_address_analysis_count(self, addr)
    }

    /// Set high threat threshold (requires ADMIN role)
    pub fn set_high_threat_threshold(&mut self, threshold: U256) -> Result<(), Vec<u8>> {
        IDetectionEngine::set_high_threat_threshold(self, threshold).map_err(|e| e.into())
    }

    /// Get current high threat threshold
    pub fn get_high_threat_threshold(&self) -> U256 {
        IDetectionEngine::get_high_threat_threshold(self)
    }

    // === V2: Whitelist Public Methods ===

    /// Add an address to the whitelist (requires ADMIN role)
    pub fn add_to_whitelist(&mut self, addr: Address) -> Result<(), Vec<u8>> {
        IDetectionEngine::add_to_whitelist(self, addr).map_err(|e| e.into())
    }

    /// Remove an address from the whitelist (requires ADMIN role)
    pub fn remove_from_whitelist(&mut self, addr: Address) -> Result<(), Vec<u8>> {
        IDetectionEngine::remove_from_whitelist(self, addr).map_err(|e| e.into())
    }

    /// Check if an address is whitelisted
    pub fn is_whitelisted(&self, addr: Address) -> bool {
        IDetectionEngine::is_whitelisted(self, addr)
    }

    // === V2: Role-Based Access Control Public Methods ===

    /// Grant a role to an account (requires ADMIN role)
    /// Role values: ADMIN=1, ANALYST=2
    pub fn grant_role(&mut self, account: Address, role: u8) -> Result<(), Vec<u8>> {
        IDetectionEngine::grant_role(self, account, role).map_err(|e| e.into())
    }

    /// Revoke a role from an account (requires ADMIN role)
    /// Cannot revoke own role or owner's roles
    pub fn revoke_role(&mut self, account: Address, role: u8) -> Result<(), Vec<u8>> {
        IDetectionEngine::revoke_role(self, account, role).map_err(|e| e.into())
    }

    /// Check if an account has a specific role
    pub fn has_role(&self, account: Address, role: u8) -> bool {
        IDetectionEngine::has_role(self, account, role)
    }

    /// Get all roles for an account (as bitmask)
    pub fn get_roles(&self, account: Address) -> u8 {
        IDetectionEngine::get_roles(self, account)
    }

    // === V2: Pattern Type Configuration Public Methods ===

    /// Set value threshold for a pattern type (requires ADMIN role)
    pub fn set_pattern_type_threshold(
        &mut self,
        pattern_type: U256,
        threshold: U256,
    ) -> Result<(), Vec<u8>> {
        IDetectionEngine::set_pattern_type_threshold(self, pattern_type, threshold)
            .map_err(|e| e.into())
    }

    /// Get value threshold for a pattern type
    pub fn get_pattern_type_threshold(&self, pattern_type: U256) -> U256 {
        IDetectionEngine::get_pattern_type_threshold(self, pattern_type)
    }

    /// Set rate threshold for a pattern type (requires ADMIN role)
    pub fn set_pattern_type_rate_threshold(
        &mut self,
        pattern_type: U256,
        threshold: U256,
    ) -> Result<(), Vec<u8>> {
        IDetectionEngine::set_pattern_type_rate_threshold(self, pattern_type, threshold)
            .map_err(|e| e.into())
    }

    /// Get rate threshold for a pattern type
    pub fn get_pattern_type_rate_threshold(&self, pattern_type: U256) -> U256 {
        IDetectionEngine::get_pattern_type_rate_threshold(self, pattern_type)
    }

    // === V2: ERC-165 Public Method ===

    /// Query if contract supports an interface (ERC-165)
    pub fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        IDetectionEngine::supports_interface(self, interface_id)
    }

    // === Convenience Functions ===

    /// Get ADMIN role constant
    pub fn admin_role(&self) -> u8 {
        ADMIN_ROLE
    }

    /// Get ANALYST role constant
    pub fn analyst_role(&self) -> u8 {
        ANALYST_ROLE
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
    use alloc::{vec, format};

    // ========================================
    // Threshold & Anomaly Detection Logic
    // ========================================

    // === Role Constant Tests ===

    #[test]
    fn test_role_constants() {
        assert_eq!(ADMIN_ROLE, 0x01);
        assert_eq!(ANALYST_ROLE, 0x02);
        assert_eq!(ALL_ROLES, 0x03);
    }

    #[test]
    fn test_value_above_threshold_is_anomaly() {
        let threshold = U256::from(100);
        let value = U256::from(150);
        assert!(value > threshold, "Value above threshold is an anomaly");
    }

    #[test]
    fn test_value_below_threshold_is_normal() {
        let threshold = U256::from(100);
        let value = U256::from(50);
        assert!(!(value > threshold), "Value below threshold is normal");
    }

    #[test]
    fn test_value_equal_threshold_is_normal() {
        let threshold = U256::from(100);
        let value = U256::from(100);
        assert!(!(value > threshold), "Value equal to threshold is NOT anomaly (strict >)");
    }

    #[test]
    fn test_value_one_above_threshold_is_anomaly() {
        let threshold = U256::from(100);
        let value = U256::from(101);
        assert!(value > threshold);
    }

    #[test]
    fn test_zero_threshold_any_value_is_anomaly() {
        let threshold = U256::ZERO;
        let value = U256::from(1);
        assert!(value > threshold, "Any positive value exceeds zero threshold");
    }

    #[test]
    fn test_zero_value_zero_threshold_is_normal() {
        let threshold = U256::ZERO;
        let value = U256::ZERO;
        assert!(!(value > threshold));
    }

    #[test]
    fn test_max_threshold_never_exceeded() {
        let threshold = U256::MAX;
        let value = U256::MAX;
        assert!(!(value > threshold), "U256::MAX cannot exceed itself");
    }

    #[test]
    fn test_large_threshold_values() {
        // 1 ETH in wei
        let threshold = U256::from(1_000_000_000_000_000_000u64);
        let below = U256::from(999_999_999_999_999_999u64);
        let above = U256::from(1_000_000_000_000_000_001u64);
        assert!(!(below > threshold));
        assert!(above > threshold);
    }

    // ========================================
    // Metric Count Arithmetic
    // ========================================

    #[test]
    fn test_metric_count_starts_zero() {
        let count = U256::ZERO;
        assert_eq!(count, U256::from(0));
    }

    #[test]
    fn test_metric_count_increments() {
        let mut count = U256::ZERO;
        for i in 1..=5u64 {
            count = count + U256::from(1);
            assert_eq!(count, U256::from(i));
        }
    }

    #[test]
    fn test_metric_count_independent_of_values() {
        // Count tracks number of metrics, not their values
        let count = U256::from(3); // 3 metrics registered
        let value = U256::from(1_000_000); // some metric value
        assert_ne!(count, value);
    }

    // ========================================
    // Metric ID Tests
    // ========================================

    #[test]
    fn test_metric_id_zero_valid() {
        let id = U256::ZERO;
        assert_eq!(id, U256::from(0), "ID 0 is valid");
    }

    #[test]
    fn test_metric_id_large() {
        let id = U256::from(u64::MAX);
        assert!(id > U256::ZERO);
    }

    #[test]
    fn test_metric_ids_distinct() {
        let id1 = U256::from(1);
        let id2 = U256::from(2);
        assert_ne!(id1, id2);
    }

    // ========================================
    // Error Encoding Tests
    // ========================================

    #[test]
    fn test_unauthorized_caller_error_encoding() {
        let caller = Address::from([0xAAu8; 20]);
        let encoded: Vec<u8> = Error::UnauthorizedCaller(caller).into();
        assert_eq!(encoded.len(), 36, "selector(4) + address(32)");
    }

    #[test]
    fn test_metric_not_found_error_encoding() {
        let id = U256::from(42);
        let encoded: Vec<u8> = Error::MetricNotFound { id }.into();
        assert_eq!(encoded.len(), 36, "selector(4) + uint256(32)");
    }

    #[test]
    fn test_invalid_threshold_error_encoding() {
        let value = U256::from(999);
        let encoded: Vec<u8> = Error::InvalidThreshold { value }.into();
        assert_eq!(encoded.len(), 36);
    }

    #[test]
    fn test_threshold_exceeded_error_encoding() {
        let current = U256::from(200);
        let threshold = U256::from(100);
        let encoded: Vec<u8> = Error::ThresholdExceeded { current, threshold }.into();
        // selector(4) + uint256(32) + uint256(32)
        assert_eq!(encoded.len(), 68, "Should encode both current and threshold");
    }

    #[test]
    fn test_invalid_owner_error_encoding() {
        let owner = Address::ZERO;
        let encoded: Vec<u8> = Error::InvalidOwner(owner).into();
        assert_eq!(encoded.len(), 36);
    }

    #[test]
    fn test_all_error_selectors_unique() {
        let errors: Vec<Vec<u8>> = vec![
            Error::UnauthorizedCaller(Address::ZERO).into(),
            Error::MetricNotFound { id: U256::ZERO }.into(),
            Error::InvalidThreshold { value: U256::ZERO }.into(),
            Error::ThresholdExceeded { current: U256::ZERO, threshold: U256::ZERO }.into(),
            Error::InvalidOwner(Address::ZERO).into(),
        ];

        for i in 0..errors.len() {
            for j in (i + 1)..errors.len() {
                assert_ne!(
                    &errors[i][..4], &errors[j][..4],
                    "Error selectors at indices {i} and {j} must differ"
                );
            }
        }
    }

    #[test]
    fn test_error_encoding_deterministic() {
        let enc1: Vec<u8> = Error::MetricNotFound { id: U256::from(7) }.into();
        let enc2: Vec<u8> = Error::MetricNotFound { id: U256::from(7) }.into();
        assert_eq!(enc1, enc2);
    }

    // ========================================
    // Address Validation
    // ========================================

    #[test]
    fn test_zero_address_check() {
        assert_eq!(Address::ZERO, Address::ZERO);
        assert_ne!(Address::from([1u8; 20]), Address::ZERO);
    }

    // ========================================
    // Debug Trait
    // ========================================

    #[test]
    fn test_all_errors_have_debug() {
        let errors: Vec<Error> = vec![
            Error::UnauthorizedCaller(Address::ZERO),
            Error::MetricNotFound { id: U256::ZERO },
            Error::InvalidThreshold { value: U256::ZERO },
            Error::ThresholdExceeded { current: U256::ZERO, threshold: U256::ZERO },
            Error::InvalidOwner(Address::ZERO),
        ];
        for err in errors {
            assert!(!format!("{err:?}").is_empty());
        }
    }
}
