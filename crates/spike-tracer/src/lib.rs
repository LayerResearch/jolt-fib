use serde::{Deserialize, Serialize};
use thiserror::Error;

// Main spike tracer integration using autocxx
mod autocxx_ffi;
pub use autocxx_ffi::*;

/// Errors that can occur during Spike execution
#[derive(Error, Debug)]
pub enum SpikeError {
    #[error("Invalid ELF file")]
    InvalidElf,
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Memory error: {0}")]
    Memory(String),
    #[error("FFI error: {0}")]
    Ffi(String),
}

/// Memory configuration for the RISC-V simulator
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub memory_size: usize,
    pub max_input_size: usize,
    pub max_output_size: usize,
    pub stack_size: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            memory_size: 64 * 1024 * 1024, // 64MB
            max_input_size: 1024,
            max_output_size: 1024,
            stack_size: 1024 * 1024, // 1MB
        }
    }
}

/// Spike commit information for trace collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpikeCommit {
    pub pc: u64,
    pub instruction: u32,
    pub privilege_level: u8,
    // TODO: Add register writes and memory accesses in later phases
}

// Re-export AutocxxSpikeTracer as the main SpikeTracer interface for compatibility
pub type SpikeTracer = AutocxxSpikeTracer;
