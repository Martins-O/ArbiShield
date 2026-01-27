//! ArbiShield - Security Contracts for Arbitrum Stylus
//!
//! This library provides three core security contracts:
//! - **DetectionEngine**: Monitors metrics and detects anomalies
//! - **CircuitBreaker**: Emergency stop mechanism for halting operations
//! - **AlertRegistry**: Permanent record of security alerts
//!
//! # Architecture
//!
//! The contracts are designed to work independently or together as a
//! coordinated security system. Each contract follows best practices for
//! Stylus development with proxy-compatible storage layouts.
//!
//! # Example Usage
//!
//! Deploy each contract independently and coordinate them externally:
//!
//! ```rust,ignore
//! // Deploy DetectionEngine
//! let engine = DetectionEngine::new();
//! engine.register_metric(U256::from(1), U256::from(1000))?;
//!
//! // Deploy CircuitBreaker
//! let breaker = CircuitBreaker::new();
//!
//! // Deploy AlertRegistry
//! let registry = AlertRegistry::new();
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

/// DetectionEngine contract module
pub mod detection_engine;

/// CircuitBreaker contract module
pub mod circuit_breaker;

/// AlertRegistry contract module
pub mod alert_registry;

// Re-export main contract types for convenience
pub use detection_engine::storage::DetectionEngine;
pub use circuit_breaker::storage::CircuitBreaker;
pub use alert_registry::storage::AlertRegistry;

#[cfg(feature = "export-abi")]
/// Export ABI for all contracts
pub fn print_from_args() {
    println!("ArbiShield contracts ABI export");
}
