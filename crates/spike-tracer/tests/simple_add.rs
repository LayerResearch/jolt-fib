use crate::common::{build_test_program, DEFAULT_MEMORY_CONFIG, SIMPLE_ADD_INSTRUCTIONS};
use log::info;
use spike_tracer::SpikeTracer;
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
    let mut tracer = SpikeTracer::new("rv32im");
    info!("Created SpikeTracer");

    // Load and execute ELF
    let elf_data = fs::read(&elf_path).expect("Failed to read simple_add.elf");
    info!("Loaded simple_add.elf: {} bytes", elf_data.len());

    let log_file = tempfile::NamedTempFile::new().expect("Failed to create temp log file");
    let log_path = log_file
        .path()
        .to_str()
        .expect("Log path is not valid UTF-8");
    info!("Created temporary log file at {}", log_path);

    let elf_str = elf_path.to_str().expect("ELF path is not valid UTF-8");
    let input = vec![0; 1024];
    let mut output = vec![0; 1024];
    let return_code = tracer.run(&elf_str, &input, &mut output, log_path);
    info!("Program terminated with return code: {}", return_code);

    info!("🎉 Rust simple_add test passed!");
}
