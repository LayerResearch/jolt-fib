use std::fs;
use spike_tracer::SpikeTracer;
use log::info;
use crate::common::{build_test_program, DEFAULT_MEMORY_CONFIG, SIMPLE_ADD_INSTRUCTIONS};

mod common;

#[test]
fn test_simple_add_elf_execution() {
    // Initialize logger
    env_logger::init();
    info!("Starting simple_add test");

    // Build the test program
    let elf_path = build_test_program("test_programs/simple_add.rs")
        .expect("Failed to build simple_add test program");
    info!("Built test program at {}", elf_path.display());

    let mut tracer = SpikeTracer::new("rv32im", &DEFAULT_MEMORY_CONFIG)
        .expect("Failed to create SpikeTracer");
    info!("Created SpikeTracer");

    // Load simple_add.elf
    let elf_data = fs::read(&elf_path).expect("Failed to read simple_add.elf");
    info!("Loaded simple_add.elf: {} bytes", elf_data.len());

    // Execute with instruction limit (Rust program with 5+7=12 computation)
    tracer.execute(&elf_data, SIMPLE_ADD_INSTRUCTIONS)
        .expect("Failed to execute simple_add.elf");
    info!("Rust simple_add program executed");
    info!("Instructions executed: {}", tracer.instruction_count());

    // Verify reasonable instruction count for simple Rust program
    assert!(tracer.instruction_count() > 0);
    assert!(tracer.instruction_count() <= SIMPLE_ADD_INSTRUCTIONS);

    info!("🎉 Rust simple_add test passed!");
} 