use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;
use rust_builder::{build_rust_program, BuildConfig};
use spike_tracer::MemoryConfig;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Environment error: {0}")]
    Env(#[from] env::VarError),
    #[error("Build error: {0}")]
    Build(String),
    #[error("Symbol error: {0}")]
    Symbol(String),
}

pub const DEFAULT_MEMORY_CONFIG: MemoryConfig = MemoryConfig {
    memory_size: 64 * 1024 * 1024, // 64MB
    max_input_size: 1024,
    max_output_size: 1024,
    stack_size: 1024 * 1024, // 1MB
};

pub const MAX_INSTRUCTIONS: u64 = 100000;
pub const SIMPLE_ADD_INSTRUCTIONS: u64 = 1000;

/// Build a single test program using rust-builder
pub fn build_test_program(source_path: &str) -> Result<PathBuf, TestError> {
    // Use OUT_DIR (standard Rust pattern for build script outputs)
    let out_dir = env::var("OUT_DIR")?;
    let build_dir = Path::new(&out_dir).join("test_programs");

    // Ensure build directory exists
    if !build_dir.exists() {
        fs::create_dir_all(&build_dir)?;
    }

    // Get output path from source path
    let source_name = Path::new(source_path).file_stem()
        .ok_or_else(|| TestError::Build("Invalid source path".to_string()))?
        .to_string_lossy();
    let output_path = build_dir.join(format!("{}.elf", source_name));

    // Only build if ELF file doesn't exist
    if !output_path.exists() {
        println!("cargo:warning=Building test program {}...", source_name);

        // Configure build
        let mut config = BuildConfig::default();
        config.target = "riscv32im-unknown-none-elf".to_string();
        config.linker_script = Some(PathBuf::from("test_programs/linker.ld"));

        // Build the program
        build_rust_program(&[source_path], &output_path, &config)
            .map_err(|e| TestError::Build(format!("Failed to build {}: {}", source_name, e)))?;

        println!("cargo:warning=Built {} successfully", source_name);
    }

    Ok(output_path)
}

pub fn get_tohost_address(elf_path: &Path) -> Result<u64, TestError> {
    if !elf_path.exists() {
        return Err(TestError::Symbol(format!("ELF file not found: {}", elf_path.display())));
    }

    let output = Command::new("riscv64-unknown-elf-nm")
        .arg(elf_path)
        .output()?;

    if !output.status.success() {
        return Err(TestError::Symbol("Failed to run nm on ELF file".into()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.contains("tohost") && !line.contains("fromhost") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[2] == "tohost" {
                let addr_str = parts[0];
                let addr = u64::from_str_radix(addr_str, 16)
                    .map_err(|_| TestError::Symbol("Invalid tohost address".into()))?;
                println!("📍 Found tohost symbol at 0x{:x}", addr);
                return Ok(addr);
            }
        }
    }

    Err(TestError::Symbol("tohost symbol not found in ELF file".into()))
} 