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

#[cfg(feature = "guest")]
pub use program::{Program, ProgramError};

#[cfg(feature = "guest")]
pub use proof::{JoltProofBundle, JoltProofWrapper};

// Re-export macros for convenience
pub use macros::*;
