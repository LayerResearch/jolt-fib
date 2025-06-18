use crate::common::{build_test_program, DEFAULT_MEMORY_CONFIG, SIMPLE_ADD_INSTRUCTIONS};
use log::info;
use spike_tracer::{new_spike_tracer, SpikeTracer};
use std::fs;

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

    // Create tracer
    let mut tracer = new_spike_tracer("rv32im");
    info!("Created SpikeTracer");

    // Load and execute ELF
    let elf_data = fs::read(&elf_path).expect("Failed to read simple_add.elf");
    info!("Loaded simple_add.elf: {} bytes", elf_data.len());

    let elf_str = elf_path.to_str().expect("ELF path is not valid UTF-8");
    let input = vec![0; 1024];
    let mut output = vec![0; 1024];
    let return_code = tracer.pin_mut().run(&elf_str, &input, &mut output);
    info!("Program terminated with return code: {}", return_code);

    info!("🎉 Rust simple_add test passed!");
}
