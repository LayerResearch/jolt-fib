use std::path::Path;

/// Configuration for Spike simulator
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Memory size in bytes
    pub memory_size: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            memory_size: 64 * 1024 * 1024, // 64MB
        }
    }
}

/// Error type for Spike operations
#[derive(Debug, thiserror::Error)]
pub enum SpikeError {
    #[error("Invalid ELF file")]
    InvalidElf,
    #[error("FFI error: {0}")]
    Ffi(String),
    #[error("Execution error: {0}")]
    Execution(String),
}

mod spike;
mod log_parser;

pub use spike::SpikeTracer;
