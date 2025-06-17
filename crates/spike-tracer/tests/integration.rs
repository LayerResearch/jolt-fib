use spike_tracer::{MemoryConfig, SpikeTracer};
use std::fs;
use std::path::PathBuf;

mod common;
use common::*;

#[test]
fn test_invalid_elf_handling() {
    let memory_config = MemoryConfig::default();
    let mut tracer =
        SpikeTracer::new("rv32im", &memory_config).expect("Failed to create SpikeTracer");

    // Test with invalid ELF data
    let invalid_elf = b"not an elf file";

    let result = tracer.execute(invalid_elf, 1000);
    assert!(result.is_err(), "Should reject invalid ELF");

    println!("✅ Invalid ELF properly rejected");
}

#[test]
fn test_spike_tracer_creation() {
    // Test various ISA configurations
    let memory_config = MemoryConfig::default();

    // Valid ISA
    let tracer = SpikeTracer::new("rv32im", &memory_config);
    assert!(tracer.is_ok(), "rv32im should be valid");

    // Invalid ISA
    let tracer = SpikeTracer::new("invalid_isa", &memory_config);
    assert!(tracer.is_err(), "invalid_isa should be rejected");

    println!("✅ ISA validation working");
}
