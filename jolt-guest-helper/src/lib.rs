//! Jolt Guest Helper
//! 
//! A helper library for developing and managing Jolt guest programs.
//! This library provides a generic interface for handling guest program compilation,
//! proving, and verification.

mod guest;
mod macros;

pub use guest::{
    Guest,
    GuestBuilder,
    GuestConfig,
    GuestError,
};

// Re-export macros for convenience
pub use macros::*;

