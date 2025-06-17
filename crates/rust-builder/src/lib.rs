use glob::{GlobError, PatternError};
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

mod compile;
mod link;

pub use compile::*;
pub use link::*;

/// Errors that can occur during compilation or linking
#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Compilation failed: {0}")]
    CompilationError(String),

    #[error("Linking failed: {0}")]
    LinkingError(String),

    #[error("Failed to find required library: {0}")]
    LibraryNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Environment error: {0}")]
    EnvError(String),

    #[error("Pattern error: {0}")]
    PatternError(#[from] PatternError),

    #[error("Glob error: {0}")]
    GlobError(#[from] GlobError),
}

/// Result type for builder operations
pub type Result<T> = std::result::Result<T, BuilderError>;

/// Configuration for program compilation
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Target triple (e.g., "riscv32im-unknown-none-elf", "x86_64-unknown-linux-gnu")
    pub target: String,
    /// Additional rustflags
    pub rustflags: Vec<String>,
    /// Path to linker script (optional)
    pub linker_script: Option<PathBuf>,
    /// Whether to clean up intermediate object files
    pub cleanup_objects: bool,
    /// Additional object files to link
    pub additional_objects: Vec<PathBuf>,
    /// Output format (e.g., "elf", "bin", "hex")
    pub output_format: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: "riscv64gc-unknown-none-elf".to_string(),
            rustflags: vec!["-C target-feature=+m,+a,+c".to_string()],
            linker_script: None,
            cleanup_objects: true,
            additional_objects: Vec::new(),
            output_format: "elf".to_string(),
        }
    }
}

/// Result of compilation
#[derive(Debug)]
pub struct BuildResult {
    /// Path to the generated output file
    pub output_path: PathBuf,
    /// Size of the output file in bytes
    pub output_size: u64,
    /// Number of instructions executed during compilation
    pub build_time_ms: Option<u64>,
}

/// Get Rust sysroot path
pub fn get_rust_sysroot() -> Result<PathBuf> {
    let output = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()?;

    if !output.status.success() {
        return Err(BuilderError::CompilationError(
            "Failed to get Rust sysroot".to_string(),
        ));
    }

    let sysroot = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(sysroot))
}

/// Get Host triple
pub fn get_host_triple() -> Result<String> {
    let output = Command::new("rustc")
        .arg("--version")
        .arg("--verbose")
        .output()?;

    if !output.status.success() {
        return Err(BuilderError::CompilationError(
            "Failed to get Rust version info".to_string(),
        ));
    }

    let output = String::from_utf8_lossy(&output.stdout);
    for line in output.lines() {
        if line.starts_with("host: ") {
            return Ok(line[6..].trim().to_string());
        }
    }

    Err(BuilderError::CompilationError(
        "Could not find host triple in rustc output".to_string(),
    ))
}
