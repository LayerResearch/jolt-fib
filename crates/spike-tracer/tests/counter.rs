use std::fs;
use spike_tracer::SpikeTracer;
use log::info;
use crate::common::{build_test_program, get_tohost_address, DEFAULT_MEMORY_CONFIG, MAX_INSTRUCTIONS};

mod common;

#[test]
fn test_counter_elf_execution() {
    // Initialize logger
    env_logger::init();
    info!("Starting counter test");

    // Build the test program
    let elf_path = build_test_program("test_programs/counter.rs")
        .expect("Failed to build counter test program");
    info!("Built test program at {}", elf_path.display());

    let mut tracer = SpikeTracer::new("rv32im", &DEFAULT_MEMORY_CONFIG)
        .expect("Failed to create SpikeTracer");
    info!("Created SpikeTracer");

    // Load counter.elf
    let elf_data = fs::read(&elf_path).expect("Failed to read counter.elf");
    info!("Loaded counter.elf: {} bytes", elf_data.len());

    // Execute until tohost
    let tohost_addr = get_tohost_address(&elf_path).expect("Failed to find tohost symbol");
    info!("Executing until tohost @ 0x{:x}...", tohost_addr);

    tracer.execute_until_tohost(&elf_data, tohost_addr)
        .expect("Failed to execute counter.elf");
    info!("Program terminated normally");
    info!("Instructions executed: {}", tracer.instruction_count());

    // Verify execution results
    assert!(tracer.instruction_count() > 0, "No instructions executed");
    assert!(
        tracer.instruction_count() < MAX_INSTRUCTIONS,
        "Too many instructions (infinite loop?)"
    );

    info!("🎉 Counter test passed!");
} 