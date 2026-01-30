//! ArbiShield Invariant Test Runner
//!
//! This is the main entry point for running all invariant tests.
//!
//! Run with:
//! ```bash
//! cargo test --test invariant_tests
//! ```

mod invariants;

// Re-export all invariant test modules for easy access
pub use invariants::*;
