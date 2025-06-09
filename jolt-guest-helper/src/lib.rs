//! Jolt Guest Helper
//!
//! A helper library for developing and managing Jolt guest programs.
//! This library provides a generic interface for handling guest program compilation,
//! proving, and verification.

mod builder;
mod macros;
mod program;
mod proof;

#[cfg(feature = "host")]
pub use builder::{Builder, BuilderError};

pub use program::{Program, ProgramError};

// Make proof structures available for both host and guest
pub use proof::{JoltProofBundle, JoltProofWrapper};

// Re-export macros for convenience
pub use macros::*;
